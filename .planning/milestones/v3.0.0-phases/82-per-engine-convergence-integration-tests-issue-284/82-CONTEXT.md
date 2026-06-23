# Phase 82: Per-Engine Convergence Integration Tests (Issue #284) - Context

**Gathered:** 2026-06-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Add end-to-end convergence tests for every single-objective engine (DeEngine, ScatterEngine, CellularEngine, AlpsEngine, CmaEngine, PsoEngine) asserting each reaches a known optimum within tolerance. Prevents silent regressions in search dynamics.

**In scope:**
- Convergence test for DeEngine (Sphere)
- Convergence test for ScatterEngine (Sphere)
- Convergence test for CellularEngine (Sphere)
- Convergence test for AlpsEngine (Sphere)
- Convergence test for CmaEngine (Sphere, no restart)
- Convergence test for CmaEngine with IPOP restart path
- Convergence test for PsoEngine (Sphere)
- All tests use fixed RNG seed for determinism
- Tests placed under `tests/engines/<engine>/`
- `cargo test` and `cargo test --features serde` pass

**Out of scope:**
- Changing any engine implementation
- Adding new engines or operators
- Multi-objective engine convergence tests
- Performance benchmarks (separate concern)

</domain>

<decisions>
## Implementation Decisions

### Benchmark Function Choice
- **D-01:** All engines use **Sphere** function (f(x) = Σ xᵢ², global minimum = 0 at origin)
- **D-02:** DeEngine: Sphere — matches existing `test_de.rs` helper
- **D-03:** ScatterEngine: Sphere — matches existing `test_scatter.rs` helper
- **D-04:** CellularEngine: Sphere — consistent with other engines
- **D-05:** AlpsEngine: Sphere — consistent with other engines
- **D-06:** CmaEngine: Sphere — matches existing `test_cma.rs` helper
- **D-07:** PsoEngine: Sphere — matches existing `test_pso.rs` helper

### Convergence Thresholds
- **D-08:** Uniform threshold across all engines: **best_fitness < 1.0** on 5-dim Sphere
- **D-09:** Threshold is loose enough for stochastic engines, tight enough to prove convergence
- **D-10:** Matches existing CMA/PSO test patterns (`fitness_target(1.0)`)

### Budget Parameters
- **D-11:** Sphere dimension: **5** (matches existing tests)
- **D-12:** Population size: **30** (matches existing DE/Scatter tests)
- **D-13:** Max generations/iterations: **300** (matches existing DE test)
- **D-14:** All tests use fixed RNG seed via `rng::set_seed(Some(seed))` for determinism

### CMA Restart Testing
- **D-15:** CMA restart tested in a **separate test** from basic convergence
- **D-16:** Only **IPOP** restart strategy tested (simpler, well-tested in existing CMA tests)
- **D-17:** Restart test asserts convergence AND that restart occurred (via observer spy or restart counter)

### Test File Organization
- **D-18:** Each engine's convergence test added to its existing test file (`tests/engines/<engine>/test_<engine>.rs`)
- **D-19:** Reuse existing helper functions (`sphere`, `random_pop`) already defined in each test file
- **D-20:** New convergence tests follow existing test naming convention: `test_<engine>_convergence`

### Agent's Discretion
- Exact test function names and internal structure
- Whether to extract shared `sphere` helper to a common module (currently duplicated)
- Specific assertion messages and error context

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope
- `.planning/ROADMAP.md` §Phase 82 — success criteria and test requirements

### Existing Test Files (must not break)
- `tests/engines/de/test_de.rs` — existing DE tests with `sphere` helper and convergence pattern
- `tests/engines/scatter/test_scatter.rs` — existing Scatter tests with `sphere` helper
- `tests/engines/cellular/test_cellular.rs` — existing Cellular tests
- `tests/engines/alps/test_alps.rs` — existing ALPS tests
- `tests/engines/cma/test_cma.rs` — existing CMA tests with restart tests (CMA-12 through CMA-16)
- `tests/engines/pso/test_pso.rs` — existing PSO tests with convergence pattern

### Engine Configurations
- `src/engines/de/configuration.rs` — `DeConfiguration` fields (population_size, max_generations, mutation_strategy, crossover_mode)
- `src/engines/scatter/configuration.rs` — `ScatterConfiguration` fields (population_size, reference_set_size, max_iterations)
- `src/engines/cellular/configuration.rs` — `CellularConfiguration` fields
- `src/engines/alps/configuration.rs` — `AlpsConfiguration` fields
- `src/engines/cma/configuration.rs` — `CmaConfiguration` fields (sigma0, lambda, restart_strategy)
- `src/engines/pso/configuration.rs` — `PsoConfiguration` fields (population_size, max_generations, inertia, topology)

### RNG Setup
- `src/rng.rs` — `set_seed()` and `make_rng()` functions for deterministic tests

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `sphere` helper function: already defined in `test_de.rs`, `test_scatter.rs`, `test_cma.rs`, `test_pso.rs` — can be reused directly
- `random_pop` helper function: already defined in all engine test files — builds random `RangeChromosome<f64>` population
- `SpyObserver` pattern: exists in `test_cma.rs` and `test_pso.rs` — can be used for CMA restart assertion

### Established Patterns
- Fixed seed setup: `rng::set_seed(Some(seed))` before population initialization
- Engine configuration: `DeConfiguration::default().with_*()` builder pattern
- Convergence assertion: `assert!(result.best_fitness < THRESHOLD, "message")`
- Determinism: all tests use seed 42 or similar fixed values

### Integration Points
- `tests/engines/<engine>/test_<engine>.rs` — add new convergence test functions to existing files
- Each test file already has the necessary imports and helpers

</code_context>

<specifics>
## Specific Ideas

- Reuse the existing `sphere` and `random_pop` helpers already defined in each test file — no need to duplicate
- For CMA restart test, use the existing `SpyObserver` pattern from `test_cma.rs` to assert restart occurred
- All convergence tests should have clear assertion messages: `"Engine should converge to sphere minimum < 1.0; got {}"`
- Consider adding a comment at the top of each convergence test explaining it's a regression test for search dynamics

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 82-per-engine-convergence-integration-tests-issue-284*
*Context gathered: 2026-06-22*
