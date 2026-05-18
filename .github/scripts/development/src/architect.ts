/**
 * Development Agent — Step 2: Architect
 *
 * Analyzes the issue requirements and the current codebase to determine
 * which files need to be created, modified, or deleted. Produces a detailed
 * architecture plan that the Writer Agent will follow.
 *
 * Outputs:
 *   - architect_passed: "true" | "false"
 *   - Artifact: architecture-plan.json
 */

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
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
  collectSourceFiles,
  listProjectFiles,
  getIssue,
  git,
  REPO_ROOT,
  type TriageResult,
  type ArchitecturePlan,
  type AIClient,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Source code extraction (public API only)
// ---------------------------------------------------------------------------

/**
 * Extract public signatures from Rust source for context.
 * Keeps: pub items, doc-comments, trait/impl headers, mod/use declarations.
 */
function extractPublicAPI(source: string): string {
  const lines = source.split("\n");
  const output: string[] = [];

  for (const line of lines) {
    const trimmed = line.trimStart();
    if (
      trimmed.startsWith("///") ||
      trimmed.startsWith("//!") ||
      trimmed.startsWith("pub ") ||
      trimmed.startsWith("pub(") ||
      trimmed.startsWith("impl ") ||
      trimmed.startsWith("impl<") ||
      trimmed.startsWith("#[") ||
      trimmed.startsWith("}") ||
      trimmed === ""
    ) {
      output.push(line);
    }
  }

  return output
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/**
 * Build a summarized source context for the AI model.
 */
function buildSourceContext(tokenBudget: number): string {
  const sources = collectSourceFiles("src");
  const parts: string[] = [];
  let totalTokens = 0;

  // Sort by relevance: operations, traits, chromosomes, genotypes first
  const priorityOrder = [
    "src/operations.rs",
    "src/traits/",
    "src/lib.rs",
    "src/configuration.rs",
    "src/ga.rs",
    "src/error.rs",
  ];

  const sortedEntries = Object.entries(sources).sort(([a], [b]) => {
    const aPriority = priorityOrder.findIndex((p) => a.startsWith(p));
    const bPriority = priorityOrder.findIndex((p) => b.startsWith(p));
    const aIdx = aPriority === -1 ? 999 : aPriority;
    const bIdx = bPriority === -1 ? 999 : bPriority;
    return aIdx - bIdx;
  });

  for (const [filepath, content] of sortedEntries) {
    const api = extractPublicAPI(content);
    const block = `### ${filepath}\n\`\`\`rust\n${api}\n\`\`\`\n`;
    const tokens = Math.ceil(block.length / 4);

    if (totalTokens + tokens > tokenBudget) {
      parts.push(`\n... (${sortedEntries.length - parts.length} more files omitted for token budget)`);
      break;
    }

    parts.push(block);
    totalTokens += tokens;
  }

  return parts.join("\n");
}

// ---------------------------------------------------------------------------
// Architecture Analysis
// ---------------------------------------------------------------------------

async function analyzeArchitecture(
  client: AIClient,
  modelName: string,
  triageResult: TriageResult,
  issueBody: string,
): Promise<ArchitecturePlan> {
  const systemPrompt = loadPrompt("architect");
  const agentInstructions = readFileSafe(
    join(REPO_ROOT, "AGENTS.md"),
  );

  // Get the project file tree
  const projectFiles = listProjectFiles(".");
  const fileTree = projectFiles
    .filter(
      (f) =>
        !f.startsWith("target/") &&
        !f.startsWith("node_modules/") &&
        !f.startsWith(".git/"),
    )
    .join("\n");

  // Build source context
  const sourceContext = buildSourceContext(4000);

  const userPrompt = `## Issue Information
- **Issue #${triageResult.issue_number}:** ${triageResult.issue_title}
- **Type:** ${triageResult.branch_type === "feature" ? "Enhancement" : "Bug fix"}
- **Branch:** ${triageResult.branch_name} (from ${triageResult.base_branch})
- **Summary:** ${triageResult.summary}

## Issue Description
${truncateToTokenBudget(issueBody, 2000)}

## Current Project File Tree
\`\`\`
${truncateToTokenBudget(fileTree, 1000)}
\`\`\`

## Current Source Code (Public API)
${sourceContext}

## Project Rules
${truncateToTokenBudget(agentInstructions, 1500)}

## Task
Analyze the issue and produce an architecture plan that specifies:
1. Which files need to be CREATED, MODIFIED, or DELETED.
2. Which parent modules need updating (pub mod + pub use).
3. Which enums need new variants.
4. Which configuration structs need changes.

Respond with the JSON structure defined in your instructions.`;

  const rawResponse = await callModel(client, modelName, systemPrompt, userPrompt);
  const cleaned = stripMarkdownFences(rawResponse);

  try {
    const plan = JSON.parse(cleaned) as ArchitecturePlan;
    plan.issue_number = triageResult.issue_number;
    return plan;
  } catch (err) {
    console.error(`::error::Failed to parse architecture plan: ${err}`);
    console.error(`Raw response (first 500 chars):\n${rawResponse.slice(0, 500)}`);
    // Return a minimal fallback plan so the pipeline can report failure gracefully
    return {
      issue_number: triageResult.issue_number,
      files_to_modify: [],
      modules_to_register: [],
      enums_to_update: [],
      configuration_changes: [],
      summary: `Architecture analysis failed: could not parse AI response. Error: ${err}`,
    };
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const repo = getEnv("REPO");

  const client = createClient(token);
  const modelName = resolveModel("ARCHITECT");

  console.log("Development Architect Agent");
  console.log(`Using model: ${modelName}`);
  console.log("=".repeat(60));

  // Load triage result
  const triagePath = "triage-result.json";
  if (!existsSync(triagePath)) {
    console.error("::error::triage-result.json not found.");
    process.exit(1);
  }
  const triageResult: TriageResult = JSON.parse(
    readFileSync(triagePath, "utf-8"),
  );

  console.log(`Issue #${triageResult.issue_number}: ${triageResult.issue_title}`);
  console.log(`Branch: ${triageResult.branch_name}`);

  // Checkout the branch
  try {
    git(`checkout ${triageResult.branch_name}`);
  } catch {
    console.log(`Branch ${triageResult.branch_name} not available locally, fetching...`);
    git("fetch origin");
    git(`checkout ${triageResult.branch_name}`);
  }

  // Fetch the issue body for full context
  const issue = await getIssue(repo, triageResult.issue_number, token);

  // Run architecture analysis
  console.log("\nAnalyzing architecture requirements...");
  const plan = await analyzeArchitecture(
    client,
    modelName,
    triageResult,
    issue.body ?? "",
  );

  // Report results
  console.log("\nArchitecture Plan:");
  console.log(`  Files to modify: ${plan.files_to_modify.length}`);
  for (const f of plan.files_to_modify) {
    console.log(`    ${f.action} ${f.path}: ${f.reason}`);
  }
  console.log(`  Modules to register: ${plan.modules_to_register.length}`);
  console.log(`  Enums to update: ${plan.enums_to_update.length}`);
  console.log(`  Configuration changes: ${plan.configuration_changes.length}`);
  console.log(`  Summary: ${plan.summary}`);

  // Save the plan
  writeFileSync(
    "architecture-plan.json",
    JSON.stringify(plan, null, 2),
    "utf-8",
  );
  console.log("\nArchitecture plan saved to architecture-plan.json");

  setGitHubOutput("architect_passed", "true");
}

main().catch((err) => {
  console.error("::error::Architect agent failed:", err);
  process.exit(1);
});
