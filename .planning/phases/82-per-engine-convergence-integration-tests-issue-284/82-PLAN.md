---
phase: 82-per-engine-convergence-integration-tests-issue-284
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - tests/engines/de/test_de.rs
  - tests/engines/scatter/test_scatter.rs
  - tests/engines/cellular/test_cellular.rs
  - tests/engines/alps/test_alps.rs
  - tests/engines/cma/test_cma.rs
  - tests/engines/pso/test_pso.rs
autonomous: true
requirements: [ISSUE-284]
user_setup: []

must_haves:
  truths:
    - "DeEngine converges to sphere minimum < 1.0 on 5D within 300 generations"
    - "ScatterEngine converges to sphere minimum < 1.0 on 5D within 300 iterations"
    - "CellularEngine converges to sphere minimum < 1.0 on 5D within 300 generations"
    - "AlpsEngine converges to sphere minimum < 1.0 on 5D within 300 generations"
    - "CmaEngine converges to sphere minimum < 1.0 on 5D within 300 generations (no restart)"
    - "CmaEngine with IPOP restart converges to sphere minimum < 1.0 and triggers at least one restart"
    - "PsoEngine converges to sphere minimum < 1.0 on 5D within 300 generations"
    - "All convergence tests use fixed RNG seed 42 for determinism"
    - "All tests pass with cargo test and cargo test --features serde"
  artifacts:
    - path: "tests/engines/de/test_de.rs"
      provides: "test_de_convergence function"
      contains: "fn test_de_convergence"
    - path: "tests/engines/scatter/test_scatter.rs"
      provides: "test_scatter_convergence function"
      contains: "fn test_scatter_convergence"
    - path: "tests/engines/cellular/test_cellular.rs"
      provides: "test_cellular_convergence function"
      contains: "fn test_cellular_convergence"
    - path: "tests/engines/alps/test_alps.rs"
      provides: "test_alps_convergence function"
      contains: "fn test_alps_convergence"
    - path: "tests/engines/cma/test_cma.rs"
      provides: "test_cma_convergence and test_cma_ipop_convergence functions"
      contains: "fn test_cma_convergence"
    - path: "tests/engines/pso/test_pso.rs"
      provides: "test_pso_convergence function"
      contains: "fn test_pso_convergence"
  key_links:
    - from: "tests/engines/de/test_de.rs"
      to: "DeEngine::new(config, init_fn, fitness_fn)"
      via: "sphere_engine helper"
      pattern: "DeEngine::new.*sphere"
    - from: "tests/engines/cma/test_cma.rs"
      to: "SpyObserver"
      via: "IPOP restart assertion"
      pattern: "spy\\.restart_count\\.load"
---

<objective>
Add end-to-end convergence tests for all 6 single-objective engines (DeEngine, ScatterEngine, CellularEngine, AlpsEngine, CmaEngine, PsoEngine) asserting each reaches sphere minimum < 1.0 on 5D within 300 generations/iterations. CMA additionally gets an IPOP restart convergence test. Closes GitHub issue #284.

Purpose: Prevents silent regressions in search dynamics across all single-objective engines.
Output: 7 new test functions added to 6 existing test files.
</objective>

<execution_context>
@/Users/luis/.config/opencode/gsd-core/workflows/execute-plan.md
@/Users/luis/.config/opencode/gsd-core/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/82-per-engine-convergence-integration-tests-issue-284/82-CONTEXT.md
@.planning/phases/82-per-engine-convergence-integration-tests-issue-284/82-RESEARCH.md
@.planning/phases/82-per-engine-convergence-integration-tests-issue-284/82-PATTERNS.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add convergence tests for DE, Scatter, Cellular, and ALPS engines</name>
  <files>tests/engines/de/test_de.rs, tests/engines/scatter/test_scatter.rs, tests/engines/cellular/test_cellular.rs, tests/engines/alps/test_alps.rs</files>
  <read_first>
    - tests/engines/de/test_de.rs (existing sphere_engine helper at lines 41-56, convergence pattern at lines 60-71)
    - tests/engines/scatter/test_scatter.rs (existing convergence pattern at lines 39-54, uses with_max_iterations NOT with_max_generations)
    - tests/engines/cellular/test_cellular.rs (existing make_engine helper at lines 43-62, CellularEngine::new returns Result — needs .expect())
    - tests/engines/alps/test_alps.rs (existing make_engine helper at lines 39-53, AlpsEngine::new returns Result — needs .expect())
  </read_first>
  <action>
