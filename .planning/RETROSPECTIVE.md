# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v2.1.0 — New Examples

**Shipped:** 2026-03-22
**Phases:** 7 (phases 6–12) | **Plans:** 15 | **Timeline:** 2026-03-20 → 2026-03-22 (~3 days)

### What Was Built

- `GenerationStats.diversity: f64` (fitness std-dev) wired into extension trigger and dynamic mutation — eliminates ad-hoc cardinality computation
- `List<T>` gene and `ListChromosome<T>` for finite symbolic alphabets, including `Mutation::ListValue` and two initializers
- `Reporter<U>` lifecycle trait with `SimpleReporter` and `DurationReporter`; zero overhead when not set
- `visualization` feature flag with `plot_fitness`, `plot_diversity`, `plot_histogram` (PNG/SVG via plotters)
- Six self-contained examples covering every major GA mode: continuous, binary, niching, NSGA-II, island, permutation
- README `## Examples` table with all 10 examples and exact `cargo run` commands

### What Worked

- **Single diversity metric for all subsystems** — computing stats once per generation and passing `gen_stats` to extension + dynamic mutation eliminated duplicate signals cleanly
- **Feature-flag isolation** — `visualization` compiles cleanly behind `#[cfg(feature = "visualization")]`; no leakage into default builds
- **Phase verifications all passed 100%** — 7/7 phases scored full marks on first verification pass; no re-verification needed
- **Examples are self-contained** — each example file has an explanatory comment block and runs standalone

### What Was Inefficient

- **Reporter/Visualization/List unused in examples** — these three features were built and tested in phases 7–9 but none of the phase 10–11 examples demonstrated them; users can't discover `with_reporter()` from examples alone
- **`job_scheduling` used `RangeChromosome` instead of `List`** — the most natural place to demonstrate `ListChromosome<char>` used a workaround instead; this was a missed integration opportunity
- **Nyquist VALIDATION.md never signed off** — all 7 VALIDATION.md files stayed at `status: draft`, `nyquist_compliant: false`; the tracking artifact has no value if never completed

### Patterns Established

- Stats computed once per generation → passed as `gen_stats` to all subsystems (do not re-compute inline)
- Optional features use `Option<Box<dyn Trait + Send>>` with `if let Some(...)` guards — zero overhead confirmed pattern
- Feature-gated modules declared in `lib.rs` with `#[cfg(feature = "X")]` — crate compiles cleanly both with and without

### Key Lessons

1. **When building a library feature, immediately wire it into one example** — don't leave verification to unit tests alone. `Reporter`, `Visualization`, and `List` all have great unit tests but zero runnable E2E paths after the milestone.
2. **Example choice drives discoverability** — users learn the API from examples first, documentation second. The example phase (10–11) should have been the integration layer for phases 7–9.
3. **Nyquist validation is either done per-phase or never** — waiting to do it in bulk at milestone end doesn't work. Consider dropping the artifact if the process doesn't support it.

### Cost Observations

- Model mix: quality profile (opus for planning/research, sonnet for execution)
- Sessions: ~3 over 3 days
- Notable: 103 commits across 3 days at 100% verification pass rate; high throughput with GSD parallelization

---

## Milestone: v2.2.1 — Performance Optimizations

**Shipped:** 2026-04-23
**Phases:** 6 (phases 19–24) | **Plans:** 13 | **Timeline:** 2026-03-30 → 2026-04-05 (~6 days)

### What Was Built

- Deferred parent clones in crossover hot path; five numeric mutation operators use `set_gene()` — eliminates full-DNA Vec allocation per mutation call
- PMX crossover replaced O(n²) position scan with O(n) `HashMap`; OX uses O(n) `HashSet`
- Rank/Boltzmann selection use `partition_point()` binary search; fitness values collected once per generation and shared across extension, niching, and stats
- On-the-fly niching eliminates O(n²) distance matrix; elite reinsertion and mass genesis use `select_nth_unstable_by()` O(n)
- RNG atomics relaxed from `SeqCst` to `Acquire`/`Relaxed`; extension regrow parallelized with rayon
- `Range` genes share `Arc<[(T,T)]>` slice per chromosome; `MassDeduplication` uses incremental `DefaultHasher`
- `GenerationStats` moved (not cloned); island migration uses `select_nth_unstable_by()` and `Arc` migrant sharing

