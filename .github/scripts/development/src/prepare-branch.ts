/**
 * Development Agent — Prepare Branch
 *
 * Triggered when an issue receives the "in progress" label.
 * Creates the feature/fix branch deterministically (no AI needed)
 * and produces the triage-result.json artifact for downstream agents.
 *
 * Outputs:
 *   - branch_created: "true" | "false"
 *   - branch_name: the created/existing branch name
 *   - base_branch: the base branch used
 *   - branch_type: "feature" | "fix"
 *   - Artifact: triage-result.json
 */

import { writeFileSync } from "node:fs";
import {
  getEnv,
  setGitHubOutput,
  getIssue,
  determineBranchType,
  buildBranchName,
  determineBaseBranch,
  branchExists,
  createBranch,
  pushBranch,
  type TriageResult,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const repo = getEnv("REPO");
  const issueNumber = parseInt(getEnv("ISSUE_NUMBER"), 10);

  console.log("Development Agent — Prepare Branch");
  console.log(`Processing issue #${issueNumber} in ${repo}`);
  console.log("=".repeat(60));

  // 1. Fetch the issue
  const issue = await getIssue(repo, issueNumber, token);

  if (issue.pull_request) {
    console.log("This is a pull request, not an issue. Skipping.");
    setGitHubOutput("branch_created", "false");
    return;
  }

  console.log(`\nIssue: ${issue.title}`);
  console.log(`Labels: ${issue.labels.map((l) => l.name).join(", ") || "None"}`);
  console.log(`Milestone: ${issue.milestone?.title ?? "None"}`);

  // 2. Determine branch info (deterministic, no AI)
  const branchType = determineBranchType(issue.labels);
  const baseBranch = determineBaseBranch(issue.milestone);
  const branchName = buildBranchName(issueNumber, issue.title, branchType);

  console.log(`\nBranch type: ${branchType}`);
  console.log(`Base branch: ${baseBranch}`);
  console.log(`Branch name: ${branchName}`);

  // 3. Create the branch if it doesn't exist
  let actualBaseBranch = baseBranch;
  if (branchExists(branchName)) {
    console.log(`\nBranch '${branchName}' already exists. Skipping creation.`);
  } else {
    console.log(`\nCreating branch '${branchName}' from '${baseBranch}'...`);
    actualBaseBranch = createBranch(branchName, baseBranch);
    pushBranch(branchName);
    if (actualBaseBranch !== baseBranch) {
      console.log(`  Note: fell back from '${baseBranch}' to '${actualBaseBranch}'.`);
    }
  }

  // 4. Build triage result for downstream agents
  const triageResult: TriageResult = {
    issue_number: issueNumber,
    issue_title: issue.title,
    branch_name: branchName,
    base_branch: actualBaseBranch,
    branch_type: branchType,
    requirements_met: true, // By definition — "in progress" means analysis passed
    questions: [],
    summary: `Development started for issue #${issueNumber}: ${issue.title}`,
  };

  writeFileSync("triage-result.json", JSON.stringify(triageResult, null, 2), "utf-8");
  console.log("\nTriage result saved to triage-result.json");

  setGitHubOutput("branch_created", "true");
  setGitHubOutput("branch_name", branchName);
  setGitHubOutput("base_branch", actualBaseBranch);
  setGitHubOutput("branch_type", branchType);
}

main().catch((err) => {
  console.error("::error::Prepare-branch failed:", err);
  process.exit(1);
});
