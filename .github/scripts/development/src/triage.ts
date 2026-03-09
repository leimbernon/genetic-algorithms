/**
 * Development Agent — Triage (Analysis Phase)
 *
 * Triggered when an issue receives the "selected for development" label,
 * or when the issue is edited / a new comment is added while that label
 * is present.
 *
 * Responsibilities:
 *   1. Read the issue description, labels, comments.
 *   2. Analyze whether requirements are sufficient (AI).
 *   3. If NOT met → post challenge questions as a comment.
 *   4. If met → swap "selected for development" → "prepared for development"
 *      and post a confirmation comment with acceptance criteria.
 *
 * Does NOT create branches or produce artifacts for downstream agents.
 *
 * Outputs:
 *   - triage_passed: "true" | "false"
 */

import {
  getEnv,
  setGitHubOutput,
  createClient,
  callModel,
  resolveModel,
  loadPrompt,
  readFileSafe,
  stripMarkdownFences,
  truncateToTokenBudget,
  getIssue,
  getIssueComments,
  postIssueComment,
  determineBranchType,
  swapLabels,
  LABELS,
  REPO_ROOT,
  type AIClient,
} from "./shared.js";
import { join } from "node:path";

// ---------------------------------------------------------------------------
// Triage Analysis
// ---------------------------------------------------------------------------

/**
 * Use the AI model to analyze the issue and determine if requirements are met.
 */
async function analyzeIssue(
  client: AIClient,
  modelName: string,
  issueTitle: string,
  issueBody: string,
  issueLabels: string[],
  issueComments: string[],
): Promise<{
  requirements_met: boolean;
  questions: string[];
  summary: string;
  acceptance_criteria: string[];
}> {
  const systemPrompt = loadPrompt("triage");
  const agentInstructions = readFileSafe(
    join(REPO_ROOT, "AGENT_INSTRUCTIONS.md"),
  );
  const contributingGuide = readFileSafe(join(REPO_ROOT, "CONTRIBUTING.md"));

  const userPrompt = `## Issue Information
- **Title:** ${issueTitle}
- **Labels:** ${issueLabels.join(", ") || "None"}
- **Description:**
${truncateToTokenBudget(issueBody || "No description provided.", 2000)}

## Comments on the Issue
${issueComments.length > 0 ? issueComments.map((c, i) => `### Comment ${i + 1}\n${truncateToTokenBudget(c, 500)}`).join("\n\n") : "No comments yet."}

## Project Context
### Agent Instructions (summary)
${truncateToTokenBudget(agentInstructions, 1500)}

### Contributing Guide (summary)
${truncateToTokenBudget(contributingGuide, 1000)}

## Task
Analyze this issue and determine:
1. Whether the requirements are clear enough to start development.
2. What questions need to be asked (if any).
3. A brief summary of the planned approach.
4. A list of acceptance criteria derived from the issue.

Respond with the JSON structure defined in your instructions.`;

  const rawResponse = await callModel(client, modelName, systemPrompt, userPrompt);
  const cleaned = stripMarkdownFences(rawResponse);

  try {
    return JSON.parse(cleaned);
  } catch (err) {
    console.error(`::error::Failed to parse triage response: ${err}`);
    console.error(`Raw response:\n${rawResponse}`);
    return {
      requirements_met: false,
      questions: [
        "The triage agent could not parse the issue. Please review the issue description and ensure it follows the template.",
      ],
      summary: "Triage analysis failed — manual review needed.",
      acceptance_criteria: [],
    };
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const repo = getEnv("REPO");
  const issueNumber = parseInt(getEnv("ISSUE_NUMBER"), 10);

  const client = createClient(token);
  const modelName = resolveModel("TRIAGE");

  console.log("Development Triage Agent — Analysis Phase");
  console.log(`Using model: ${modelName}`);
  console.log(`Processing issue #${issueNumber} in ${repo}`);
  console.log("=".repeat(60));

  // 1. Fetch the issue
  const issue = await getIssue(repo, issueNumber, token);

  // Skip pull requests (issues API can return PRs)
  if (issue.pull_request) {
    console.log("This is a pull request, not an issue. Skipping.");
    setGitHubOutput("triage_passed", "false");
    return;
  }

  console.log(`\nIssue: ${issue.title}`);
  console.log(`Labels: ${issue.labels.map((l) => l.name).join(", ") || "None"}`);
  console.log(`Milestone: ${issue.milestone?.title ?? "None"}`);

  // 2. Determine branch type (informational for the confirmation comment)
  const branchType = determineBranchType(issue.labels);

  // 3. Fetch existing comments for context (exclude bot comments)
  const comments = await getIssueComments(repo, issueNumber, token);
  const commentBodies = comments
    .filter((c) => !c.user.login.endsWith("[bot]"))
    .map((c) => c.body);

  // 4. Analyze the issue with AI
  console.log("\nAnalyzing issue requirements...");
  const analysis = await analyzeIssue(
    client,
    modelName,
    issue.title,
    issue.body ?? "",
    issue.labels.map((l) => l.name),
    commentBodies,
  );

  console.log(`\nRequirements met: ${analysis.requirements_met}`);
  console.log(`Summary: ${analysis.summary}`);
  if (analysis.questions.length > 0) {
    console.log(`Questions (${analysis.questions.length}):`);
    for (const q of analysis.questions) {
      console.log(`  - ${q}`);
    }
  }

  // 5. Post questions if requirements are NOT met
  if (!analysis.requirements_met && analysis.questions.length > 0) {
    const commentBody = [
      "## 🤖 Triage Agent — Clarification Needed\n",
      `I've reviewed this issue and have some questions before development can begin:\n`,
      ...analysis.questions.map((q, i) => `${i + 1}. ${q}`),
      "",
      `**Summary:** ${analysis.summary}`,
      "",
      "---",
      '_Please address these questions by editing the issue description or adding a comment. The analysis will re-run automatically while the "selected for development" label is present._',
    ].join("\n");

    await postIssueComment(repo, issueNumber, commentBody, token);
    console.log("\nPosted clarification questions on the issue.");
  }

  // 6. If requirements ARE met → swap labels and post confirmation
  if (analysis.requirements_met) {
    // Swap: "selected for development" → "prepared for development"
    await swapLabels(repo, issueNumber, LABELS.SELECTED, LABELS.PREPARED, token);

    const confirmBody = [
      "## 🤖 Triage Agent — Analysis Complete\n",
      `**Type:** ${branchType === "feature" ? "New feature" : "Bug fix"}`,
      `**Summary:** ${analysis.summary}\n`,
      "**Acceptance criteria:**",
      ...analysis.acceptance_criteria.map((c) => `- [ ] ${c}`),
      "",
      "---",
      '_Requirements are clear. The issue is now marked as "prepared for development". When you are ready, add the "in progress" label to start automated development._',
    ].join("\n");

    await postIssueComment(repo, issueNumber, confirmBody, token);
    console.log("\nPosted confirmation comment. Labels swapped to 'prepared for development'.");
  }

  setGitHubOutput("triage_passed", analysis.requirements_met ? "true" : "false");
}

main().catch((err) => {
  console.error("::error::Triage agent failed:", err);
  process.exit(1);
});
