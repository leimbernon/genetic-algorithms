# Phase 64: Test & Doc Quality - Context

**Gathered:** 2026-06-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 64 adds line-coverage measurement (≥80% for `src/engines/` and `src/operations/`) to GitHub Actions CI, eliminates all `#[allow(...)]` suppressions from non-generated source files by fixing their root causes, and adds rustdoc `# Examples` blocks to all user-facing public entry points. No new features, no API additions.

**Out of scope:** Doc examples for trait impl items, re-exported type aliases, enum variants, or private items. WASM-branch coverage (tested separately via `cargo check --target wasm32`). Inline tests in `src/` — all new tests go in `tests/`.

</domain>

<decisions>
## Implementation Decisions

### Coverage Tool

- **D-01:** Use `cargo llvm-cov`. Install before execution: `cargo install cargo-llvm-cov` + `rustup component add llvm-tools-preview`. Matches ROADMAP spec exactly.
- **D-02:** Coverage gate lives in **GitHub Actions CI only** — a new step (or updated `ci.yml`) that fails the build if `src/engines/` or `src/operations/` line coverage drops below 80%.
- **D-03:** Coverage run uses `--all-features` (covers serde, visualization, observer-tracing, observer-metrics, benchmarks paths).
- **D-04:** WASM-gated branches (`#[cfg(not(target_arch = "wasm32"))]`) are **excluded from the 80% target** via llvm-cov exclusion patterns or `#[coverage(off)]`. WASM correctness is verified separately by the existing WASM CI check.
- **D-05:** Coverage additions are **data-driven**: generate a baseline `cargo llvm-cov --all-features` report first, rank modules by coverage, then write tests for the lowest-coverage modules. No guessing where gaps are.

### `#[allow(...)]` Suppressions

All 23 suppressions are fixed by addressing root causes:

- **D-06:** `#[allow(dead_code)]` on struct fields in engine modules (sms_emoa, ibea, moead, spea2, cma, ga.rs): fix root cause — make fields used in the algorithm or delete them if genuinely unused.
- **D-07:** `#[allow(clippy::too_many_arguments)]` on DE mutation functions: introduce a `DeMutationParams` struct grouping the mutation parameters (F, CR, strategy, bounds, etc.). Removes the suppression and improves readability.
- **D-08:** `#[allow(deprecated)]` in cellular/alps configs and ga.rs (referencing `Reporter<U>`): **remove the deprecated `Reporter` code entirely** — v3.0.0 is a major version break and `Reporter` removal is an active requirement (ARCH-03, already complete per REQUIREMENTS.md). The allows disappear with the code.
- **D-09:** `#[allow(clippy::type_complexity)]` in ga.rs complex function signatures: introduce type aliases (e.g., `type FitnessFn<U> = Arc<dyn Fn(&U) -> f64 + Send + Sync>`).
- **D-10:** Remaining suppressions (`#[allow(clippy::should_implement_trait)]` on CompositeObserver, `#[allow(unused_variables)]` in mutation.rs, `#[allow(unused_mut)]` in ga.rs tests): Claude fixes root cause for each — rename to avoid trait conflict, use `_var` pattern, remove the mut declaration.

### Doc Example Scope

- **D-11:** **Scope = user-facing entry points only**: `pub fn`, `pub struct`, `pub trait`, `pub enum` at module root that users call directly. Excludes: trait impl items, internal re-exports, enum variants, type aliases, and `pub(crate)` items. Target: ~100–150 items.
- **D-12:** Complex items requiring full GA configuration (most of the library) use **```` ```rust,no_run ```` annotation** — syntax-checked by `cargo test --doc` but not executed. Shows real usage without requiring a full runtime setup in each doc test.
- **D-13:** Simple leaf items (gene types, error variants, small utilities) use **fully-runnable** ```` ```rust ```` examples where the setup is concise.

### New Tests

- **D-14:** All new tests go in `tests/` directory only, following the existing `tests/test_*.rs` pattern. Zero inline `#[cfg(test)] mod tests` in `src/`.

### Claude's Discretion

- Exact llvm-cov exclusion pattern syntax for WASM branches (e.g., `--exclude-files` glob vs `#[coverage(off)]` attribute)
- Whether CI coverage step runs in a separate job or extends the existing `cargo test` job
- Exact `DeMutationParams` field names and which DE functions share the struct
- Which specific tests close the biggest coverage gaps (determined from baseline report)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active Requirements
- `.planning/REQUIREMENTS.md` §ARCH — ARCH-03 (Reporter removal, already complete) applies to D-08

### Roadmap
- `.planning/ROADMAP.md` §Phase 64 — Goal and success criteria (coverage ≥80%, zero allows, every pub item doc example)

### Existing Source
- `src/engines/` — coverage target; contains sms_emoa, ibea, moead, spea2, cma, de, cellular, alps, ga, island, nsga2, nsga3, scatter
- `src/operations/` — coverage target; contains crossover/, mutation/, selection/, survivor/, extension/
- `src/engines/de/mutation.rs` — the `#[allow(clippy::too_many_arguments)]` functions (D-07)
- `src/engines/ga.rs` — deprecated allow (D-08), type_complexity (D-09), dead_code (D-06)
- `src/observe/observer/composite.rs` — `#[allow(clippy::should_implement_trait)]` (D-10)
- `src/operations/mutation.rs` — `#[allow(unused_variables)]` (D-10)

### CI
- `.github/workflows/` — existing CI configuration; new coverage gate goes here

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tests/structures.rs` — shared test chromosome types; new tests should reuse these helpers
- `cargo-tarpaulin` is installed (v0.32.0) — available as fallback if llvm-cov has issues
- Existing `tests/test_*.rs` files show the pattern for new test files

### Established Patterns
- Test file naming: `tests/test_<module>.rs` for new modules; sub-tests via `mod` inside if the module is large
- Seeded RNG via `.with_seed(42)` for deterministic tests
- All CI gates must pass: `cargo test`, `cargo test --features serde`, `cargo clippy`, `cargo doc --no-deps`

### Integration Points
- New CI step integrates into `.github/workflows/` — likely extend `ci.yml` or add `coverage.yml`
- `#[allow(deprecated)]` removal in cellular/alps configs touches builder methods — verify backwards compatibility within milestone branch (breaking changes accepted in v3.0.0)

</code_context>

<specifics>
## Specific Ideas

- No specific references mentioned in discussion — open to standard approaches for coverage reporting and doc example patterns

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 64-test-doc-quality*
*Context gathered: 2026-06-10*
