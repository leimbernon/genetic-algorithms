/**
 * Shared utilities for development agents.
 *
 * Provides: environment helpers, GitHub API client, AI model caller,
 * file I/O helpers, prompt loading, and Git operations.
 */

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { join, relative, resolve, dirname as pathDirname } from "node:path";
import { appendFileSync } from "node:fs";
import { execSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import OpenAI from "openai";
import Anthropic from "@anthropic-ai/sdk";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/** Supported AI providers. Set via `AI_PROVIDER` env var. */
export type AIProvider = "github" | "anthropic";

export const GITHUB_MODELS_ENDPOINT = "https://models.github.ai/inference";
export const DEFAULT_GITHUB_MODEL = "openai/gpt-4.1";
export const DEFAULT_ANTHROPIC_MODEL = "claude-sonnet-4-6";
export const MAX_RETRIES = 3;
export const RETRY_DELAY_SECONDS = 5;

/**
 * Resolve which AI provider to use.
 *
 * Set `AI_PROVIDER=anthropic` to use the Anthropic API directly.
 * Defaults to `"github"` (GitHub Models API).
 */
export function resolveProvider(): AIProvider {
  const env = (process.env.AI_PROVIDER ?? "").toLowerCase().trim();
  if (env === "anthropic") return "anthropic";
  return "github";
}

/**
 * Resolve the repository root directory.
 *
 * In GitHub Actions the scripts run with working-directory set to
 * `.github/scripts/development/`, so `process.cwd()` points there instead
 * of the repo root. We use `GITHUB_WORKSPACE` when available (CI), otherwise
 * walk up from this file's directory until we find `Cargo.toml`.
 */
export const REPO_ROOT: string = (() => {
  if (process.env.GITHUB_WORKSPACE) {
    return process.env.GITHUB_WORKSPACE;
  }
  const thisDir =
    import.meta.dirname ?? pathDirname(fileURLToPath(import.meta.url));
  let dir = resolve(thisDir, "..");
  for (let i = 0; i < 10; i++) {
    if (existsSync(join(dir, "Cargo.toml"))) return dir;
    const parent = resolve(dir, "..");
    if (parent === dir) break;
    dir = parent;
  }
  return process.cwd();
})();

/**
 * Agent role identifiers used for per-agent model configuration.
 * Each maps to an environment variable: `{ROLE}_MODEL`.
 */
export type AgentRole =
  | "TRIAGE"
  | "ARCHITECT"
  | "WRITER"
  | "REVIEWER"
  | "PULL_REQUEST";

/**
 * Resolve the AI model name for a given agent role.
 *
 * Resolution order:
 *   1. Agent-specific env var (e.g. `TRIAGE_MODEL`)
 *   2. Shared env var `DEFAULT_MODEL`
 *   3. Hardcoded fallback (depends on provider)
 */
export function resolveModel(role: AgentRole): string {
  const agentEnv = process.env[`${role}_MODEL`];
  if (agentEnv) return agentEnv;

  const defaultEnv = process.env["DEFAULT_MODEL"];
  if (defaultEnv) return defaultEnv;

  const provider = resolveProvider();
  return provider === "anthropic"
    ? DEFAULT_ANTHROPIC_MODEL
    : DEFAULT_GITHUB_MODEL;
}

// ---------------------------------------------------------------------------
// Environment helpers
// ---------------------------------------------------------------------------

/** Read an environment variable. Exits with error if required and missing. */
export function getEnv(name: string, required = true): string {
  const value = process.env[name] ?? "";
  if (required && !value) {
    console.error(`::error::Environment variable ${name} is not set.`);
    process.exit(1);
  }
  return value;
}

/** Set a GitHub Actions output variable. */
export function setGitHubOutput(name: string, value: string): void {
  const outputFile = process.env.GITHUB_OUTPUT ?? "";
  if (outputFile) {
    appendFileSync(outputFile, `${name}=${value}\n`, "utf-8");
  } else {
    console.log(`OUTPUT: ${name}=${value}`);
  }
}

// ---------------------------------------------------------------------------
// File helpers
// ---------------------------------------------------------------------------

/** Read a file safely; returns empty string on error. */
export function readFileSafe(filePath: string): string {
  try {
    return readFileSync(filePath, "utf-8");
  } catch {
    return "";
  }
}

/** Resolve directory of this source file (Node 18+ compatible). */
function thisFileDir(): string {
  return (
    import.meta.dirname ?? pathDirname(fileURLToPath(import.meta.url))
  );
}

/**
 * Load a prompt from a markdown file inside the `prompts/` directory.
 * Returns the raw markdown content as a string.
 */
export function loadPrompt(promptName: string): string {
  const promptDir = join(thisFileDir(), "..", "prompts");
  const promptPath = join(promptDir, `${promptName}.md`);
  return readFileSafe(promptPath);
}

/**
 * Collect all Rust source files under a directory (relative to REPO_ROOT).
 * Returns a mapping of relative path (from repo root) -> file content.
 */
export function collectSourceFiles(rootDir: string): Record<string, string> {
  const sources: Record<string, string> = {};
  const absRoot = resolve(REPO_ROOT, rootDir);

  function walk(dir: string): void {
    if (!existsSync(dir)) return;
    for (const entry of readdirSync(dir)) {
      const fullPath = join(dir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else if (entry.endsWith(".rs")) {
        const rel = relative(REPO_ROOT, fullPath);
        try {
          sources[rel] = readFileSync(fullPath, "utf-8");
        } catch (e) {
          console.warn(`::warning::Could not read ${fullPath}: ${e}`);
        }
      }
    }
  }

  walk(absRoot);
  return sources;
}

/**
 * List all files in a directory tree, returning relative paths from REPO_ROOT.
 */
export function listProjectFiles(rootDir: string): string[] {
  const files: string[] = [];
  const absRoot = resolve(REPO_ROOT, rootDir);

  function walk(dir: string): void {
    if (!existsSync(dir)) return;
    for (const entry of readdirSync(dir)) {
      if (entry === "target" || entry === "node_modules" || entry === ".git")
        continue;
      const fullPath = join(dir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else {
        files.push(relative(REPO_ROOT, fullPath));
      }
    }
  }

  walk(absRoot);
  return files;
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/** Execute a git command in the repo root and return stdout. */
export function git(command: string): string {
  return execSync(`git ${command}`, {
    cwd: REPO_ROOT,
    encoding: "utf-8",
    maxBuffer: 10 * 1024 * 1024,
  }).trim();
}

/** Check if a branch exists locally or remotely. */
export function branchExists(branchName: string): boolean {
  try {
    git(`rev-parse --verify ${branchName}`);
    return true;
  } catch {
    try {
      git(`rev-parse --verify origin/${branchName}`);
      return true;
    } catch {
      return false;
    }
  }
}

/** Create and checkout a new branch from the specified base.
 *  Returns the actual base branch used (may differ from input if fallback occurred). */
export function createBranch(branchName: string, baseBranch: string): string {
  // Fetch latest from origin
  git("fetch origin");

  // Resolve the base branch — if the requested branch doesn't exist
  // (e.g. a milestone branch that hasn't been created yet), fall back
  // to main so the feature/fix branch can still be created.
  let resolvedBase = baseBranch;
  try {
    git(`checkout ${baseBranch}`);
    git(`pull origin ${baseBranch}`);
  } catch {
    try {
      git(`checkout -b ${baseBranch} origin/${baseBranch}`);
    } catch {
      // Base branch doesn't exist locally or remotely — fall back to main
      console.log(
        `::warning::Base branch '${baseBranch}' not found locally or on origin. Falling back to 'main'.`,
      );
      resolvedBase = "main";
      try {
        git("checkout main");
        git("pull origin main");
      } catch {
        git("checkout -b main origin/main");
      }
    }
  }

  // Create new branch
  git(`checkout -b ${branchName}`);
  console.log(`  Created branch '${branchName}' from '${resolvedBase}'`);
  return resolvedBase;
}

/** Push the current branch to origin. */
export function pushBranch(branchName: string): void {
  git(`push -u origin ${branchName}`);
  console.log(`  Pushed branch '${branchName}' to origin`);
}

/** Stage and commit changes. Returns true if there were changes to commit. */
export function commitChanges(message: string): boolean {
  git("add -A");
  const status = git("status --porcelain");
  if (!status) {
    console.log("  No changes to commit.");
    return false;
  }
  git(`commit -m "${message.replace(/"/g, '\\"')}"`);
  console.log(`  Committed: ${message}`);
  return true;
}

// ---------------------------------------------------------------------------
// GitHub API helpers
// ---------------------------------------------------------------------------

/** GitHub issue/ticket type. */
export interface GitHubIssue {
  number: number;
  title: string;
  body: string | null;
  state: string;
  labels: Array<{ name: string }>;
  milestone: {
    title: string;
    number: number;
  } | null;
  assignee: {
    login: string;
  } | null;
  assignees: Array<{ login: string }>;
  html_url: string;
  draft?: boolean;
  pull_request?: unknown;
}

/** GitHub milestone type. */
export interface GitHubMilestone {
  title: string;
  number: number;
  description: string | null;
  state: string;
}

/** GitHub comment type. */
export interface GitHubComment {
  id: number;
  body: string;
  user: { login: string };
  created_at: string;
}

/** Perform a GET request against the GitHub API with pagination support. */
export async function githubApiGet<T>(
  url: string,
  token: string,
): Promise<T> {
  const results: unknown[] = [];
  let nextUrl: string | null = url;

  while (nextUrl) {
    const resp: Response = await fetch(nextUrl, {
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github.v3+json",
      },
    });

    if (!resp.ok) {
      throw new Error(`GitHub API error: ${resp.status} ${resp.statusText}`);
    }

    const data: unknown = await resp.json();

    if (Array.isArray(data)) {
      results.push(...data);
      nextUrl = null;
      const linkHeader: string | null = resp.headers.get("link");
      if (linkHeader) {
        for (const part of linkHeader.split(",")) {
          if (part.includes('rel="next"')) {
            const match: RegExpMatchArray | null = part.match(/<([^>]+)>/);
            if (match) nextUrl = match[1];
          }
        }
      }
    } else {
      return data as T;
    }
  }

  return results as T;
}

/** Post a comment on a GitHub issue. */
export async function postIssueComment(
  repo: string,
  issueNumber: number,
  body: string,
  token: string,
): Promise<void> {
  const url = `https://api.github.com/repos/${repo}/issues/${issueNumber}/comments`;
  const resp = await fetch(url, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github.v3+json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ body }),
  });

  if (!resp.ok) {
    throw new Error(
      `Failed to post comment: ${resp.status} ${resp.statusText}`,
    );
  }
}

