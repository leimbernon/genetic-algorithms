/**
 * Development Agent — Step 5: Pull Request Creator
 *
 * Creates a GitHub pull request with the correct metadata:
 * title matching the issue, labels copied from the issue,
 * milestone from the issue, assignee from the issue, and
 * a clear description summarizing the changes.
 *
 * Outputs:
 *   - pr_created: "true" | "false"
 *   - pr_url: the URL of the created PR
 *   - pr_number: the PR number
 */

import { readFileSync, existsSync } from "node:fs";
import {
  getEnv,
  setGitHubOutput,
  createClient,
  callModel,
  resolveModel,
  loadPrompt,
  stripMarkdownFences,
  truncateToTokenBudget,
  getIssue,
  githubApiGet,
  swapLabels,
  LABELS,
  type TriageResult,
  type ArchitecturePlan,
  type WriterResult,
  type AIClient,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface PRDescription {
  title: string;
  body: string;
  labels: string[];
  milestone: string | null;
  assignees: string[];
  base_branch: string;
}

interface GitHubPR {
  number: number;
  html_url: string;
}

// ---------------------------------------------------------------------------
// PR Description Generation
// ---------------------------------------------------------------------------

/**
 * Use the AI model to generate the PR title and body.
 */
async function generatePRDescription(
  client: AIClient,
  modelName: string,
  triageResult: TriageResult,
  plan: ArchitecturePlan,
  writerResult: WriterResult,
  issueBody: string,
  issueLabels: string[],
  milestoneName: string | null,
  assignees: string[],
): Promise<PRDescription> {
  const systemPrompt = loadPrompt("pull-request");

  const userPrompt = `## Source Issue
- **Issue #${triageResult.issue_number}:** ${triageResult.issue_title}
- **Type:** ${triageResult.branch_type === "feature" ? "Enhancement" : "Bug fix"}
- **Labels:** ${issueLabels.join(", ") || "None"}
- **Milestone:** ${milestoneName ?? "None"}
- **Assignees:** ${assignees.join(", ") || "None"}

### Issue Description
${truncateToTokenBudget(issueBody, 1000)}

## Changes Made
### Architecture Plan Summary
${plan.summary}

### Files Changed
${writerResult.files_written.map((f) => `- **${f.action}** \`${f.path}\`: ${f.description}`).join("\n")}

### Metrics
- Tests added: ${writerResult.tests_added}
- Benchmarks added: ${writerResult.benchmarks_added}
- Version bump: ${writerResult.version_bump ? `${writerResult.version_bump.from} → ${writerResult.version_bump.to} (${writerResult.version_bump.bump_type})` : "None"}

### Validation Status
- cargo fmt: ${writerResult.validation.fmt ? "PASS" : "FAIL"}
- cargo clippy: ${writerResult.validation.clippy ? "PASS" : "FAIL"}
- cargo test: ${writerResult.validation.tests ? "PASS" : "FAIL"}
- cargo test --doc: ${writerResult.validation.doc_tests ? "PASS" : "FAIL"}
- cargo bench --no-run: ${writerResult.validation.bench_compile ? "PASS" : "FAIL"}

## Branch Information
- **Head branch:** ${triageResult.branch_name}
- **Base branch:** ${triageResult.base_branch}

## Task
Generate a pull request title and description following the format in your instructions.
Respond with the JSON structure defined in your instructions.`;

  const rawResponse = await callModel(
    client,
    modelName,
    systemPrompt,
    userPrompt,
  );
  const cleaned = stripMarkdownFences(rawResponse);

  try {
    const parsed = JSON.parse(cleaned) as PRDescription;
    // Ensure correct metadata regardless of AI output
    parsed.labels = issueLabels;
    parsed.milestone = milestoneName;
    parsed.assignees = assignees;
    parsed.base_branch = triageResult.base_branch;
    return parsed;
  } catch (err) {
    console.error(`::error::Failed to parse PR description: ${err}`);
    console.error(`Raw response:\n${rawResponse}`);

    // Fallback: generate a basic PR description
    return {
      title: triageResult.issue_title.replace(/^\[(BUG|REQUEST)\]\s*/i, ""),
      body: [
        `## Summary\n`,
        `${triageResult.branch_type === "feature" ? "Implement" : "Fix"} ${triageResult.issue_title}.\n`,
        `Closes #${triageResult.issue_number}\n`,
        `## Changes\n`,
        ...writerResult.files_written.map(
          (f) => `- ${f.action} \`${f.path}\`: ${f.description}`,
        ),
        `\n## Testing\n`,
        `- ${writerResult.tests_added} test(s) added`,
        `- ${writerResult.benchmarks_added} benchmark(s) added`,
        `- All cargo checks: ${Object.values(writerResult.validation).every(Boolean) ? "PASS" : "PARTIAL"}`,
      ].join("\n"),
      labels: issueLabels,
      milestone: milestoneName,
      assignees,
      base_branch: triageResult.base_branch,
    };
  }
}

// ---------------------------------------------------------------------------
// GitHub PR Creation
// ---------------------------------------------------------------------------

/**
 * Create a pull request via the GitHub API.
 */
