/**
 * Development Agent — Handle PR Review
 *
 * Triggered when a pull_request_review is submitted on a PR that is
 * linked to an issue with the "in review" label.
 *
 * Reads the review comments, feeds them to the Writer agent as feedback,
 * runs validation, and pushes fixes to the PR branch.
 *
 * Outputs:
 *   - review_handled: "true" | "false"
 */

import { resolve } from "node:path";
import {
  getEnv,
  setGitHubOutput,
  createClient,
  callModel,
  resolveModel,
  loadPrompt,
  readFileSafe,
  truncateToTokenBudget,
  collectSourceFiles,
  getPullRequest,
  getPRReviews,
  getPRReviewComments,
  parseLinkedIssueNumber,
  git,
  commitChanges,
  pushBranch,
  DailyRateLimitError,
  REPO_ROOT,
  type AIClient,
} from "./shared.js";
import {
  parseFileBlocks,
  applyFileBlocks,
  runValidation,
} from "./writer.js";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const MAX_VALIDATION_FIX_ITERATIONS = 5;

// ---------------------------------------------------------------------------
// Build Review Feedback
// ---------------------------------------------------------------------------

interface ReviewFeedback {
  generalComments: string[];
  fileComments: Array<{ path: string; line: number | null; body: string }>;
}

async function collectReviewFeedback(
  repo: string,
  prNumber: number,
  token: string,
): Promise<ReviewFeedback> {
  const reviews = await getPRReviews(repo, prNumber, token);
  const comments = await getPRReviewComments(repo, prNumber, token);

  // Get the latest non-bot reviews
  const humanReviews = reviews.filter(
    (r) => !r.user.login.endsWith("[bot]") && r.state !== "approved",
  );

  const generalComments = humanReviews
    .filter((r) => r.body && r.body.trim())
    .map((r) => `[${r.state}] ${r.body!.trim()}`);

  const fileComments = comments
    .filter((c) => !c.user.login.endsWith("[bot]"))
    .map((c) => ({
      path: c.path,
      line: c.line,
      body: c.body,
    }));

  return { generalComments, fileComments };
}

function formatFeedbackForWriter(feedback: ReviewFeedback): string {
  const parts: string[] = [];

  if (feedback.generalComments.length > 0) {
    parts.push("## General Review Comments");
    for (const comment of feedback.generalComments) {
      parts.push(`- ${comment}`);
    }
  }

  if (feedback.fileComments.length > 0) {
    parts.push("\n## Inline Code Review Comments");
    for (const comment of feedback.fileComments) {
      const location = comment.line ? `${comment.path}:${comment.line}` : comment.path;
      parts.push(`- **${location}**: ${comment.body}`);
    }
  }

  return parts.join("\n");
}

// ---------------------------------------------------------------------------
// Code Fix Generation
// ---------------------------------------------------------------------------

async function generateFixes(
  client: AIClient,
  modelName: string,
  feedbackText: string,
  currentFiles: Record<string, string>,
): Promise<string> {
  const systemPrompt = loadPrompt("writer");

  const fileSection = Object.entries(currentFiles)
    .map(
      ([path, content]) =>
        `### ${path}\n\`\`\`rust\n${truncateToTokenBudget(content, 1000)}\n\`\`\``,
    )
    .join("\n\n");

  const userPrompt = `## PR Review Feedback — Address These Issues

A human reviewer has submitted feedback on the pull request. Fix ALL issues raised.

${feedbackText}

## Current Source Files

${fileSection}

## Instructions
1. Address EVERY piece of feedback from the reviewer.
2. Output COMPLETE file contents using the === FILE: path === format.
3. Only output files that need changes.
4. Do NOT break existing functionality.
5. Run through your mental checklist: fmt, clippy, tests, doc-tests, benchmarks.`;

  return callModel(client, modelName, systemPrompt, userPrompt, 0.2);
}

async function generateValidationFixes(
  client: AIClient,
  modelName: string,
  errors: string[],
  currentFiles: Record<string, string>,
): Promise<string> {
  const systemPrompt = loadPrompt("writer");

  const fileSection = Object.entries(currentFiles)
    .map(
      ([path, content]) =>
        `### ${path}\n\`\`\`rust\n${truncateToTokenBudget(content, 1000)}\n\`\`\``,
    )
    .join("\n\n");

  const userPrompt = `## Validation Errors to Fix

The following cargo checks failed. Fix ALL errors and output corrected files.

### Errors
${errors.map((e) => `\`\`\`\n${truncateToTokenBudget(e, 500)}\n\`\`\``).join("\n\n")}

### Current Files
${fileSection}