/** Fetch a single issue by number. */
export async function getIssue(
  repo: string,
  issueNumber: number,
  token: string,
): Promise<GitHubIssue> {
  const url = `https://api.github.com/repos/${repo}/issues/${issueNumber}`;
  return githubApiGet<GitHubIssue>(url, token);
}

/** Fetch comments on an issue. */
export async function getIssueComments(
  repo: string,
  issueNumber: number,
  token: string,
): Promise<GitHubComment[]> {
  const url = `https://api.github.com/repos/${repo}/issues/${issueNumber}/comments?per_page=100`;
  return githubApiGet<GitHubComment[]>(url, token);
}

// ---------------------------------------------------------------------------
// Label constants
// ---------------------------------------------------------------------------

/** Workflow label names used by the development agent lifecycle. */
export const LABELS = {
  BACKLOG: "backlog",
  SELECTED: "selected for development",
  PREPARED: "prepared for development",
  IN_PROGRESS: "in progress",
  IN_REVIEW: "in review",
  DONE: "done",
} as const;

// ---------------------------------------------------------------------------
// Label management
// ---------------------------------------------------------------------------

/** Add a label to a GitHub issue. */
export async function addLabel(
  repo: string,
  issueNumber: number,
  label: string,
  token: string,
): Promise<void> {
  const url = `https://api.github.com/repos/${repo}/issues/${issueNumber}/labels`;
  const resp = await fetch(url, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github.v3+json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ labels: [label] }),
  });
  if (!resp.ok) {
    console.warn(
      `::warning::Failed to add label '${label}': ${resp.status} ${resp.statusText}`,
    );
  }
}

