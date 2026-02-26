/**
 * Documentation Agent — Step 0: Guard
 *
 * Validates that the /docs directory structure matches the rules defined in
 * DOCUMENTATION_STRUCTURE.md. Performs a fast programmatic check first (file
 * presence), then uses the AI model to evaluate template compliance of
 * existing files.
 *
 * Outputs:
 *   - structure_valid: "true" | "false"
 *   - needs_initialization: "true" | "false"
 */

import { existsSync } from "node:fs";
import {
  getEnv,
  setGitHubOutput,
  createClient,
  callModel,
  resolveModel,
  collectExistingDocs,
  loadPrompt,
  loadStructureDefinition,
  stripMarkdownFences,
  REQUIRED_DOC_FILES,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface NonCompliantFile {
  path: string;
  issues: string[];
}

interface GuardResult {
  structure_valid: boolean;
  needs_initialization: boolean;
  missing_files: string[];
  unexpected_files: string[];
  non_compliant_files: NonCompliantFile[];
  summary: string;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const apiKey = getEnv("MODELS_API_KEY");
  const client = createClient(apiKey);
  const modelName = resolveModel("GUARD");

  console.log("Documentation Guard Agent");
  console.log(`Using model: ${modelName}`);
  console.log("=".repeat(60));

  // 1. Programmatic check: which required files are missing?
  const missingFiles: string[] = [];
  for (const requiredFile of REQUIRED_DOC_FILES) {
    if (!existsSync(requiredFile)) {
      missingFiles.push(requiredFile);
    }
  }

  console.log(`\nRequired files: ${REQUIRED_DOC_FILES.length}`);
  console.log(`Missing files:  ${missingFiles.length}`);

  // 2. Collect existing docs
  const existingDocs = collectExistingDocs("docs");
  const existingPaths = Object.keys(existingDocs);

  // 3. Find unexpected files (in docs/ but not in required list)
  const requiredSet = new Set(REQUIRED_DOC_FILES);
  const unexpectedFiles = existingPaths.filter((p) => !requiredSet.has(p));

  if (unexpectedFiles.length > 0) {
    console.log(`Unexpected files: ${unexpectedFiles.join(", ")}`);
  }

  // 4. Fast path: if no docs exist at all, skip AI call
  if (existingPaths.length === 0) {
    console.log("\nNo documentation files found. Initialization required.");
    setGitHubOutput("structure_valid", "false");
    setGitHubOutput("needs_initialization", "true");
    return;
  }

  // 5. Fast path: if all required files exist and no unexpected files,
  //    still use AI to check template compliance
  //    If many files are missing, skip AI and go straight to initialization.
  if (missingFiles.length > REQUIRED_DOC_FILES.length / 2) {
    console.log(
      `\nMore than half of required files are missing (${missingFiles.length}/${REQUIRED_DOC_FILES.length}). Initialization required.`,
    );
    setGitHubOutput("structure_valid", "false");
    setGitHubOutput("needs_initialization", "true");
    return;
  }

  // 6. Use AI model to validate template compliance of existing files
  console.log("\nCalling AI model to validate template compliance...");

  const structureDefinition = loadStructureDefinition();
  const systemPrompt = loadPrompt("guard");

  let docsListing: string;
  if (existingPaths.length > 0) {
    const parts = Object.entries(existingDocs).map(([path, content]) => {
      const preview =
        content.length > 3000
          ? content.slice(0, 3000) + "\n... (truncated)"
          : content;
      return `### ${path}\n\`\`\`markdown\n${preview}\n\`\`\``;
    });
    docsListing = parts.join("\n\n");
  } else {
    docsListing = "No documentation files found.";
  }

  const userPrompt = `## Structure Definition

\`\`\`markdown
${structureDefinition}
\`\`\`

## Existing Documentation in /docs
${docsListing}

## Missing Files
${missingFiles.length > 0 ? missingFiles.map((f) => `- ${f}`).join("\n") : "None — all required files are present."}

## Unexpected Files
${unexpectedFiles.length > 0 ? unexpectedFiles.map((f) => `- ${f}`).join("\n") : "None."}

## Task
Validate the documentation structure and template compliance.
Respond with the JSON structure as defined in your instructions.`;

  const rawResponse = await callModel(client, modelName, systemPrompt, userPrompt);
  const cleaned = stripMarkdownFences(rawResponse);

  let result: GuardResult;
  try {
    result = JSON.parse(cleaned) as GuardResult;
  } catch (err) {
    console.error(`::error::Failed to parse guard response as JSON: ${err}`);
    console.error(`Raw response:\n${rawResponse}`);
    // Conservative fallback: assume initialization is needed
    setGitHubOutput("structure_valid", "false");
    setGitHubOutput("needs_initialization", "true");
    return;
  }

  // 7. Report results
  console.log(`\nGuard Assessment:`);
  console.log(`  Structure valid:      ${result.structure_valid}`);
  console.log(`  Needs initialization: ${result.needs_initialization}`);
  console.log(`  Missing files:        ${result.missing_files?.length ?? 0}`);
  console.log(`  Unexpected files:     ${result.unexpected_files?.length ?? 0}`);
  console.log(`  Non-compliant files:  ${result.non_compliant_files?.length ?? 0}`);
  console.log(`  Summary: ${result.summary}`);

  if (result.non_compliant_files?.length > 0) {
    console.log("\n  Non-compliant files:");
    for (const file of result.non_compliant_files) {
      console.log(`    - ${file.path}:`);
      for (const issue of file.issues) {
        console.log(`        * ${issue}`);
      }
    }
  }

  setGitHubOutput(
    "structure_valid",
    result.structure_valid ? "true" : "false",
  );
  setGitHubOutput(
    "needs_initialization",
    result.needs_initialization ? "true" : "false",
  );
}

main().catch((err) => {
  console.error("::error::Guard agent failed:", err);
  process.exit(1);
});
