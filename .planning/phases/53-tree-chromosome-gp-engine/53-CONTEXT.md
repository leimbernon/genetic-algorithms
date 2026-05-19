---
phase: 53-tree-chromosome-gp-engine
status: planned
branch: feat/53-tree-chromosome-gp-engine
target: milestone/v3.0.0
depends_on: [50]
---

## Goal

Introduce tree-structured chromosome support for genetic programming.

- `TreeChromosome: ChromosomeT` supertrait — implements ChromosomeT WITHOUT flat-slice methods
- `GpGa<U>` engine — dedicated GP engine with ramped half-and-half initialisation
- Subtree crossover and subtree mutation operators
- Bloat control (configurable depth/node limit)
- Full checkpoint support via `serde_stacker` to handle deep trees
- `Display` impl as expression string

## Requirements

CHR-03, CHR-04, CHR-05, CHR-06, CHR-07