/** Remove a label from a GitHub issue. Silently ignores if label is not present. */
export async function removeLabel(
  repo: string,
  issueNumber: number,
  label: string,
  token: string,
): Promise<void> {
  const encodedLabel = encodeURIComponent(label);
  const url = `https://api.github.com/repos/${repo}/issues/${issueNumber}/labels/${encodedLabel}`;
  const resp = await fetch(url, {
    method: "DELETE",
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github.v3+json",
    },
  });
  if (!resp.ok && resp.status !== 404) {
    console.warn(
      `::warning::Failed to remove label '${label}': ${resp.status} ${resp.statusText}`,
    );
  }
}

/** Atomically swap one label for another on a GitHub issue. */
export async function swapLabels(
  repo: string,
  issueNumber: number,
  oldLabel: string,
  newLabel: string,
  token: string,
): Promise<void> {
  await addLabel(repo, issueNumber, newLabel, token);
  await removeLabel(repo, issueNumber, oldLabel, token);
  console.log(`  Labels: '${oldLabel}' → '${newLabel}' on issue #${issueNumber}`);
}

/** Close a GitHub issue. */
export async function closeIssue(
  repo: string,
  issueNumber: number,
  token: string,
  stateReason: "completed" | "not_planned" = "completed",
): Promise<void> {
  const url = `https://api.github.com/repos/${repo}/issues/${issueNumber}`;
  const resp = await fetch(url, {
    method: "PATCH",
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github.v3+json",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ state: "closed", state_reason: stateReason }),
  });
  if (!resp.ok) {
    console.warn(
      `::warning::Failed to close issue #${issueNumber}: ${resp.status} ${resp.statusText}`,
    );
  } else {
    console.log(`  Closed issue #${issueNumber} (${stateReason})`);
  }
}

