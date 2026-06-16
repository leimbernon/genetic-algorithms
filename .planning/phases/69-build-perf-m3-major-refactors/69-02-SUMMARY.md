---
phase: 69-build-perf-m3-major-refactors
plan: "02"
subsystem: infra
tags: [cargo, rayon, parallel, wasm32, feature-flags, ci]

# Dependency graph
requires:
  - phase: 69-01
    provides: divan bench harness foundation needed before gating parallel in bench code
provides:
  - "parallel Cargo.toml feature gating rayon as optional dep (D-05 canonical)"
  - "Three Pattern B ungated rayon import files gated with D-06 combined cfg"
  - "parallel-off CI matrix entry in feature-matrix.yml (will be green after 69-03)"
  - "Canonical gate strings for 69-03 to copy verbatim"
affects:
  - 69-03 (gate remaining 17 rayon call-sites using same D-06 canonical strings)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "D-05: parallel = [dep:rayon] in Cargo.toml features with default = [logging, parallel]"
    - "D-06: parallel arm = #[cfg(all(not(target_arch = wasm32), feature = parallel))]; sequential arm = #[cfg(any(target_arch = wasm32, not(feature = parallel)))]"

key-files:
  created: []
  modified:
    - Cargo.toml
    - src/population.rs
    - src/traits/common.rs
    - src/engines/island/nsga2.rs
    - .github/workflows/feature-matrix.yml

key-decisions:
  - "D-05 canonical Cargo.toml pattern: parallel = [dep:rayon]; rayon = { version = 1.10, optional = true }"
  - "D-06 canonical gate: parallel arm uses all(not(wasm32), feature=parallel); sequential arm uses any(wasm32, not(feature=parallel))"
  - "Feature added to default set so existing users see no behavior change (rayon still on by default)"
  - "parallel-off CI matrix entry added intentionally before 69-03 — will be red until 69-03 gates remaining 17 files"
  - "evolve_islands_one_generation: both par and seq arms end with })?; then Ok(()) to unify return path"

patterns-established:
  - "Pattern B fix: top-level ungated use rayon::prelude::* gets #[cfg(all(not(target_arch = wasm32), feature = parallel))] attribute added before it"
  - "Pattern B call-site: par_iter_mut()/into_par_iter() call kept in parallel arm; iter_mut()/into_iter() added as sequential arm with identical closure body"
  - "Block-local rayon import in function body: remove local use rayon::prelude::*; from block; wrap entire block (without import) in parallel cfg; add sequential sibling block"

requirements-completed: []

# Metrics
duration: 45min
completed: 2026-06-16
---

# Phase 69 Plan 02: parallel feature scaffold + three ungated rayon imports gated

**rayon made optional behind new parallel Cargo feature (D-05); three Pattern B ungated imports in population.rs, traits/common.rs, and engines/island/nsga2.rs gated with D-06 combined cfg; parallel-off CI matrix entry added**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-06-16T09:30:00Z
- **Completed:** 2026-06-16T10:15:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `parallel = ["dep:rayon"]` feature and made rayon optional in Cargo.toml per D-05
- Gated the three Pattern B (ungated) rayon import files with the D-06 canonical combined cfg pattern
- Added parallel-off CI matrix entry to feature-matrix.yml so 69-03 can flip it from red to green
- `cargo check --lib` (default features, all features) passes; wasm32 check passes

## Task Commits

Each task was committed atomically:

1. **Cargo.toml feature declaration** - `27090f2` (build)
2. **Gate three ungated rayon imports** - `670baeb` (fix)
3. **Add parallel-off CI matrix entry** - `3cac835` (ci)

## Files Created/Modified

- `Cargo.toml` - Added parallel = ["dep:rayon"] feature; changed rayon to optional = true; added "parallel" to default feature set
- `src/population.rs` - Gated top-level use rayon::prelude::*; gated par_iter_mut block with parallel arm + sequential iter_mut fallback
- `src/traits/common.rs` - Gated top-level use rayon::prelude::*; converted into_par_iter to parallel/sequential cfg pair
- `src/engines/island/nsga2.rs` - Gated top-level import; converted three rayon usage blocks (initial ranking, evolve loop, island init) to cfg-gated arm pairs
- `.github/workflows/feature-matrix.yml` - Added parallel-off matrix entry with --no-default-features --features logging

## Canonical Gate Strings (for 69-03 to copy verbatim)

**Parallel arm:**
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
```

**Sequential arm:**
```rust
#[cfg(any(target_arch = "wasm32", not(feature = "parallel")))]
```

**Import gate:**
```rust
#[cfg(all(not(target_arch = "wasm32"), feature = "parallel"))]
use rayon::prelude::*;
```

**DO NOT use** the negated form `#[cfg(not(all(not(target_arch = "wasm32"), feature = "parallel")))]` for the sequential arm — this is the forbidden form per D-06 and must not appear.

## Decisions Made

- Used `any(target_arch = "wasm32", not(feature = "parallel"))` for sequential arm (not negated form) — mandated by D-06 and CONTEXT.md
- Added "parallel" to default features so all existing callers see no behavior change; rayon still on by default
- CI matrix entry for parallel-off intentionally added red to ensure 69-03's PR flips it green
- `evolve_islands_one_generation` split into two `try_for_each` arms (par + seq) each ending with `})?;` followed by a shared `Ok(())`

## Deviations from Plan

None — plan executed exactly as written. Both task commits made, canonical gate strings applied as specified.

## Issues Encountered

None — the three Pattern B files were straightforward to gate. The `evolve_islands_one_generation` function had two block-local `use rayon::prelude::*;` imports (one in `run()` and one in `evolve_islands_one_generation()`) that were each converted to cfg-gated blocks without the local import.

## Next Phase Readiness

- Plan 69-03 can copy the canonical gate strings from this SUMMARY verbatim
- 69-03 must gate the remaining ~17 rayon call-sites in Pattern A and Pattern C files
- After 69-03 lands, `cargo test --no-default-features --features logging` will pass and parallel-off CI will go green
- WASM check (`cargo check --target wasm32-unknown-unknown --lib`) passes after this plan

---
*Phase: 69-build-perf-m3-major-refactors*
*Completed: 2026-06-16*
