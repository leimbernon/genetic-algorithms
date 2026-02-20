"""
Documentation Agent - Step 1: Analyze Changes

This script analyzes the diff of a merged pull request and compares it
against the existing documentation in /docs to determine what documentation
needs to be created, updated, or deleted.

Output: doc-plan.json with a structured list of documentation actions.
"""

import json
import os
import sys
import time
from pathlib import Path

from openai import OpenAI

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

MODELS_ENDPOINT = "https://models.inference.ai.azure.com"
MODEL_NAME = "claude-sonnet-4-5"
MAX_RETRIES = 3
RETRY_DELAY_SECONDS = 5

# Only analyze files in these paths (relevant for public documentation)
ANALYZABLE_PREFIXES = ("src/", "examples/", "Cargo.toml")

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def get_env(name: str, required: bool = True) -> str:
    """Read an environment variable, fail loudly if required and missing."""
    value = os.environ.get(name, "")
    if required and not value:
        print(f"::error::Environment variable {name} is not set.")
        sys.exit(1)
    return value


def github_api_get(url: str, token: str) -> dict | list:
    """Perform a GET request against the GitHub API with pagination support."""
    import requests

    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github.v3+json",
    }
    results: list = []
    while url:
        resp = requests.get(url, headers=headers, timeout=30)
        resp.raise_for_status()
        data = resp.json()
        if isinstance(data, list):
            results.extend(data)
            # Handle pagination via Link header
            url = None
            if "Link" in resp.headers:
                for part in resp.headers["Link"].split(","):
                    if 'rel="next"' in part:
                        url = part.split(";")[0].strip().strip("<>")
        else:
            return data
    return results


def get_pr_changed_files(repo: str, pr_number: str, token: str) -> list[dict]:
    """Fetch the list of files changed in a pull request."""
    url = f"https://api.github.com/repos/{repo}/pulls/{pr_number}/files?per_page=100"
    return github_api_get(url, token)


def collect_existing_docs(docs_dir: Path) -> dict[str, str]:
    """
    Walk the docs/ directory and return a mapping of
    relative path -> file content for all .md files.
    """
    docs: dict[str, str] = {}
    if not docs_dir.exists():
        return docs
    for md_file in sorted(docs_dir.rglob("*.md")):
        rel = md_file.relative_to(docs_dir.parent)  # e.g. docs/traits.md
        try:
            docs[str(rel)] = md_file.read_text(encoding="utf-8")
        except Exception as exc:
            print(f"::warning::Could not read {md_file}: {exc}")
    return docs


def read_file_safe(path: str) -> str:
    """Read a file from the repository checkout, return empty string on error."""
    try:
        return Path(path).read_text(encoding="utf-8")
    except Exception:
        return ""


def call_model(client: OpenAI, system_prompt: str, user_prompt: str) -> str:
    """Call the AI model with retry logic."""
    for attempt in range(1, MAX_RETRIES + 1):
        try:
            response = client.chat.completions.create(
                model=MODEL_NAME,
                messages=[
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_prompt},
                ],
                temperature=0.2,
            )
            return response.choices[0].message.content.strip()
        except Exception as exc:
            print(f"::warning::Model call attempt {attempt}/{MAX_RETRIES} failed: {exc}")
            if attempt < MAX_RETRIES:
                time.sleep(RETRY_DELAY_SECONDS * attempt)
            else:
                raise