// ---------------------------------------------------------------------------
// Pull Request helpers
// ---------------------------------------------------------------------------

/** GitHub pull request type (subset of fields we use). */
export interface GitHubPullRequest {
  number: number;
  title: string;
  body: string | null;
  head: { ref: string; sha: string };
  base: { ref: string };
  html_url: string;
  merged: boolean;
  state: string;
}

/** GitHub PR review type. */
export interface GitHubReview {
  id: number;
  user: { login: string };
  body: string | null;
  state: string; // "approved" | "changes_requested" | "commented" | "dismissed"
  submitted_at: string;
}

/** GitHub PR review comment type. */
export interface GitHubReviewComment {
  id: number;
  body: string;
  path: string;
  line: number | null;
  user: { login: string };
  created_at: string;
}

/** Fetch a pull request by number. */
export async function getPullRequest(
  repo: string,
  prNumber: number,
  token: string,
): Promise<GitHubPullRequest> {
  const url = `https://api.github.com/repos/${repo}/pulls/${prNumber}`;
  return githubApiGet<GitHubPullRequest>(url, token);
}

/** Fetch reviews on a pull request. */
export async function getPRReviews(
  repo: string,
  prNumber: number,
  token: string,
): Promise<GitHubReview[]> {
  const url = `https://api.github.com/repos/${repo}/pulls/${prNumber}/reviews?per_page=100`;
  return githubApiGet<GitHubReview[]>(url, token);
}

/** Fetch review comments (inline code comments) on a pull request. */
export async function getPRReviewComments(
  repo: string,
  prNumber: number,
  token: string,
): Promise<GitHubReviewComment[]> {
  const url = `https://api.github.com/repos/${repo}/pulls/${prNumber}/comments?per_page=100`;
  return githubApiGet<GitHubReviewComment[]>(url, token);
}

