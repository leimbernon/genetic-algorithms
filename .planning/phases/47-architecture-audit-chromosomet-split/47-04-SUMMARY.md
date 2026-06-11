---
phase: 47-architecture-audit-chromosomet-split
plan: 04
subsystem: configuration, initializers, chromosome types
tags:
  - rust
  - configuration
  - enum-type
  - breaking-change
  - pr-2-start

dependency_graph:
  requires:
    - "47-03 (engine bounds upgraded to LinearChromosome; PR 1 GREEN)"
  provides:
    - "ChromosomeLength enum — Fixed(usize) + Variable { min, max } — public, re-exported from crate root (ARCH-05)"
    - "LimitConfiguration.chromosome_length: ChromosomeLength replaces genes_per_chromosome: usize"
    - "LimitConfiguration.needs_unique_ids and alleles_can_be_repeated fields removed (ARCH-04 partial)"
    - "ConfigurationT trait: with_chromosome_length replaces three removed builder methods"
    - "InitializationFn<G> type alias (src/traits/common.rs) drops Option<bool> third parameter"
    - "Three built-in initializers drop _needs_unique_ids parameter"
    - "24 expected compile errors surfaced at downstream call sites — scoped hand-off to 47-05 and 47-06"
  affects:
    - "PR 2 (ARCH-04 + ARCH-05) — first plan; lib intentionally non-GREEN post-task-2"
    - "47-05 (StoppingCriteria flattening) — resolves ga.rs accessor errors"
    - "47-06 (engine/example migration) — resolves remaining 24 call-site errors"

tech_stack:
  added: []
  patterns:
    - "ChromosomeLength derives Copy+Clone+Debug+PartialEq + cfg_attr serde — same pattern as Selection enum in src/operations.rs"
    - "impl Default for ChromosomeLength returns Fixed(0) — enables embedding in Default-deriving config structs"
    - "InitializationFn<G> is dyn Fn (not fn pointer) — allows closures; location is src/traits/common.rs not src/initializers/mod.rs"

key_files:
  created:
    - src/types/chromosomes/length.rs
    - tests/test_chromosome_length.rs
  modified:
    - src/types/chromosomes/mod.rs
    - src/lib.rs
    - src/configuration.rs
    - src/traits/configuration.rs
    - src/traits/common.rs
    - src/initializers/binary_initializer.rs
    - src/initializers/range_initializer.rs
    - src/initializers/list_initializer.rs
    - src/engines/ga.rs

decisions:
  - "ChromosomeLength landed in src/types/chromosomes/length.rs (not src/chromosomes/length.rs as planned) — directory was src/types/chromosomes/ per the actual repo layout at execution time"
  - "InitializationFn<G> lives in src/traits/common.rs as a dyn Fn trait object (not a fn pointer in src/initializers/mod.rs) — the plan's grep target was wrong; the real location was found and updated correctly"
  - "ga.rs ConfigurationT impl updated to with_chromosome_length in this plan rather than deferring — the impl was adjacent to the field being changed, so it was correct to fix in place"
  - "24 compile errors at downstream sites (engines + tests + examples) are the expected and documented hand-off state to 47-05 and 47-06; no unrelated breakage"

metrics:
  duration: "~4 minutes"
  completed_date: "2026-05-20"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 9
---

# Phase 47 Plan 04: ChromosomeLength Enum + LimitConfiguration Migration — Summary

**One-liner:** Introduced `ChromosomeLength` as a public first-class crate type, replaced `LimitConfiguration.genes_per_chromosome: usize` with `chromosome_length: ChromosomeLength`, removed `needs_unique_ids` and `alleles_can_be_repeated` fields, and updated all three built-in initializer signatures — surfacing 24 expected compile errors at downstream call sites as the documented hand-off to plans 47-05 and 47-06.

## What Was Built

### Task 1: ChromosomeLength enum + Wave 0 tests + re-export

Created `src/types/chromosomes/length.rs` with the `ChromosomeLength` enum:

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}

impl Default for ChromosomeLength {
    fn default() -> Self { ChromosomeLength::Fixed(0) }
}
```

Updated `src/types/chromosomes/mod.rs` to declare the module and re-export `ChromosomeLength`. Updated `src/lib.rs` to add `pub use chromosomes::ChromosomeLength` alongside `ChromosomeT`.

Created `tests/test_chromosome_length.rs` with three Wave 0 tests:
- `test_chromosome_length_variants` — constructs `Fixed(8)` and `Variable { min: 2, max: 16 }`, checks equality
- `test_chromosome_length_default_is_fixed_zero` — asserts `ChromosomeLength::default() == ChromosomeLength::Fixed(0)`
- `test_chromosome_length_serde_roundtrip` (gated `#[cfg(feature = "serde")]`) — JSON round-trip via `serde_json`

`cargo check --lib` GREEN; `cargo test --test test_chromosome_length` GREEN; WASM GREEN after Task 1.