def set_github_output(name: str, value: str) -> None:
    """Set a GitHub Actions output variable."""
    output_file = os.environ.get("GITHUB_OUTPUT", "")
    if output_file:
        with open(output_file, "a", encoding="utf-8") as f:
            f.write(f"{name}={value}\n")
    else:
        # Fallback for local testing
        print(f"OUTPUT: {name}={value}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    # Read environment
    api_key = get_env("MODELS_API_KEY")
    pr_number = get_env("PR_NUMBER")
    pr_title = get_env("PR_TITLE")
    pr_body = get_env("PR_BODY", required=False)
    token = get_env("GITHUB_TOKEN")
    repo = get_env("REPO")

    client = OpenAI(base_url=MODELS_ENDPOINT, api_key=api_key)

    # ── 1. Fetch changed files from the PR ──────────────────────────────
    print(f"Fetching changed files for PR #{pr_number} in {repo}...")
    changed_files = get_pr_changed_files(repo, pr_number, token)

    # Filter to only analyzable paths
    relevant_files = [
        f
        for f in changed_files
        if any(f["filename"].startswith(prefix) for prefix in ANALYZABLE_PREFIXES)
    ]

    if not relevant_files:
        print("No relevant source files changed. Skipping documentation analysis.")
        set_github_output("has_changes", "false")
        return

    print(f"Found {len(relevant_files)} relevant changed files.")

    # ── 2. Build a summary of changes ───────────────────────────────────
    changes_summary = []
    for f in relevant_files:
        file_info = {
            "filename": f["filename"],
            "status": f["status"],  # added, modified, removed, renamed
            "additions": f.get("additions", 0),
            "deletions": f.get("deletions", 0),
        }
        # Include the patch (diff) if available and not too large
        patch = f.get("patch", "")
        if len(patch) < 8000:
            file_info["patch"] = patch
        else:
            file_info["patch"] = patch[:8000] + "\n... (truncated)"

        # Read current file content for added/modified files
        if f["status"] in ("added", "modified"):
            content = read_file_safe(f["filename"])
            if len(content) < 12000:
                file_info["current_content"] = content
            else:
                file_info["current_content"] = content[:12000] + "\n... (truncated)"

        changes_summary.append(file_info)

    # ── 3. Collect existing documentation ───────────────────────────────
    docs_dir = Path("docs")
    existing_docs = collect_existing_docs(docs_dir)

    docs_listing = "No documentation files found in /docs yet."
    if existing_docs:
        parts = []
        for path, content in existing_docs.items():
            preview = content[:2000] + "\n... (truncated)" if len(content) > 2000 else content
            parts.append(f"### {path}\n```markdown\n{preview}\n```")
        docs_listing = "\n\n".join(parts)

    # ── 4. Build the prompt ─────────────────────────────────────────────
    system_prompt = """You are a documentation analyst for a public Rust genetic algorithms library.

Your job is to analyze code changes from a merged pull request and determine what
documentation files in the /docs directory need to be CREATED, UPDATED, or DELETED.

Rules:
- Only suggest documentation changes that are directly relevant to the code changes.
- Documentation lives in /docs as markdown files organized by topic.
- Each major module or feature should have its own documentation file.
- Think about what an external developer using this library would need to know.
- If no documentation changes are needed, return an empty actions array.
- Respond ONLY with valid JSON, no markdown fences, no extra text."""

    user_prompt = f"""## Pull Request Information
- **PR Number:** #{pr_number}
- **Title:** {pr_title}
- **Description:** {pr_body or "No description provided."}

## Changed Files
```json
{json.dumps(changes_summary, indent=2)}
```

## Existing Documentation in /docs
{docs_listing}

## Task
Analyze the code changes and produce a JSON plan with documentation actions.
Each action must specify:
- "action": one of "CREATE", "UPDATE", or "DELETE"
- "path": the documentation file path (e.g. "docs/genotypes/range.md")
- "reason": why this action is needed
- "relevant_source_files": list of source file paths related to this doc
- "key_points": list of key points to cover in the documentation

Respond with this exact JSON structure:
{{
  "pr_number": {pr_number},
  "pr_title": "{pr_title}",
  "actions": [
    {{
      "action": "CREATE | UPDATE | DELETE",
      "path": "docs/...",
      "reason": "...",
      "relevant_source_files": ["src/..."],
      "key_points": ["point 1", "point 2"]
    }}
  ]
}}"""

    # ── 5. Call the model ───────────────────────────────────────────────
    print("Calling AI model to analyze changes...")
    raw_response = call_model(client, system_prompt, user_prompt)

    # Strip markdown fences if the model wraps its response
    cleaned = raw_response.strip()
    if cleaned.startswith("```"):
        cleaned = "\n".join(cleaned.split("\n")[1:])
    if cleaned.endswith("```"):
        cleaned = "\n".join(cleaned.split("\n")[:-1])
    cleaned = cleaned.strip()

    # ── 6. Parse and validate ───────────────────────────────────────────
    try:
        plan = json.loads(cleaned)
    except json.JSONDecodeError as exc:
        print(f"::error::Failed to parse model response as JSON: {exc}")
        print(f"Raw response:\n{raw_response}")
        sys.exit(1)

    # Ensure required fields
    if "actions" not in plan:
        plan["actions"] = []
    plan.setdefault("pr_number", int(pr_number))
    plan.setdefault("pr_title", pr_title)

    actions = plan["actions"]
    if not actions:
        print("Analyst agent determined no documentation changes are needed.")
        set_github_output("has_changes", "false")
        return

    # ── 7. Save the plan ────────────────────────────────────────────────
    output_path = Path("doc-plan.json")
    output_path.write_text(json.dumps(plan, indent=2, ensure_ascii=False), encoding="utf-8")

    print(f"Documentation plan saved to {output_path}")
    print(f"  Actions: {len(actions)}")
    for action in actions:
        print(f"    - {action['action']} {action['path']}: {action['reason']}")

    set_github_output("has_changes", "true")


if __name__ == "__main__":
    main()
