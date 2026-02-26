/**
 * Shared utilities for documentation agents.
 *
 * Provides: environment helpers, GitHub API client, AI model caller,
 * file I/O helpers, and prompt loading from markdown files.
 */

import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { appendFileSync } from "node:fs";
import OpenAI from "openai";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

export const MODELS_ENDPOINT = "https://models.inference.ai.azure.com";
export const MODEL_NAME = "claude-sonnet-4-5";
export const MAX_RETRIES = 3;
export const RETRY_DELAY_SECONDS = 5;

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
  const promptDir = join(import.meta.dirname!, "prompts");
  const promptPath = join(promptDir, `${promptName}.md`);
  return readFileSafe(promptPath);
}

/**
 * Walk the docs/ directory and return a mapping of
 * relative path -> file content for all .md files.
 */
export function collectExistingDocs(docsDir: string): Record<string, string> {
  const docs: Record<string, string> = {};
  if (!existsSync(docsDir)) return docs;

  function walk(dir: string): void {
    for (const entry of readdirSync(dir)) {
      const fullPath = join(dir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else if (entry.endsWith(".md")) {
        const rel = relative(join(docsDir, ".."), fullPath); // e.g. docs/traits.md
        try {
          docs[rel] = readFileSync(fullPath, "utf-8");
        } catch (e) {
          console.warn(`::warning::Could not read ${fullPath}: ${e}`);
        }
      }
    }
  }

  walk(docsDir);
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

/** Create an OpenAI client configured for Azure AI Models. */
export function createClient(apiKey: string): OpenAI {
  return new OpenAI({
    baseURL: MODELS_ENDPOINT,
    apiKey,
  });
}

/** Call the AI model with retry logic. */
export async function callModel(
  client: OpenAI,
  systemPrompt: string,
  userPrompt: string,
  temperature = 0.2,
): Promise<string> {
  for (let attempt = 1; attempt <= MAX_RETRIES; attempt++) {
    try {
      const response = await client.chat.completions.create({
        model: MODEL_NAME,
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
