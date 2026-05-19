---
phase: 51-multi-parent-crossover-self-adaptive-mutation
status: planned
branch: feat/51-multi-parent-crossover-self-adaptive-mutation
target: milestone/v3.0.0
depends_on: [47]
---

## Goal

Extend the real-valued chromosome operator set with multi-parent crossover and self-adaptive mutation.

Crossover operators (require `RealValued` marker trait; binary/permutation chromosomes return `GaError`):
- `Crossover::Undx { num_parents }` — Unimodal Normal Distribution Crossover
- `Crossover::Spx { num_parents }` — Simplex Crossover
- `Crossover::Pcx { num_parents }` — Parent-Centric Crossover

Self-adaptive mutation:
- `SelfAdaptive: ChromosomeT` — per-chromosome sigma vector co-evolves alongside the solution
- `Mutation::SelfAdaptiveGaussian` — log-normal sigma update (strategy parameter evolution)

## Requirements

CRS-02, CRS-03, CRS-04, MUT-05, TRAITS-02
