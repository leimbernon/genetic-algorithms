/**
 * Shared utilities for documentation agents.
 *
 * Provides: environment helpers, GitHub API client, AI model caller,
 * file I/O helpers, and prompt loading from markdown files.
 */

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { join, relative, resolve, dirname as pathDirname } from "node:path";
import { appendFileSync } from "node:fs";
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
 * `.github/scripts/documentation/`, so `process.cwd()` points there instead
 * of the repo root. We use `GITHUB_WORKSPACE` when available (CI), otherwise
 * walk up from this file's directory until we find `Cargo.toml`.
 */
export const REPO_ROOT: string = (() => {
  if (process.env.GITHUB_WORKSPACE) {
    return process.env.GITHUB_WORKSPACE;
  }
  // Local dev: walk up from this file until we find Cargo.toml
  // Use fileURLToPath for Node 18 compatibility (import.meta.dirname requires Node 21+)
  const thisDir = import.meta.dirname ?? pathDirname(fileURLToPath(import.meta.url));
  let dir = resolve(thisDir, "..");
  for (let i = 0; i < 10; i++) {
    if (existsSync(join(dir, "Cargo.toml"))) return dir;
    const parent = resolve(dir, "..");
    if (parent === dir) break; // reached filesystem root
    dir = parent;
  }
  // Fallback to cwd
  return process.cwd();
})();

/**
 * Agent role identifiers used for per-agent model configuration.
 * Each maps to an environment variable: `{ROLE}_MODEL` (e.g. `ANALYST_MODEL`).
 */
export type AgentRole = "GUARD" | "INITIALIZER" | "ANALYST" | "WRITER" | "REVIEWER";

/**
 * Resolve the AI model name for a given agent role.
 *
 * Resolution order:
 *   1. Agent-specific env var (e.g. `ANALYST_MODEL`)
 *   2. Shared env var `DEFAULT_MODEL`
 *   3. Hardcoded fallback (depends on provider)
 */
export function resolveModel(role: AgentRole): string {
  const agentEnv = process.env[`${role}_MODEL`];
  if (agentEnv) return agentEnv;

  const defaultEnv = process.env["DEFAULT_MODEL"];
  if (defaultEnv) return defaultEnv;

  const provider = resolveProvider();
  return provider === "anthropic" ? DEFAULT_ANTHROPIC_MODEL : DEFAULT_GITHUB_MODEL;
}

/** Only analyze files in these paths (relevant for public documentation). */
export const ANALYZABLE_PREFIXES = ["src/", "examples/", "Cargo.toml"];

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
    // Fallback for local testing
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
  return import.meta.dirname ?? pathDirname(fileURLToPath(import.meta.url));
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
 * Load the DOCUMENTATION_STRUCTURE.md definition file.
 */
export function loadStructureDefinition(): string {
  const structurePath = join(thisFileDir(), "..", "DOCUMENTATION_STRUCTURE.md");
  return readFileSafe(structurePath);
}

/**
 * The required documentation files as defined in DOCUMENTATION_STRUCTURE.md.
 * This is the canonical list used by the guard agent to validate structure.
 */
export const REQUIRED_DOC_FILES: string[] = [
  "docs/getting-started.md",
  "docs/configuration.md",
  "docs/chromosomes.md",
  "docs/genotypes.md",
  "docs/operators/selection.md",
  "docs/operators/crossover.md",
  "docs/operators/mutation.md",
  "docs/operators/survivor.md",
  "docs/fitness.md",
  "docs/population.md",
  "docs/traits.md",
  "docs/validators.md",
  "docs/examples.md",
  "docs/api-reference.md",
];

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
 * Walk the docs/ directory and return a mapping of
 * relative path (from repo root) -> file content for all .md files.
 */