## Instructions
1. Fix ALL validation errors.
2. Output COMPLETE file contents using the === FILE: ... === format.
3. Only output files that need changes.
4. Do NOT break existing functionality.`;

  return callModel(client, modelName, systemPrompt, userPrompt, 0.2);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const repo = getEnv("REPO");
  const prNumber = parseInt(getEnv("PR_NUMBER"), 10);

  const client = createClient(token);
  const writerModel = resolveModel("WRITER");

  console.log("Development Agent — Handle PR Review");
  console.log(`Writer model: ${writerModel}`);
  console.log(`Processing PR #${prNumber} in ${repo}`);
  console.log("=".repeat(60));

  // 1. Fetch PR info
  const pr = await getPullRequest(repo, prNumber, token);
  console.log(`\nPR: ${pr.title}`);
  console.log(`Branch: ${pr.head.ref} → ${pr.base.ref}`);

  // 2. Find linked issue
  const issueNumber = parseLinkedIssueNumber(pr.body);
  if (issueNumber) {
    console.log(`Linked issue: #${issueNumber}`);
  } else {
    console.log("No linked issue found in PR body.");
  }

  // 3. Checkout the PR branch
  git("fetch origin");
  try {
    git(`checkout ${pr.head.ref}`);
    git(`pull origin ${pr.head.ref}`);
  } catch {
    git(`checkout -b ${pr.head.ref} origin/${pr.head.ref}`);
  }

  // 4. Collect review feedback
  console.log("\nCollecting review feedback...");
  const feedback = await collectReviewFeedback(repo, prNumber, token);

  const totalComments = feedback.generalComments.length + feedback.fileComments.length;
  console.log(`  General comments: ${feedback.generalComments.length}`);
  console.log(`  Inline comments: ${feedback.fileComments.length}`);

  if (totalComments === 0) {
    console.log("\nNo actionable review feedback found. Skipping.");
    setGitHubOutput("review_handled", "false");
    return;
  }

  const feedbackText = formatFeedbackForWriter(feedback);

  // 5. Collect current source files that might need changes
  const affectedPaths = new Set<string>();
  for (const comment of feedback.fileComments) {
    affectedPaths.add(comment.path);
  }

  // Also include all src/ files for context
  const allSources = collectSourceFiles("src");
  const currentFiles: Record<string, string> = { ...allSources };

  // Add test and bench files
  const testSources = collectSourceFiles("tests");
  const benchSources = collectSourceFiles("benches");
  Object.assign(currentFiles, testSources, benchSources);

  // 6. Generate fixes
  console.log("\n  [Writer] Generating fixes for review feedback...");
  let response: string;
  try {
    response = await generateFixes(client, writerModel, feedbackText, currentFiles);
  } catch (err) {
    if (err instanceof DailyRateLimitError) throw err;
    console.error(`::error::Fix generation failed: ${err}`);
    setGitHubOutput("review_handled", "false");
    return;
  }

  const blocks = parseFileBlocks(response);
  if (blocks.length === 0) {
    console.warn("::warning::Writer produced no file blocks from review feedback.");
    setGitHubOutput("review_handled", "false");
    return;
  }

  console.log(`\n  [Writer] Generated ${blocks.length} file fixes:`);
  applyFileBlocks(blocks);

  // 7. Validation loop
  for (let fixIteration = 1; fixIteration <= MAX_VALIDATION_FIX_ITERATIONS; fixIteration++) {
    console.log(`\n  --- Validation attempt ${fixIteration}/${MAX_VALIDATION_FIX_ITERATIONS} ---`);

    const validation = runValidation();

    if (validation.errors.length === 0) {
      console.log("  All cargo checks passed!");
      break;
    }

    console.log(`  ${validation.errors.length} check(s) failed.`);

    if (fixIteration < MAX_VALIDATION_FIX_ITERATIONS) {
      const filesToFix: Record<string, string> = {};
      for (const block of blocks) {
        if (block.action !== "DELETE") {
          const content = readFileSafe(resolve(REPO_ROOT, block.path));
          if (content) filesToFix[block.path] = content;
        }
      }

      try {
        const fixResponse = await generateValidationFixes(
          client,
          writerModel,
          validation.errors,
          filesToFix,
        );
        const fixBlocks = parseFileBlocks(fixResponse);
        if (fixBlocks.length > 0) {
          console.log(`  [Writer] Applying ${fixBlocks.length} validation fixes...`);
          applyFileBlocks(fixBlocks);
        }
      } catch (err) {
        if (err instanceof DailyRateLimitError) throw err;
        console.error(`::error::Validation fix failed: ${err}`);
        break;
      }
    }
  }

  // 8. Commit and push
  console.log("\n  Committing review fixes...");
  const commitMsg = `fix: address PR review feedback (#${prNumber})`;
  if (commitChanges(commitMsg)) {
    pushBranch(pr.head.ref);
    console.log("  Changes pushed to PR branch.");
  } else {
    console.log("  No changes to commit.");
  }

  setGitHubOutput("review_handled", "true");
}

main().catch((err) => {
  console.error("::error::Handle-review agent failed:", err);
  process.exit(1);
});
