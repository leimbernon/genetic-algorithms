---
phase: 49-unified-strategy-engines
status: planned
branch: feat/49-unified-strategy-engines
target: milestone/v3.0.0
depends_on: [47]
---

## Goal

Introduce a `Strategy<U>` trait so users can swap between GA, hill-climbing, and exhaustive permutation search at runtime via `Box<dyn Strategy<U>>` without rewriting application code.

Engines to implement:
- `HillClimbEngine<U>` — Stochastic hill climbing and SteepestAscent variants
- `PermutateEngine<U>` — Exhaustive permutation search with safety gate (size limit)

Observer hooks wired throughout all new engines.

## Requirements

STR-01, STR-02, STR-03, STR-04
