# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- 🚧 **v2.1.0 — New Examples** — Phases 10-12 (in progress)

## Phases

<details>
<summary>✅ v2.1 — Improve Usability, partial (Phases 1-5) — SHIPPED 2026-03-20</summary>

Phases 1-5 predate GSD tracking. Issues closed: #165, #166, #167, #168, #169.

- [x] Extension strategies (MassExtinction, MassGenesis, MassDegeneration, MassDeduplication)
- [x] Dynamic mutation probability based on population cardinality
- [x] Clone crossover strategy
- [x] Rejuvenate crossover operator
- [x] LRU fitness cache

</details>

<details>
<summary>✅ v2.2 — Improve Usability, completion (Phases 6-9) — SHIPPED 2026-03-21</summary>

Issues closed: #170, #171, #178, #179.

- [x] **Phase 6: Diversity Estimation** — Expose a diversity metric in statistics and wire it into the GA's adaptive subsystems (completed 2026-03-20)
- [x] **Phase 7: List Genotype** — Add a `List<T>` gene and chromosome type for finite symbolic alphabets (completed 2026-03-21)
- [x] **Phase 8: Reporter Trait** — Add a `Reporter` trait with lifecycle hooks and two built-in implementations (completed 2026-03-21)
- [x] **Phase 9: Visualization** — Add an optional `visualization` feature that renders fitness and diversity charts to PNG/SVG (completed 2026-03-21)

</details>

### 🚧 v2.1.0 — New Examples (In Progress)

**Milestone goal:** Add six runnable examples covering all major GA modes and operators, and update the README to document them with `cargo run --example` commands.

- [x] **Phase 10: Single-population Examples** - Rastrigin continuous optimization, Feature Selection with adaptive GA, and Niching / Fitness Sharing — all using `Ga<U>` (completed 2026-03-22)
- [x] **Phase 11: Advanced Mode Examples** - NSGA-II multi-objective (ZDT1), Island Model multi-population, and Job Scheduling permutation — using `Nsga2Ga` and `IslandGa` (completed 2026-03-22)
- [x] **Phase 12: Documentation** - README updated with examples section and `cargo run --example <name>` commands (completed 2026-03-22)

## Phase Details

### Phase 6: Diversity Estimation
**Goal**: Population diversity is a first-class observable metric that both users and the GA's internal subsystems can read and act on
**Depends on**: Nothing (first GSD-tracked phase; builds on shipped v2.1 codebase)
**Requirements**: DIV-01, DIV-02, DIV-03
**Success Criteria** (what must be TRUE):
  1. After each generation, `stats.diversity` returns a `f64` value the user can read and log
  2. The extension strategy (e.g., MassExtinction) triggers only when the per-generation diversity value falls below the configured threshold — not based on ad-hoc heuristics
  3. The dynamic mutation probability module uses the per-generation diversity value when deciding how to scale mutation probability
  4. All existing tests pass with no change to the public `ChromosomeT` or operator trait signatures
**Plans:** 2/2 plans complete

Plans:
- [x] 06-01-PLAN.md — Add diversity field to GenerationStats with serde backward-compat
- [x] 06-02-PLAN.md — Reorder GA loop and wire subsystems to read gen_stats.diversity

### Phase 7: List Genotype
**Goal**: Users can solve problems over finite symbolic alphabets using a `List<T>` gene and chromosome that plug into the existing operator pipeline without modification
**Depends on**: Phase 6
**Requirements**: LIST-01, LIST-02, LIST-03, LIST-04
**Success Criteria** (what must be TRUE):
  1. User can define a `List<T>` gene by specifying a finite allele set and obtain gene instances drawn from it
  2. User can construct a `ListChromosome<T>` that implements `ChromosomeT` and carries a fitness value
  3. A `ListChromosome<T>` works as input to all existing selection, crossover, mutation, and survivor operators without any operator code change
  4. User can initialize a full `List` population with a built-in initializer (equivalent to `BinaryChromosome` and `RangeChromosome` initializers)
  5. Diversity estimation from Phase 6 is computed correctly for `List` populations
**Plans:** 2/2 plans complete

Plans:
- [x] 07-01-PLAN.md — List<T> gene type and ListChromosome<T> with GeneT/ChromosomeT impls
- [x] 07-02-PLAN.md — ListValue mutation operator, list initializer, integration tests

### Phase 8: Reporter Trait
**Goal**: Users can attach structured lifecycle observers to `Ga` that receive hooks at key execution points, with zero cost when no reporter is configured
**Depends on**: Phase 6
**Requirements**: REP-01, REP-02, REP-03, REP-04
**Success Criteria** (what must be TRUE):
  1. User can call `.with_reporter(Box::new(my_reporter))` on a `Ga` builder and have `on_start`, `on_generation_complete`, `on_new_best`, and `on_finish` invoked at the corresponding execution points
  2. A `Ga` without a reporter configured compiles and runs with zero overhead (the `NoopReporter` is the default and the compiler eliminates it)
  3. `SimpleReporter` prints a one-line progress summary to stdout every N generations (N configurable by the user)
  4. `DurationReporter` reports wall-clock time spent in each execution phase (selection, crossover, mutation, survivor) at the end of the run