/**
 * Parse the linked issue number from a PR body.
 * Looks for patterns like "Closes #42", "Fixes #42", "Resolves #42".
 */
export function parseLinkedIssueNumber(prBody: string | null): number | null {
  if (!prBody) return null;
  const match = prBody.match(
    /(?:closes|fixes|resolves|close|fix|resolve)\s+#(\d+)/i,
  );
  return match ? parseInt(match[1], 10) : null;
}

// ---------------------------------------------------------------------------
// AI Model helpers
// ---------------------------------------------------------------------------

/**
 * Provider-agnostic AI client wrapper.
 */
export interface AIClient {
  provider: AIProvider;
  openai?: OpenAI;
  anthropic?: Anthropic;
}

/**
 * Create an AI client based on the configured provider.
 */
export function createClient(token: string): AIClient {
  const provider = resolveProvider();

  if (provider === "anthropic") {
    const apiKey = process.env.ANTHROPIC_API_KEY;
    if (!apiKey) {
      console.error(
        "::error::AI_PROVIDER is set to 'anthropic' but ANTHROPIC_API_KEY is not set.",
      );
      process.exit(1);
    }
    return {
      provider,
      anthropic: new Anthropic({ apiKey }),
    };
  }

  return {
    provider,
    openai: new OpenAI({
      baseURL: GITHUB_MODELS_ENDPOINT,
      apiKey: token,
    }),
  };
}

/**
 * Custom error thrown when the daily rate limit is exhausted.
 */
export class DailyRateLimitError extends Error {
  public retryAfterSeconds: number;
  constructor(retryAfterSeconds: number) {
    super(
      `Daily rate limit reached. Retry after ${retryAfterSeconds}s (~${Math.round(retryAfterSeconds / 3600)}h).`,
    );
    this.name = "DailyRateLimitError";
    this.retryAfterSeconds = retryAfterSeconds;
  }
}

/**
 * Simple rate limiter that enforces a minimum delay between API calls.
 */
const MIN_CALL_INTERVAL_MS = 5_000;
let lastCallTimestamp = 0;

async function waitForRateLimit(): Promise<void> {
  const now = Date.now();
  const elapsed = now - lastCallTimestamp;
  if (lastCallTimestamp > 0 && elapsed < MIN_CALL_INTERVAL_MS) {
    const waitMs = MIN_CALL_INTERVAL_MS - elapsed;
    console.log(
      `  [Rate limit] Waiting ${Math.ceil(waitMs / 1000)}s before next API call...`,
    );
    await sleep(waitMs);
  }
  lastCallTimestamp = Date.now();
}

/** Call the AI model via the Anthropic Messages API. */
async function callAnthropic(
  client: Anthropic,
  modelName: string,
  systemPrompt: string,
  userPrompt: string,
  temperature: number,
): Promise<string> {
  const response = await client.messages.create({
    model: modelName,
    max_tokens: 16384,
    system: systemPrompt,
    messages: [{ role: "user", content: userPrompt }],
    temperature,
  });

  const textBlocks = response.content.filter(
    (block): block is Anthropic.TextBlock => block.type === "text",
  );
  return textBlocks.map((b) => b.text).join("").trim();
}

/** Call the AI model via the OpenAI-compatible API (GitHub Models). */
async function callOpenAI(
  client: OpenAI,
  modelName: string,
  systemPrompt: string,
  userPrompt: string,
  temperature: number,
): Promise<string> {
  const response = await client.chat.completions.create({
    model: modelName,
    messages: [
      { role: "system", content: systemPrompt },
      { role: "user", content: userPrompt },
    ],
    temperature,
  });
  return response.choices[0]?.message?.content?.trim() ?? "";
}

/**
 * Call the AI model with retry logic and rate limit awareness.
 */
