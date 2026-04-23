---
plan: 23-02
phase: 23-memory-layout
status: complete
completed: 2026-04-04
requirements: [MEM-03, MEM-05]
---

# Plan 23-02: Remove generation_numbers + DefaultHasher Dedup

## Objective
Remove unused `generation_numbers` field from `Population` struct (including serde impls) and replace `HashSet<Vec<i32>>` with incremental `DefaultHasher` in `MassDeduplication`.

## What Was Built

### Task 1: Remove generation_numbers from Population
- Removed `pub generation_numbers: Vec<usize>` from `Population` struct definition and doc comment
- Removed `generation_numbers: vec![]` from `Population::new_empty()` and `Population::new()`
- Removed `generation_numbers: self.generation_numbers.clone()` from `Clone` impl
- Updated serde `Serialize`: changed field count 6→5, removed `generation_numbers` field
- Updated serde `Deserialize`: removed `GenerationNumbers` variant, local var, match arm, construction field, and `FIELDS` entry
- `FIELDS` array now contains exactly 5 entries: chromosomes, best_chromosome, best_chromosome_is_set, f_avg, f_max

### Task 2: DefaultHasher-based MassDeduplication
- Replaced `HashSet<Vec<i32>>` with `HashMap<u64, Vec<i32>>` for dedup tracking
- Implements incremental hashing via `DefaultHasher`: hashes each gene's `id()` without allocating a `Vec<i32>` on the common path
- Collision-safe: on hash collision, falls back to exact `Vec<i32>` comparison to distinguish true duplicates from hash collisions
- Semantics preserved: sort by fitness (best first) ensures the first-seen chromosome is the best one

## Key Files

**key-files.modified:**
- `src/population.rs` — Population struct without generation_numbers; serde with 5 fields
- `src/operations/extension/mass_deduplication.rs` — DefaultHasher-based dedup with HashMap

## Verification

- `grep "generation_numbers" src/population.rs` → no matches ✓
- `grep "HashSet" src/operations/extension/mass_deduplication.rs` → no matches ✓
- `grep "DefaultHasher" src/operations/extension/mass_deduplication.rs` → matches ✓
- `cargo test` → 22 passed, 0 failed ✓
- `cargo test --features serde` → 22 passed, 0 failed ✓

## Self-Check: PASSED

All acceptance criteria met. No deviations from plan.
