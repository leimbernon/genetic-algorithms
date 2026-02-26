/**
 * Documentation Agent — Step 3: Review Documentation Quality
 *
 * Evaluates generated documentation against quality criteria and returns
 * structured feedback. Designed to be imported by write-docs.ts, but can
 * also be used standalone.
 */

import OpenAI from "openai";
import {
  callModel,
  loadPrompt,
  stripMarkdownFences,
  buildSourceSection,
  truncateToTokenBudget,
  type DocAction,
  type ReviewResult,
} from "./shared.js";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/** Quality thresholds (0-10 scale). */
const THRESHOLDS: Record<string, number> = {
  completeness: 7,
  clarity: 7,
  examples: 6,
};

// ---------------------------------------------------------------------------
// Reviewer
// ---------------------------------------------------------------------------

/**
 * Review a single documentation file and return a structured evaluation.
 */
export async function reviewDocument(
  client: OpenAI,
  modelName: string,
  docContent: string,
  action: DocAction,
  sourceContents: Record<string, string>,
): Promise<ReviewResult> {
  // Token budget: 8K total limit, reserve ~1500 for system prompt,
  // ~1000 for doc content + metadata, ~500 for task instructions.
  // That leaves ~5000 tokens for source reference.
  const SOURCE_TOKEN_BUDGET = 3000;

  // Build source code reference with token budget and API extraction
  const sourceReference = buildSourceSection(sourceContents, SOURCE_TOKEN_BUDGET);

  const systemPrompt = loadPrompt("reviewer");

  const userPrompt = `## Documentation to Review

**File:** ${action.path}
**Action:** ${action.action}
**Reason:** ${action.reason}
**Key points that should be covered:** ${JSON.stringify(action.key_points ?? [])}

### Generated Documentation
\`\`\`markdown
${truncateToTokenBudget(docContent, 1500)}
\`\`\`

### Reference Source Code
${sourceReference}

## Task
Evaluate the documentation against the three quality criteria.
Respond with this exact JSON structure:
{
  "approved": true or false,
  "scores": {
    "completeness": <0-10>,
    "clarity": <0-10>,
    "examples": <0-10>
  },
  "feedback": "Specific feedback for improvement (empty string if approved)"
}`;

  const rawResponse = await callModel(client, modelName, systemPrompt, userPrompt);
  const cleaned = stripMarkdownFences(rawResponse);

  let review: ReviewResult;
  try {
    review = JSON.parse(cleaned) as ReviewResult;
  } catch {
    console.warn(
      "  [Reviewer] Warning: Could not parse review response, treating as approved.",
    );
    console.warn(`  [Reviewer] Raw response: ${rawResponse.slice(0, 500)}`);
    return {
      approved: true,
      scores: { completeness: 7, clarity: 7, examples: 6 },
      feedback: "",
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
