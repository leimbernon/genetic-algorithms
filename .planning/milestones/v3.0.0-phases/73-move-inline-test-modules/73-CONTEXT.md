# Phase 73: Move Inline #[cfg(test)] Modules to tests/ - Context

**Gathered:** 2026-06-18
**Status:** Ready for planning

<domain>
## Phase Boundary

All 10 `#[cfg(test)] mod tests { ... }` blocks across `src/` are migrated to corresponding files under `tests/`, so that `grep -rn '#\[cfg(test)\]' src/` returns zero matches with no coverage regression.

**Files to migrate:**
- `src/aos.rs`
- `src/benchmarks/dtlz.rs`
- `src/benchmarks/single_objective.rs`
- `src/benchmarks/zdt.rs`
- `src/engines/multi_objective/indicators/generational_distance.rs`
- `src/engines/multi_objective/indicators/hypervolume.rs`
- `src/engines/multi_objective/indicators/inverted_generational_distance.rs`
- `src/engines/multi_objective/indicators/spread.rs`
- `src/operations/local_search.rs`
- `src/operations/mutation/levy_flight.rs`

**Out of scope:**
- Writing new tests for untested items
- Changing any source code logic
- Promoting `pub(crate)` helpers to `pub`

</domain>

<decisions>
## Implementation Decisions

### Private item access (5 files use `use super::*;` for pub(crate) helpers)
- **D-01:** Tests that access `pub(crate)` helpers (`validate_non_empty`, `validate_dimension_consistency`, `nearest_distance`, `squared_euclidean_distance`, etc.) are rewritten to exercise the same behavior through the public API. No visibility promotions.
- **D-02:** For each helper test that is dropped, write an equivalent public-API assertion that exercises the same invariant. Example: instead of `validate_non_empty("gd", &[]) == Err(...)`, assert that `generational_distance(&[], &front)` returns the expected error. Same coverage, different entry point.
- **D-03:** `src/operations/local_search.rs` and `src/operations/mutation/levy_flight.rs` also use `use super::*;` — apply the same rewrite strategy.

### File placement (where migrated tests land in tests/)
- **D-04:** Follow the mirrored-subdirectory pattern for all files (consistent with `tests/engines/` mirroring `src/engines/`). New subdirs are created as needed.
- **D-05:** `src/benchmarks/dtlz.rs`, `zdt.rs`, `single_objective.rs` → `tests/benchmarks/dtlz.rs`, `tests/benchmarks/zdt.rs`, `tests/benchmarks/single_objective.rs` (new `tests/benchmarks/` directory).
- **D-06:** `src/engines/multi_objective/indicators/*.rs` → `tests/engines/multi_objective/indicators/generational_distance.rs`, etc. (new `tests/engines/multi_objective/indicators/` directory).
- **D-07:** `src/aos.rs` → `tests/engines/aos/` (directory already exists; add a new test file, e.g., `tests/engines/aos/test_aos_strategy.rs`).
- **D-08:** `src/operations/local_search.rs` → `tests/engines/local_search.rs` already exists; merge or add alongside.
- **D-09:** `src/operations/mutation/levy_flight.rs` → `tests/operations/test_mutation_levy_flight.rs` (inside existing `tests/operations/` dir, following its `test_mutation_*.rs` naming).

### Serde-gated nested tests
- **D-10:** Files with nested `#[cfg(feature = "serde")]` blocks inside `#[cfg(test)]` (benchmarks: dtlz, zdt, single_objective) carry the feature gate over: wrap those test functions in `#[cfg(feature = "serde")]` at the test-file level.

### Claude's Discretion
- Exact file name for the AOS test file inside `tests/engines/aos/` (e.g., `strategy.rs`, `test_aos_strategy.rs`).
- Whether to merge `src/operations/local_search.rs` tests into the existing `tests/engines/local_search.rs` or create a separate file.
- Order of test functions within the new test files (mirror source order is fine).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source files being migrated
- `src/aos.rs` — AOS inline tests (uses `use super::*;`; AosStrategy variants are public, but internal reward state may not be)
- `src/benchmarks/dtlz.rs` — DTLZ benchmark inline tests (uses `use super::*;`, nested serde block)
- `src/benchmarks/single_objective.rs` — Single-objective benchmark inline tests (uses `use super::*;`, nested serde block)
- `src/benchmarks/zdt.rs` — ZDT benchmark inline tests (uses `use super::*;`, nested serde block)
- `src/engines/multi_objective/indicators/generational_distance.rs` — Uses `pub(crate)` `nearest_distance`, `validate_*`
- `src/engines/multi_objective/indicators/hypervolume.rs` — Uses `pub(crate)` `validate_non_empty`, `validate_dimension`
- `src/engines/multi_objective/indicators/inverted_generational_distance.rs` — Uses `pub(crate)` helpers
- `src/engines/multi_objective/indicators/spread.rs` — Uses `pub(crate)` `squared_euclidean_distance`
- `src/operations/local_search.rs` — Uses `use super::*;`
- `src/operations/mutation/levy_flight.rs` — Uses `use super::*;`

### Existing tests/ structure to follow
- `tests/engines/` — Mirrored engine subdirectory pattern
- `tests/engines/local_search.rs` — Existing local_search tests (check for duplication before merging)
- `tests/operations/` — Existing mutation test naming: `test_mutation_*.rs`

### Validation commands
- `cargo test` — Must pass with same test count before and after
- `grep -rn '#\[cfg(test)\]' src/` — Must return zero matches when done

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tests/engines/multi_objective/` — already exists; add `indicators/` subdir
- `tests/engines/aos/` — already exists as a directory; add the new test file inside
- `tests/operations/` — already exists with `test_mutation_*.rs` pattern

### Established Patterns
- Integration tests import via `use genetic_algorithms::...` (no `use super::*;` available)
- Serde feature gates in test files: `#[cfg(feature = "serde")]` wrapping test functions
- Test file naming under `tests/operations/`: `test_mutation_<name>.rs`

### Integration Points
- `Cargo.toml` `[[test]]` sections: no explicit declarations needed — Cargo auto-discovers `tests/*.rs` and `tests/**/mod.rs`
- New subdirectory test files need a `mod.rs` or be listed in a parent `mod.rs` if using the module approach; otherwise use `tests/benchmarks/dtlz.rs` as a standalone integration test file (Cargo discovers `tests/**/*.rs` directly in Rust 2018+)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — mechanical migration following the decisions above.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 73-move-inline-test-modules*
*Context gathered: 2026-06-18*
