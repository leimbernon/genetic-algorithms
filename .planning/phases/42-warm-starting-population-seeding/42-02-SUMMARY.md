---
phase: 42-warm-starting-population-seeding
plan: 02
type: execute
generated: 2026-05-13
---

# Plan 42-02 — Seed-Based Population Initialization

## Objective

Implement seed-based population initialization with genotypic dedup, trusted fitness preservation, and Hall of Fame seed admission.

## What Was Built

### 1. `src/engines/ga.rs` — Seed-aware initialization

- **`initialize_random()`** — extracted the existing random initialization logic into a private helper method. Called when `seeds` is `None` (zero behavioral change).

- **`initialize_with_seeds()`** — new seed-aware initialization method:
  - Consumes `self.seeds` (seeds only used at init time) and places them at the front of the population.
  - Generates random fill chromosomes for remaining slots (`population_size - seeds.len()`).
  - Genotypic dedup: each fill chromosome is compared against all seed DNA (via `gene.id()`) and against other fill chromosomes. Duplicates are discarded and regenerated.
  - Max retry bound (`fill_count * 10`) prevents infinite loops; returns `InitializationError` if exhausted.
  - Seed fitness is trusted (skips `calculate_fitness`); fill chromosomes ARE evaluated.
  - When HallOfFame is configured, all population chromosomes are admitted during initialization (generation 0).

- **`initialization()`** — modified to branch on `self.seeds.is_some()` → delegates to seed-aware or random path. Repair operator still applied post-init.

- **Builder validation** (from 42-01):
  - `with_seeds()` builder method
  - `build()` validates seeds count does not exceed `population_size`
  - `build()` validates mutual exclusivity with `with_checkpoint()`

### 2. `tests/engines/warm_starting/test_warm_starting.rs` — Integration tests

- Builder validation tests: `test_wsm_with_seeds_builds_successfully`, `test_wsm_with_seeds_exceeds_population_errors`
- Mutual exclusivity test: `test_wsm_seeds_and_checkpoint_mutually_exclusive`
- Seed init tests: `test_wsm_seeds_population_size_matches` (exact fit), `test_wsm_seeds_admitted_to_hall_of_fame`, `test_wsm_seeds_without_hall_of_fame`

## Self-Check

| Check | Result |
|-------|--------|
| `cargo check` | PASS \
| Seed fitness preserved (not re-evaluated) | YES — trusted fitness per D-07 |
| Seeds placed at front of population | YES |
| Fill dedup against seed DNA | YES — genotypic via gene.id() |
| Fill dedup against other fill | YES — same comparison pattern |
| HOF admission during init | YES — generation 0 |
| WASM compatible | YES — pure data operations |

## Files Changed

- `src/engines/ga.rs` — +242/-8 lines (seed-aware init, builder methods, validation)
- `tests/engines/warm_starting/test_warm_starting.rs` — new file (255 lines, builder + seed init tests)

## Deviation Notes

- Test design simplified: uses exact-fit population (seeds count == population_size) for deterministic seed-fitness verification, avoiding fill dedup randomness in tests.
