/**
 * Shared utilities for documentation agents.
 *
 * Provides: environment helpers, GitHub API client, AI model caller,
 * file I/O helpers, and prompt loading from markdown files.
 */

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { appendFileSync } from "node:fs";
import OpenAI from "openai";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

export const MODELS_ENDPOINT = "https://models.github.ai/inference";
export const DEFAULT_MODEL = "openai/gpt-5";
export const MAX_RETRIES = 3;
export const RETRY_DELAY_SECONDS = 5;

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
  let dir = resolve(import.meta.dirname!, "..");
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
 *   3. Hardcoded fallback (`openai/gpt-4.1`)
 */
export function resolveModel(role: AgentRole): string {
  const agentEnv = process.env[`${role}_MODEL`];
  if (agentEnv) return agentEnv;

  const defaultEnv = process.env["DEFAULT_MODEL"];
  if (defaultEnv) return defaultEnv;

  return DEFAULT_MODEL;
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

/**
 * Load a prompt from a markdown file inside the `prompts/` directory.
 * Returns the raw markdown content as a string.
 */
export function loadPrompt(promptName: string): string {
  const promptDir = join(import.meta.dirname!, "..", "prompts");
  const promptPath = join(promptDir, `${promptName}.md`);
  return readFileSafe(promptPath);
}

/**
 * Load the DOCUMENTATION_STRUCTURE.md definition file.
 */
export function loadStructureDefinition(): string {
  const structurePath = join(import.meta.dirname!, "..", "DOCUMENTATION_STRUCTURE.md");
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

/** Create an OpenAI-compatible client configured for GitHub Models API. */
export function createClient(token: string): OpenAI {
  return new OpenAI({
    baseURL: MODELS_ENDPOINT,
    apiKey: token,
  });
}

/** Call the AI model with retry logic. */
export async function callModel(
  client: OpenAI,
  modelName: string,
  systemPrompt: string,
  userPrompt: string,
  temperature = 0.2,
): Promise<string> {
  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    try {
      const response = await client.chat.completions.create({
        model: modelName,
        messages: [
          { role: "system", content: systemPrompt },
          { role: "user", content: userPrompt },
        ],
        temperature,
      });
      return response.choices[0]?.message?.content?.trim() ?? "";
    } catch (err) {
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