export async function callModel(
  client: AIClient,
  modelName: string,
  systemPrompt: string,
  userPrompt: string,
  temperature = 0.2,
): Promise<string> {
  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    try {
      await waitForRateLimit();

      if (client.provider === "anthropic" && client.anthropic) {
        return await callAnthropic(
          client.anthropic,
          modelName,
          systemPrompt,
          userPrompt,
          temperature,
        );
      }

      if (client.openai) {
        return await callOpenAI(
          client.openai,
          modelName,
          systemPrompt,
          userPrompt,
          temperature,
        );
      }

      throw new Error("AI client not properly initialized.");
    } catch (err: unknown) {
      if (err instanceof DailyRateLimitError) throw err;

      const status = (err as { status?: number }).status;
      const headers = (err as { headers?: Record<string, string> }).headers;

      if (status === 429) {
        const retryAfter = parseInt(headers?.["retry-after"] ?? "0", 10);
        const limitType = headers?.["x-ratelimit-type"] ?? "unknown";

        if (retryAfter > 60 || limitType.includes("Day")) {
          console.error(
            `::error::Daily rate limit hit (type=${limitType}, retry-after=${retryAfter}s). Cannot continue.`,
          );
          throw new DailyRateLimitError(retryAfter);
        }

        const waitSeconds = Math.max(
          retryAfter,
          RETRY_DELAY_SECONDS * attempt,
        );
        console.warn(
          `::warning::Rate limit hit (attempt ${attempt}/${MAX_RETRIES}). Waiting ${waitSeconds}s...`,
        );
        await sleep(waitSeconds * 1000);
        continue;
      }

      if (status === 529) {
        const waitSeconds = RETRY_DELAY_SECONDS * attempt;
        console.warn(
          `::warning::API overloaded (attempt ${attempt}/${MAX_RETRIES}). Waiting ${waitSeconds}s...`,
        );
        await sleep(waitSeconds * 1000);
        continue;
      }

      console.warn(
        `::warning::Model call attempt ${attempt}/${MAX_RETRIES} failed: ${err}`,
      );
      if (attempt < MAX_RETRIES) {
        await sleep(RETRY_DELAY_SECONDS * attempt * 1000);
      } else {
        throw err;
      }
    }
  }
  throw new Error("Max retries exceeded");
}

/** Strip markdown fences if the model wraps its response in them. */
export function stripMarkdownFences(text: string): string {
  let cleaned = text.trim();
  if (cleaned.startsWith("```")) {
    cleaned = cleaned.split("\n").slice(1).join("\n");
  }
  if (cleaned.endsWith("```")) {
    cleaned = cleaned.split("\n").slice(0, -1).join("\n");
  }
  return cleaned.trim();
}

/** Estimate the number of tokens in a string (~4 chars per token). */
export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

/** Truncate a text string to fit within a token budget. */
export function truncateToTokenBudget(
  text: string,
  tokenBudget: number,
): string {
  const currentTokens = estimateTokens(text);
  if (currentTokens <= tokenBudget) return text;

  const allowedChars = tokenBudget * 4;
  return text.slice(0, allowedChars) + "\n... (truncated to fit token budget)";
}

// ---------------------------------------------------------------------------
// Issue analysis helpers
// ---------------------------------------------------------------------------

/**
 * Determine the branch type from issue labels.
 * - "enhancement" label -> "feature"
 * - "bug" label -> "fix"
 * - fallback -> "feature"
 */
export function determineBranchType(
  labels: Array<{ name: string }>,
): "feature" | "fix" {
  const labelNames = labels.map((l) => l.name.toLowerCase());
  if (labelNames.includes("bug")) return "fix";
  return "feature";
}

/**
 * Build the branch name for an issue.
 *
 * Format: `{type}/{issue_number}-{slug}`
 * where slug is a kebab-case version of the issue title (max 50 chars).
 */
export function buildBranchName(
  issueNumber: number,
  issueTitle: string,
  branchType: "feature" | "fix",
): string {
  const slug = issueTitle
    .toLowerCase()
    .replace(/\[.*?\]/g, "") // remove [BUG], [REQUEST], etc.
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 50)
    .replace(/-+$/, "");

  return `${branchType}/${issueNumber}-${slug}`;
}

/**
 * Determine the base branch for the new feature/fix branch.
 *
 * - If the issue has a milestone, the base is `milestone/{milestone-slug}`.
 * - Otherwise, the base is `main`.
 */
