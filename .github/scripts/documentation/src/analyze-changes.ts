/**
 * Documentation Agent — Step 1: Analyze Changes
 *
 * Analyzes the diff of a merged pull request and compares it against the
 * existing documentation in /docs to determine what documentation needs
 * to be created, updated, or deleted.
 *
 * Output: doc-plan.json with a structured list of documentation actions.
 */

import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  getEnv,
  setGitHubOutput,
  createClient,
  callModel,
  resolveModel,
  getPRChangedFiles,
  collectExistingDocs,
  readFileSafe,
  loadPrompt,
  stripMarkdownFences,
  ANALYZABLE_PREFIXES,
  REPO_ROOT,
  type DocPlan,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  // Read environment
  const token = getEnv("GITHUB_TOKEN");
  const prNumber = getEnv("PR_NUMBER");
  const prTitle = getEnv("PR_TITLE");
  const prBody = getEnv("PR_BODY", false);
  const repo = getEnv("REPO");

  const client = createClient(token);
  const modelName = resolveModel("ANALYST");

  // 1. Fetch changed files from the PR
  console.log(`Fetching changed files for PR #${prNumber} in ${repo}...`);
  console.log(`Using model: ${modelName}`);
  const changedFiles = await getPRChangedFiles(repo, prNumber, token);

  // Filter to only analyzable paths
  const relevantFiles = changedFiles.filter((f) =>
    ANALYZABLE_PREFIXES.some((prefix) => f.filename.startsWith(prefix)),
  );

  if (relevantFiles.length === 0) {
    console.log(
      "No relevant source files changed. Skipping documentation analysis.",
    );
    setGitHubOutput("has_changes", "false");
    return;
  }

  console.log(`Found ${relevantFiles.length} relevant changed files.`);

  // 2. Build a summary of changes
  const changesSummary = relevantFiles.map((f) => {
    const fileInfo: Record<string, unknown> = {
      filename: f.filename,
      status: f.status,
      additions: f.additions ?? 0,
      deletions: f.deletions ?? 0,
    };

    // Include the patch (diff) if available and not too large
    const patch = f.patch ?? "";
    fileInfo.patch =
      patch.length < 8000 ? patch : patch.slice(0, 8000) + "\n... (truncated)";

    // Read current file content for added/modified files
    if (f.status === "added" || f.status === "modified") {
      const content = readFileSafe(resolve(REPO_ROOT, f.filename));
      fileInfo.current_content =
        content.length < 12000
          ? content
          : content.slice(0, 12000) + "\n... (truncated)";
    }

    return fileInfo;
  });

  // 3. Collect existing documentation
  const existingDocs = collectExistingDocs("docs");

  let docsListing = "No documentation files found in /docs yet.";
  if (Object.keys(existingDocs).length > 0) {
    const parts = Object.entries(existingDocs).map(([path, content]) => {
      const preview =
        content.length > 2000
          ? content.slice(0, 2000) + "\n... (truncated)"
          : content;
      return `### ${path}\n\`\`\`markdown\n${preview}\n\`\`\``;
    });
    docsListing = parts.join("\n\n");
  }

  // 4. Build the prompt
  const systemPrompt = loadPrompt("analyst");

  const userPrompt = `## Pull Request Information
- **PR Number:** #${prNumber}
- **Title:** ${prTitle}
- **Description:** ${prBody || "No description provided."}

## Changed Files
\`\`\`json
${JSON.stringify(changesSummary, null, 2)}
\`\`\`

## Existing Documentation in /docs
${docsListing}

## Task
Analyze the code changes and produce a JSON plan with documentation actions.
Each action must specify:
- "action": one of "CREATE", "UPDATE", or "DELETE"
- "path": the documentation file path (e.g. "docs/genotypes/range.md")
- "reason": why this action is needed
- "relevant_source_files": list of source file paths related to this doc
- "key_points": list of key points to cover in the documentation

Respond with this exact JSON structure:
{
  "pr_number": ${prNumber},
  "pr_title": "${prTitle}",
  "actions": [
    {
      "action": "CREATE | UPDATE | DELETE",
      "path": "docs/...",
      "reason": "...",
      "relevant_source_files": ["src/..."],
      "key_points": ["point 1", "point 2"]
    }
  ]
}`;

  // 5. Call the model
  console.log("Calling AI model to analyze changes...");
  const rawResponse = await callModel(client, modelName, systemPrompt, userPrompt);
  const cleaned = stripMarkdownFences(rawResponse);

  // 6. Parse and validate
  let plan: DocPlan;
  try {
    plan = JSON.parse(cleaned) as DocPlan;
  } catch (err) {
    console.error(`::error::Failed to parse model response as JSON: ${err}`);
    console.error(`Raw response:\n${rawResponse}`);
    process.exit(1);
  }

  // Ensure required fields
  if (!plan.actions) plan.actions = [];
  plan.pr_number ??= Number(prNumber);
  plan.pr_title ??= prTitle;

  if (plan.actions.length === 0) {
    console.log(
      "Analyst agent determined no documentation changes are needed.",
    );
    setGitHubOutput("has_changes", "false");
    return;
  }

  // 7. Save the plan
  writeFileSync("doc-plan.json", JSON.stringify(plan, null, 2), "utf-8");

  console.log("Documentation plan saved to doc-plan.json");
  console.log(`  Actions: ${plan.actions.length}`);
  for (const action of plan.actions) {
    console.log(`    - ${action.action} ${action.path}: ${action.reason}`);
  }

  setGitHubOutput("has_changes", "true");
}

main().catch((err) => {
  console.error("::error::Analyst agent failed:", err);
  process.exit(1);
});
