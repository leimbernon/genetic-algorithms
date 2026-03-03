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
import { execSync } from "node:child_process";
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
  buildSourceSection,
  estimateTokens,
  truncateToTokenBudget,
  setGitHubOutput,
  githubApiGet,
  DailyRateLimitError,
  REQUIRED_DOC_FILES,
  REPO_ROOT,
  type ReviewResult,
  type AIClient,
} from "./shared.js";
import { reviewDocument } from "./review-docs.js";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const MAX_REVIEW_ITERATIONS = 5;

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
// Git & PR helpers for incremental initialization
// ---------------------------------------------------------------------------

const INIT_BRANCH = "docs/initialization";

/** Execute a git command in the repo root and return trimmed stdout. */
function git(command: string): string {
  return execSync(`git ${command}`, {
    cwd: REPO_ROOT,
    encoding: "utf-8",
    stdio: ["pipe", "pipe", "pipe"],
  }).trim();
}

/** Check if a branch exists on origin. */
function remoteBranchExists(branch: string): boolean {
  try {
    git(`ls-remote --exit-code origin refs/heads/${branch}`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Set up the local initialization branch.
 *
 * Strategy: start from HEAD (which has the latest code and scripts from
 * the base branch checkout), then carry forward any docs that already
 * exist on the remote initialization branch.
 */
function setupInitBranch(baseBranch: string): void {
  git("fetch origin");

  // If the remote init branch exists, carry forward its docs/ directory
  if (remoteBranchExists(INIT_BRANCH)) {
    console.log(`Remote branch ${INIT_BRANCH} exists. Pulling existing docs...`);
    git(`fetch origin ${INIT_BRANCH}`);

    // Overlay docs/ from the remote branch onto the current working tree
    try {
      git(`checkout origin/${INIT_BRANCH} -- docs/`);
      console.log("Carried forward existing docs from remote init branch.");
    } catch {
      console.log("No docs/ directory on remote init branch yet.");
    }
  } else {
    console.log(`Remote branch ${INIT_BRANCH} does not exist yet. Starting fresh.`);
  }

  // Create (or reset) the local init branch from the current state
  try {
    git(`branch -D ${INIT_BRANCH}`);
  } catch {
    // Branch didn't exist locally — that's fine
  }
  git(`checkout -b ${INIT_BRANCH}`);

  // If we carried forward docs, commit them as the base of this branch
  const docsDir = resolve(REPO_ROOT, "docs");
  if (existsSync(docsDir)) {
    try {
      git("add docs/");
      git("diff --cached --quiet");
    } catch {
      // diff --cached exits 1 when there ARE staged changes — commit them
      git('commit -m "docs: carry forward existing documentation from previous run"');
      console.log("Committed carried-forward docs as branch base.");
    }
  }
}

/** Stage and commit a single documentation file. */
function commitSingleDoc(docPath: string): void {
  const absPath = resolve(REPO_ROOT, docPath);
  git(`add "${absPath}"`);
  git(`commit -m "docs: initialize ${docPath}"`);
  console.log(`  [Git] Committed ${docPath}`);
}

/** Force-push the initialization branch to origin. */
function pushInitBranch(): void {
  git(`push origin ${INIT_BRANCH} --force`);
  console.log(`  [Git] Pushed ${INIT_BRANCH} to origin.`);
}

/** Build a PR body with a progress checklist. */
function buildInitPRBody(
  existingDocs: string[],
  missingDocs: string[],
  allDone: boolean,
): string {
  const lines: string[] = [
    "## Documentation Initialization",
    "",
    "This PR is automatically managed by the **Documentation Initializer Agent**.",
    "It bootstraps the full documentation structure incrementally across multiple workflow runs.",
    "",
    "### Progress",
    "",
  ];

  for (const doc of REQUIRED_DOC_FILES) {
    const done = existingDocs.includes(doc);
    lines.push(`- [${done ? "x" : " "}] \`${doc}\``);
  }

  lines.push("");
  if (allDone) {
    lines.push("> **All documents have been initialized.** This PR is ready for review.");
  } else {
    lines.push(
      `> **${existingDocs.length}/${REQUIRED_DOC_FILES.length}** documents completed. ` +
        `Remaining: ${missingDocs.length}. The next workflow run will continue where this left off.`,
    );
  }

  lines.push("");
  lines.push("### Review checklist");
  lines.push("- [ ] Documentation accurately reflects the current codebase");
  lines.push("- [ ] Examples are correct and runnable");
  lines.push("- [ ] Language is clear and accessible to external developers");
  lines.push("");
  lines.push("---");
  lines.push("*Generated automatically by the Documentation Agent workflow.*");

  return lines.join("\n");
}

/**
 * Create a new PR or update the existing one for the initialization branch.
 * Returns the PR number.
 */
async function createOrUpdatePR(
  repo: string,
  token: string,
  baseBranch: string,
  title: string,
  body: string,
): Promise<number> {
  // Search for an existing open PR from the init branch
  interface PRNode {
    number: number;
  }
  const searchResult = await githubApiGet<PRNode[]>(
    `https://api.github.com/repos/${repo}/pulls?head=${repo.split("/")[0]}:${INIT_BRANCH}&state=open&per_page=5`,
    token,
  );

  if (searchResult.length > 0) {
    const prNumber = searchResult[0].number;
    console.log(`Updating existing PR #${prNumber}...`);

    // Update title and body
    const res = await fetch(
      `https://api.github.com/repos/${repo}/pulls/${prNumber}`,
      {
        method: "PATCH",
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: "application/vnd.github+json",
          "X-GitHub-Api-Version": "2022-11-28",
        },
        body: JSON.stringify({ title, body }),
      },
    );
    if (!res.ok) {
      console.error(`Failed to update PR #${prNumber}: ${res.status} ${res.statusText}`);
    }
    return prNumber;
  }

  // Create new PR
  console.log("Creating new initialization PR...");
  const res = await fetch(
    `https://api.github.com/repos/${repo}/pulls`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
      body: JSON.stringify({
        title,
        body,
        head: INIT_BRANCH,
        base: baseBranch,
      }),
    },
  );

  if (!res.ok) {
    const errBody = await res.text();
    throw new Error(`Failed to create PR: ${res.status} ${res.statusText}\n${errBody}`);
  }

  const pr = (await res.json()) as { number: number };
  console.log(`Created PR #${pr.number}.`);

  // Try to add the "documentation" label (non-fatal if it fails)
  try {
    await fetch(
      `https://api.github.com/repos/${repo}/issues/${pr.number}/labels`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: "application/vnd.github+json",
          "X-GitHub-Api-Version": "2022-11-28",
        },
        body: JSON.stringify({ labels: ["documentation"] }),
      },
    );
  } catch {
    console.log("Could not add 'documentation' label (non-fatal).");
  }

  return pr.number;
}

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
  client: AIClient,
  initializerModel: string,
  reviewerModel: string,
  docPath: string,
  sourceContents: Record<string, string>,
  structureDefinition: string,
  existingContent: string,
): Promise<string> {
  const systemPrompt = loadPrompt("initializer");

  // Token budget: 8K total limit, reserve ~1500 for system prompt,
  // ~500 for structure definition overhead, ~500 for other prompt parts.
  // That leaves ~5500 tokens for source code + structure definition content.
  const structTokens = estimateTokens(structureDefinition);
  const SOURCE_TOKEN_BUDGET = Math.max(1000, 5500 - structTokens);

  // Build source code section with token budget and API extraction
  const sourceSection = buildSourceSection(sourceContents, SOURCE_TOKEN_BUDGET);

  let feedback = "";
  let finalContent = "";

  for (let iteration = 1; iteration <= MAX_REVIEW_ITERATIONS; iteration++) {
    console.log(`  --- Iteration ${iteration}/${MAX_REVIEW_ITERATIONS} ---`);

    // Build user prompt
    const promptParts: string[] = [
      `## Target File: ${docPath}`,
      "",
      "## Structure Definition",
      `\`\`\`markdown\n${truncateToTokenBudget(structureDefinition, 800)}\n\`\`\``,
      "",
      "## Relevant Source Code",
      sourceSection,
    ];

    if (existingContent) {
      promptParts.push(
        "",
        "## Existing Content (to be adapted)",
        `\`\`\`markdown\n${truncateToTokenBudget(existingContent, 500)}\n\`\`\``,
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
  const repo = getEnv("REPO");
  const baseBranch = process.env.BASE_BRANCH ?? "main";
  const client = createClient(token);
  const initializerModel = resolveModel("INITIALIZER");
  const reviewerModel = resolveModel("REVIEWER");

  console.log("Documentation Initializer Agent");
  console.log(`Initializer model: ${initializerModel}`);
  console.log(`Reviewer model:    ${reviewerModel}`);
  console.log(`Base branch:       ${baseBranch}`);
  console.log("=".repeat(60));

  // 0. Set up the incremental initialization branch
  console.log("\nSetting up initialization branch...");
  setupInitBranch(baseBranch);

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
  //    After each successful generation, commit and push individually so
  //    progress is preserved even if the workflow is killed or rate-limited.
  let created = 0;
  let skipped = 0;
  let rateLimited = false;
  let prNumber: number | undefined;

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

    try {
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
      created++;

      // Commit this doc individually and push to preserve progress
      commitSingleDoc(docPath);
      pushInitBranch();

      // Create or update the PR after the first doc is pushed
      const existingDocs = REQUIRED_DOC_FILES.filter((d) => {
        const p = resolve(REPO_ROOT, d);
        return existsSync(p) && readFileSync(p, "utf-8").trim().length > 0;
      });
      const missingDocs = REQUIRED_DOC_FILES.filter((d) => !existingDocs.includes(d));
      const allDone = missingDocs.length === 0;
      const title = allDone
        ? "[Documentation] initialization complete"
        : `[Documentation] initialization in progress (${existingDocs.length}/${REQUIRED_DOC_FILES.length})`;
      const body = buildInitPRBody(existingDocs, missingDocs, allDone);
      prNumber = await createOrUpdatePR(repo, token, baseBranch, title, body);
    } catch (err) {
      if (err instanceof DailyRateLimitError) {
        console.error(`\n::warning::${err.message}`);
        console.log(`  Documents already created (${created}) have been committed and pushed.`);
        console.log(`  Remaining ${REQUIRED_DOC_FILES.length - i} docs will be created on the next run.`);
        rateLimited = true;
        break;
      }
      throw err; // Re-throw non-rate-limit errors
    }
  }

  // 4. Final summary and PR update
  const existingDocs = REQUIRED_DOC_FILES.filter((d) => {
    const p = resolve(REPO_ROOT, d);
    return existsSync(p) && readFileSync(p, "utf-8").trim().length > 0;
  });
  const missingDocs = REQUIRED_DOC_FILES.filter((d) => !existingDocs.includes(d));
  const allDone = missingDocs.length === 0;

  console.log(`\n${"=".repeat(60)}`);
  console.log("Documentation Initialization Summary");
  console.log(`  Created this run:  ${created}`);
  console.log(`  Skipped (already exist): ${skipped}`);
  console.log(`  Total complete: ${existingDocs.length}/${REQUIRED_DOC_FILES.length}`);
  if (rateLimited) {
    console.log("  Stopped early due to daily rate limit.");
  }
  console.log("=".repeat(60));

  // Update PR with final state (if we pushed anything or if we're just updating status)
  if (prNumber != null || (created === 0 && skipped > 0)) {
    const title = allDone
      ? "[Documentation] initialization complete"
      : `[Documentation] initialization in progress (${existingDocs.length}/${REQUIRED_DOC_FILES.length})`;
    const body = buildInitPRBody(existingDocs, missingDocs, allDone);

    if (prNumber != null) {
      await createOrUpdatePR(repo, token, baseBranch, title, body);
    }
  }

  // 5. Set output for downstream jobs
  setGitHubOutput("initialization_complete", allDone ? "true" : "false");

  if (allDone) {
    console.log("\nAll documentation files have been initialized!");
  } else {
    console.log(`\n${missingDocs.length} documents remaining. Next workflow run will continue.`);
  }
}

main().catch((err) => {
  console.error("::error::Initializer agent failed:", err);
  process.exit(1);
});
