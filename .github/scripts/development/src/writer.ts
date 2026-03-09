/**
 * Development Agent — Step 3: Writer
 *
 * Implements the code changes defined by the Architecture Plan.
 * Follows a test-first approach: writes tests and benchmarks first,
 * then writes the implementation, then validates with cargo tools.
 *
 * Includes a review loop with Agent 4 (Reviewer) — iterates until
 * the code passes quality checks or max iterations is reached.
 *
 * Outputs:
 *   - writer_passed: "true" | "false"
 *   - Artifact: writer-result.json
 */

import {
  readFileSync,
  writeFileSync,
  mkdirSync,
  existsSync,
  unlinkSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
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
  git,
  commitChanges,
  pushBranch,
  cargoFmt,
  cargoFmtCheck,
  cargoClippy,
  cargoTest,
  cargoTestDoc,
  cargoBenchCheck,
  getIssue,
  DailyRateLimitError,
  REPO_ROOT,
  type TriageResult,
  type ArchitecturePlan,
  type WriterResult,
  type QualityReview,
  type AIClient,
} from "./shared.js";
import { reviewCode } from "./reviewer.js";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const MAX_WRITE_REVIEW_ITERATIONS = 3;
const MAX_VALIDATION_FIX_ITERATIONS = 5;

// ---------------------------------------------------------------------------
// File Generation
// ---------------------------------------------------------------------------

export interface FileBlock {
  path: string;
  action: "CREATE" | "MODIFY" | "DELETE";
  content: string;
  description: string;
}

/**
 * Parse the AI model response to extract file blocks.
 *
 * Expected format:
 * === FILE: path/to/file.rs ===
 * ACTION: CREATE
 * DESCRIPTION: What this file does
 * ---
 * <file content>
 * ===
 */
export function parseFileBlocks(response: string): FileBlock[] {
  const blocks: FileBlock[] = [];
  const regex =
    /=== FILE:\s*(.+?)\s*===\s*\nACTION:\s*(CREATE|MODIFY|DELETE)\s*\nDESCRIPTION:\s*(.+?)\s*\n---\n([\s\S]*?)(?====\s*$|=== FILE:)/gm;

  let match;
  while ((match = regex.exec(response + "\n=== FILE:")) !== null) {
    blocks.push({
      path: match[1].trim(),
      action: match[2].trim() as "CREATE" | "MODIFY" | "DELETE",
      description: match[3].trim(),
      content: match[4].trim(),
    });
  }

  return blocks;
}

/**
 * Apply file blocks to the filesystem.
 */
export function applyFileBlocks(blocks: FileBlock[]): void {
  for (const block of blocks) {
    const absPath = resolve(REPO_ROOT, block.path);

    if (block.action === "DELETE") {
      if (existsSync(absPath)) {
        unlinkSync(absPath);
        console.log(`    DELETED: ${block.path}`);
      }
      continue;
    }

    mkdirSync(dirname(absPath), { recursive: true });
    writeFileSync(absPath, block.content + "\n", "utf-8");
    console.log(`    ${block.action}: ${block.path} (${block.description})`);
  }
}

// ---------------------------------------------------------------------------
// Code Generation
// ---------------------------------------------------------------------------

/**
 * Ask the AI to generate code based on the architecture plan.
 */
