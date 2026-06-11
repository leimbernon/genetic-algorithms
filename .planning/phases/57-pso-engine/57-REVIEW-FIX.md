---
phase: 57
fixed_at: 2026-06-03T00:00:00Z
review_path: .planning/phases/57-pso-engine/57-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 57: Code Review Fix Report

**Fixed at:** 2026-06-03
**Source review:** .planning/phases/57-pso-engine/57-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6 (CR-01, CR-02, WR-01, WR-02, WR-03, WR-04)
- Fixed: 6 findings across 7 commits (WR-03 required an additional test-update commit)
- Skipped: 0

## Fixed Issues

### CR-01: `best` chromosome in result may not match `gbest_position`

**Files modified:** `src/engines/pso/engine.rs`
**Commit:** dc14bcc
**Applied fix:** Added `gbest_owner: usize` field to `PsoState` (initialized to `best_idx` in `PsoState::new`). Updated the synchronous gbest-update pass to set `state.gbest_owner = j` whenever `gbest_fitness` improves. Replaced the `find_best(&pop)` call in the best-update block with a reconstruction: clone `pop[owner]`, set its DNA to `gbest_position` via `Cow::Owned`, and set its fitness to `gbest_fitness`. This ensures `result.best` and `result.best_fitness` always refer to the same particle — the one whose pbest actually is the global best.

### CR-02: Empty-population guard both logs "returning empty result" and then panics

**Files modified:** `src/engines/pso/engine.rs`
**Commit:** dc14bcc
**Applied fix:** Removed the misleading `log::warn!("returning empty result")` line and the premature `self.notify(|obs| obs.on_run_end(...))` observer call that preceded the panic. The guard block now contains only `panic!("PsoEngine: init_fn returned an empty population")` — consistent message and behavior.

### WR-01: `Range<f64>::bounds()` returns only the first range tuple

**Files modified:** `src/traits/real_gene.rs`
**Commit:** 41b70c7
**Applied fix:** Added a doc comment to the `Range<f64>` implementation of `bounds()` clarifying: "Returns the first `(lo, hi)` range entry for this gene. For genes constructed with multiple range entries, only the first entry is used. The caller is responsible for constructing such genes with a representative range in position 0." Implementation unchanged (Option A from review).

### WR-02: `Ring::neighborhood_size` documentation contradicts implementation

**Files modified:** `src/engines/pso/configuration.rs`
**Commit:** 2a2a057
**Applied fix:** Replaced the incorrect doc comment on `Ring::neighborhood_size` ("Number of neighbors on each side to include", implying `2*k+1` total) with the correct description matching the engine's `lbest_position` implementation: `k` total neighbors split `floor(k/2)` left and `ceil(k/2)` right, giving `k+1` particles total including `i` itself. Added a concrete example: `neighborhood_size = 2` gives `{ i-1, i, i+1 }`. Updated the `Ring` variant doc to use the correct common values (2 and 4 instead of 3 and 5).

### WR-03: `inertia_weight` exported as public API unnecessarily

**Files modified:** `src/engines/pso/configuration.rs`, `src/engines/pso/mod.rs`, `tests/engines/pso/test_pso.rs`
**Commits:** 2a2a057 (configuration + mod), b87ad70 (test update)
**Applied fix:** Changed `pub fn inertia_weight` to `pub(crate) fn inertia_weight` in `configuration.rs`. Removed `inertia_weight` from the `pub use configuration::...` re-export in `mod.rs`. The function is no longer part of the `genetic_algorithms::pso::` public surface. Because the integration test `tests/engines/pso/test_pso.rs` was importing and calling `inertia_weight` directly for PSO-09, an additional commit updated the test to use a local `inertia_weight_test` helper that replicates the same arithmetic, keeping all 11 PSO tests green.

### WR-04: Example `init_population` mutates global RNG seed as a side effect

**Files modified:** `examples/pso_rastrigin.rs`
**Commit:** bd604a7
**Applied fix:** Removed `rng::set_seed(Some(99))` from inside `init_population()`. Added `rng::set_seed(Some(99));` as the first statement in `main()`. The seed is now set exactly once before engine construction, eliminating the hidden side effect that would reset the global RNG counter on every invocation of the init function.

## Skipped Issues

None — all in-scope findings were successfully fixed.

---

**Post-fix verification:**
- `cargo test --test test_pso`: 11 passed
- `cargo clippy --all-targets -- -D warnings`: no issues
- `cargo check --target wasm32-unknown-unknown`: clean

_Fixed: 2026-06-03_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