Add one convergence test function to each of the 4 test files. Per D-01 through D-05, all use Sphere function. Per D-08, threshold is best_fitness < 1.0. Per D-11, dimension is 5. Per D-12, population size is 30. Per D-13, max generations/iterations is 300. Per D-14, seed is 42 (already embedded in random_pop helper). Per D-18 through D-20, tests go in existing files with naming convention test_<engine>_convergence.

**tests/engines/de/test_de.rs** — Append `test_de_convergence` after the existing tests (before line 206 cache tests section):
- Reuse existing `sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Binomial)` helper (line 41)
- Assert `result.best_fitness < 1.0` with message "DE should converge to sphere minimum < 1.0; got {}"
- DeEngine::new does NOT return Result; engine.run() does NOT return Result — no .expect() needed

**tests/engines/scatter/test_scatter.rs** — Append `test_scatter_convergence` after existing tests:
- Use `ScatterConfiguration::default().with_population_size(30).with_reference_set_size(6).with_max_iterations(300).with_problem_solving(ProblemSolving::Minimization).with_fitness_target(1.0)` (per D-08/D-13)
- CRITICAL: Scatter uses `with_max_iterations()` NOT `with_max_generations()`
- CRITICAL: Must include `.with_reference_set_size(6)` — required by ScatterConfiguration
- ScatterEngine::new does NOT return Result; engine.run() does NOT return Result — no .expect() needed
- Assert `result.best_fitness < 1.0`

**tests/engines/cellular/test_cellular.rs** — Append `test_cellular_convergence` after existing tests:
- Use `CellularConfiguration::default().with_grid(6, 6).with_neighborhood(Neighborhood::Moore).with_update_mode(UpdateMode::Asynchronous).with_max_generations(300).with_selection(Selection::Tournament).with_crossover(Crossover::Uniform).with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.5) })).with_problem_solving(ProblemSolving::Minimization).with_fitness_target(1.0)`
- CRITICAL: CellularEngine::new() RETURNS Result — MUST use `.expect("valid test config")`
- Grid 6x6 = 36 individuals (population = rows * cols)
- engine.run() does NOT return Result — no .expect() on run

**tests/engines/alps/test_alps.rs** — Append `test_alps_convergence` after existing tests:
- Use `AlpsConfiguration::default().with_n_layers(4).with_layer_size(15).with_age_scheme(AlpsAgeScheme::Linear).with_age_gap(5).with_injection_interval(10).with_max_generations(300).with_crossover(Crossover::Uniform).with_mutation(Mutation::Gaussian(GaussianParams { sigma: Some(0.5) })).with_problem_solving(ProblemSolving::Minimization).with_fitness_target(1.0)`
- CRITICAL: AlpsEngine::new() RETURNS Result — MUST use `.expect("valid test config")`
- Uses AlpsAgeScheme::Linear (simplest scheme)
- engine.run() does NOT return Result — no .expect() on run

