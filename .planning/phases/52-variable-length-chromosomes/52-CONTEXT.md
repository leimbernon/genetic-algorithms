---
phase: 52-variable-length-chromosomes
status: planned
branch: feat/52-variable-length-chromosomes
target: milestone/v3.0.0
depends_on: [47]
---

## Goal

Allow chromosome length to vary between individuals, enabling structural evolution.

- `ChromosomeLength::Variable { min, max }` configuration
- `Mutation::Insertion` — add a gene at a random position (clamped to `max`)
- `Mutation::Deletion` — remove a gene at a random position (clamped to `min`)
- `Crossover::VariableLength(AlignmentStrategy)` — length-aware crossover with configurable alignment
- Parsimony pressure survivor config — penalises longer chromosomes to prevent unbounded growth

## Requirements

MUT-06, CHR-01, CHR-02
