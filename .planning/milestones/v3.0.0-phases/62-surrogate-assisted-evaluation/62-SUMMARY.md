---
phase: 62-surrogate-assisted-evaluation
plan: "phase"
subsystem: fitness
tags: [surrogate, prescreening, trait, stats, wasm32, tests, example]

dependency_graph:
  requires:
    - phase: 60-batch-fitness-evaluation
      provides: BatchFitnessEvaluator pattern, FitnessCache pipeline, GenerationStats.cache_hits/cache_misses
    - phase: 61-performance-clone-reduction-parallel-survivor
      provides: parallel-survivor patterns, WASM-safe cfg gates
  provides:
    - SurrogateModel<U: ChromosomeT>: Send + Sync trait at crate root (src/fitness/surrogate.rs)
    - Ga::with_surrogate(model, prescreening_fraction) builder method
    - GenerationStats.true_fitness_calls: Option<u64> field
    - 12 integration tests in tests/test_surrogate.rs (11 active + 1 serde-gated)
    - Runnable example: examples/surrogate_rastrigin.rs
  affects:
    - Phase 63 (visualization): true_fitness_calls can be plotted per-generation
    - Phase 64 (doctests): SurrogateModel trait needs doctests
    - Future phases wiring surrogate to CmaEngine or IslandGa

tech-stack:
  added: []
  patterns:
    - "SurrogateModel<U> mirrors BatchFitnessEvaluator<U> trait layout (same file location, same Send+Sync bounds, same Arc<dyn> usage)"
    - "GenerationStats optional field pattern: Option<u64> with #[cfg_attr(feature = 'serde', serde(default))]"
    - "Sequential sort_unstable_by for WASM safety — no cfg gate needed, always sequential"
    - "NaN substitution to NEG_INFINITY before sort: if raw.is_nan() { f64::NEG_INFINITY } else { raw }"
    - "Index restoration after sort: second sort_unstable_by_key to restore original order"

key-files:
  created:
    - src/fitness/surrogate.rs
    - tests/test_surrogate.rs
    - examples/surrogate_rastrigin.rs
  modified:
    - src/fitness.rs
    - src/lib.rs
    - src/stats.rs
    - src/engines/ga.rs
    - src/engines/hill_climb/engine.rs
    - src/engines/permutate/engine.rs
    - Cargo.toml

key-decisions:
  - "D-01: SurrogateModel<U: ChromosomeT>: Send + Sync with single predict() method — training is user-managed"
  - "D-02: Trait in src/fitness/surrogate.rs, parallel to BatchFitnessEvaluator in src/fitness/batch.rs"
  - "D-03: Builder .with_surrogate(Arc<dyn SurrogateModel<U>>, prescreening_fraction: f64) on Ga"
  - "D-04: Rejected offspring dropped permanently — surrogate is a pure filter, not fitness predictor"
  - "D-05: Minimum floor max(1, floor(n * fraction)) ensures at least 1 offspring passes"
  - "D-06: Prescreening applies to offspring batch only — existing population never re-screened"
  - "D-07: Ga only in Phase 62 — CmaEngine and IslandGa support deferred"
  - "D-08: Pipeline order: surrogate → FitnessCache → BatchFitnessEvaluator/scalar fitness_fn"
  - "D-09: Surrogate + BatchFitnessEvaluator compose cleanly — no mutual exclusivity"
  - "D-10: GenerationStats.true_fitness_calls: Option<u64> — None when no surrogate, Some(n) post-prescreening count"
  - "D-11: GaObserver receives true_fitness_calls via existing on_generation_complete(stats: &GenerationStats) — no new method needed"

requirements-completed: []

duration: ~120 min (3 plans × ~40 min avg)
completed: "2026-06-09"
---

# Phase 62: Surrogate-Assisted Evaluation Summary

**SurrogateModel<U: ChromosomeT>: Send + Sync trait, Ga::with_surrogate() builder, and GenerationStats.true_fitness_calls field deliver offspring prescreening that reduces true fitness evaluations by up to 60% per generation, demonstrated on 10D Rastrigin with 12 green integration tests and a runnable example.**

## Phase Goal

Phase 62 delivers a first-class surrogate-assisted evaluation API for `Ga<U>`. Users attach a
`SurrogateModel<U>` to any `Ga` run to rank newly generated offspring by predicted fitness before
the expensive true evaluator runs. Only the top `prescreening_fraction` survive to true evaluation;
rejected offspring are dropped permanently. The true fitness call count per generation is tracked in
`GenerationStats.true_fitness_calls` and flows through `GaObserver` via the existing
`on_generation_complete` callback — no new observer methods required.

## Phase Outcome

All deliverables shipped across three plans:
- Plan 01: `SurrogateModel<U>` trait + `GenerationStats.true_fitness_calls` + Wave 0 tests
- Plan 02: `Ga::with_surrogate()` builder + prescreening hot-path + engine-runtime tests
- Plan 03: `examples/surrogate_rastrigin.rs` example + full CI gate run

