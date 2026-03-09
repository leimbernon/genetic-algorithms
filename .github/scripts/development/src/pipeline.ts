/**
 * Development Pipeline — Local Orchestrator
 *
 * Runs the full development agent pipeline sequentially for local testing.
 * Supports two modes:
 *
 *   1. "analyze" — Run triage analysis only (Phase 1)
 *   2. "develop" — Run full development pipeline (Phase 2: branch + architect + writer + PR)
 *   3. (default) — Run both: analyze first, then develop if analysis passes
 *
 * Usage:
 *   # Full pipeline:
 *   GITHUB_TOKEN=... REPO=owner/repo ISSUE_NUMBER=42 npx tsx src/pipeline.ts
 *
 *   # Triage analysis only:
 *   GITHUB_TOKEN=... REPO=owner/repo ISSUE_NUMBER=42 npx tsx src/pipeline.ts analyze
 *
 *   # Development only (skip analysis, assume requirements are met):
 *   GITHUB_TOKEN=... REPO=owner/repo ISSUE_NUMBER=42 npx tsx src/pipeline.ts develop
 */

import { execSync } from "node:child_process";
import { getEnv } from "./shared.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function runStep(name: string, script: string): void {
  console.log(`\n${"#".repeat(72)}`);
  console.log(`# STEP: ${name}`);
  console.log(`${"#".repeat(72)}\n`);

  try {
    execSync(`npx tsx src/${script}`, {
      cwd: import.meta.dirname ?? process.cwd(),
      stdio: "inherit",
      env: process.env,
      timeout: 600_000, // 10 minutes per step
    });
    console.log(`\n  Step "${name}" completed successfully.`);
  } catch (err) {
    console.error(`\n::error::Step "${name}" failed.`);
    throw err;
  }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

async function main(): Promise<void> {
  // Validate required environment
  getEnv("GITHUB_TOKEN");
  getEnv("REPO");
  getEnv("ISSUE_NUMBER");

  const mode = process.argv[2] ?? "full";

  console.log("Development Agent Pipeline");
  console.log("=".repeat(72));
  console.log(`Repository: ${process.env.REPO}`);
  console.log(`Issue:      #${process.env.ISSUE_NUMBER}`);
  console.log(`Mode:       ${mode}`);
  console.log(`Provider:   ${process.env.AI_PROVIDER ?? "github"}`);
  console.log(`Model:      ${process.env.DEFAULT_MODEL ?? "(default)"}`);
  console.log("=".repeat(72));

  const startTime = Date.now();

  // Phase 1: Triage Analysis
  if (mode === "analyze" || mode === "full") {
    runStep("Triage — Analyze issue requirements", "triage.ts");

    if (mode === "analyze") {
      console.log("\nAnalysis-only mode. Pipeline stopped.");
      return;
    }
  }

  // Phase 2: Full Development
  if (mode === "develop" || mode === "full") {
    // Step 2a: Prepare branch
    runStep("Prepare — Create branch", "prepare-branch.ts");

    // Check if branch was created
    const { readFileSync } = await import("node:fs");
    const triageResult = JSON.parse(readFileSync("triage-result.json", "utf-8"));

    if (!triageResult.requirements_met) {
      console.log("\nBranch preparation failed. Pipeline stopped.");
      return;
    }

    // Step 2b: Architect
    runStep("Architect — Plan file changes", "architect.ts");

    // Step 2c: Writer (includes Reviewer loop)
    runStep("Writer — Implement code + Review", "writer.ts");

    // Step 2d: Pull Request
    runStep("Pull Request — Create PR", "pull-request.ts");
  }

  const elapsed = Math.round((Date.now() - startTime) / 1000);
  console.log(`\n${"=".repeat(72)}`);
  console.log(`Pipeline completed in ${elapsed}s`);
  console.log("=".repeat(72));
}

main().catch((err) => {
  console.error("\n::error::Pipeline failed:", err);
  process.exit(1);
});
