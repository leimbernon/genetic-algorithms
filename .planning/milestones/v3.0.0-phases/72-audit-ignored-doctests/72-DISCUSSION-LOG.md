# Phase 72: Audit and Fix Ignored Doctests - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-18
**Phase:** 72-audit-ignored-doctests
**Areas discussed:** Doctest restoration strategy

---

## Doctest restoration strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Compile-only for heavy tests | Keep `# ignore` only for tests needing external resources (GPU, network); convert long-running ones to compile-only | ✓ |
| Remove all ignores | Make ALL 29 doctests run fully — no exceptions. Accept longer test times. | |
| You decide | Agent picks the pragmatic approach for each individual doctest | |

**User's choice:** Compile-only for heavy tests
**Notes:** Doctests that need external resources or have long runtime are converted to `no_run` with a comment. All others run fully.

---

## Deferred Ideas

None — discussion stayed within phase scope.
