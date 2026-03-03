/**
 * Development Agent — Handle PR Merge
 *
 * Triggered when a pull request is closed and merged.
 * Finds the linked issue, swaps "in review" → "done", and closes the issue.
 *
 * Outputs:
 *   - merge_handled: "true" | "false"
 */

import {
  getEnv,
  setGitHubOutput,
  getIssue,
  getPullRequest,
  parseLinkedIssueNumber,
  swapLabels,
  closeIssue,
  LABELS,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const repo = getEnv("REPO");
  const prNumber = parseInt(getEnv("PR_NUMBER"), 10);

  console.log("Development Agent — Handle PR Merge");
  console.log(`Processing PR #${prNumber} in ${repo}`);
  console.log("=".repeat(60));

  // 1. Fetch PR info
  const pr = await getPullRequest(repo, prNumber, token);
  console.log(`\nPR: ${pr.title}`);
  console.log(`Merged: ${pr.merged}`);
  console.log(`State: ${pr.state}`);

  if (!pr.merged) {
    console.log("PR was closed without merging. No action needed.");
    setGitHubOutput("merge_handled", "false");
    return;
  }

  // 2. Find linked issue number from PR body
  const issueNumber = parseLinkedIssueNumber(pr.body);

  if (!issueNumber) {
    console.log("No linked issue found in PR body. Cannot update labels.");
    setGitHubOutput("merge_handled", "false");
    return;
  }

  console.log(`Linked issue: #${issueNumber}`);

  // 3. Verify the issue exists and has the "in review" label
  const issue = await getIssue(repo, issueNumber, token);
  const hasInReview = issue.labels.some((l) => l.name === LABELS.IN_REVIEW);

  if (!hasInReview) {
    console.log(`Issue #${issueNumber} does not have "${LABELS.IN_REVIEW}" label. Skipping label swap.`);
    // Still close the issue if it's open
    if (issue.state === "open") {
      await closeIssue(repo, issueNumber, token);
    }
    setGitHubOutput("merge_handled", "true");
    return;
  }

  // 4. Swap labels: "in review" → "done"
  await swapLabels(repo, issueNumber, LABELS.IN_REVIEW, LABELS.DONE, token);

  // 5. Close the issue
  if (issue.state === "open") {
    await closeIssue(repo, issueNumber, token);
  }

  console.log(`\nIssue #${issueNumber} marked as "${LABELS.DONE}" and closed.`);
  setGitHubOutput("merge_handled", "true");
}

main().catch((err) => {
  console.error("::error::Handle-merge failed:", err);
  process.exit(1);
});
