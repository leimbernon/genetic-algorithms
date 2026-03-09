/**
 * Development Agent — Step 4: Reviewer
 *
 * Evaluates code quality, test coverage, documentation, architecture
 * compliance, and error handling. Returns structured feedback for the
 * Writer Agent to iterate on.
 *
 * This module is imported by writer.ts for the review loop, but can
 * also run standalone for verification.
 */

import { readFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";
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
  REPO_ROOT,
  type TriageResult,
  type ArchitecturePlan,
  type QualityReview,
  type AIClient,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Quality Thresholds
// ---------------------------------------------------------------------------

const THRESHOLDS: Record<string, number> = {
  code_quality: 7,
  test_coverage: 7,
  documentation: 7,
  architecture_compliance: 8,
  error_handling: 7,
};

// ---------------------------------------------------------------------------
// Review Function (exported for use by writer.ts)
// ---------------------------------------------------------------------------

/**
 * Review code changes and return a structured quality assessment.
 */
export async function reviewCode(
  client: AIClient,
  modelName: string,
  triageResult: TriageResult,
  plan: ArchitecturePlan,
  changedFiles: Record<string, string>,
): Promise<QualityReview> {
  const systemPrompt = loadPrompt("reviewer");

  // Build the changed files section
  const filesSection = Object.entries(changedFiles)
    .map(
      ([path, content]) =>
        `### ${path}\n\`\`\`rust\n${truncateToTokenBudget(content, 1200)}\n\`\`\``,
    )
    .join("\n\n");

  // Load reference files
  const agentInstructions = readFileSafe(
    resolve(REPO_ROOT, "AGENT_INSTRUCTIONS.md"),
  );

  const userPrompt = `## Code Review Request

### Issue
- **Issue #${triageResult.issue_number}:** ${triageResult.issue_title}
- **Type:** ${triageResult.branch_type === "feature" ? "Enhancement" : "Bug fix"}

### Architecture Plan
**Files expected:**
${plan.files_to_modify.map((f) => `- ${f.action} \`${f.path}\`: ${f.reason}`).join("\n")}

**Modules to register:**
${plan.modules_to_register.map((m) => `- \`${m.parent_module}\`: ${m.new_module}`).join("\n") || "None"}

**Enums to update:**
${plan.enums_to_update.map((e) => `- \`${e.enum_path}\`: ${e.new_variant} in ${e.enum_name}`).join("\n") || "None"}

### Changed Files
${filesSection}

### Project Standards Reference
${truncateToTokenBudget(agentInstructions, 1500)}

## Task
Evaluate the code against the five quality criteria defined in your instructions.
Respond with the JSON structure specified in your instructions.`;

  const rawResponse = await callModel(
    client,
    modelName,
    systemPrompt,
    userPrompt,
  );
  const cleaned = stripMarkdownFences(rawResponse);

  let review: QualityReview;
  try {
    review = JSON.parse(cleaned) as QualityReview;
  } catch {
    console.warn(
      "  [Reviewer] Warning: Could not parse review response, treating as needs-review.",
    );
    console.warn(`  [Reviewer] Raw response: ${rawResponse.slice(0, 500)}`);
    return {
      approved: false,
      scores: {
        code_quality: 5,
        test_coverage: 5,
        documentation: 5,
        architecture_compliance: 5,
        error_handling: 5,
      },
      issues: [],
      feedback:
        "Review agent could not parse the code properly. Manual review recommended.",
    };
  }

  // Validate scores against thresholds
  const scores = review.scores ?? {};
  let allPass = true;
  for (const [criterion, threshold] of Object.entries(THRESHOLDS)) {
    const score =
      (scores as unknown as Record<string, number>)[criterion] ?? 0;
    if (score < threshold) {
      allPass = false;
    }
  }

  // Override the model's approved flag based on actual threshold checks
  review.approved = allPass;

  return review;
}

// ---------------------------------------------------------------------------
// Standalone Main (for direct execution)
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  // REPO is validated to ensure we're in a valid environment
  getEnv("REPO");

  const client = createClient(token);
  const modelName = resolveModel("REVIEWER");

  console.log("Development Reviewer Agent (standalone)");
  console.log(`Using model: ${modelName}`);
  console.log("=".repeat(60));

  // Load artifacts
  const triageResult: TriageResult = JSON.parse(
    readFileSync("triage-result.json", "utf-8"),
  );
  const plan: ArchitecturePlan = JSON.parse(
    readFileSync("architecture-plan.json", "utf-8"),
  );

  // Collect changed files from the plan
  const changedFiles: Record<string, string> = {};
  for (const f of plan.files_to_modify) {
    if (f.action !== "DELETE") {
      const content = readFileSafe(resolve(REPO_ROOT, f.path));
      if (content) changedFiles[f.path] = content;
    }
  }

  console.log(`Reviewing ${Object.keys(changedFiles).length} files...`);

  const review = await reviewCode(
    client,
    modelName,
    triageResult,
    plan,
    changedFiles,
  );

  console.log("\nReview Results:");
  console.log(`  Approved: ${review.approved}`);
  console.log(`  Scores:`);
  for (const [k, v] of Object.entries(review.scores)) {
    const threshold = THRESHOLDS[k] ?? 0;
    const status = v >= threshold ? "PASS" : "FAIL";
    console.log(`    ${k}: ${v}/10 (threshold: ${threshold}) [${status}]`);
  }

  if (review.issues.length > 0) {
    console.log(`\n  Issues (${review.issues.length}):`);
    for (const issue of review.issues) {
      console.log(
        `    [${issue.severity}] ${issue.file}${issue.line ? `:${issue.line}` : ""}: ${issue.description}`,
      );
      console.log(`      Suggestion: ${issue.suggestion}`);
    }
  }

  if (review.feedback) {
    console.log(`\n  Feedback: ${review.feedback}`);
  }

  setGitHubOutput("reviewer_approved", review.approved ? "true" : "false");
}

// Only run main when executed directly
const isMainModule =
  process.argv[1]?.endsWith("reviewer.ts") ||
  process.argv[1]?.endsWith("reviewer.js");
if (isMainModule) {
  main().catch((err) => {
    console.error("::error::Reviewer agent failed:", err);
    process.exit(1);
  });
}
