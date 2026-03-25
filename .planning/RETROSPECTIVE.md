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

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v2.0.0 | Pre-GSD | — | Initial GSD adoption |
| v2.1.0 | 7 | 15 | First full GSD-tracked milestone; established reporter/feature-flag patterns |

### Cumulative Quality

| Milestone | Test Pass Rate | Verification Pass Rate | Tech Debt Items |
|-----------|---------------|----------------------|----------------|
| v2.1.0 | 100% | 7/7 (100%) | 3 (examples don't demo Phase 7–9) |

### Top Lessons (Verified Across Milestones)

1. Build library features and their example demonstrations in the same phase — don't separate them