All tests use doc-comment: /// Convergence regression test: <Engine> must reach sphere minimum < 1.0 on 5 dimensions within 300 <generations|iterations>. Prevents silent regressions in search dynamics. (Closes issue #284)
  </action>
  <verify>
    <automated>cargo test engines::de::test_de::test_de_convergence engines::scatter::test_scatter::test_scatter_convergence engines::cellular::test_cellular::test_cellular_convergence engines::alps::test_alps::test_alps_convergence -- --exact 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - tests/engines/de/test_de.rs contains `fn test_de_convergence`
    - tests/engines/scatter/test_scatter.rs contains `fn test_scatter_convergence`
    - tests/engines/cellular/test_cellular.rs contains `fn test_cellular_convergence`
    - tests/engines/alps/test_alps.rs contains `fn test_alps_convergence`
    - Each test asserts `result.best_fitness < 1.0`
    - All 4 tests pass with `cargo test`
    - All 4 tests pass with `cargo test --features serde`
    - No existing tests are broken
  </acceptance_criteria>
  <done>4 new convergence test functions exist and pass. Each asserts best_fitness < 1.0 on 5D Sphere with seed 42. No existing tests broken.</done>
</task>

<task type="auto">
  <name>Task 2: Add convergence tests for CMA and PSO engines (including CMA IPOP restart)</name>
  <files>tests/engines/cma/test_cma.rs, tests/engines/pso/test_pso.rs</files>
  <read_first>
    - tests/engines/cma/test_cma.rs (existing convergence pattern at lines 124-147, IPOP restart pattern at lines 401-433, SpyObserver at lines 50-118, CmaEngine::new does NOT return Result but engine.run() RETURNS Result)
    - tests/engines/pso/test_pso.rs (existing convergence pattern at lines 346-362, PSO uses rng::set_seed BEFORE random_pop, uses g.real_value() not g.value(), PsoEngine::new does NOT return Result but engine.run() RETURNS Result)
  </read_first>
  <action>
Add 3 convergence test functions across 2 test files. Per D-06/D-07, both use Sphere. Per D-08, threshold < 1.0. Per D-15/D-16, CMA IPOP restart tested in separate test. Per D-17, restart test asserts convergence AND restart occurred.

**tests/engines/cma/test_cma.rs** — Append `test_cma_convergence` and `test_cma_ipop_convergence` after existing tests (before batch_and_cache_tests module at line 672):

Test 1: `test_cma_convergence`
- Use `CmaConfiguration::default_for_dim(5).with_max_generations(300).with_problem_solving(ProblemSolving::Minimization).with_sigma0(0.3).with_fitness_target(1.0)`
- CRITICAL: Use `CmaConfiguration::default_for_dim(5)` NOT `CmaConfiguration::default()`
- CRITICAL: Must include `.with_sigma0(0.3)` — required for CMA initialization
- CmaEngine::new does NOT return Result — no .expect() on construction
- engine.run() RETURNS Result — MUST use `.expect("engine run should succeed")`
- Assert `result.best_fitness < 1.0`

Test 2: `test_cma_ipop_convergence`
- Use `CmaConfiguration::default_for_dim(5).with_max_generations(500).with_problem_solving(ProblemSolving::Minimization).with_sigma0(0.3).with_fitness_target(1.0).with_restart_strategy(RestartStrategy::Ipop { population_scale: 2.0, stagnation_threshold: 10, max_restarts: 3 })`
- Use existing SpyObserver (already defined in file, lines 50-118): `let spy = Arc::new(SpyObserver::default());`
- Wire observer: `.with_observer(spy.clone())`
- Assert convergence: `result.best_fitness < 1.0`
- Assert restart occurred: `spy.restart_count.load(Ordering::SeqCst) >= 1`

**tests/engines/pso/test_pso.rs** — Append `test_pso_convergence` after existing tests:
- MUST call `rng::set_seed(Some(42))` BEFORE `random_pop()` — PSO tests require explicit seed
- Create init_pop: `let init_pop = random_pop(30, 5, -5.0, 5.0, 42);`
- Use `PsoConfiguration::default().with_population_size(30).with_max_generations(300).with_problem_solving(ProblemSolving::Minimization).with_fitness_target(1.0)`
- CRITICAL: PSO uses `move |_n| init_pop.clone()` pattern — population created BEFORE engine
- PsoEngine::new does NOT return Result — no .expect() on construction
- engine.run() RETURNS Result — MUST use `.expect("engine run should succeed")`
- Assert `result.best_fitness < 1.0`
- PSO sphere helper uses `g.real_value()` not `g.value()` — both equivalent for RangeGene<f64> but must match file convention

