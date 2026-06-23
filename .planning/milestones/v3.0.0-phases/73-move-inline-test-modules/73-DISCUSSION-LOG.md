# Phase 73: Move Inline #[cfg(test)] Modules to tests/ - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-18
**Phase:** 73-move-inline-test-modules
**Areas discussed:** Private item access, File placement

---

## Private item access

| Option | Description | Selected |
|--------|-------------|----------|
| Rewrite via public API | Drop direct helper tests; replace with equivalent coverage through public functions. No API surface changes. | ✓ |
| Promote helpers to pub | Mark pub(crate) helpers as pub. v3.0.0 allows breaking changes, but exposes implementation details. | |
| You decide | Agent picks the cleanest approach per file. | |

**User's choice:** Rewrite via public API

### Follow-up: How to handle helper tests with no clean public-API equivalent?

| Option | Description | Selected |
|--------|-------------|----------|
| Replace with equivalent public-API assertions | For each dropped helper test, write an equivalent test through the public function exercising the same invariant. | ✓ |
| Drop helper-only tests | Drop tests with no clean public-API equivalent. Less migration work, marginally less unit-level coverage. | |

**User's choice:** Replace with equivalent public-API assertions

---

## File placement

### Where should src/benchmarks/ tests land?

| Option | Description | Selected |
|--------|-------------|----------|
| tests/benchmarks/ mirrored subdir | Create tests/benchmarks/dtlz.rs etc. Mirrors src/ structure. | ✓ |
| Flat tests/test_benchmarks_*.rs | Flat naming following test_rng.rs, test_stats.rs pattern. Less consistent for deeper modules. | |

**User's choice:** Mirrored subdir

### Where should indicator tests land?

| Option | Description | Selected |
|--------|-------------|----------|
| tests/engines/multi_objective/indicators/*.rs | Full mirror of src/ path. Add indicators/ subdir inside existing tests/engines/multi_objective/. | ✓ |
| Inline into tests/engines/multi_objective/test_indicators.rs | Consolidate all 4 into one file. Fewer files, mixed concerns. | |

**User's choice:** tests/engines/multi_objective/indicators/*.rs

---

## Claude's Discretion

- Exact file name for AOS test file inside `tests/engines/aos/`
- Whether to merge local_search tests into existing `tests/engines/local_search.rs` or create a separate file
- Order of test functions within new test files

## Deferred Ideas

None — discussion stayed within phase scope.
