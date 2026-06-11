---
plan: 25-01
phase: 25-alternative-metaheuristics
status: complete
committed: 2258a9f
note: mark-and-skip — implementation committed 2026-04-26 before SUMMARY was created
---

## What Was Built

Moved `src/chromosomes/` and `src/genotypes/` into `src/types/` group directory (`src/types/chromosomes/` and `src/types/genotypes/`). All existing public paths preserved via `#[path]` attributes in `src/lib.rs`.

## Key Files

- `src/types/chromosomes.rs` — re-exports all chromosome types
- `src/types/genotypes.rs` — re-exports all genotype types
- `src/lib.rs` — updated `#[path]` attributes to map old paths to new locations

## Self-Check: PASSED

All public API paths preserved. Build and tests pass.
