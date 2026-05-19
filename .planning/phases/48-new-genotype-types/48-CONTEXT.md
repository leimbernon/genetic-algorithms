---
phase: 48-new-genotype-types
status: planned
branch: feat/48-new-genotype-types
target: milestone/v3.0.0
depends_on: [47]
---

## Goal

Introduce three purpose-built chromosome types that replace ad-hoc hacks for common problem categories:

- `UniqueChromosome<T>` — permutation problems (no duplicate genes, runtime guard against single-point/uniform crossover)
- `MultiRangeChromosome<T>` — heterogeneous real-valued spaces with per-gene bounds
- `MultiUniqueChromosome<T>` — multiple independent permutation groups in one chromosome

Migrate `job_scheduling` example to `UniqueChromosome`.

## Requirements

GEN-01, GEN-02, GEN-03, GEN-04
