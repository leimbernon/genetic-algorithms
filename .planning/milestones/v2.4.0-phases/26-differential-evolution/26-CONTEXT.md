# Phase 26: Differential Evolution Engine - Context

**Gathered:** 2026-04-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 26 closes the remaining gaps on the Differential Evolution engine that was stubbed in Phase 25. The engine implementation (5 mutation strategies, 2 crossover modes, JADE, L-SHADE) and its 11 integration tests are already in place and passing. This phase delivers:

1. **DE-07 gap**: Add a DE-vs-standard-GA convergence comparison benchmark to `benches/de.rs`
2. **Observer integration**: Wire `Option<Arc<dyn GaObserver<U>>>` into `DeEngine` — reusing the existing `GaObserver<U>` trait with full parity (on_start, on_generation_complete, on_new_best, on_finish)
3. **Quality gate**: `cargo clippy` zero warnings, `cargo doc --no-deps` zero rustdoc warnings, all tests passing

</domain>

<decisions>
## Implementation Decisions

### Phase Scope
- **D-01:** Phase 26 is a gap-closure phase. Do NOT rewrite or restructure the existing DE implementation. Audit requirements DE-01–DE-07, identify what's missing, fill only those gaps.

### Benchmark (DE-07)
- **D-02:** Add a new benchmark group to `benches/de.rs` that runs `DeEngine` (Rand1/Binomial) and the standard `Ga` engine on the same sphere function. The output comparison should make it clear whether DE or GA converges faster. A separate benchmark file is NOT needed.

### Bounds Clamping
- **D-03:** No clamping added to the engine. It is the user's responsibility via `DeGene::with_de_value()`. Do not add `with_bounds_clamping()` to `DeConfiguration`.

### Observer Integration
- **D-04:** `DeEngine` should accept `Option<Arc<dyn GaObserver<U>>>` — the same interface as the standard `Ga` engine. No new `DeObserver` sub-trait is needed.
- **D-05:** Observer hooks to call (full parity with Ga engine): `on_start`, `on_generation_complete`, `on_new_best`, `on_finish`. Pass `GenerationStats` to `on_generation_complete`.

### Claude's Discretion
- How to build `GenerationStats` from the DE run loop (population size, best fitness, generation index, diversity estimation) — follow whatever pattern is simplest given existing `GenerationStats` fields
- Whether to update `tests/test_de.rs` with observer smoke tests or create a separate `tests/test_de_observer.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing DE Engine
- `src/engines/de/mod.rs` — public exports
- `src/engines/de/engine.rs` — `DeEngine` struct and `run()` loop (where observer hooks go)
- `src/engines/de/configuration.rs` — `DeConfiguration`, `DeAdaptive`, `DeMutationStrategy`, `DeCrossoverMode`
- `src/engines/de/mutation.rs` — `JadeState`, `LShadeState`, mutation strategies
- `src/engines/de/crossover.rs` — binomial/exponential crossover
- `src/engines/de/gene.rs` — `DeGene` trait

### Observer System
- `src/traits/` — `GaObserver<U>` trait definition (find exact file)
- `src/engines/ga.rs` — reference for how observer is wired in the standard Ga engine (on_start, on_generation_complete, on_new_best, on_finish call sites)

### Tests & Benchmarks
- `tests/test_de.rs` — existing 11 tests; may need observer smoke test added
- `benches/de.rs` — benchmark to extend with DE-vs-GA comparison

### Requirements
- `.planning/REQUIREMENTS.md` — DE-01 through DE-07 (all must be satisfied)

### Project Patterns
- `CLAUDE.md` — tests in `tests/`, never inline; use `target=` in log macros

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `GaObserver<U>` trait: already has all required hooks; `DeEngine` should mirror how `ga.rs` wraps it in `Option<Arc<dyn GaObserver<U>>>`
- `GenerationStats`: already defined; reuse for `on_generation_complete` payload
- Standard `Ga` engine benchmark in `benches/ga_run.rs`: reference for the DE-vs-GA comparison structure

### Established Patterns
- Observer wired as `Option<Arc<...>>`: zero overhead when None; consistent with existing engines
- `DeConfiguration` builder pattern is already in place — no new builder methods needed for this phase
- Tests: `tests/test_de.rs` file exists; add observer tests there or in a new `tests/test_de_observer.rs`

### Integration Points
- `src/lib.rs`: `pub mod de;` already present — no lib.rs changes needed
- `Cargo.toml`: `[[bench]] name = "de"` already registered — only extend the existing bench file

</code_context>

<specifics>
## Specific Ideas

- Benchmark comparison: run DeEngine(Rand1/Binomial, 100 gen) vs Ga engine on sphere(5d, pop=30) — print wall-clock time per 100 generations side by side
- Observer: model the `DeEngine::new_with_observer(config, init_fn, fitness_fn, observer)` constructor after the existing Ga engine pattern

</specifics>

<deferred>
## Deferred Ideas

- `DeObserver<U>` sub-trait with JADE/L-SHADE-specific hooks (on_mutation_factor_update) — future phase if needed
- Bounds clamping via `DeConfiguration::with_bounds_clamping(lo, hi)` — future phase
- Scatter Search shared utilities — Phase 27's concern

</deferred>

---

*Phase: 26-differential-evolution*
*Context gathered: 2026-04-26*
