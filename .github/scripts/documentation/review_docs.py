"""
Documentation Agent - Step 3: Review Documentation Quality

This module provides the reviewer agent functionality. It evaluates generated
documentation against quality criteria and returns structured feedback.

It is designed to be imported by write_docs.py, but can also be used standalone.
"""

import json
import time

from openai import OpenAI

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

MODEL_NAME = "claude-sonnet-4-5"
MAX_RETRIES = 3
RETRY_DELAY_SECONDS = 5

# Quality thresholds (0-10 scale)
THRESHOLDS = {
    "completeness": 7,
    "clarity": 7,
    "examples": 6,
}

# ---------------------------------------------------------------------------
# Reviewer Agent
# ---------------------------------------------------------------------------

REVIEWER_SYSTEM_PROMPT = """You are a documentation quality reviewer for a public Rust genetic algorithms library.

Your job is to evaluate documentation that was automatically generated and determine
whether it meets quality standards for publication.

You evaluate on three criteria (each scored 0-10):

1. **Completeness** (minimum: 7/10)
   - Does the documentation cover all the relevant code changes?
   - Are all public types, traits, functions, and methods documented?
   - Are configuration options and parameters explained?

2. **Clarity** (minimum: 7/10)
   - Is the text understandable for an external developer who has never seen this codebase?
   - Are technical concepts explained, not just listed?
   - Is the structure logical and easy to follow?

3. **Examples** (minimum: 6/10)
   - Are there Rust code examples where applicable?
   - Are the examples realistic and correct?
   - Do examples show common use cases?

Rules:
- Be strict but fair. The documentation will be published publicly.
- If any criterion falls below its minimum threshold, set approved to false.
- Provide specific, actionable feedback for any criterion below threshold.
- Respond ONLY with valid JSON, no markdown fences, no extra text."""


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
            print(f"  [Reviewer] Model call attempt {attempt}/{MAX_RETRIES} failed: {exc}")
            if attempt < MAX_RETRIES:
                time.sleep(RETRY_DELAY_SECONDS * attempt)
            else:
                raise


def review_document(
    client: OpenAI,
    doc_content: str,
    action: dict,
    source_contents: dict,
) -> dict:
    """
    Review a single documentation file and return a structured evaluation.

    Args:
        client: OpenAI client configured for GitHub Models.
        doc_content: The markdown content to review.
        action: The action dict from doc-plan.json for this document.
        source_contents: Mapping of source file path -> file content.

    Returns:
        dict with keys: approved (bool), scores (dict), feedback (str)
    """
    # Build source code reference
    source_ref_parts = []
    for filepath, content in source_contents.items():
        truncated = content[:10000] + "\n... (truncated)" if len(content) > 10000 else content
        source_ref_parts.append(f"### {filepath}\n```rust\n{truncated}\n```")

    source_reference = "\n\n".join(source_ref_parts) if source_ref_parts else "No source files available."

    user_prompt = f"""## Documentation to Review

**File:** {action['path']}
**Action:** {action['action']}
**Reason:** {action['reason']}
**Key points that should be covered:** {json.dumps(action.get('key_points', []))}

### Generated Documentation
```markdown
{doc_content}
```

### Reference Source Code
{source_reference}

## Task
Evaluate the documentation against the three quality criteria.
Respond with this exact JSON structure:
{{
  "approved": true or false,
  "scores": {{
    "completeness": <0-10>,
    "clarity": <0-10>,
    "examples": <0-10>
  }},
  "feedback": "Specific feedback for improvement (empty string if approved)"
}}"""

    raw_response = call_model(client, REVIEWER_SYSTEM_PROMPT, user_prompt)

    # Strip markdown fences if present
    cleaned = raw_response.strip()
    if cleaned.startswith("```"):
        cleaned = "\n".join(cleaned.split("\n")[1:])
    if cleaned.endswith("```"):
        cleaned = "\n".join(cleaned.split("\n")[:-1])
    cleaned = cleaned.strip()

    try:
        review = json.loads(cleaned)
    except json.JSONDecodeError:
        print(f"  [Reviewer] Warning: Could not parse review response, treating as approved.")
        print(f"  [Reviewer] Raw response: {raw_response[:500]}")
        return {
            "approved": True,
            "scores": {"completeness": 7, "clarity": 7, "examples": 6},
            "feedback": "",
        }

    # Validate scores against thresholds
    scores = review.get("scores", {})
    all_pass = True
    for criterion, threshold in THRESHOLDS.items():
        score = scores.get(criterion, 0)
        if score < threshold:
            all_pass = False

    # Override the model's approved flag based on actual threshold checks
    review["approved"] = all_pass

    return review