async function generateCode(
  client: AIClient,
  modelName: string,
  triageResult: TriageResult,
  plan: ArchitecturePlan,
  issueBody: string,
  existingCode: Record<string, string>,
  feedback: string,
): Promise<string> {
  const systemPrompt = loadPrompt("writer");

  // Build context of files that need modification
  const relevantPaths = plan.files_to_modify.map((f) => f.path);
  const existingCodeSection = relevantPaths
    .filter((p) => existingCode[p])
    .map((p) => `### ${p}\n\`\`\`rust\n${truncateToTokenBudget(existingCode[p], 800)}\n\`\`\``)
    .join("\n\n");

  // Also include related files for context
  const allSources = collectSourceFiles("src");
  const contextFiles = Object.entries(allSources)
    .filter(([path]) => {
      // Include files referenced by the plan
      return (
        plan.modules_to_register.some((m) => path.includes(m.parent_module)) ||
        plan.enums_to_update.some((e) => path.includes(e.enum_path)) ||
        plan.configuration_changes.some((c) => path.includes(c.file)) ||
        path === "src/lib.rs" ||
        path === "src/operations.rs" ||
        path === "src/error.rs"
      );
    })
    .map(
      ([path, content]) =>
        `### ${path}\n\`\`\`rust\n${truncateToTokenBudget(content, 600)}\n\`\`\``,
    )
    .join("\n\n");

  // Include test structures for reference
  const testStructures = readFileSafe(
    resolve(REPO_ROOT, "tests/structures.rs"),
  );

  // Read Cargo.toml for version
  const cargoToml = readFileSafe(resolve(REPO_ROOT, "Cargo.toml"));

  const promptParts: string[] = [
    `## Issue #${triageResult.issue_number}: ${triageResult.issue_title}`,
    `**Type:** ${triageResult.branch_type === "feature" ? "Enhancement" : "Bug fix"}`,
    `**Summary:** ${triageResult.summary}\n`,
    "## Issue Description",
    truncateToTokenBudget(issueBody, 1500),
    "",
    "## Architecture Plan",
    "### Files to modify:",
    ...plan.files_to_modify.map(
      (f) => `- **${f.action}** \`${f.path}\`: ${f.reason}`,
    ),
    "### Modules to register:",
    ...plan.modules_to_register.map(
      (m) =>
        `- In \`${m.parent_module}\`: add \`pub mod ${m.new_module}\` and export ${m.exports.join(", ")}`,
    ),
    "### Enums to update:",
    ...plan.enums_to_update.map(
      (e) =>
        `- In \`${e.enum_path}\`: add variant \`${e.new_variant}\` to enum \`${e.enum_name}\``,
    ),
    "### Configuration changes:",
    ...plan.configuration_changes.map((c) => `- In \`${c.file}\`: ${c.description}`),
  ];

  if (existingCodeSection) {
    promptParts.push(
      "",
      "## Existing Code (files to modify)",
      existingCodeSection,
    );
  }

  if (contextFiles) {
    promptParts.push("", "## Related Source Files (for context)", contextFiles);
  }

  promptParts.push(
    "",
    "## Test Structures (use these for tests)",
    `\`\`\`rust\n${truncateToTokenBudget(testStructures, 500)}\n\`\`\``,
  );

  promptParts.push(
    "",
    "## Cargo.toml (current version)",
    `\`\`\`toml\n${truncateToTokenBudget(cargoToml, 300)}\n\`\`\``,
  );

  if (feedback) {
    promptParts.push(
      "",
      "## Reviewer Feedback (address these issues)",
      feedback,
      "",
      "Please revise ALL files to address the feedback. Output complete file contents.",
    );
  }

  promptParts.push(
    "",
    "## Instructions",
    "1. Write tests FIRST (unit tests, integration tests, benchmarks).",
    "2. Then write the implementation code.",
    "3. Follow the output format from your system instructions (=== FILE: ... ===).",
    "4. Output COMPLETE file contents for each file (not diffs).",
    "5. Include version bump in Cargo.toml if needed.",
    "6. End with a JSON summary block.",
  );

  const userPrompt = promptParts.join("\n");

  return callModel(client, modelName, systemPrompt, userPrompt, 0.3);
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

export interface ValidationResult {
  fmt: boolean;
  clippy: boolean;
  tests: boolean;
  doc_tests: boolean;
  bench_compile: boolean;
  errors: string[];
}

/**
 * Run the full cargo validation suite.
 */
export function runValidation(): ValidationResult {
  const errors: string[] = [];

  // 1. Format
  console.log("  [Validation] Running cargo fmt...");
  cargoFmt(); // auto-fix
  const fmtCheck = cargoFmtCheck();
  if (!fmtCheck.success) {
    errors.push(`cargo fmt: ${fmtCheck.stdout}`);
  }

  // 2. Clippy
  console.log("  [Validation] Running cargo clippy...");
  const clippy = cargoClippy();
  if (!clippy.success) {
    errors.push(`cargo clippy:\n${clippy.stdout}`);
  }

  // 3. Tests
  console.log("  [Validation] Running cargo test...");
  const tests = cargoTest();
  if (!tests.success) {
    errors.push(`cargo test:\n${tests.stdout}`);
  }

  // 4. Doc tests
  console.log("  [Validation] Running cargo test --doc...");
  const docTests = cargoTestDoc();
  if (!docTests.success) {
    errors.push(`cargo test --doc:\n${docTests.stdout}`);
  }

  // 5. Bench compile check
  console.log("  [Validation] Running cargo bench --no-run...");
  const bench = cargoBenchCheck();
  if (!bench.success) {
    errors.push(`cargo bench --no-run:\n${bench.stdout}`);
  }

  return {
    fmt: fmtCheck.success,
    clippy: clippy.success,
    tests: tests.success,
    doc_tests: docTests.success,
    bench_compile: bench.success,
    errors,
  };
}

/**
 * Ask the AI to fix validation errors.
 */
async function fixValidationErrors(
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

  const client = createClient(token);
  const writerModel = resolveModel("WRITER");
  const reviewerModel = resolveModel("REVIEWER");

  console.log("Development Writer Agent");
  console.log(`Writer model: ${writerModel}`);
  console.log(`Reviewer model: ${reviewerModel}`);
  console.log("=".repeat(60));

  // Load artifacts from previous steps
  const triageResult: TriageResult = JSON.parse(
    readFileSync("triage-result.json", "utf-8"),
  );
  const plan: ArchitecturePlan = JSON.parse(
    readFileSync("architecture-plan.json", "utf-8"),
  );

  console.log(`Issue #${triageResult.issue_number}: ${triageResult.issue_title}`);
  console.log(`Branch: ${triageResult.branch_name}`);

  // Checkout the branch
  git(`checkout ${triageResult.branch_name}`);

  // Fetch the issue body
  const issue = await getIssue(repo, triageResult.issue_number, token);

  // Collect existing code that will be modified
  const existingCode: Record<string, string> = {};
  for (const f of plan.files_to_modify) {
    const content = readFileSafe(resolve(REPO_ROOT, f.path));
    if (content) existingCode[f.path] = content;
  }

  // Write-Review loop
  let reviewFeedback = "";
  let finalValidation: ValidationResult | null = null;
  let filesWritten: FileBlock[] = [];

  for (
    let iteration = 1;
    iteration <= MAX_WRITE_REVIEW_ITERATIONS;
    iteration++
  ) {
    console.log(
      `\n${"=".repeat(60)}\nWrite-Review Iteration ${iteration}/${MAX_WRITE_REVIEW_ITERATIONS}\n${"=".repeat(60)}`,
    );

    // 1. Generate code
    console.log("\n  [Writer] Generating code...");
    let response: string;
    try {
      response = await generateCode(
        client,
        writerModel,
        triageResult,
        plan,
        issue.body ?? "",
        existingCode,
        reviewFeedback,
      );
    } catch (err) {
      if (err instanceof DailyRateLimitError) throw err;
      console.error(`::error::Code generation failed: ${err}`);
      break;
    }

    // 2. Parse and apply file blocks
    const blocks = parseFileBlocks(response);
    if (blocks.length === 0) {
      console.warn(
        "::warning::Writer produced no file blocks. Attempting to extract code from response...",
      );
      // Fallback: try to extract the response as a single file
      break;
    }

    console.log(`\n  [Writer] Generated ${blocks.length} files:`);
    applyFileBlocks(blocks);
    filesWritten = blocks;

    // 3. Validation loop
    for (
      let fixIteration = 1;
      fixIteration <= MAX_VALIDATION_FIX_ITERATIONS;
      fixIteration++
    ) {
      console.log(
        `\n  --- Validation attempt ${fixIteration}/${MAX_VALIDATION_FIX_ITERATIONS} ---`,
      );

      const validation = runValidation();
      finalValidation = validation;

      if (validation.errors.length === 0) {
        console.log("  All cargo checks passed!");
        break;
      }

      console.log(
        `  ${validation.errors.length} check(s) failed. Attempting fix...`,
      );

      if (fixIteration < MAX_VALIDATION_FIX_ITERATIONS) {
        // Collect current content of files we wrote
        const currentFiles: Record<string, string> = {};
        for (const block of blocks) {
          if (block.action !== "DELETE") {
            const content = readFileSafe(resolve(REPO_ROOT, block.path));
            if (content) currentFiles[block.path] = content;
          }
        }

        try {
          const fixResponse = await fixValidationErrors(
            client,
            writerModel,
            validation.errors,
            currentFiles,
          );
          const fixBlocks = parseFileBlocks(fixResponse);
          if (fixBlocks.length > 0) {
            console.log(`  [Writer] Applying ${fixBlocks.length} fixes...`);
            applyFileBlocks(fixBlocks);
          }
        } catch (err) {
          if (err instanceof DailyRateLimitError) throw err;
          console.error(`::error::Fix attempt failed: ${err}`);
          break;
        }
      }
    }

    // 4. Quality review
    console.log("\n  [Reviewer] Reviewing code quality...");
    const changedFiles: Record<string, string> = {};
    for (const block of blocks) {
      if (block.action !== "DELETE") {
        const content = readFileSafe(resolve(REPO_ROOT, block.path));
        if (content) changedFiles[block.path] = content;
      }
    }

    let review: QualityReview;
    try {
      review = await reviewCode(
        client,
        reviewerModel,
        triageResult,
        plan,
        changedFiles,
      );
    } catch (err) {
      if (err instanceof DailyRateLimitError) throw err;
      console.error(`::error::Review failed: ${err}`);
      break;
    }

    const scores = review.scores;
    console.log(
      `  [Reviewer] Scores: quality=${scores.code_quality}, tests=${scores.test_coverage}, ` +
        `docs=${scores.documentation}, arch=${scores.architecture_compliance}, errors=${scores.error_handling}`,
    );
    console.log(`  [Reviewer] Approved: ${review.approved}`);

    if (review.approved) {
      console.log("  Code approved by reviewer!");
      break;
    }

    if (iteration < MAX_WRITE_REVIEW_ITERATIONS) {
      reviewFeedback = review.feedback;
      if (review.issues.length > 0) {
        reviewFeedback +=
          "\n\nSpecific issues:\n" +
          review.issues
            .map(
              (i) =>
                `- [${i.severity}] ${i.file}${i.line ? `:${i.line}` : ""}: ${i.description} → ${i.suggestion}`,
            )
            .join("\n");
      }
      console.log(`  [Reviewer] Feedback: ${reviewFeedback}`);
      console.log("  Retrying with reviewer feedback...");
    } else {
      console.log("  Max iterations reached. Accepting current version.");
    }
  }

  // 5. Commit and push
  console.log("\n  Committing changes...");
  const commitMsg = `${triageResult.branch_type}: ${triageResult.issue_title} (#${triageResult.issue_number})`;
  if (commitChanges(commitMsg)) {
    pushBranch(triageResult.branch_name);
  }

  // 6. Save result
  const result: WriterResult = {
    files_written: filesWritten.map((b) => ({
      path: b.path,
      action: b.action,
      description: b.description,
    })),
    tests_added: filesWritten.filter(
      (b) => b.path.startsWith("tests/") || b.path.startsWith("benches/"),
    ).length,
    benchmarks_added: filesWritten.filter((b) =>
      b.path.startsWith("benches/"),
    ).length,
    version_bump: null, // TODO: parse from response
    validation: {
      fmt: finalValidation?.fmt ?? false,
      clippy: finalValidation?.clippy ?? false,
      tests: finalValidation?.tests ?? false,
      doc_tests: finalValidation?.doc_tests ?? false,
      bench_compile: finalValidation?.bench_compile ?? false,
    },
    summary: `Implemented ${filesWritten.length} files for issue #${triageResult.issue_number}`,
  };

  writeFileSync("writer-result.json", JSON.stringify(result, null, 2), "utf-8");
  console.log("\nWriter result saved to writer-result.json");

  const allPassed = Object.values(result.validation).every(Boolean);
  setGitHubOutput("writer_passed", allPassed ? "true" : "false");
}

main().catch((err) => {
  console.error("::error::Writer agent failed:", err);
  process.exit(1);
});
