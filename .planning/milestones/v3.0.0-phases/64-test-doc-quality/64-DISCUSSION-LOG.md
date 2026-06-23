# Phase 64: Test & Doc Quality - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-10
**Phase:** 64-test-doc-quality
**Areas discussed:** Coverage tool, #[allow] policy, Doc example scope, Coverage approach

---

## Coverage Tool

| Option | Description | Selected |
|--------|-------------|----------|
| Install llvm-cov | Matches ROADMAP exactly. More accurate source-based coverage. cargo install + rustup component | ✓ |
| Use tarpaulin | Already installed (v0.32.0). Slightly less accurate but works without extra install | |
| Accept either in CI | Flexible fallback approach — adds complexity | |

**User's choice:** Install llvm-cov (Recommended)
**Notes:** Matches the ROADMAP wording exactly.

---

| Option | Description | Selected |
|--------|-------------|----------|
| GitHub Actions CI only | New step in ci.yml; fails if coverage drops below 80% | ✓ |
| Local script + CI | scripts/check-coverage.sh wrapper + CI step | |
| CI only, no hard fail | Informational only — does not block merging | |

**User's choice:** GitHub Actions CI only (Recommended)

---

| Option | Description | Selected |
|--------|-------------|----------|
| All features (--all-features) | Covers serde, visualization, observer-tracing, observer-metrics, benchmarks | ✓ |
| Core only | Faster, avoids font/ttf deps, but misses serde and observer paths | |
| Core + serde only | Most commonly-tested combination in current CI | |

**User's choice:** All features (Recommended)

---

## `#[allow]` Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Fix: make fields used or delete | Fixes root cause. May require small algorithmic changes | ✓ |
| Accept: reserved for future phases | Keep with doc comment explaining intent | |
| Audit each one individually | Per-field decision | |

**User's choice (dead_code):** Fix root cause (Recommended)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Introduce DeMutationParams struct | Groups DE mutation parameters. Removes suppress, improves readability | ✓ |
| Keep the suppress | DE mutation inherently has many params; struct adds indirection | |
| You decide | Claude picks based on actual signatures | |

**User's choice (too_many_arguments):** Introduce params struct (Recommended)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Remove deprecated items entirely | v3.0.0 allows it; Reporter removal is active requirement | ✓ |
| Keep deprecated, restructure allow | Feature-gate approach | |
| You decide per item | Claude judges each | |

**User's choice (deprecated):** Remove deprecated items entirely (Recommended)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Introduce type aliases | e.g. type FitnessFn<U> = Arc<dyn Fn(&U) -> f64 + Send + Sync> | ✓ |
| Keep the suppress | Type complexity is real; alias obscures what's happening | |
| You decide | Claude reads signatures and picks cleanest fix | |

**User's choice (type_complexity):** Introduce type aliases (Recommended)

---

## Doc Example Scope

| Option | Description | Selected |
|--------|-------------|----------|
| User-facing entry points only | pub fn, struct, trait, enum at module root. ~100-150 items | ✓ |
| Every pub fn and method | ~300-400 items including operator impls | |
| Literally every pub item | 684 items including variants and aliases | |

**User's choice:** User-facing entry points only (Recommended)
**Notes:** Excludes trait impl items, re-exports, enum variants, type aliases.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Use no_run with realistic snippets | Syntax-checked but not executed. Shows real usage | ✓ |
| Use ignore for complex items | Skips the example in cargo test --doc | |
| Write fully-runnable examples for all items | Every example compiles and runs | |

**User's choice:** Use `no_run` with realistic snippets (Recommended)

---

## Coverage Approach

| Option | Description | Selected |
|--------|-------------|----------|
| tests/ directory only | All new tests follow existing pattern. Zero inline tests in src/ | ✓ |
| Mixed inline + tests/ | Inline for unit-level gaps, tests/ for integration | |

**User's choice:** tests/ directory only (Recommended)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Exclude wasm-gated branches | llvm-cov exclusion patterns / #[coverage(off)]. WASM tested separately | ✓ |
| Count them as gaps | Add native tests for non-wasm paths | |
| You decide | Claude picks most practical approach | |

**User's choice:** Exclude wasm-gated branches (Recommended)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Run llvm-cov first, fix lowest modules | Data-driven. Generate baseline, rank by coverage, fix bottom | ✓ |
| Focus on new engines (ALPS, Cellular, Scatter, DE, CMA) | Target v2.3-v3.0 additions first | |
| Focus on src/operations/ operators | Edge cases are easy to miss here | |

**User's choice:** Data-driven from baseline report (Recommended)

---

## Claude's Discretion

- Exact llvm-cov exclusion pattern syntax for WASM branches
- Whether CI coverage step is a new job or extends existing cargo test job
- Exact `DeMutationParams` field names
- `#[allow(clippy::should_implement_trait)]`, `unused_variables`, `unused_mut` — fix approach per item
- Which specific tests close the biggest coverage gaps (determined post-baseline)

## Deferred Ideas

None — discussion stayed within phase scope.
