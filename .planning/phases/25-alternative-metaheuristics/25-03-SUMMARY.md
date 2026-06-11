---
plan: 25-03
phase: 25-alternative-metaheuristics
status: complete
committed: 2258a9f
note: mark-and-skip — implementation committed 2026-04-26 before SUMMARY was created
---

## What Was Built

Moved ga, island, and nsga2 engine modules into `src/engines/` group directory. Created placeholder stubs for future engines: `de/`, `scatter/`, `cellular/`, `alps/`. All public paths preserved. Pre-existing rustdoc warnings and clippy issues fixed.

## Key Files

- `src/engines/ga.rs` — main GA engine
- `src/engines/island/` — island model
- `src/engines/nsga2/` — NSGA-II multi-objective
- `src/engines/de/mod.rs` — placeholder stub
- `src/engines/scatter/mod.rs` — placeholder stub
- `src/engines/cellular/mod.rs` — placeholder stub
- `src/engines/alps/mod.rs` — placeholder stub
- `src/lib.rs` — updated `#[path]` attributes

## Self-Check: PASSED

Full test suite passes including serde, clippy, and rustdoc. All public API paths preserved.