The entire CI matrix is green. The example demonstrates 60% evaluation reduction (40
`true_fitness_calls` vs. 100 theoretical offspring per generation at `fraction=0.4`).

## Performance

- **Duration:** ~120 min total across 3 plans
- **Plans completed:** 3 of 3
- **Tasks completed:** 8 of 8
- **Files modified:** 9 total (7 in Plan 01, 2 in Plan 02, 2 in Plan 03)

## Decisions Implemented: D-01 through D-11

D-01: `src/fitness/surrogate.rs:76` — `pub trait SurrogateModel<U: ChromosomeT>: Send + Sync { fn predict(&self, chromosome: &U) -> f64; }`
D-02: `src/fitness/surrogate.rs` (new file) + `src/fitness.rs:24` `pub mod surrogate;` — parallel to `batch.rs`
D-03: `src/engines/ga.rs:1015` `pub fn with_surrogate(model, prescreening_fraction: f64) -> Self`
D-04: `src/engines/ga.rs:1801` rejected entries removed via `offspring = scores.into_iter().map(|(idx,_)| offspring[idx].clone()).collect()`
D-05: `src/engines/ga.rs:1797` `let keep = ((offspring.len() as f64 * fraction).floor() as usize).max(1)`
D-06: prescreening block at `src/engines/ga.rs:1781` fires on `offspring` Vec only (post-crossover+mutation, pre-merge)
D-07: `CmaEngine` and `IslandGa` have no `surrogate` field — explicitly deferred
D-08: `src/engines/ga.rs:1781-1806` surrogate block, then line 1808 `batch_evaluate(...)` — surrogate first in pipeline
D-09: `batch_evaluate` at line 1810 runs unconditionally after surrogate narrows `offspring` — no mutual exclusivity guard
D-10: `src/stats.rs:74` `pub true_fitness_calls: Option<u64>` with `#[cfg_attr(feature = "serde", serde(default))]`
D-11: `src/engines/ga.rs:2185` `gen_stats.true_fitness_calls = true_fitness_calls;` flows to observer via `GenerationStats`

## Public API Additions

| Addition | Location | Description |
|----------|----------|-------------|
| `SurrogateModel<U>` | `src/fitness/surrogate.rs`, re-exported from `src/lib.rs:343` | Cheap approximation oracle for prescreening |
| `Ga::with_surrogate()` | `src/engines/ga.rs:1015` | Builder method: takes `Arc<dyn SurrogateModel<U>>` and `f64` fraction |
| `GenerationStats.true_fitness_calls` | `src/stats.rs:74` | `Option<u64>` — None when no surrogate, Some(n) post-prescreening count |

## Tests Added

| File | Count | Test IDs |
|------|-------|---------|
| `tests/test_surrogate.rs` (Wave 0, Plan 01) | 4 (3 + 1 serde) | SC-1a (predict ordering), SC-1d (floor formula), SC-1g (NaN→NEG_INF), SC-2c (serde round-trip) |
| `tests/test_surrogate.rs` (engine runtime, Plan 02) | 8 | SC-1b (no surrogate → None), SC-1c (with surrogate → Some), SC-1e (boundary fraction=1.0), SC-1f (invalid fraction→Err), SC-2a (stat Some on run), SC-2b (stat None without surrogate), SC-3 (true_fitness_calls ≤ offspring_count) |
| **Total** | **12** | 11 active + 1 serde-gated |

## CI Gate Results

| Command | Result |
|---------|--------|
| `cargo test` | PASS — 56 lib + 389 engine + 13 surrogate + other = all passed, 0 failed |
| `cargo test --features serde` | PASS — 16 serde-gated tests pass (+ SC-2c); 0 failed |
| `cargo clippy --all-targets -- -D warnings` | PASS — no issues found |
| `cargo doc --no-deps 2>&1 \| grep -c '^warning:'` | PASS — 0 warnings |
| `cargo check --target wasm32-unknown-unknown` | PASS — no rayon, no Instant in prescreening path |
| `cargo run --example surrogate_rastrigin --release` | PASS — assertion holds (true_fitness_calls=40 < 100 per generation) |

## Out-of-Scope Confirmations

- **D-07: CmaEngine surrogate** — no `surrogate` field or prescreening in `src/engines/cma/`. Deferred.
- **D-07: IslandGa surrogate** — no surrogate support in `src/engines/island/`. Deferred.
- **Online surrogate learning** (`update` hook) — not added. Users implement via `Arc<Mutex<model>>` interior mutability.
- **Initial population prescreening** — generation 0 is not prescreened. Only per-generation offspring batches.

## Files Modified Summary (All Plans)