### What Worked

- **Incremental, non-breaking optimizations** — all 24 requirements closed with zero public API changes; pure internal patch philosophy held throughout
- **Independent phase ordering** — phases 20–24 had no dependency on each other, allowing flexible reordering when any phase proved simpler/harder than expected
- **Nyquist audit caught real gaps** — Phase 19 CLONE-02 Nyquist tests (added post-execution) revealed that MultiPoint/Uniform/Cycle/SinglePoint crossover operators were not yet building children directly; the audit drove targeted gap closure
- **On-the-fly niching was the right trade-off** — preserving the old `compute_distance_matrix`/`apply_fitness_sharing` API while replacing them in ga.rs allowed the optimization with zero breaking risk

### What Was Inefficient

- **SUMMARY.md files lacked `one_liner` field** — the gsd-tools `milestone complete` command extracted 0 accomplishments automatically; had to write them manually. Add `one_liner:` to SUMMARY.md frontmatter going forward.
- **Phase 19 had three separate plans where two could have been one** — gap closure for CLONE-02 (crossover child construction) was scoped separately from the initial clone work, adding an extra planning/execution cycle
- **Audit discovered operator gaps post-execution** — Phase 19-02 and 19-03 were gap-closure plans driven by the Nyquist audit. A stronger pre-execution checklist per operator type would have caught these in Phase 19-01 planning.

### Patterns Established

- `select_nth_unstable_by()` / `partition_point()` as the standard O(n) replacement for sort+first or linear position scan — applies to selection, elitism, migration, stats
- `Arc<[T]>` for shared immutable slice data constructed once, read-many (Range genes, migrant vectors)
- Collect fitness values once per generation in ga.rs, then pass by reference to all subsystems — prevents redundant O(n) allocations in niching, extension, and stats

### Key Lessons

1. **Add `one_liner:` to every SUMMARY.md frontmatter immediately after writing** — this is the only automatic source for milestone accomplishment extraction; missing it requires manual reconstruction.
2. **Pre-plan operator scope per phase** — when a phase targets "all X operators," list them explicitly in the plan rather than discovering coverage gaps via audit. A checklist of all 9 crossover operators in Phase 19 planning would have prevented the 2-plan gap closure cycle.
3. **Nyquist audit works — run it before marking a phase complete, not after** — the audit caught real missed coverage in Phase 19; that feedback loop should happen at phase-close time, not milestone-close time.

### Cost Observations

- Model mix: balanced profile (sonnet for most execution, opus for planning/research phases)
- Sessions: ~8 over 6 days
- Notable: 21 perf/refactor commits with 24/24 requirements closed and zero regressions; conservative approach (preserve old APIs, add new paths) kept blast radius minimal

---

## Milestone: v2.3.0 — Alternative Metaheuristics & Population Models

**Shipped:** 2026-04-27
**Phases:** 5 (phases 25–29) | **Plans:** 8 | **Timeline:** 2026-04-26 → 2026-04-27 (2 days)

### What Was Built

- Non-breaking `src/` restructure via `#[path]` re-exports — all existing paths preserved without semver bump
- Differential Evolution engine: DeGene trait, 5 mutation strategies, JADE/L-SHADE adaptive variants, 11 tests, benchmark
- Scatter Search engine: diversification, reference set, combination, optional local search, 7 tests, benchmark
- Cellular GA engine: 2D toroidal grid with 4 neighborhoods and sync/async update modes, 10 tests, benchmark
- ALPS engine: 3 age schemes, cross-layer mating, periodic injection, 11 tests, benchmark