async function createPullRequest(
  repo: string,
  token: string,
  description: PRDescription,
  headBranch: string,
): Promise<GitHubPR> {
  // 1. Create the PR
  const createUrl = `https://api.github.com/repos/${repo}/pulls`;
  const createResp = await fetch(createUrl, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github.v3+json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      title: description.title,
      body: description.body,
      head: headBranch,
      base: description.base_branch,
    }),
  });

  if (!createResp.ok) {
    const errorBody = await createResp.text();
    throw new Error(
      `Failed to create PR: ${createResp.status} ${createResp.statusText}\n${errorBody}`,
    );
  }

  const pr = (await createResp.json()) as GitHubPR;
  console.log(`  Created PR #${pr.number}: ${pr.html_url}`);

  // 2. Add labels (if any)
  if (description.labels.length > 0) {
    const labelsUrl = `https://api.github.com/repos/${repo}/issues/${pr.number}/labels`;
    const labelsResp = await fetch(labelsUrl, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github.v3+json",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ labels: description.labels }),
    });
    if (labelsResp.ok) {
      console.log(`  Added labels: ${description.labels.join(", ")}`);
    } else {
      console.warn(
        `::warning::Failed to add labels: ${labelsResp.status} ${labelsResp.statusText}`,
      );
    }
  }

  // 3. Set milestone (if any)
  if (description.milestone) {
    // Find milestone number by name
    const milestonesUrl = `https://api.github.com/repos/${repo}/milestones?state=open&per_page=100`;
    const milestones = await githubApiGet<
      Array<{ number: number; title: string }>
    >(milestonesUrl, token);
    const milestone = milestones.find(
      (m) =>
        m.title.toLowerCase() === description.milestone!.toLowerCase(),
    );

    if (milestone) {
      const updateUrl = `https://api.github.com/repos/${repo}/issues/${pr.number}`;
      const updateResp = await fetch(updateUrl, {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: "application/vnd.github.v3+json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ milestone: milestone.number }),
      });
      if (updateResp.ok) {
        console.log(`  Set milestone: ${description.milestone}`);
      } else {
        console.warn(
          `::warning::Failed to set milestone: ${updateResp.status}`,
        );
      }
    } else {
      console.warn(
        `::warning::Milestone '${description.milestone}' not found.`,
      );
    }
  }

  // 4. Add assignees (if any)
  if (description.assignees.length > 0) {
    const assigneesUrl = `https://api.github.com/repos/${repo}/issues/${pr.number}/assignees`;
    const assigneesResp = await fetch(assigneesUrl, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github.v3+json",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ assignees: description.assignees }),
    });
    if (assigneesResp.ok) {
      console.log(`  Added assignees: ${description.assignees.join(", ")}`);
    } else {
      console.warn(
        `::warning::Failed to add assignees: ${assigneesResp.status}`,
      );
    }
  }

  return pr;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const repo = getEnv("REPO");

  const client = createClient(token);
  const modelName = resolveModel("PULL_REQUEST");

  console.log("Development Pull Request Agent");
  console.log(`Using model: ${modelName}`);
  console.log("=".repeat(60));

  // Load artifacts
  const triageResult: TriageResult = JSON.parse(
    readFileSync("triage-result.json", "utf-8"),
  );
  const plan: ArchitecturePlan = JSON.parse(
    readFileSync("architecture-plan.json", "utf-8"),
  );
  const writerResult: WriterResult = JSON.parse(
    readFileSync("writer-result.json", "utf-8"),
  );

  console.log(`Issue #${triageResult.issue_number}: ${triageResult.issue_title}`);
  console.log(`Branch: ${triageResult.branch_name} → ${triageResult.base_branch}`);

  // Fetch issue for metadata
  const issue = await getIssue(repo, triageResult.issue_number, token);
  const issueLabels = issue.labels.map((l) => l.name);
  const milestoneName = issue.milestone?.title ?? null;
  const assignees = issue.assignees.map((a) => a.login);

  // Check if PR already exists for this branch
  const existingPRsUrl = `https://api.github.com/repos/${repo}/pulls?head=${repo.split("/")[0]}:${triageResult.branch_name}&state=open`;
  const existingPRs = await githubApiGet<GitHubPR[]>(existingPRsUrl, token);

  if (existingPRs.length > 0) {
    const existing = existingPRs[0];
    console.log(
      `\nPR already exists: #${existing.number} (${existing.html_url})`,
    );
    setGitHubOutput("pr_created", "true");
    setGitHubOutput("pr_url", existing.html_url);
    setGitHubOutput("pr_number", String(existing.number));
    return;
  }

  // Generate PR description
  console.log("\nGenerating PR description...");
  const description = await generatePRDescription(
    client,
    modelName,
    triageResult,
    plan,
    writerResult,
    issue.body ?? "",
    issueLabels,
    milestoneName,
    assignees,
  );

  console.log(`\nPR Title: ${description.title}`);
  console.log(`Base: ${description.base_branch}`);
  console.log(`Labels: ${description.labels.join(", ") || "None"}`);
  console.log(`Milestone: ${description.milestone ?? "None"}`);
  console.log(`Assignees: ${description.assignees.join(", ") || "None"}`);

  // Create the PR
  console.log("\nCreating pull request...");
  const pr = await createPullRequest(
    repo,
    token,
    description,
    triageResult.branch_name,
  );

  setGitHubOutput("pr_created", "true");
  setGitHubOutput("pr_url", pr.html_url);
  setGitHubOutput("pr_number", String(pr.number));

  // Swap labels: "in progress" → "in review"
  await swapLabels(
    repo,
    triageResult.issue_number,
    LABELS.IN_PROGRESS,
    LABELS.IN_REVIEW,
    token,
  );

  console.log(`\nPull request created successfully: ${pr.html_url}`);
}

main().catch((err) => {
  console.error("::error::Pull Request agent failed:", err);
  process.exit(1);
});
