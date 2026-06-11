---
phase: 23-memory-layout
plan: 01
subsystem: genotypes
tags: [arc, copy, inline, range, performance, serde]

# Dependency graph
requires: []
provides:
  - "Arc<[(T,T)]> shared range slice in Range gene (eliminates per-gene heap allocation)"
  - "Copy-specialized Range::value() returning by value without clone()"
  - "#[inline] on FitnessFnWrapper::call() for compiler inlining"
  - "serde rc feature enabling Arc deserialization"
affects: [operations, crossover, mutation, survivor, fitness]

# Tech tracking
tech-stack:
  added: ["serde rc feature"]
  patterns:
    - "Arc<[T]> for shared immutable slice data — prefer over Vec when data is constructed once and read-many"
    - "Copy bound over Clone when all concrete types are Copy — enables returning by value directly"
    - "#[inline] on thin wrapper methods for hot paths"

key-files:
  created: []
  modified:
    - src/genotypes/range.rs
    - src/chromosomes/range.rs
    - src/fitness/fitness_fn_wrapper.rs
    - Cargo.toml
    - tests/chromosomes/test_range.rs

key-decisions:
  - "Range.ranges changed to Arc<[(T,T)]>; constructor signature unchanged (still takes Vec<(T,T)>) to preserve public API"
  - "Copy bound replaces Clone on Range<T> impl blocks and chromosomes::Range<T> — all concrete users (f64, i32) satisfy Copy"
  - "Hash impl updated to use .iter() on Arc<[(T,T)]> since Arc does not impl IntoIterator via &Arc"
  - "chromosomes::Range<T> bounds updated from Clone to Copy to remain consistent with genotypes::Range<T>"

patterns-established:
  - "Arc<[T]> from Vec<T>: use vec.into_boxed_slice().into() in constructors; Arc::from([]) for empty default"
  - "Copy-specialize method bounds: change impl<T: Clone + Default> to impl<T: Copy + Default> when return-by-value is desired"

requirements-completed: [MEM-01, MEM-02, MEM-04]

# Metrics
duration: 8min
completed: 2026-04-03
---

# Phase 23 Plan 01: Memory Layout Summary

**Arc<[(T,T)]> shared range slice, Copy-specialized value(), and #[inline] on fitness call — eliminating per-gene heap allocations and redundant clones**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-03T15:24:44Z
- **Completed:** 2026-04-03T15:32:47Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Migrated `Range.ranges` from `Vec<(T,T)>` to `Arc<[(T,T)]>`, making range bounds shared via ref-count instead of heap-allocated per gene
- Copy-specialized `Range::value()` to return `self.value` directly instead of calling `.clone()` on every access
- Added `#[inline]` to `FitnessFnWrapper::call()` to enable compiler inlining of the thin wrapper
- Enabled serde `rc` feature for `Arc<[(T,T)]>` deserialization support

## Task Commits

Each task was committed atomically:

1. **Task 1: Migrate Range.ranges to Arc and add serde rc feature** - `d4bc8e4` (feat)
2. **Task 2: Copy-specialize Range::value() and inline FitnessFnWrapper::call()** - `eca2202` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified
- `src/genotypes/range.rs` - ranges field changed to Arc<[(T,T)]>, value() returns by copy, bounds updated to Copy
- `src/chromosomes/range.rs` - All impl bounds updated from Clone to Copy for consistency
- `src/fitness/fitness_fn_wrapper.rs` - #[inline] added to call()
- `Cargo.toml` - serde dependency gains "rc" feature
- `tests/chromosomes/test_range.rs` - Struct literal Vec fields converted to Arc via .into()

## Decisions Made
- Constructor signature `new(id, ranges: Vec<(T,T)>, value)` kept unchanged — internal conversion via `into_boxed_slice().into()` preserves public API
- `Copy` replaces `Clone` throughout `Range<T>` impl blocks (genotypes and chromosomes) — `Copy: Clone` so no capability lost; all existing uses (`f64`, `i32`) satisfy `Copy`
- `Hash` impl updated from `for (lo, hi) in &self.ranges` to `self.ranges.iter()` — `&Arc<[T]>` does not implement `IntoIterator`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Hash impl iteration over Arc slice**
- **Found during:** Task 1 (Migrate Range.ranges to Arc)
- **Issue:** `for (lo, hi) in &self.ranges` fails — `&Arc<[(T,T)]>` does not implement `IntoIterator`
- **Fix:** Changed to `self.ranges.iter()` which works via Deref to slice
- **Files modified:** src/genotypes/range.rs
- **Verification:** cargo test exits 0
- **Committed in:** d4bc8e4 (Task 1 commit)

**2. [Rule 1 - Bug] Fixed test struct literals constructing ranges as Vec**
- **Found during:** Task 1 (after Hash fix)
- **Issue:** `tests/chromosomes/test_range.rs` constructs `RangeGenotype { ranges: vec![...] }` struct literals which fail after field type change
- **Fix:** Added `.into()` call on each vec literal to convert to Arc<[(T,T)]>
- **Files modified:** tests/chromosomes/test_range.rs
- **Verification:** cargo test exits 0
- **Committed in:** d4bc8e4 (Task 1 commit)

**3. [Rule 1 - Bug] Updated chromosomes::Range<T> bounds from Clone to Copy**
- **Found during:** Task 2 (Copy-specialize value())
- **Issue:** `chromosomes::Range<T>` uses `T: Clone + Default` bounds on all impl blocks; `phenotype()` calls `gene.value()` which now requires `T: Copy` — compile error
- **Fix:** Updated all `impl<T: Sync + Send + Clone + Default>` blocks in chromosomes/range.rs to use `Copy` instead of `Clone`
- **Files modified:** src/chromosomes/range.rs
- **Verification:** cargo test and cargo clippy both exit 0
- **Committed in:** eca2202 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (3 Rule 1 bugs)
**Impact on plan:** All fixes necessary for compilation correctness. The plan's instruction to change the bound to Copy correctly anticipated the need but didn't enumerate the downstream ripple into chromosomes/range.rs and test struct literals.

## Issues Encountered
- None beyond the auto-fixed compilation errors above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- MEM-01, MEM-02, MEM-04 complete; ready for plan 23-02 (remaining memory layout optimizations)
- All tests pass including serde round-trip

---
*Phase: 23-memory-layout*
*Completed: 2026-04-03*
