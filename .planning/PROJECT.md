# genetic_algorithms

## What This Is

A Rust library for genetic algorithms published on crates.io as `genetic_algorithms`. Provides single-objective (`Ga<U>`), multi-objective (NSGA-II), and island model execution modes. Generic over chromosome/gene types via traits, with a rich operator library for selection, crossover, mutation, and survivor selection.

## Core Value

The simplest correct way to run a genetic algorithm in Rust — generic enough for any problem domain, fast enough for real workloads.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Population diversity estimation metric — v2.1.0 (Phase 6)
- ✓ List genotype `List<T>` / `ListChromosome<T>` — v2.1.0 (Phase 7)
- ✓ Reporter trait with lifecycle hooks — v2.1.0 (Phase 8)
- ✓ Visualization module (`plot_fitness`, `plot_diversity`, `plot_histogram`) — v2.1.0 (Phase 9)
- ✓ Single-population GA (`Ga<U>`) with builder pattern — core library
- ✓ Multi-objective NSGA-II (`Nsga2Ga`) — pareto front, crowding distance
- ✓ Island model (`IslandGa`) — multi-population + migration topologies
- ✓ 6 selection operators: Tournament, FitnessProportionate, Rank, Boltzmann, Truncation, Random
- ✓ 11 crossover operators: SinglePoint, Multipoint, Uniform, Cycle, Order, PMX, SBX, BlendAlpha, Arithmetic, Clone, Rejuvenate
- ✓ 10 mutation operators: Swap, Inversion, Scramble, Value, BitFlip, Creep, Gaussian, Polynomial, NonUniform, Insertion
- ✓ 4 survivor operators: Fitness, Age, MuPlusLambda, MuCommaLambda
- ✓ 4 extension/diversity operators: MassExtinction, MassGenesis, MassDegeneration, MassDeduplication
- ✓ LRU fitness caching — avoids redundant evaluations for identical DNA
- ✓ Dynamic mutation probability based on population cardinality
- ✓ Extension strategies (diversity threshold-based rescue)
- ✓ Serde checkpoint/restore (feature-gated)
- ✓ Adaptive GA parameter updates
- ✓ Fitness sharing / niching

### Active

<!-- Current scope — Milestone v2.1.0 New Examples -->

- [ ] Rastrigin continuous optimization example using `Range<f64>` chromosomes (#154)
- [ ] NSGA-II multi-objective example (ZDT1 benchmark) (#155)
- [ ] Island Model GA example (parallel multi-population) (#156)
- [ ] Job Scheduling example (minimize makespan, permutation) (#157)
- [ ] Feature Selection example (Binary chromosomes + adaptive GA) (#158)
- [ ] Niching / Fitness Sharing example (multimodal optimization) (#159)
- [ ] README updated to document all examples with run commands

### Out of Scope

- Breaking API changes — deferred to milestone v3.0+ (Advanced Representations)
- NSGA-III / MOEA/D / SPEA2 — deferred to Advanced Multi-Objective milestone
- Differential Evolution engine — deferred to Alt. Metaheuristics milestone

## Context

- Crate version: 2.1.0 on crates.io
- Rust library, no binary or UI component
- All operators follow enum + factory-function pattern (runtime dispatch)
- `ChromosomeT` uses `Cow<[Gene]>` for zero-copy DNA operations
- Parallel fitness evaluation via rayon; seeded RNG for reproducibility
- No breaking changes allowed without explicit milestone designation

## Constraints

- **Compatibility**: No breaking changes to public traits (`ChromosomeT`, operator traits) — new features via new enums, builder methods, or optional traits
- **Feature flags**: New optional dependencies must be behind feature flags (e.g., `visualization`, `observer-tracing`)
- **Testing**: All PRs must pass `cargo test`, `cargo test --features serde`, `cargo clippy`, zero rustdoc warnings

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Enum + factory for operators | Runtime dispatch without dyn Trait overhead; easy to extend | ✓ Good |
| `Cow<[Gene]>` in ChromosomeT | Zero-copy DNA reads; only clone on mutation | ✓ Good |
| LRU cache keyed on Debug string | Simple to implement; correctness risk if Debug is non-deterministic | ⚠️ Revisit |
| Rayon for parallelism | Fits workload; overhead on small populations | ✓ Good |

## Current Milestone: v2.1.0 — New Examples

**Goal:** Add runnable examples covering all major GA modes and operators, and update README to document them.

**Target features:**
- 6 runnable examples: Rastrigin, NSGA-II (ZDT1), Island Model, Job Scheduling, Feature Selection, Niching
- README updated with examples section and `cargo run --example <name>` commands

---
*Last updated: 2026-03-21 — milestone v2.1.0 New Examples started*
