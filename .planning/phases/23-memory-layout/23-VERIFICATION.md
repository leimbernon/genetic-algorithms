---
phase: 23-memory-layout
verified: 2026-04-04T00:00:00Z
status: passed
score: 5/5 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 2/5
  gaps_closed:
    - "Range genes share a single Arc-allocated range slice instead of per-gene Vec heap allocation"
    - "Range::value() returns by value for Copy types without calling .clone()"
    - "FitnessFnWrapper::call() is annotated with #[inline]"
  gaps_remaining: []
  regressions: []
---

# Phase 23: Memory Layout Verification Report

**Phase Goal:** Improve memory layout and reduce allocations — migrate Range gene to Arc shared slice, specialize value() for Copy types, inline FitnessFnWrapper::call(), remove unused generation_numbers field, and replace HashSet<Vec<i32>> with incremental DefaultHasher in MassDeduplication.
**Verified:** 2026-04-04
**Status:** passed
**Re-verification:** Yes — after gap closure (previous score 2/5, now 5/5)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Range genes share a single Arc-allocated range slice instead of per-gene Vec heap allocation | VERIFIED | `src/genotypes/range.rs` line 11: `use std::sync::Arc;`; line 46: `pub ranges: Arc<[(T, T)]>`; line 73: `ranges: Arc::from([])`; line 102: `ranges.into_boxed_slice().into()` |
| 2 | Range::value() returns by value for Copy types without calling .clone() | VERIFIED | `src/genotypes/range.rs` line 89: `impl<T: Copy + Default> Range<T>`; line 107: `self.value` (no .clone()) |
| 3 | FitnessFnWrapper::call() is annotated with #[inline] | VERIFIED | `src/fitness/fitness_fn_wrapper.rs` line 53: `#[inline]` immediately before `pub fn call(&self, dna: &[G]) -> f64` at line 54 |
| 4 | Population struct has no generation_numbers field | VERIFIED | `src/population.rs` grep returns no matches for `generation_numbers`; serde uses `serialize_struct("Population", 5)` |
| 5 | MassDeduplication uses incremental DefaultHasher | VERIFIED | `src/operations/extension/mass_deduplication.rs` line 11: `use std::hash::{DefaultHasher, Hash, Hasher}`; line 48: `HashMap<u64, Vec<i32>>`; lines 51-53: `DefaultHasher::new()`, `g.id().hash(&mut hasher)`, `hasher.finish()` |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/genotypes/range.rs` | Arc<[(T,T)]> range field and Copy-specialized value() | VERIFIED | Arc import at line 11; `pub ranges: Arc<[(T, T)]>` at line 46; `impl<T: Copy + Default>` at line 89; `self.value` (no clone) at line 107 |
| `src/fitness/fitness_fn_wrapper.rs` | #[inline] annotation on call() | VERIFIED | `#[inline]` at line 53 directly above `pub fn call` at line 54 |
| `Cargo.toml` | serde rc feature for Arc deserialization | VERIFIED | `serde = { version = "1", features = ["derive", "rc"], optional = true }` at line 32 |
| `src/population.rs` | Population struct without generation_numbers field | VERIFIED | No `generation_numbers` in file; `serialize_struct("Population", 5)` at line 292 |
| `src/operations/extension/mass_deduplication.rs` | DefaultHasher-based deduplication | VERIFIED | `DefaultHasher`, `Hash`, `Hasher` imported; `HashMap<u64, Vec<i32>>` used; no `HashSet<Vec<i32>>` present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/genotypes/range.rs` | `Cargo.toml` | serde rc feature enables Arc<[T]> derive Deserialize | WIRED | Cargo.toml line 32: `features = ["derive", "rc"]`; Arc field in range.rs at line 46 |
| `src/population.rs` | `tests/test_serde.rs` | serde round-trip validates Population without generation_numbers | WIRED | `serde_population_binary_round_trip` test exercises Population serde; struct serializes 5 fields |
| `src/operations/extension/mass_deduplication.rs` | `tests/extension/test_extension.rs` | mass_deduplication tests validate dedup behavior | WIRED | Four tests present: `mass_deduplication_removes_duplicates`, `mass_deduplication_keeps_best_minimization`, `mass_deduplication_all_unique`, `mass_deduplication_empty_population` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| MEM-01 | 23-01 | Range genes use Arc<[(T,T)]> shared range slice | SATISFIED | `pub ranges: Arc<[(T, T)]>` at line 46; `use std::sync::Arc` at line 11; Cargo.toml serde rc feature present |
| MEM-02 | 23-01 | Range::value() returns by value for Copy types | SATISFIED | `impl<T: Copy + Default>` at line 89; `self.value` (no `.clone()`) at line 107 |
| MEM-03 | 23-02 | Unused generation_numbers removed from Population | SATISFIED | No `generation_numbers` anywhere in `src/population.rs`; serde count is 5 |
| MEM-04 | 23-01 | FitnessFnWrapper::call() annotated with #[inline] | SATISFIED | `#[inline]` at line 53 of `src/fitness/fitness_fn_wrapper.rs` |
| MEM-05 | 23-02 | MassDeduplication uses incremental DefaultHasher | SATISFIED | `DefaultHasher::new()`, incremental `g.id().hash(&mut hasher)`, `hasher.finish()`, `HashMap<u64, Vec<i32>>` all present |

**Note on REQUIREMENTS.md status:** The requirements file still shows MEM-01 and MEM-02 as `[x]` (complete) and MEM-03 and MEM-05 as `[ ]` (pending), which was an inversion documented in the initial verification. Now that all five are implemented in code, the requirements file should be updated to mark all five as `[x]`.

### Anti-Patterns Found

None. All previously flagged anti-patterns have been resolved:
- `Vec<(T, T)>` replaced with `Arc<[(T, T)]>` in `src/genotypes/range.rs`
- `self.value.clone()` replaced with `self.value` in `src/genotypes/range.rs`
- `#[inline]` added to `FitnessFnWrapper::call()` in `src/fitness/fitness_fn_wrapper.rs`

### Human Verification Required

None — all verification items are code-level checks confirmed programmatically.

### Re-verification Summary

All three gaps from the initial verification (2026-04-04 score 2/5) have been closed:

1. **MEM-01 (Arc migration)** — `src/genotypes/range.rs` now uses `Arc<[(T, T)]>` for the ranges field; `use std::sync::Arc` is imported; `new()` converts via `into_boxed_slice().into()`; `Default` uses `Arc::from([])`; Cargo.toml has `"rc"` in serde features.

2. **MEM-02 (Copy specialization)** — `impl<T: Copy + Default> Range<T>` replaces the previous `Clone + Default` bound; `value()` returns `self.value` directly without `.clone()`.

3. **MEM-04 (#[inline])** — `#[inline]` is present at line 53 of `src/fitness/fitness_fn_wrapper.rs` immediately before `pub fn call`.

No regressions found in the two previously verified items (MEM-03 and MEM-05).

---

_Verified: 2026-04-04_
_Verifier: Claude (gsd-verifier)_
