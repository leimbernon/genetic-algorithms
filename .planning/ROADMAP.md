# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- 🚧 **v2.2 — Improve Usability (completion)** — Phases 6-9 (in progress)

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

### 🚧 v2.2 — Improve Usability, completion (In Progress)

**Milestone goal:** Expose diversity as a first-class metric, add a symbolic List genotype, give users structured lifecycle reporting, and provide an optional chart-generation module.

- [x] **Phase 6: Diversity Estimation** — Expose a diversity metric in statistics and wire it into the GA's adaptive subsystems (completed 2026-03-20)
- [x] **Phase 7: List Genotype** — Add a `List<T>` gene and chromosome type for finite symbolic alphabets (completed 2026-03-21)
- [ ] **Phase 8: Reporter Trait** — Add a `Reporter` trait with lifecycle hooks and two built-in implementations
- [ ] **Phase 9: Visualization** — Add an optional `visualization` feature that renders fitness and diversity charts to PNG/SVG

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
- [ ] 06-01-PLAN.md — Add diversity field to GenerationStats with serde backward-compat
- [ ] 06-02-PLAN.md — Reorder GA loop and wire subsystems to read gen_stats.diversity

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
- [ ] 07-01-PLAN.md — List<T> gene type and ListChromosome<T> with GeneT/ChromosomeT impls
- [ ] 07-02-PLAN.md — ListValue mutation operator, list initializer, integration tests

### Phase 8: Reporter Trait
**Goal**: Users can attach structured lifecycle observers to `Ga` that receive hooks at key execution points, with zero cost when no reporter is configured
**Depends on**: Phase 6
**Requirements**: REP-01, REP-02, REP-03, REP-04
**Success Criteria** (what must be TRUE):
  1. User can call `.with_reporter(Box::new(my_reporter))` on a `Ga` builder and have `on_start`, `on_generation_complete`, `on_new_best`, and `on_finish` invoked at the corresponding execution points
  2. A `Ga` without a reporter configured compiles and runs with zero overhead (the `NoopReporter` is the default and the compiler eliminates it)
  3. `SimpleReporter` prints a one-line progress summary to stdout every N generations (N configurable by the user)
  4. `DurationReporter` reports wall-clock time spent in each execution phase (selection, crossover, mutation, survivor) at the end of the run
**Plans**: TBD

### Phase 9: Visualization
**Goal**: Users who opt into the `visualization` feature flag can generate PNG or SVG charts of fitness and diversity trends directly from GA statistics
**Depends on**: Phase 6, Phase 7, Phase 8
**Requirements**: VIZ-01, VIZ-02, VIZ-03, VIZ-04
**Success Criteria** (what must be TRUE):
  1. User can call a visualization function with a `Vec<Stats>` to produce a fitness-over-generations chart (best, worst, average lines) saved as PNG or SVG
  2. User can produce a diversity-over-generations chart from the same `Vec<Stats>` using the diversity values populated in Phase 6
  3. User can produce a fitness-distribution histogram for a chosen generation from the run statistics
  4. All visualization functions are absent from the compiled binary unless the `visualization` feature flag is explicitly enabled — the crate compiles cleanly with `cargo test` (no feature) and with `cargo test --features visualization`
**Plans**: TBD

## Progress

**Execution order:** 6 → 7 → 8 → 9

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 6. Diversity Estimation | 2/2 | Complete    | 2026-03-20 | - |
| 7. List Genotype | 2/2 | Complete   | 2026-03-21 | - |
| 8. Reporter Trait | v2.2 | 0/TBD | Not started | - |
| 9. Visualization | v2.2 | 0/TBD | Not started | - |