export function determineBaseBranch(
  milestone: GitHubIssue["milestone"],
): string {
  if (!milestone) return "main";

  const milestoneSlug = milestone.title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return `milestone/${milestoneSlug}`;
}

// ---------------------------------------------------------------------------
// Rust shell command helpers
// ---------------------------------------------------------------------------

/** Run a shell command in the repo root and return { stdout, success }. */
export function runCommand(
  command: string,
): { stdout: string; success: boolean } {
  try {
    const stdout = execSync(command, {
      cwd: REPO_ROOT,
      encoding: "utf-8",
      maxBuffer: 10 * 1024 * 1024,
      timeout: 300_000, // 5 min
    });
    return { stdout: stdout.trim(), success: true };
  } catch (err: unknown) {
    const stdout =
      (err as { stdout?: string }).stdout ??
      (err as { message?: string }).message ??
      "";
    return { stdout: stdout.toString().trim(), success: false };
  }
}

/** Run cargo fmt --check and return results. */
export function cargoFmtCheck(): { stdout: string; success: boolean } {
  return runCommand("cargo fmt --check");
}

/** Run cargo fmt to fix formatting. */
export function cargoFmt(): { stdout: string; success: boolean } {
  return runCommand("cargo fmt");
}

/** Run cargo clippy and return results. */
export function cargoClippy(): { stdout: string; success: boolean } {
  return runCommand(
    "cargo clippy --all-targets --all-features -- -D warnings 2>&1",
  );
}

/** Run cargo test and return results. */
export function cargoTest(): { stdout: string; success: boolean } {
  return runCommand("cargo test 2>&1");
}

/** Run cargo test --doc and return results. */
export function cargoTestDoc(): { stdout: string; success: boolean } {
  return runCommand("cargo test --doc 2>&1");
}

/** Run cargo bench --no-run and return results. */
export function cargoBenchCheck(): { stdout: string; success: boolean } {
  return runCommand("cargo bench --no-run 2>&1");
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** The triage result produced by Agent 1. */
export interface TriageResult {
  issue_number: number;
  issue_title: string;
  branch_name: string;
  base_branch: string;
  branch_type: "feature" | "fix";
  requirements_met: boolean;
  questions: string[];
  summary: string;
}

/** Architecture plan produced by Agent 2. */
export interface ArchitecturePlan {
  issue_number: number;
  files_to_modify: Array<{
    path: string;
    reason: string;
    action: "CREATE" | "MODIFY" | "DELETE";
  }>;
  modules_to_register: Array<{
    parent_module: string;
    new_module: string;
    exports: string[];
  }>;
  enums_to_update: Array<{
    enum_path: string;
    enum_name: string;
    new_variant: string;
  }>;
  configuration_changes: Array<{
    file: string;
    description: string;
  }>;
  summary: string;
}

/** Code writer result produced by Agent 3. */
export interface WriterResult {
  files_written: Array<{
    path: string;
    action: "CREATE" | "MODIFY" | "DELETE";
    description: string;
  }>;
  tests_added: number;
  benchmarks_added: number;
  version_bump: {
    from: string;
    to: string;
    bump_type: "patch" | "minor" | "major";
  } | null;
  validation: {
    fmt: boolean;
    clippy: boolean;
    tests: boolean;
    doc_tests: boolean;
    bench_compile: boolean;
  };
  summary: string;
}

/** Quality review result produced by Agent 4. */
export interface QualityReview {
  approved: boolean;
  scores: {
    code_quality: number;
    test_coverage: number;
    documentation: number;
    architecture_compliance: number;
    error_handling: number;
  };
  issues: Array<{
    severity: "critical" | "major" | "minor";
    file: string;
    line?: number;
    description: string;
    suggestion: string;
  }>;
  feedback: string;
}

/** PR creation result produced by Agent 5. */
export interface PRResult {
  pr_number: number;
  pr_url: string;
  title: string;
  base_branch: string;
  head_branch: string;
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