**Plans:** 2/2 plans complete

Plans:
- [x] 08-01-PLAN.md — Reporter trait definition, NoopReporter, and Ga integration (hook wiring)
- [x] 08-02-PLAN.md — SimpleReporter, DurationReporter, and integration tests

### Phase 9: Visualization
**Goal**: Users who opt into the `visualization` feature flag can generate PNG or SVG charts of fitness and diversity trends directly from GA statistics
**Depends on**: Phase 6, Phase 7, Phase 8
**Requirements**: VIZ-01, VIZ-02, VIZ-03, VIZ-04
**Success Criteria** (what must be TRUE):
  1. User can call a visualization function with a `Vec<Stats>` to produce a fitness-over-generations chart (best, worst, average lines) saved as PNG or SVG
  2. User can produce a diversity-over-generations chart from the same `Vec<Stats>` using the diversity values populated in Phase 6
  3. User can produce a fitness-distribution histogram for a chosen generation from the run statistics
  4. All visualization functions are absent from the compiled binary unless the `visualization` feature flag is explicitly enabled — the crate compiles cleanly with `cargo test` (no feature) and with `cargo test --features visualization`
**Plans:** 2/2 plans complete

Plans:
- [x] 09-01-PLAN.md — Feature flag setup, VisualizationError, and plot_fitness function
- [x] 09-02-PLAN.md — plot_diversity and plot_histogram functions

### Phase 10: Single-population Examples
**Goal**: Users can run three self-contained examples that demonstrate `Ga<U>` on continuous optimization, binary feature selection with adaptive parameters, and multimodal niching
**Depends on**: Phase 9 (all prior library work is in place)
**Requirements**: EX-01, EX-05, EX-06
**Success Criteria** (what must be TRUE):
  1. `cargo run --example rastrigin` executes without error, prints per-generation fitness, and converges toward the global minimum (fitness near 0)
  2. `cargo run --example feature_selection` executes without error and prints the best binary feature mask found along with its evaluated fitness
  3. `cargo run --example niching` executes without error and the reported best solutions include multiple distinct peaks rather than converging to a single one
  4. Each example file is self-contained with an explanatory comment block describing the problem, chromosome type, and operators used
**Plans:** 3/3 plans complete

Plans:
- [ ] 10-01-PLAN.md — Rastrigin continuous optimization example (EX-01)
- [ ] 10-02-PLAN.md — Feature selection with adaptive GA example (EX-05)
- [ ] 10-03-PLAN.md — Niching / fitness sharing example (EX-06)

### Phase 11: Advanced Mode Examples
**Goal**: Users can run three self-contained examples demonstrating NSGA-II multi-objective optimization, island model parallel evolution, and permutation-based job scheduling
**Depends on**: Phase 10
**Requirements**: EX-02, EX-03, EX-04
**Success Criteria** (what must be TRUE):
  1. `cargo run --example nsga2_zdt1` executes without error and prints a non-dominated Pareto front approximation showing the trade-off between the two ZDT1 objectives
  2. `cargo run --example island_model` executes without error and prints per-island best fitness values plus the global best after migration rounds complete
  3. `cargo run --example job_scheduling` executes without error and prints the best job ordering found along with its makespan value
  4. Each example file is self-contained with an explanatory comment block describing the problem, GA mode used, and key configuration choices
**Plans:** 3/3 plans complete

Plans:
- [ ] 11-01-PLAN.md — NSGA-II ZDT1 multi-objective example (EX-02)
- [ ] 11-02-PLAN.md — Island model Rastrigin 20D example (EX-03)
- [ ] 11-03-PLAN.md — Job scheduling permutation example (EX-04)

### Phase 12: Documentation
**Goal**: The README documents all six examples so users can discover and run them without reading source code
**Depends on**: Phase 11
**Requirements**: DOC-01
**Success Criteria** (what must be TRUE):
  1. The README contains an Examples section listing all six examples with a one-line description of each
  2. Every example entry in the README includes the exact `cargo run --example <name>` command needed to execute it
  3. A first-time user reading only the README can identify which example matches their problem domain (continuous, multi-objective, parallel, permutation, binary, multimodal)
**Plans:** 1/1 plans complete

Plans:
- [ ] 12-01-PLAN.md — Add Examples section to README with all 10 examples table

## Progress

**Execution order:** 6 -> 7 -> 8 -> 9 -> 10 -> 11 -> 12

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 6. Diversity Estimation | v2.2 | 2/2 | Complete | 2026-03-20 |
| 7. List Genotype | v2.2 | 2/2 | Complete | 2026-03-21 |
| 8. Reporter Trait | v2.2 | 2/2 | Complete | 2026-03-21 |
| 9. Visualization | v2.2 | 2/2 | Complete | 2026-03-21 |
| 10. Single-population Examples | 3/3 | Complete    | 2026-03-22 | - |
| 11. Advanced Mode Examples | 3/3 | Complete    | 2026-03-22 | - |
| 12. Documentation | 1/1 | Complete   | 2026-03-22 | - |