### Task 2: LimitConfiguration migration + initializer cleanup

**`src/configuration.rs` — LimitConfiguration:**
- Removed `pub genes_per_chromosome: usize`
- Removed `pub needs_unique_ids: bool`
- Removed `pub alleles_can_be_repeated: bool`
- Added `pub chromosome_length: ChromosomeLength` (pub(crate) for external callers; accessed via builder)
- Updated `impl Default` to set `chromosome_length: ChromosomeLength::default()`

**`src/traits/configuration.rs` — ConfigurationT trait:**
- Removed `fn with_genes_per_chromosome`, `fn with_needs_unique_ids`, `fn with_alleles_can_be_repeated`
- Added `fn with_chromosome_length(self, length: ChromosomeLength) -> Self`

**`src/traits/common.rs` — InitializationFn type alias:**
- Before: `pub type InitializationFn<G> = dyn Fn(usize, Option<&[G]>, Option<bool>) -> Vec<G> + Send + Sync`
- After: `pub type InitializationFn<G> = dyn Fn(usize, Option<&[G]>) -> Vec<G> + Send + Sync`

**Three built-in initializers** — dropped trailing `_needs_unique_ids: Option<bool>` parameter:
- `binary_random_initialization(genes_per_chromosome, _alleles)` — 2 params
- `range_random_initialization(genes_per_chromosome, alleles)` — 2 params
- `list_random_initialization(genes_per_chromosome, alleles)` — 2 params

**`src/engines/ga.rs`** — `with_chromosome_length` impl updated in the GaConfiguration builder block (adjacent to the changed field).

### Post-Task 2 compile state (documented hand-off)

`cargo check --lib` fails with **24 field-not-found errors** at downstream sites:
- `genes_per_chromosome` reads in `src/engines/ga.rs` and multi-objective engines
- `needs_unique_ids` and `alleles_can_be_repeated` reads in `src/engines/nsga2`, `nsga3`, `moead`, `spea2`
- Affected test and example files

These errors are **exclusively** about the three removed field names — no unrelated breakage. Plans 47-05 (ga.rs accessors + StoppingCriteria) and 47-06 (full engine/example migration) resolve all 24 sites.

## Deviations from Plan

### Auto-navigated (no plan change required)

**1. ChromosomeLength module path: `src/types/chromosomes/` not `src/chromosomes/`**
- The plan assumed `src/chromosomes/length.rs` based on the project guide's module map, but the actual codebase uses `src/types/chromosomes/` (introduced during prior milestone restructuring).
- The executor created the file at the correct path and wired the mod declarations accordingly.

**2. InitializationFn location: `src/traits/common.rs` not `src/initializers/mod.rs`**
- The plan directed `grep -rn 'type InitializationFn' src/initializers/` — the type was not there.
- The executor found it in `src/traits/common.rs` and updated it in place. The `dyn Fn` form (not a bare `fn` pointer) was preserved.

## Deferred Items (47-05 and 47-06)

| Error site | Resolving plan |
|---|---|
| `ga.rs` reads of old fields via StoppingCriteria accessors | 47-05 |
| 8 multi-objective engine reads of `alleles_can_be_repeated` | 47-06 |
| All example and test call sites using old builder methods | 47-06 |

## Known Stubs

None. The `Variable { min, max }` variant of `ChromosomeLength` is wired into the public type and configuration layer but has no behavioral implementation yet — that is Phase 52 scope, as documented in CONTEXT.md D-07.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes. The `serde` derive on `ChromosomeLength` is cfg-gated and verified WASM-clean. No new crate dependencies added.

## Self-Check: PASSED

- `src/types/chromosomes/length.rs` — FOUND with `pub enum ChromosomeLength`, `Copy + Clone + Debug + PartialEq`, serde cfg_attr, `Default` impl
- `src/lib.rs` — FOUND `pub use chromosomes::ChromosomeLength`
- `src/configuration.rs` — FOUND `pub chromosome_length: ChromosomeLength`; no `genes_per_chromosome: usize`, no `needs_unique_ids`, no `alleles_can_be_repeated` fields
- `src/traits/configuration.rs` — FOUND `fn with_chromosome_length`; no `with_genes_per_chromosome`, `with_needs_unique_ids`, `with_alleles_can_be_repeated`
- `src/traits/common.rs` — FOUND `InitializationFn<G>` as `dyn Fn(usize, Option<&[G]>) -> Vec<G> + Send + Sync` (2-param form)
- `binary_random_initialization` — CONFIRMED 2-param signature
- `range_random_initialization` — CONFIRMED 2-param signature
- `list_random_initialization` — CONFIRMED 2-param signature
- Wave 0 tests — CONFIRMED passing (plans 47-05 and 47-06 ran successfully on top of this state)
- Commit `bacc190` (test), `aef50db` (feat), `3d00083` (refactor) — CONFIRMED in git log
