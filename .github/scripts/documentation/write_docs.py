"""
Documentation Agent - Step 2: Write Documentation

This script reads the documentation plan (doc-plan.json) produced by the
analyst agent, generates or updates markdown files in /docs, and uses the
reviewer agent to validate quality. If the reviewer rejects, the writer
retries with feedback (up to 2 iterations per document).
"""

import json
import os
import sys
import time
from pathlib import Path

from openai import OpenAI

# Import the reviewer module (same directory)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from review_docs import review_document  # noqa: E402

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

MODELS_ENDPOINT = "https://models.inference.ai.azure.com"
MODEL_NAME = "claude-sonnet-4-5"
MAX_RETRIES = 3
RETRY_DELAY_SECONDS = 5
MAX_REVIEW_ITERATIONS = 2

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
                temperature=0.3,
            )
            return response.choices[0].message.content.strip()
        except Exception as exc:
            print(f"  [Writer] Model call attempt {attempt}/{MAX_RETRIES} failed: {exc}")
            if attempt < MAX_RETRIES:
                time.sleep(RETRY_DELAY_SECONDS * attempt)
            else:
                raise


# ---------------------------------------------------------------------------
# Writer Agent
# ---------------------------------------------------------------------------

WRITER_SYSTEM_PROMPT = """You are a technical documentation writer for a public Rust genetic algorithms library.

Your job is to write clear, accurate, and helpful documentation in Markdown format.

Guidelines:
- Write for an external developer who is using this library for the first time.
- Include a brief overview/introduction at the top of each document.
- Document all public types, traits, functions, and their parameters.
- Include Rust code examples that are realistic and demonstrate common use cases.
- Use proper markdown formatting: headings, code blocks with ```rust, tables where appropriate.
- Keep the tone professional and concise.
- Do NOT include any meta-commentary about the documentation itself.
- Respond ONLY with the markdown content. No preamble, no closing remarks, just the document."""


def generate_document(
    client: OpenAI,
    action: dict,
    source_contents: dict,
    existing_content: str,
    feedback: str,
) -> str:
    """
    Call the writer agent to generate or update a documentation file.

    Args:
        client: OpenAI client configured for GitHub Models.
        action: The action dict from doc-plan.json.
        source_contents: Mapping of source file path -> file content.
        existing_content: Current content of the doc file (empty for CREATE).
        feedback: Reviewer feedback from a previous iteration (empty on first pass).

    Returns:
        The generated markdown content as a string.
    """
    # Build source code section
    source_parts = []
    for filepath, content in source_contents.items():
        truncated = content[:15000] + "\n... (truncated)" if len(content) > 15000 else content
        source_parts.append(f"### {filepath}\n```rust\n{truncated}\n```")

    source_section = "\n\n".join(source_parts) if source_parts else "No source files available."

    # Build the user prompt
    prompt_parts = [
        f"## Task: {action['action']} documentation file",
        f"**Target file:** {action['path']}",
        f"**Reason:** {action['reason']}",
        f"**Key points to cover:** {json.dumps(action.get('key_points', []))}",
        "",
        "## Relevant Source Code",
        source_section,
    ]

    if existing_content and action["action"] == "UPDATE":
        prompt_parts.extend([
            "",
            "## Current Documentation (to be updated)",
            f"```markdown\n{existing_content}\n```",
        ])

    if feedback:
        prompt_parts.extend([
            "",
            "## Reviewer Feedback (address these issues)",
            feedback,
            "",
            "Please revise the documentation to address all the feedback above.",
        ])

    user_prompt = "\n".join(prompt_parts)

    return call_model(client, WRITER_SYSTEM_PROMPT, user_prompt)


# ---------------------------------------------------------------------------
# Main orchestration
# ---------------------------------------------------------------------------


