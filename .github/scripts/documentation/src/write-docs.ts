/**
 * Documentation Agent — Step 2: Write Documentation
 *
 * Reads the documentation plan (doc-plan.json) produced by the analyst agent,
 * generates or updates markdown files in /docs, and uses the reviewer agent
 * to validate quality. If the reviewer rejects, the writer retries with
 * feedback (up to MAX_REVIEW_ITERATIONS per document).
 */

import { readFileSync, writeFileSync, unlinkSync, mkdirSync, existsSync } from "node:fs";
import { dirname } from "node:path";
import {
  getEnv,
  createClient,
  callModel,
  resolveModel,
  readFileSafe,
  loadPrompt,
  stripMarkdownFences,
  type DocPlan,
  type DocAction,
} from "./shared.js";
import { reviewDocument } from "./review-docs.js";
import type OpenAI from "openai";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const MAX_REVIEW_ITERATIONS = 2;

// ---------------------------------------------------------------------------
// Writer Agent
// ---------------------------------------------------------------------------

/**
 * Generate or update a documentation file using the AI model.
 */
async function generateDocument(
  client: OpenAI,
  modelName: string,
  action: DocAction,
  sourceContents: Record<string, string>,
  existingContent: string,
  feedback: string,
): Promise<string> {
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

  const systemPrompt = loadPrompt("writer");

  // Build the user prompt
  const promptParts: string[] = [
    `## Task: ${action.action} documentation file`,
    `**Target file:** ${action.path}`,
    `**Reason:** ${action.reason}`,
    `**Key points to cover:** ${JSON.stringify(action.key_points ?? [])}`,
    "",
    "## Relevant Source Code",
    sourceSection,
  ];

  if (existingContent && action.action === "UPDATE") {
    promptParts.push(
      "",
      "## Current Documentation (to be updated)",
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

  return callModel(client, modelName, systemPrompt, userPrompt, 0.3);
}

// ---------------------------------------------------------------------------
// Action processing
// ---------------------------------------------------------------------------

/**
 * Process a single documentation action: write the document, review it,
 * and retry if needed (up to MAX_REVIEW_ITERATIONS).
 */
async function processAction(
  client: OpenAI,
  writerModel: string,
  reviewerModel: string,
  action: DocAction,
): Promise<void> {
  const docPath = action.path;
  const actionType = action.action;

  console.log(`\n${"=".repeat(60)}`);
  console.log(`Processing: ${actionType} ${docPath}`);
  console.log(`Reason: ${action.reason}`);
  console.log("=".repeat(60));

  // Handle DELETE actions
  if (actionType === "DELETE") {
    if (existsSync(docPath)) {
      unlinkSync(docPath);
      console.log(`  Deleted: ${docPath}`);
    } else {
      console.log(`  File already does not exist: ${docPath}`);
    }
    return;
  }

  // Read relevant source files
  const sourceContents: Record<string, string> = {};
  for (const srcFile of action.relevant_source_files ?? []) {
    const content = readFileSafe(srcFile);
    if (content) {
      sourceContents[srcFile] = content;
    }
  }

  // Read existing content (for UPDATE)
  let existingContent = "";
  if (actionType === "UPDATE" && existsSync(docPath)) {
    existingContent = readFileSync(docPath, "utf-8");
  }

  // Write + Review loop
  let feedback = "";
  let finalContent = "";

  for (let iteration = 1; iteration <= MAX_REVIEW_ITERATIONS; iteration++) {
    console.log(`\n  --- Iteration ${iteration}/${MAX_REVIEW_ITERATIONS} ---`);

    // Writer Agent
    console.log("  [Writer] Generating documentation...");
    let content = await generateDocument(
      client,
      writerModel,
      action,
      sourceContents,
      existingContent,
      feedback,
    );

    // Strip markdown fences if the model wraps the entire response
    content = stripMarkdownFences(content);
    finalContent = content;

    // Ensure parent directories exist and write the file
    mkdirSync(dirname(docPath), { recursive: true });
    writeFileSync(docPath, content + "\n", "utf-8");
    console.log(`  [Writer] Wrote ${content.length} chars to ${docPath}`);

    // Reviewer Agent
    console.log("  [Reviewer] Evaluating documentation quality...");
    const review = await reviewDocument(
      client,
      reviewerModel,
      content,
      action,
      sourceContents,
    );

    const scores = review.scores ?? {};
    const approved = review.approved;
    feedback = review.feedback ?? "";

    console.log(
      `  [Reviewer] Scores: completeness=${scores.completeness ?? "?"}, ` +
        `clarity=${scores.clarity ?? "?"}, ` +
        `examples=${scores.examples ?? "?"}`,
    );
    console.log(`  [Reviewer] Approved: ${approved}`);

    if (approved) {
      console.log("  Documentation approved!");
      break;
    }

    if (iteration < MAX_REVIEW_ITERATIONS) {
      console.log(`  [Reviewer] Feedback: ${feedback}`);
      console.log("  Retrying with reviewer feedback...");
    } else {
      console.log("  Max iterations reached. Accepting current version.");
      console.log(
        `  [Reviewer] Final feedback (for manual review): ${feedback}`,
      );
    }
  }

  // Ensure the final content is written
  mkdirSync(dirname(docPath), { recursive: true });
  writeFileSync(docPath, finalContent + "\n", "utf-8");
  console.log(`  Final document saved: ${docPath}`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  const apiKey = getEnv("MODELS_API_KEY");
  const client = createClient(apiKey);
  const writerModel = resolveModel("WRITER");
  const reviewerModel = resolveModel("REVIEWER");

  console.log(`Writer model: ${writerModel}`);
  console.log(`Reviewer model: ${reviewerModel}`);

  // Load the documentation plan
  const planPath = "doc-plan.json";
  if (!existsSync(planPath)) {
    console.error("::error::doc-plan.json not found.");
    process.exit(1);
  }

  const plan: DocPlan = JSON.parse(readFileSync(planPath, "utf-8"));
  const actions = plan.actions ?? [];

  if (actions.length === 0) {
    console.log("No documentation actions to process.");
    return;
  }

  const prNumber =
    plan.pr_number ?? process.env.PR_NUMBER ?? "unknown";
  console.log(
    `Documentation Writer Agent - Processing ${actions.length} actions for PR #${prNumber}`,
  );

  // Process each action
  for (let i = 0; i < actions.length; i++) {
    process.stdout.write(`\n[${i + 1}/${actions.length}] `);
    await processAction(client, writerModel, reviewerModel, actions[i]);
  }

  console.log(`\n${"=".repeat(60)}`);
  console.log(`All ${actions.length} documentation actions processed.`);
  console.log("=".repeat(60));
}

main().catch((err) => {
  console.error("::error::Writer agent failed:", err);
  process.exit(1);
});
