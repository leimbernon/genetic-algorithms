---
phase: 40-constraint-handling-penalty-functions-feasibility-rules-repa
plan: 01
status: complete
subsystem: tests
tags: [constraints, tests, adaptive-penalty, feasibility-rules]
metrics:
  tests_passed: 8
  tasks_completed: 2
key-files:
  created:
    - tests/test_constraints.rs
  updated:
    - tests/test_constraints.rs
---

## Summary

Fixed all 10 compilation errors in `tests/test_constraints.rs` caused by API changes (RangeGene `id_values()` → `value()`, i32 → usize conversions, import paths). Added two new GA integration tests for adaptive penalty and feasibility rules.

## Tasks

1. **Fixed compilation errors** — Updated `id_values()` calls to `value()`, fixed `ProblemSolving` import path, added `ChromosomeT` import, converted `n` i32 to usize, resolved unused import warnings.
2. **Added missing integration tests** — `test_constraint_handling_adaptive_penalty` and `test_constraint_handling_feasibility_rules` both pass.

## Self-Check: PASSED

- All 8 tests pass via `cargo test --test test_constraints`
- `cargo clippy --tests` reports zero warnings in test_constraints.rs
