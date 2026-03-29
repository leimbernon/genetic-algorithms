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

## Milestone: v2.2.0 — Observability & Traceability

**Shipped:** 2026-03-28
**Phases:** 6 (phases 13–18) | **Plans:** 14 | **Timeline:** 2026-03-25 → 2026-03-28 (~4 days)

### What Was Built

- `GaObserver<U>` trait with 12 `Send+Sync` hooks (run lifecycle, operator timing, new best, stagnation, extension); zero overhead via `Option<Arc<dyn GaObserver>>`
- `LogObserver` reproducing all pre-v2.2.0 `log!()` output; all 16 hardcoded macros removed from `ga.rs` execution paths
- `TracingObserver` behind `observer-tracing` feature flag — structured spans per generation, compatible with OpenTelemetry / Jaeger / any `tracing` subscriber
- `IslandGaObserver<U>` and `Nsga2Observer<U>` sub-traits with engine-specific hooks; wired into `IslandGa<U>` and `Nsga2Ga<U>` run loops
- `CompositeObserver<U>` fan-out (19 hooks dispatched to N observers via `AllObserver` blanket impl) + `MetricsObserver` behind `observer-metrics` feature flag (11 per-generation gauges/histograms/counters)
- Phase 18 gap closure: `TracingObserver` composable, hook ordering fixed, real elapsed `Duration` in operator hooks, crate-root re-exports

### What Worked

- **Layered dependency order** — phases 13→14→15→16→17→18 built a clean dependency chain; each phase produced a working artifact that the next phase could depend on without guessing
- **Blanket impl for AllObserver** — composability fell out naturally; adding empty `IslandGaObserver`/`Nsga2Observer` impls to `TracingObserver` in Phase 18 required < 5 lines
- **Phase 18 as explicit gap closure** — running `gsd:audit-milestone` before Phase 18 gave a precise list of what to fix; execution was fast because the scope was pre-defined
- **Background test runs** — the background task during Phase 18 execution caught `test_observer_on_new_best_fires` flakiness immediately, before pushing to remote

### What Was Inefficient

- **Flaky test shipped in Phase 18-02** — `test_observer_on_new_best_fires` used random initialization with ~7.6% failure probability; this should have been caught in review before committing, not by a background CI run
- **Stale "Pending" entries in REQUIREMENTS.md** — Phase 18 completed but didn't update the gap-closure section of the traceability table; left misleading "Pending" rows in the archive
- **Missing per-operator timing** — `on_mutation_complete` and `on_fitness_evaluation_complete` receive the combined crossover+mutation+fitness elapsed, not individual operator timing; deferred to EXT-01 but represents a known accuracy gap in MetricsObserver histograms

### Patterns Established

- Observer via `Option<Arc<dyn Trait + Send + Sync>>` with `notify()` helper — zero overhead when `None`, clean fan-out when `Some`
- Feature-gated observer implementations: `#[cfg(feature = "observer-tracing")]` / `#[cfg(feature = "observer-metrics")]` — compile cleanly without pulling optional deps
- `AllObserver` blanket impl unlocks composability — any type implementing all three observer traits automatically satisfies `AllObserver` and can be added to `CompositeObserver`
- Test determinism via `rng::set_seed()` or deterministic initialization — never rely on random population improving within N generations for a required assertion

### Key Lessons

1. **Audit-driven gap closure works** — running `gsd:audit-milestone` after the main phases produced a precise, actionable list of gaps; Phase 18 closed them all in 2 plans. The pattern is worth repeating at every milestone.
2. **Probabilistic tests are a footgun** — any test asserting "this stochastic event happened at least once" in a short run can flake in CI. Fix deterministically at authoring time, not after CI failure.
3. **Operator timing needs its own timing block** — the combined-elapsed approach satisfied the requirement at v2.2.0 level, but users attaching `MetricsObserver` will expect per-operator histograms. EXT-01 should be scheduled early in the next performance milestone.

### Cost Observations

- Model mix: sonnet for execution, sonnet for verification
- Sessions: ~2 over 4 days
- Notable: Phase 18 gap closure executed in < 20 min wall time after the audit defined exact scope; audit investment pays back immediately

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v2.0.0 | Pre-GSD | — | Initial GSD adoption |
| v2.1.0 | 7 | 15 | First full GSD-tracked milestone; established reporter/feature-flag patterns |
| v2.2.0 | 6 | 14 | Introduced audit-driven gap closure (Phase 18); established observer/blanket-impl patterns |

### Cumulative Quality

| Milestone | Test Pass Rate | Verification Pass Rate | Tech Debt Items |
|-----------|---------------|----------------------|----------------|
| v2.1.0 | 100% | 7/7 (100%) | 3 (examples don't demo Phase 7–9) |
| v2.2.0 | 100% | 6/6 (100%) | 1 (EXT-01: per-operator timing separation) |

### Top Lessons (Verified Across Milestones)

1. Build library features and their example demonstrations in the same phase — don't separate them
2. Audit-driven gap closure (pre-milestone audit → explicit Phase N.x) is faster than discovering gaps during verification
3. Probabilistic tests must use seeded RNG or deterministic initialization — flaky tests erode CI confidence