def process_action(
    client: OpenAI,
    action: dict,
) -> None:
    """
    Process a single documentation action: write the document, review it,
    and retry if needed (up to MAX_REVIEW_ITERATIONS).
    """
    doc_path = Path(action["path"])
    action_type = action["action"]

    print(f"\n{'='*60}")
    print(f"Processing: {action_type} {doc_path}")
    print(f"Reason: {action['reason']}")
    print(f"{'='*60}")

    # Handle DELETE actions
    if action_type == "DELETE":
        if doc_path.exists():
            doc_path.unlink()
            print(f"  Deleted: {doc_path}")
        else:
            print(f"  File already does not exist: {doc_path}")
        return

    # Read relevant source files
    source_contents = {}
    for src_file in action.get("relevant_source_files", []):
        content = read_file_safe(src_file)
        if content:
            source_contents[src_file] = content

    # Read existing content (for UPDATE)
    existing_content = ""
    if action_type == "UPDATE" and doc_path.exists():
        existing_content = doc_path.read_text(encoding="utf-8")

    # Write + Review loop
    feedback = ""
    final_content = ""

    for iteration in range(1, MAX_REVIEW_ITERATIONS + 1):
        print(f"\n  --- Iteration {iteration}/{MAX_REVIEW_ITERATIONS} ---")

        # ── Writer Agent ────────────────────────────────────────────
        print(f"  [Writer] Generating documentation...")
        content = generate_document(
            client, action, source_contents, existing_content, feedback
        )

        # Strip markdown fences if the model wraps the entire response
        if content.startswith("```markdown"):
            content = "\n".join(content.split("\n")[1:])
        if content.startswith("```md"):
            content = "\n".join(content.split("\n")[1:])
        if content.endswith("```"):
            content = "\n".join(content.split("\n")[:-1])
        content = content.strip()

        final_content = content

        # Ensure parent directories exist and write the file
        doc_path.parent.mkdir(parents=True, exist_ok=True)
        doc_path.write_text(content + "\n", encoding="utf-8")
        print(f"  [Writer] Wrote {len(content)} chars to {doc_path}")

        # ── Reviewer Agent ──────────────────────────────────────────
        print(f"  [Reviewer] Evaluating documentation quality...")
        review = review_document(client, content, action, source_contents)

        scores = review.get("scores", {})
        approved = review.get("approved", False)
        feedback = review.get("feedback", "")

        print(f"  [Reviewer] Scores: completeness={scores.get('completeness', '?')}, "
              f"clarity={scores.get('clarity', '?')}, "
              f"examples={scores.get('examples', '?')}")
        print(f"  [Reviewer] Approved: {approved}")

        if approved:
            print(f"  Documentation approved!")
            break

        if iteration < MAX_REVIEW_ITERATIONS:
            print(f"  [Reviewer] Feedback: {feedback}")
            print(f"  Retrying with reviewer feedback...")
        else:
            print(f"  Max iterations reached. Accepting current version.")
            print(f"  [Reviewer] Final feedback (for manual review): {feedback}")

    # Ensure the final content is written
    doc_path.parent.mkdir(parents=True, exist_ok=True)
    doc_path.write_text(final_content + "\n", encoding="utf-8")
    print(f"  Final document saved: {doc_path}")


def main() -> None:
    # Read environment
    api_key = get_env("MODELS_API_KEY")

    client = OpenAI(base_url=MODELS_ENDPOINT, api_key=api_key)

    # Load the documentation plan
    plan_path = Path("doc-plan.json")
    if not plan_path.exists():
        print("::error::doc-plan.json not found.")
        sys.exit(1)

    plan = json.loads(plan_path.read_text(encoding="utf-8"))
    actions = plan.get("actions", [])

    if not actions:
        print("No documentation actions to process.")
        return

    pr_number = plan.get("pr_number", os.environ.get("PR_NUMBER", "unknown"))
    print(f"Documentation Writer Agent - Processing {len(actions)} actions for PR #{pr_number}")

    # Process each action
    for i, action in enumerate(actions, 1):
        print(f"\n[{i}/{len(actions)}] ", end="")
        process_action(client, action)

    print(f"\n{'='*60}")
    print(f"All {len(actions)} documentation actions processed.")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