All tests use doc-comment explaining convergence regression purpose and closing issue #284.
  </action>
  <verify>
    <automated>cargo test engines::cma::test_cma::test_cma_convergence engines::cma::test_cma::test_cma_ipop_convergence engines::pso::test_pso::test_pso_convergence -- --exact 2>&1 | tail -20</automated>
  </verify>
  <acceptance_criteria>
    - tests/engines/cma/test_cma.rs contains `fn test_cma_convergence`
    - tests/engines/cma/test_cma.rs contains `fn test_cma_ipop_convergence`
    - tests/engines/pso/test_pso.rs contains `fn test_pso_convergence`
    - test_cma_convergence asserts `result.best_fitness < 1.0` using CmaConfiguration::default_for_dim(5)
    - test_cma_ipop_convergence asserts `result.best_fitness < 1.0` AND `spy.restart_count >= 1`
    - test_pso_convergence asserts `result.best_fitness < 1.0` with explicit rng::set_seed(Some(42))
    - All 3 tests pass with `cargo test`
    - All 3 tests pass with `cargo test --features serde`
    - No existing tests are broken (including SpyObserver-dependent CMA-12 through CMA-17)
  </acceptance_criteria>
  <done>3 new convergence test functions exist and pass. CMA IPOP test confirms both convergence and restart triggering. No existing tests broken.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| (none) | Testing-only phase — no production code, no external inputs, no security-sensitive paths |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation |
|-----------|----------|-----------|-------------|------------|
| T-82-SC | Tampering | npm/pip/cargo installs | accept | No external packages installed in this phase — testing only, all dependencies are existing crate internals |
</threat_model>

<verification>
## Phase Verification Gate

All 7 new convergence tests pass with both `cargo test` and `cargo test --features serde`. No existing tests broken. Full validation sequence:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --features serde
cargo test --doc
cargo check --target wasm32-unknown-unknown
cargo bench --no-run
cargo doc --no-deps
```
</verification>

<success_criteria>
- 7 new test functions exist across 6 test files
- Each test asserts best_fitness < 1.0 on 5D Sphere
- CMA IPOP test additionally asserts restart_count >= 1
- All tests use fixed RNG seed 42 for determinism
- `cargo test` passes with 0 failures
- `cargo test --features serde` passes with 0 failures
- No existing tests broken
</success_criteria>

<artifacts_this_phase_produces>
## Artifacts This Phase Produces

| Symbol | Type | File | Description |
|--------|------|------|-------------|
| `test_de_convergence` | test function | `tests/engines/de/test_de.rs` | DE convergence on 5D Sphere |
| `test_scatter_convergence` | test function | `tests/engines/scatter/test_scatter.rs` | Scatter convergence on 5D Sphere |
| `test_cellular_convergence` | test function | `tests/engines/cellular/test_cellular.rs` | Cellular GA convergence on 5D Sphere |
| `test_alps_convergence` | test function | `tests/engines/alps/test_alps.rs` | ALPS convergence on 5D Sphere |
| `test_cma_convergence` | test function | `tests/engines/cma/test_cma.rs` | CMA-ES convergence on 5D Sphere (no restart) |
| `test_cma_ipop_convergence` | test function | `tests/engines/cma/test_cma.rs` | CMA-ES IPOP restart convergence on 5D Sphere |
| `test_pso_convergence` | test function | `tests/engines/pso/test_pso.rs` | PSO convergence on 5D Sphere |
</artifacts_this_phase_produces>

<output>
Create `.planning/phases/82-per-engine-convergence-integration-tests-issue-284/82-01-SUMMARY.md` when done
</output>