| File | Plans | What Changed |
|------|-------|--------------|
| `src/fitness/surrogate.rs` | 01 (created) | SurrogateModel<U> trait definition |
| `src/fitness.rs` | 01 | Added `pub mod surrogate; pub use surrogate::SurrogateModel;` |
| `src/lib.rs` | 01 | Added `pub use fitness::SurrogateModel;` at crate root |
| `src/stats.rs` | 01 | Added `true_fitness_calls: Option<u64>` field with serde(default) |
| `src/engines/hill_climb/engine.rs` | 01 | Added `true_fitness_calls: None` to direct GenerationStats constructor |
| `src/engines/permutate/engine.rs` | 01 | Added `true_fitness_calls: None` to direct GenerationStats constructor |
| `tests/test_surrogate.rs` | 01 (created), 02 (appended) | 12 integration tests across Wave 0 and engine-runtime categories |
| `src/engines/ga.rs` | 02 | surrogate field, with_surrogate() builder, build() validation, prescreening hot-path block, gen_stats assignment |
| `examples/surrogate_rastrigin.rs` | 03 (created) | End-to-end demonstration with LinearSurrogate and assertion |
| `Cargo.toml` | 03 | Added `[[example]] name = "surrogate_rastrigin"` |

## WASM Compatibility

The prescreening block uses `sort_unstable_by` unconditionally — no `par_iter` or `par_sort` in
the hot path. `cargo check --target wasm32-unknown-unknown` passes clean. The `SurrogateModel`
trait is WASM-safe: `predict(&self, chromosome: &U) -> f64` has no stdlib dependencies.

## Deviations from Plan

### Plan 01 Auto-fixed Issues

1. **[Rule 1 - Bug] Missing true_fitness_calls in hill_climb and permutate constructors** (commit 71c2aa1)
   - GenerationStats direct construction sites needed `true_fitness_calls: None`

2. **[Rule 1 - Bug] NanSurrogate used Cell<usize> which is !Sync** (commit 7ec1be0)
   - Replaced with AtomicUsize to satisfy SurrogateModel: Send + Sync bounds

3. **[Rule 1 - Bug] Comment in test matched #[ignore] acceptance grep** (commit 7ec1be0)
   - Changed comment text to avoid grep false positive

### Plan 02 Auto-fixed Issues

None. TDD cycle executed cleanly.

### Plan 03 Auto-fixed Issues

1. **[Rule 1 - Bug] LinearSurrogate.predict() used .dna() without LinearChromosome in scope** (commit 742db9f)
   - Fixed by importing LinearChromosome trait inline in impl block with `use genetic_algorithms::traits::LinearChromosome`

2. **[Rule 1 - Bug] Clippy unnecessary_map_or in test_surrogate.rs:438** (commit 73bf904)
   - Replaced `.map_or(false, |c| c <= 10)` with `.is_some_and(|c| c <= 10)` per clippy suggestion

## Hand-off Notes for Downstream Phases

- **Phase 63 (visualization):** `ga.stats()` now returns `true_fitness_calls: Option<u64>` per generation. Plotting this alongside `best_fitness` shows the cost-reduction trajectory of the surrogate over generations. The field is always `None` when no surrogate is configured, so downstream code must handle the `Option`.

- **Phase 64 (doctests):** The `SurrogateModel` trait in `src/fitness/surrogate.rs` has a `# Example` doctest block marked `rust,ignore`. A future phase may promote this to a runnable doctest by importing the concrete chromosome type. The `LinearSurrogate` in `examples/surrogate_rastrigin.rs` is a ready-made template.

- **Future surrogate phases (CmaEngine/IslandGa):** The prescreening block in `src/engines/ga.rs:1781–1806` is the canonical implementation reference. The `SurrogateModel<U>` trait is already at crate root; new engines only need to add the `surrogate: Option<(Arc<dyn SurrogateModel<U>>, f64)>` field, builder method, and the same prescreening block.

## Known Stubs

None. All 12 tests pass, the example runs, and no placeholder values remain in the created or modified files.

## Threat Flags

None. `SurrogateModel` is a pure in-memory trait — no network endpoints, file I/O, auth paths, or schema changes at trust boundaries introduced in Phase 62.

## Self-Check

- `src/fitness/surrogate.rs`: FOUND
- `src/stats.rs` (true_fitness_calls field): FOUND
- `src/engines/ga.rs` (with_surrogate builder): FOUND
- `src/engines/ga.rs` (prescreening block): FOUND
- `tests/test_surrogate.rs` (12 tests): FOUND
- `examples/surrogate_rastrigin.rs`: FOUND
- `Cargo.toml` (surrogate_rastrigin example entry): FOUND
- Plan 01 commits bb6b921, 71c2aa1, 7ec1be0: present on milestone/v3.0.0
- Plan 02 commits c39f7cc, e6bfbc4, 350faa7: present on milestone/v3.0.0
- Plan 03 commits 742db9f (example), 73bf904 (clippy fix): FOUND

CI gate matrix: cargo test PASS, cargo test --features serde PASS, clippy PASS, doc PASS, wasm32 PASS, example PASS.

---
*Phase: 62-surrogate-assisted-evaluation*
*Completed: 2026-06-09*