export function collectExistingDocs(docsDir: string): Record<string, string> {
  const absDocsDir = resolve(REPO_ROOT, docsDir);
  const docs: Record<string, string> = {};
  if (!existsSync(absDocsDir)) return docs;

  function walk(dir: string): void {
    for (const entry of readdirSync(dir)) {
      const fullPath = join(dir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else if (entry.endsWith(".md")) {
        const rel = relative(REPO_ROOT, fullPath); // e.g. docs/traits.md
        try {
          docs[rel] = readFileSync(fullPath, "utf-8");
        } catch (e) {
          console.warn(`::warning::Could not read ${fullPath}: ${e}`);
        }
      }
    }
  }

  walk(absDocsDir);
  return docs;
}

// ---------------------------------------------------------------------------
// GitHub API helpers
// ---------------------------------------------------------------------------

/** Perform a GET request against the GitHub API with pagination support. */
export async function githubApiGet<T>(url: string, token: string): Promise<T> {
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
      // Handle pagination via Link header
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

/** Fetch the list of files changed in a pull request. */
export async function getPRChangedFiles(
  repo: string,
  prNumber: string,
  token: string,
): Promise<GitHubFile[]> {
  const url = `https://api.github.com/repos/${repo}/pulls/${prNumber}/files?per_page=100`;
  return githubApiGet<GitHubFile[]>(url, token);
}

export interface GitHubFile {
  filename: string;
  status: string; // added, modified, removed, renamed
  additions: number;
  deletions: number;
  patch?: string;
}

// ---------------------------------------------------------------------------
// AI Model helpers
// ---------------------------------------------------------------------------

/**
 * Provider-agnostic AI client wrapper.
 *
 * Agents receive this from `createClient()` and pass it to `callModel()`.
 * They never need to know which provider is being used.
 */
export interface AIClient {
  provider: AIProvider;
  openai?: OpenAI;
  anthropic?: Anthropic;
}

/**
 * Create an AI client based on the configured provider.
 *
 * - `github` (default): Uses the OpenAI SDK pointed at GitHub Models API.
 *   Authenticated with `GITHUB_TOKEN`.
 * - `anthropic`: Uses the Anthropic SDK directly.
 *   Authenticated with `ANTHROPIC_API_KEY`.
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

  // Default: GitHub Models (OpenAI-compatible)
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
 * Callers should catch this to save progress and exit gracefully.
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
 *
 * GitHub Models free tier: 15 req/min → ~4s between requests.
 * Anthropic: generous limits, but a small delay avoids bursts.
 */
const MIN_CALL_INTERVAL_MS = 5_000;
let lastCallTimestamp = 0;

async function waitForRateLimit(): Promise<void> {
  const now = Date.now();
  const elapsed = now - lastCallTimestamp;
  if (lastCallTimestamp > 0 && elapsed < MIN_CALL_INTERVAL_MS) {
    const waitMs = MIN_CALL_INTERVAL_MS - elapsed;
    console.log(`  [Rate limit] Waiting ${Math.ceil(waitMs / 1000)}s before next API call...`);
    await sleep(waitMs);
  }
  lastCallTimestamp = Date.now();
}

/**
 * Call the AI model via the Anthropic Messages API.
 */
async function callAnthropic(
  client: Anthropic,
  modelName: string,
  systemPrompt: string,
  userPrompt: string,
  temperature: number,
): Promise<string> {
  const response = await client.messages.create({
    model: modelName,
    max_tokens: 8192,
    system: systemPrompt,
    messages: [{ role: "user", content: userPrompt }],
    temperature,
  });

  // Extract text from content blocks
  const textBlocks = response.content.filter(
    (block): block is Anthropic.TextBlock => block.type === "text",
  );
  return textBlocks.map((b) => b.text).join("").trim();
}

/**
 * Call the AI model via the OpenAI-compatible API (GitHub Models).
 */
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
 *
 * Works with both GitHub Models (OpenAI-compatible) and Anthropic providers.
 *
 * - Proactively waits between calls to avoid per-minute rate limits.
 * - Detects daily rate limits (GitHub) and throws `DailyRateLimitError`.
 * - Retries transient errors with exponential backoff.
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
      // Re-throw our own errors
      if (err instanceof DailyRateLimitError) throw err;

      const status = (err as { status?: number }).status;
      const headers = (err as { headers?: Record<string, string> }).headers;

      // Handle rate limits (both providers return 429)
      if (status === 429) {
        const retryAfter = parseInt(headers?.["retry-after"] ?? "0", 10);
        const limitType = headers?.["x-ratelimit-type"] ?? "unknown";

        // GitHub daily limit: retry-after is huge (> 60s) or type contains "Day"
        if (retryAfter > 60 || limitType.includes("Day")) {
          console.error(
            `::error::Daily rate limit hit (type=${limitType}, retry-after=${retryAfter}s). Cannot continue.`,
          );
          throw new DailyRateLimitError(retryAfter);
        }

        // Per-minute limit: wait and retry
        const waitSeconds = Math.max(retryAfter, RETRY_DELAY_SECONDS * attempt);
        console.warn(
          `::warning::Rate limit hit (attempt ${attempt}/${MAX_RETRIES}). ` +
            `Waiting ${waitSeconds}s...`,
        );
        await sleep(waitSeconds * 1000);
        continue;
      }

      // Anthropic overloaded (529)
      if (status === 529) {
        const waitSeconds = RETRY_DELAY_SECONDS * attempt;
        console.warn(
          `::warning::Anthropic API overloaded (attempt ${attempt}/${MAX_RETRIES}). ` +
            `Waiting ${waitSeconds}s...`,
        );
        await sleep(waitSeconds * 1000);
        continue;
      }

      // Non-rate-limit error
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
  // Unreachable, but satisfies TypeScript
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

// ---------------------------------------------------------------------------
// Token budget utilities
// ---------------------------------------------------------------------------

/**
 * Estimate the number of tokens in a string.
 *
 * Uses a conservative heuristic of ~4 characters per token, which is a
 * reasonable approximation for English/code text with GPT-family tokenizers.
 */
export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

/**
 * Extract the public API surface from a Rust source file.
 *
 * Keeps:
 *   - Doc comments (`///`, `//!`)
 *   - `pub` item signatures (structs, enums, traits, functions, type aliases)
 *   - `impl` block headers
 *   - Struct/enum fields and variant definitions
 *   - `use` / `mod` declarations that are `pub`
 *   - Derive and attribute macros on kept items
 *
 * Strips:
 *   - Function/method bodies (replaced with `{ ... }`)
 *   - Non-doc comments
 *   - Private items
 *   - Blank lines (collapsed)
 */
export function extractRustPublicAPI(source: string): string {
  const lines = source.split("\n");
  const output: string[] = [];
  let braceDepth = 0;
  let inFnBody = false;
  let fnBraceStart = 0;
  let pendingAttrs: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trimStart();

    // Always keep module-level doc comments
    if (trimmed.startsWith("//!")) {
      output.push(line);
      continue;
    }

    // Always keep doc comments (they precede pub items)
    if (trimmed.startsWith("///")) {
      pendingAttrs.push(line);
      continue;
    }

    // Collect attributes (#[...]) — they may decorate a pub item
    if (trimmed.startsWith("#[")) {
      pendingAttrs.push(line);
      continue;
    }

    // If we're inside a function body, just count braces to find the end
    if (inFnBody) {
      for (const ch of line) {
        if (ch === "{") braceDepth++;
        if (ch === "}") braceDepth--;
      }
      if (braceDepth <= fnBraceStart) {
        inFnBody = false;
        // We already emitted `{ ... }` when we entered the body
      }
      continue;
    }

    // Detect `pub` items and `impl` blocks
    const isPubItem =
      trimmed.startsWith("pub ") ||
      trimmed.startsWith("pub(");
    const isImplBlock =
      trimmed.startsWith("impl ") || trimmed.startsWith("impl<");
    const isUseOrMod =
      trimmed.startsWith("use ") || trimmed.startsWith("mod ");

    if (isPubItem || isImplBlock) {
      // Flush pending doc comments / attributes
      output.push(...pendingAttrs);
      pendingAttrs = [];

      // Determine if this is a function/method signature
      const isFn =
        trimmed.includes(" fn ") ||
        trimmed.startsWith("pub fn ") ||
        trimmed.startsWith("pub(crate) fn ") ||
        trimmed.startsWith("pub(super) fn ");

      if (isFn) {
        // Emit the signature, replacing the body with `{ ... }`
        const sigLine = extractFnSignature(line);
        output.push(sigLine);

        // Count braces to skip the body
        const openCount = countChar(line, "{");
        const closeCount = countChar(line, "}");
        if (openCount > closeCount) {
          inFnBody = true;
          fnBraceStart = braceDepth; // remember the depth before entering
          braceDepth += openCount - closeCount;
        }
        // If body is on same line (open+close equal), we're done
        continue;
      }

      // For struct/enum/trait/impl — emit the header line
      output.push(line);

      // Track braces for struct/enum bodies — we KEEP field definitions
      const openCount = countChar(line, "{");
      const closeCount = countChar(line, "}");
      braceDepth += openCount - closeCount;
      continue;
    }

    // Inside a struct/enum/trait/impl body (braceDepth > 0)
    if (braceDepth > 0) {
      const openCount = countChar(line, "{");
      const closeCount = countChar(line, "}");

      // Keep doc comments, pub fields, method signatures, closing braces,
      // variant definitions, type/const declarations
      const isDocComment = trimmed.startsWith("///");
      const isPubField = trimmed.startsWith("pub ");
      const isVariant = /^\s*\w+/.test(line) && !trimmed.startsWith("//");
      const isClosingBrace = trimmed.startsWith("}");
      const isFnSig = trimmed.startsWith("fn ") || (isPubField && trimmed.includes(" fn "));
      const isAttr = trimmed.startsWith("#[");
      const isType = trimmed.startsWith("type ") || trimmed.startsWith("const ");

      if (isDocComment || isAttr) {
        pendingAttrs.push(line);
        braceDepth += openCount - closeCount;
        continue;
      }

      if (isFnSig || isPubField) {
        // Flush pending attrs
        output.push(...pendingAttrs);
        pendingAttrs = [];

        if (trimmed.includes(" fn ") || trimmed.startsWith("fn ")) {
          const sigLine = extractFnSignature(line);
          output.push(sigLine);
          if (openCount > closeCount) {
            inFnBody = true;
            fnBraceStart = braceDepth;
            braceDepth += openCount - closeCount;
          }
        } else {
          output.push(line);
          braceDepth += openCount - closeCount;
        }
        continue;
      }

      if (isClosingBrace || isType) {
        output.push(...pendingAttrs);
        pendingAttrs = [];
        output.push(line);
        braceDepth += openCount - closeCount;
        continue;
      }

      // Inside an enum body, keep variant lines (they don't start with //)
      if (isVariant && braceDepth === 1) {
        output.push(...pendingAttrs);
        pendingAttrs = [];
        output.push(line);
        braceDepth += openCount - closeCount;
        continue;
      }

      braceDepth += openCount - closeCount;
      // Discard other lines (private fields, inner logic)
      pendingAttrs = [];
      continue;
    }

    // Top-level non-pub items: discard (including private fns, use statements)
    // But keep pub use/mod
    if (isUseOrMod && isPubItem) {
      output.push(...pendingAttrs);
      pendingAttrs = [];
      output.push(line);
      continue;
    }

    // Discard pending attrs that don't attach to a pub item
    pendingAttrs = [];
  }

  // Collapse consecutive blank lines
  return output
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

/** Extract a function signature line, replacing the body with `{ ... }`. */
function extractFnSignature(line: string): string {
  const braceIdx = line.indexOf("{");
  if (braceIdx === -1) {
    // Signature continues on next line or is a trait method with `;`
    return line;
  }
  return line.slice(0, braceIdx).trimEnd() + " { ... }";
}

/** Count occurrences of a character in a string. */
function countChar(s: string, ch: string): number {
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    if (s[i] === ch) count++;
  }
  return count;
}

/**
 * Build a source code section that fits within a token budget.
 *
 * For each source file, extracts the public API surface first. If the total
 * still exceeds the budget, truncates proportionally across files.
 *
 * @param sources   Map of filepath -> full source content
 * @param tokenBudget  Maximum tokens allowed for the source section
 * @returns Formatted markdown string with source code blocks
 */
export function buildSourceSection(
  sources: Record<string, string>,
  tokenBudget: number,
): string {
  if (Object.keys(sources).length === 0) {
    return "No source files available.";
  }

  // Step 1: Extract public API for each file
  const extracted: Record<string, string> = {};
  for (const [filepath, content] of Object.entries(sources)) {
    if (filepath.endsWith(".rs")) {
      extracted[filepath] = extractRustPublicAPI(content);
    } else {
      // Non-Rust files (Cargo.toml, etc.) — keep as-is but truncate
      extracted[filepath] =
        content.length > 2000
          ? content.slice(0, 2000) + "\n... (truncated)"
          : content;
    }
  }

  // Step 2: Calculate total tokens after extraction
  const parts: Array<{ filepath: string; content: string; tokens: number }> = [];
  let totalTokens = 0;
  for (const [filepath, content] of Object.entries(extracted)) {
    // Account for markdown wrapper: ### filepath\n```rust\n...\n```
    const wrapper = `### ${filepath}\n\`\`\`rust\n\`\`\`\n\n`;
    const tokens = estimateTokens(content + wrapper);
    parts.push({ filepath, content, tokens });
    totalTokens += tokens;
  }

  // Step 3: If within budget, return everything
  if (totalTokens <= tokenBudget) {
    return parts
      .map((p) => `### ${p.filepath}\n\`\`\`rust\n${p.content}\n\`\`\``)
      .join("\n\n");
  }

  // Step 4: Proportionally truncate each file to fit the budget
  const ratio = tokenBudget / totalTokens;
  const truncatedParts: string[] = [];

  for (const part of parts) {
    const allowedChars = Math.floor(part.content.length * ratio);
    const truncated =
      allowedChars >= part.content.length
        ? part.content
        : part.content.slice(0, allowedChars) + "\n... (truncated to fit token budget)";
    truncatedParts.push(
      `### ${part.filepath}\n\`\`\`rust\n${truncated}\n\`\`\``,
    );
  }

  return truncatedParts.join("\n\n");
}

/**
 * Truncate a text string to fit within a token budget.
 * Appends a truncation notice if the text was shortened.
 */
export function truncateToTokenBudget(text: string, tokenBudget: number): string {
  const currentTokens = estimateTokens(text);
  if (currentTokens <= tokenBudget) return text;

  const allowedChars = tokenBudget * 4; // inverse of the 4-chars-per-token heuristic
  return text.slice(0, allowedChars) + "\n... (truncated to fit token budget)";
}

// ---------------------------------------------------------------------------
// Misc
// ---------------------------------------------------------------------------

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface DocAction {
  action: "CREATE" | "UPDATE" | "DELETE";
  path: string;
  reason: string;
  relevant_source_files: string[];
  key_points: string[];
}

export interface DocPlan {
  pr_number: number;
  pr_title: string;
  actions: DocAction[];
}

export interface ReviewResult {
  approved: boolean;
  scores: {
    completeness: number;
    clarity: number;
    examples: number;
  };
  feedback: string;
}
