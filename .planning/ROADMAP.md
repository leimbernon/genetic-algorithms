# Roadmap: genetic_algorithms

## Milestones

- ✅ **v2.1 — Improve Usability (partial)** — Phases 1-5 (shipped 2026-03-20)
- ✅ **v2.2 — Improve Usability (completion)** — Phases 6-9 (shipped 2026-03-21)
- ✅ **v2.1.0 — New Examples** — Phases 10-12 (shipped 2026-03-22)
- 🚧 **v2.2.0 — Observability & Traceability** — Phases TBD (in progress)

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

- [x] **Phase 6: Diversity Estimation** — `GenerationStats.diversity` wired into extension trigger and dynamic mutation (completed 2026-03-20)
- [x] **Phase 7: List Genotype** — `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets (completed 2026-03-21)
- [x] **Phase 8: Reporter Trait** — `Reporter<U>` with 4 lifecycle hooks, `SimpleReporter`, `DurationReporter` (completed 2026-03-21)
- [x] **Phase 9: Visualization** — `visualization` feature flag, `plot_fitness`, `plot_diversity`, `plot_histogram` (completed 2026-03-21)

</details>

<details>
<summary>✅ v2.1.0 — New Examples (Phases 10-12) — SHIPPED 2026-03-22</summary>

- [x] **Phase 10: Single-population Examples** — `rastrigin`, `feature_selection`, `niching` (completed 2026-03-22)
- [x] **Phase 11: Advanced Mode Examples** — `nsga2_zdt1`, `island_model`, `job_scheduling` (completed 2026-03-22)
- [x] **Phase 12: Documentation** — README `## Examples` table with all 10 examples and `cargo run` commands (completed 2026-03-22)

Full archive: `.planning/milestones/v2.1.0-ROADMAP.md`

</details>

### 🚧 v2.2.0 — Observability & Traceability (In Progress)

**Milestone goal:** Implement a generic, telemetry-agnostic observability system — `GaObserver` trait, `LogObserver`, `TracingObserver`, Island/NSGA-II sub-traits, `CompositeObserver`, `MetricsObserver`.

Issues: #182, #183, #184, #185, #186

Phases to be defined via `/gsd:new-milestone`.

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 6. Diversity Estimation | v2.2 | 2/2 | Complete | 2026-03-20 |
| 7. List Genotype | v2.2 | 2/2 | Complete | 2026-03-21 |
| 8. Reporter Trait | v2.2 | 2/2 | Complete | 2026-03-21 |
| 9. Visualization | v2.2 | 2/2 | Complete | 2026-03-21 |
| 10. Single-population Examples | v2.1.0 | 3/3 | Complete | 2026-03-22 |
| 11. Advanced Mode Examples | v2.1.0 | 3/3 | Complete | 2026-03-22 |
| 12. Documentation | v2.1.0 | 1/1 | Complete | 2026-03-22 |
