/**
 * Documentation Agent — Step 0b: Initialize Documentation
 *
 * Generates or adapts all required documentation files to conform to the
 * structure defined in DOCUMENTATION_STRUCTURE.md. Called when the guard
 * agent determines that docs need initialization.
 *
 * For each required file, it:
 *   1. Identifies the relevant source files
 *   2. Generates documentation using the AI model
 *   3. Validates quality via the reviewer agent
 *   4. Writes the final file to disk
 */

import {
  readFileSync,
  writeFileSync,
  mkdirSync,
  existsSync,
} from "node:fs";
import { dirname, resolve } from "node:path";
import {
  getEnv,
  createClient,
  callModel,
  resolveModel,
  readFileSafe,
  loadPrompt,
  loadStructureDefinition,
  collectSourceFiles,
  stripMarkdownFences,
  REQUIRED_DOC_FILES,
  REPO_ROOT,
  type ReviewResult,
} from "./shared.js";
import { reviewDocument } from "./review-docs.js";
import type OpenAI from "openai";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const MAX_REVIEW_ITERATIONS = 2;

/**
 * Mapping from documentation file paths to the source file glob patterns
 * that are relevant for generating that documentation.
 */
const DOC_SOURCE_MAP: Record<string, string[]> = {
  "docs/getting-started.md": [
    "src/lib.rs",
    "src/ga.rs",
    "src/configuration.rs",
    "examples/",
    "Cargo.toml",
  ],
  "docs/configuration.md": [
    "src/configuration.rs",
    "src/traits/configuration.rs",
    "src/ga.rs",
  ],
  "docs/chromosomes.md": [
    "src/chromosomes.rs",
    "src/chromosomes/binary.rs",
    "src/chromosomes/range.rs",
  ],
  "docs/genotypes.md": [
    "src/genotypes.rs",
    "src/genotypes/binary.rs",
    "src/genotypes/range.rs",
  ],
  "docs/operators/selection.md": [
    "src/operations/selection.rs",
    "src/operations/selection/tournament.rs",
    "src/operations/selection/fitness_proportionate.rs",
    "src/operations/selection/random.rs",
  ],
  "docs/operators/crossover.md": [
    "src/operations/crossover.rs",
    "src/operations/crossover/uniform_crossover.rs",
    "src/operations/crossover/multipoint.rs",
    "src/operations/crossover/cycle.rs",
  ],
  "docs/operators/mutation.md": [
    "src/operations/mutation.rs",
    "src/operations/mutation/swap.rs",
    "src/operations/mutation/scramble.rs",
    "src/operations/mutation/inversion.rs",
    "src/operations/mutation/value.rs",
  ],
  "docs/operators/survivor.md": [
    "src/operations/survivor.rs",
    "src/operations/survivor/age.rs",
    "src/operations/survivor/fitness.rs",
  ],
  "docs/fitness.md": [
    "src/fitness.rs",
    "src/fitness/count_true.rs",
    "src/fitness/fitness_fn_wrapper.rs",
  ],
  "docs/population.md": [
    "src/population.rs",
  ],
  "docs/traits.md": [
    "src/traits.rs",
    "src/traits/gene.rs",
    "src/traits/chromosome.rs",
    "src/traits/configuration.rs",
  ],
  "docs/validators.md": [
    "src/validators.rs",
    "src/validators/generic_validator.rs",
    "src/validators/validator_factory.rs",
  ],
  "docs/examples.md": [
    "examples/",
    "src/ga.rs",
  ],
  "docs/api-reference.md": [
    "src/lib.rs",
  ],
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Collect the source file contents relevant to a given doc file.
 * Handles both exact file paths and directory prefixes (e.g., "examples/").
 */
function getRelevantSources(
  docPath: string,
  allSources: Record<string, string>,
): Record<string, string> {
  const patterns = DOC_SOURCE_MAP[docPath] ?? [];
  const result: Record<string, string> = {};

  for (const pattern of patterns) {
    if (pattern.endsWith("/")) {
      // Directory prefix match
      for (const [srcPath, content] of Object.entries(allSources)) {
        if (srcPath.startsWith(pattern)) {
          result[srcPath] = content;
        }
      }
    } else {
      // Exact file match
      if (allSources[pattern]) {
        result[pattern] = allSources[pattern];
      }
    }
  }

  return result;
}

/**
 * Generate a single documentation file using the initializer agent.
 */
async function generateDocFile(
  client: OpenAI,
  initializerModel: string,
  reviewerModel: string,
  docPath: string,
  sourceContents: Record<string, string>,
  structureDefinition: string,
  existingContent: string,
): Promise<string> {
  const systemPrompt = loadPrompt("initializer");

  // Build source code section
  const sourceParts = Object.entries(sourceContents).map(
    ([filepath, content]) => {
      const truncated =
        content.length > 15000
          ? content.slice(0, 15000) + "\n... (truncated)"
          : content;
      return `### ${filepath}\n\`\`\`rust\n${truncated}\n\`\`\``;
    },
  );

  const sourceSection =
    sourceParts.length > 0
      ? sourceParts.join("\n\n")
      : "No source files available.";

  let feedback = "";
  let finalContent = "";

  for (let iteration = 1; iteration <= MAX_REVIEW_ITERATIONS; iteration++) {
    console.log(`  --- Iteration ${iteration}/${MAX_REVIEW_ITERATIONS} ---`);

    // Build user prompt
    const promptParts: string[] = [
      `## Target File: ${docPath}`,
      "",
      "## Structure Definition",
      `\`\`\`markdown\n${structureDefinition}\n\`\`\``,
      "",
      "## Relevant Source Code",
      sourceSection,
    ];

    if (existingContent) {
      promptParts.push(
        "",
        "## Existing Content (to be adapted)",
        `\`\`\`markdown\n${existingContent}\n\`\`\``,
      );
    }

    if (feedback) {
      promptParts.push(
        "",
        "## Reviewer Feedback (address these issues)",
        feedback,
        "",
        "Please revise the documentation to address all the feedback above.",
      );
    }

    const userPrompt = promptParts.join("\n");

    // Generate
    console.log("  [Initializer] Generating documentation...");
    let content = await callModel(
      client,
      initializerModel,
      systemPrompt,
      userPrompt,
      0.3,
    );
    content = stripMarkdownFences(content);
    finalContent = content;

    // Write to disk
    const absDocPath = resolve(REPO_ROOT, docPath);
    mkdirSync(dirname(absDocPath), { recursive: true });
    writeFileSync(absDocPath, content + "\n", "utf-8");
    console.log(`  [Initializer] Wrote ${content.length} chars to ${docPath}`);

    // Review
    console.log("  [Reviewer] Evaluating documentation quality...");
    const review: ReviewResult = await reviewDocument(
      client,
      reviewerModel,
      content,
      {
        action: existingContent ? "UPDATE" : "CREATE",
        path: docPath,
        reason: "Documentation initialization",
        relevant_source_files: Object.keys(sourceContents),
        key_points: ["Complete documentation following DOCUMENTATION_STRUCTURE.md template"],
      },
      sourceContents,
    );

    const scores = review.scores ?? {};
    console.log(
      `  [Reviewer] Scores: completeness=${scores.completeness ?? "?"}, ` +
        `clarity=${scores.clarity ?? "?"}, ` +
        `examples=${scores.examples ?? "?"}`,
    );
    console.log(`  [Reviewer] Approved: ${review.approved}`);

    if (review.approved) {
      console.log("  Documentation approved!");
      break;
    }

    feedback = review.feedback ?? "";
    if (iteration < MAX_REVIEW_ITERATIONS) {
      console.log(`  [Reviewer] Feedback: ${feedback}`);
      console.log("  Retrying with reviewer feedback...");
    } else {
      console.log("  Max iterations reached. Accepting current version.");
    }
  }

  return finalContent;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const token = getEnv("GITHUB_TOKEN");
  const client = createClient(token);
  const initializerModel = resolveModel("INITIALIZER");
  const reviewerModel = resolveModel("REVIEWER");

  console.log("Documentation Initializer Agent");
  console.log(`Initializer model: ${initializerModel}`);
  console.log(`Reviewer model:    ${reviewerModel}`);
  console.log("=".repeat(60));

  // 1. Collect all source files
  console.log("\nCollecting source files...");
  const allSources = collectSourceFiles("src");

  // Also collect examples
  const exampleSources = collectSourceFiles("examples");
  Object.assign(allSources, exampleSources);

  // Read Cargo.toml as well
  const cargoToml = readFileSafe(resolve(REPO_ROOT, "Cargo.toml"));
  if (cargoToml) {
    allSources["Cargo.toml"] = cargoToml;
  }

  console.log(`Found ${Object.keys(allSources).length} source files.`);

  // 2. Load structure definition
  const structureDefinition = loadStructureDefinition();
  if (!structureDefinition) {
    console.error("::error::DOCUMENTATION_STRUCTURE.md not found.");
    process.exit(1);
  }

  // 3. Process each required doc file — only generate files that don't exist.
  //    The Writer agent handles updates to existing docs; the Initializer
  //    is responsible for bootstrapping missing files from scratch.
  let created = 0;
  let skipped = 0;

  for (let i = 0; i < REQUIRED_DOC_FILES.length; i++) {
    const docPath = REQUIRED_DOC_FILES[i];
    console.log(`\n${"=".repeat(60)}`);
    console.log(`[${i + 1}/${REQUIRED_DOC_FILES.length}] ${docPath}`);
    console.log("=".repeat(60));

    // Skip files that already exist — the Writer agent handles updates
    const absPath = resolve(REPO_ROOT, docPath);
    if (existsSync(absPath)) {
      const existing = readFileSync(absPath, "utf-8").trim();
      if (existing.length > 0) {
        console.log(`  File already exists (${existing.length} chars). Skipping — Writer handles updates.`);
        skipped++;
        continue;
      }
    }

    // Get relevant source files for this doc (may be empty for some docs)
    const sources = getRelevantSources(docPath, allSources);
    if (Object.keys(sources).length > 0) {
      console.log(`  Relevant sources: ${Object.keys(sources).join(", ")}`);
    } else {
      console.log("  No specific source files mapped. Will generate from general project context.");
    }

    console.log("  File does not exist. Creating from scratch.");
    created++;

    // Generate the documentation and validate with reviewer
    await generateDocFile(
      client,
      initializerModel,
      reviewerModel,
      docPath,
      sources,
      structureDefinition,
      "", // No existing content — always creating from scratch
    );
  }

  console.log(`\n${"=".repeat(60)}`);
  console.log("Documentation Initialization Complete");
  console.log(`  Created:  ${created}`);
  console.log(`  Skipped (already exist): ${skipped}`);
  console.log("=".repeat(60));
}

main().catch((err) => {
  console.error("::error::Initializer agent failed:", err);
  process.exit(1);
});