### What Worked

- **Fast-tracked implementation via direct commits** — all 4 engines were implemented in single atomic commits rather than incremental GSD plan/execute cycles; 2 days for 4 engines + restructure is high throughput
- **Non-breaking restructure was zero-drama** — `#[path]` attribute in lib.rs is the right Rust idiom for this; no downstream churn
- **Tests in `tests/`** — 39 integration tests across 4 engines, all isolated in tests/ as required

### What Was Inefficient

- **GSD planning artifacts not updated during execution** — phases 27/28/29 have no phase directories; SUMMARY.md files had to be auto-filled at milestone close; planning and execution were decoupled
- **REQUIREMENTS.md traceability table never updated** — all 31 requirements stayed "Pending" until milestone close; lost traceability during the milestone
- **Plans 26-01 and 26-02 created but never executed** — observer wiring and DE-vs-GA benchmark were planned but the implementation outpaced the planning workflow; resulted in plan/code mismatch
- **Observer hooks deferred across 4 engines** — the v2.3.0 scope included observer integration (via plans 26-01+), but fast-track commits skipped it; now a concrete tech debt item for next milestone

### Patterns Established

- When fast-tracking implementation (direct commits), still create minimal phase directories + SUMMARY.md at commit time — 5 minutes at commit time saves 30 minutes at milestone close
- New engines should match the `#[path]` re-export pattern in lib.rs — don't create new public modules without updating lib.rs

### Key Lessons

1. **GSD artifacts and git commits should stay in sync** — when you bypass `gsd-execute-phase` and commit directly, update the planning artifacts in the same commit (even a one-paragraph SUMMARY.md). The cleanup cost at milestone close is non-trivial.
2. **Observer integration is a first-class feature, not an afterthought** — any engine that doesn't hook into GaObserver is a second-class citizen. The pattern is clear and cheap; defer it deliberately with a tracked issue, not silently.
3. **Plans created after implementation have `<done>` markers that don't match reality** — the 26-01 and 26-02 plan files have `<done>` tags written for work that wasn't done; this erodes trust in planning artifacts. Either execute the plan or delete the `<done>` tag.

### Cost Observations

- Model mix: execution-heavy (sonnet); minimal planning overhead
- Sessions: ~2 over 2 days
- Notable: 4 engines shipped in 2 days at 2 days total — fast-track mode trades artifact quality for delivery speed; the debt is real

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v2.0.0 | Pre-GSD | — | Initial GSD adoption |
| v2.1.0 | 7 | 15 | First full GSD-tracked milestone; established reporter/feature-flag patterns |
| v2.2.0 | 6 | 13 | Observer trait system; feature-flag optional deps |
| v2.2.1 | 6 | 13 | Pure internal optimization; Nyquist audit drove gap closure |
| v2.3.0 | 5 | 8 | Fast-track delivery; planning artifacts decoupled from execution |

### Cumulative Quality

| Milestone | Test Pass Rate | Verification Pass Rate | Tech Debt Items |
|-----------|---------------|----------------------|----------------|
| v2.1.0 | 100% | 7/7 (100%) | 3 (examples don't demo Phase 7–9) |
| v2.2.0 | 100% | 6/6 (100%) | 1 (EXT-01 per-operator timing deferred to v2.3+) |
| v2.2.1 | 100% | 6/6 (100%) | 1 (SUMMARY one_liner field missing across all phases) |
| v2.3.0 | 100% | 0/0 (no verifications run) | 2 (observer not wired in new engines; DE-vs-GA benchmark missing) |

### Top Lessons (Verified Across Milestones)

1. Build library features and their example demonstrations in the same phase — don't separate them
2. Add `one_liner:` to SUMMARY.md frontmatter at write time — the only automated source for milestone accomplishment extraction
3. List all operator/target names explicitly in phase planning — coverage gaps discovered by post-hoc audit should be caught at planning time
