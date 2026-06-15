---
phase: 68-build-perf-m2-dependency-hygiene
reviewed: 2026-06-15T00:00:00Z
depth: standard
files_reviewed: 73
files_reviewed_list:
  - .github/workflows/feature-matrix.yml
  - CHANGELOG.md
  - Cargo.toml
  - README.md
  - src/engines/cma/engine.rs
  - src/engines/ga.rs
  - src/engines/gp/engine.rs
  - src/engines/island/migration.rs
  - src/engines/island/nsga2.rs
  - src/engines/permutate/engine.rs
  - src/hall_of_fame.rs
  - src/lib.rs
  - src/niching/sharing.rs
  - src/observe/observer/mod.rs
  - src/operations/crossover/arithmetic.rs
  - src/operations/crossover/blend_alpha.rs
  - src/operations/crossover/clone.rs
  - src/operations/crossover/cycle.rs
  - src/operations/crossover/edge_recombination.rs
  - src/operations/crossover/multipoint.rs
  - src/operations/crossover/order.rs
  - src/operations/crossover/pcx.rs
  - src/operations/crossover/pmx.rs
  - src/operations/crossover/rejuvenate.rs
  - src/operations/crossover/sbx.rs
  - src/operations/crossover/single_point.rs
  - src/operations/crossover/spx.rs
  - src/operations/crossover/undx.rs
  - src/operations/crossover/uniform_crossover.rs
  - src/operations/crossover/variable_length.rs
  - src/operations/extension/mass_deduplication.rs
  - src/operations/extension/mass_degeneration.rs
  - src/operations/extension/mass_extinction.rs
  - src/operations/extension/mass_genesis.rs
  - src/operations/mutation.rs
  - src/operations/mutation/bit_flip.rs
  - src/operations/mutation/cauchy.rs
  - src/operations/mutation/differential.rs
  - src/operations/mutation/insertion.rs
  - src/operations/mutation/inversion.rs
  - src/operations/mutation/length_mutation.rs
  - src/operations/mutation/levy_flight.rs
  - src/operations/mutation/non_uniform.rs
  - src/operations/mutation/polynomial.rs
  - src/operations/mutation/scramble.rs
  - src/operations/mutation/self_adaptive_gaussian.rs
  - src/operations/mutation/swap.rs
  - src/operations/mutation/uniform.rs
  - src/operations/selection.rs
  - src/operations/selection/boltzmann.rs
  - src/operations/selection/clearing.rs
  - src/operations/selection/fitness_proportionate.rs
  - src/operations/selection/lexicase.rs
  - src/operations/selection/random.rs
  - src/operations/selection/rank.rs
  - src/operations/selection/tournament.rs
  - src/operations/selection/truncation.rs
  - src/operations/survivor/age.rs
  - src/operations/survivor/deterministic_crowding.rs
  - src/operations/survivor/fitness.rs
  - src/operations/survivor/mu_comma_lambda.rs
  - src/operations/survivor/mu_plus_lambda.rs
  - src/operations/survivor/parsimony.rs
  - src/population.rs
  - src/traits/linear_chromosome.rs
  - src/types/genotypes/list.rs
  - tests/engines/ibea/test_ibea.rs
  - tests/engines/moead/test_moead.rs
  - tests/engines/sms_emoa/test_sms_emoa.rs
  - tests/engines/spea2/test_spea2.rs
  - tests/observe/observer/test_composite_observer.rs
  - tests/observe/observer/test_observer.rs
  - tests/observe/observer/test_sub_trait_observers.rs
  - tests/test_no_logger_installed.rs
findings:
  critical: 1
  warning: 3
  info: 1
  total: 5
status: issues_found
---

# Phase 68: Code Review Report

**Reviewed:** 2026-06-15
**Depth:** standard
**Files Reviewed:** 73
**Status:** issues_found

## Summary

This phase introduced the `logging` optional feature: a `crate::log_*!` macro family in
`src/lib.rs` that delegates to `::log::*` when the feature is enabled and expands to `()`
when disabled, converting 183 call sites across `src/`. `LogObserver` is gated behind
`#[cfg(feature = "logging")]` at both the module level (`observe/observer/mod.rs`) and the
public re-export (`lib.rs`). The CI matrix (`feature-matrix.yml`) gains `no-default-features`
and `logging-explicit` matrix legs.

The macro migration is mechanically sound: all 183 `crate::log_*!` call sites correctly
use the new macros, no bare `use log::` imports remain in `src/` outside expected files,
and all test files that use `LogObserver` correctly gate their usage under
`#[cfg(feature = "logging")]`. The `test_no_logger_installed` integration test properly
validates that the library does not auto-install a logger.

Four issues were found.

## Critical Issues

### CR-01: README documents `LogObserver` as requiring "No feature flags" — now false

**File:** `README.md:312`
**Issue:** The README states `**LogObserver** — logs every hook via the log crate. **No feature
flags required**.` This was true before Phase 68, but `LogObserver` is now gated behind the
`logging` feature. Users who follow this documentation and add
`use genetic_algorithms::LogObserver;` to a crate compiled with
`default-features = false` will get a compile error because `LogObserver` is not in scope.
The same snippet at README lines 314–320 references `LogObserver` in a code block without
any mention of the required feature.

**Fix:**
```markdown
**`LogObserver`** — logs every hook via the `log` crate. Requires the `logging` feature
(enabled by default). Implements `GaObserver`, `IslandGaObserver`, and `Nsga2Observer`.

```toml
# If you disabled default features, re-enable logging explicitly:
genetic_algorithms = { version = "3.0.0", default-features = false, features = ["logging"] }
```
```

Also update the code block at line 314 to add the crate feature requirement in a comment
or guard, consistent with how `observer-metrics` and `observer-tracing` snippets handle it
in the same section.

## Warnings

### WR-01: `logging` feature silently pulls `serde_core` + `serde_derive` as transitive deps

**File:** `Cargo.toml:44`
**Issue:** The `log` dependency is declared as:
```toml
log = { version = "0.4.22", features = ["std", "serde", "kv_unstable"], optional = true }
```
The `"serde"` sub-feature of the `log` crate activates `serde`'s serialization support
for `Level` and `LevelFilter`. Cargo's resolved lock confirms this:
```
name = "log"
version = "0.4.29"
dependencies = ["serde_core", "value-bag"]
```
`serde_core` 1.0.228 (with `serde_derive`) is therefore a transitive dependency of the
`logging` feature, even when the user has not enabled the crate's `serde` feature. This
contradicts the documented promise that disabling default features "sheds `log` for minimal
binary size". More concretely, a WASM or embedded user who enables `logging` but not `serde`
still compiles serde proc-macros as a transitive dep.

Neither `Level` nor `LevelFilter` appear to be serialized anywhere in this codebase;
the `serde` feature on `log` is not needed.

**Fix:**
```toml
# Remove the "serde" sub-feature from the log dependency:
log = { version = "0.4.22", features = ["std", "kv_unstable"], optional = true }
```
This eliminates `serde_core` + `serde_derive` from the transitive closure of the `logging`
feature when the user has not also enabled the `serde` feature.

### WR-02: `feature-matrix.yml` runs only on `push`, not on PRs — `no-default-features` never tested at PR time

**File:** `.github/workflows/feature-matrix.yml:3-5`
**Issue:** The workflow trigger is:
```yaml
on:
  push:
    branches: [main, "milestone/**"]
```
The `no-default-features` and `logging-explicit` matrix legs (which verify that the library
compiles and all tests pass without the `log` crate) only run after merge. A PR that
accidentally re-introduces a `use log::info!` call at a non-gated site would compile fine
with default features (the PR CI only runs `rust-unit-tests.yml` which does not test
`--no-default-features`) and only fail after landing. The `feature-matrix.yml` should also
trigger on `pull_request` for at minimum `main` and `milestone/**` targets.

**Fix:**
```yaml
on:
  push:
    branches: [main, "milestone/**"]
  pull_request:
    branches: [main, "milestone/**"]
```

### WR-03: wasm32 check does not test `--no-default-features` (logging=off) path

**File:** `.github/workflows/wasm-check.yml` (cross-reference with `feature-matrix.yml:46-48`)
**Issue:** `wasm-check.yml` runs three `cargo check --target wasm32-unknown-unknown` steps:
default features, `--features serde`, `--features visualization`. It does not run
`--no-default-features`, which is the path that removes the `log` crate entirely. Since
the `crate::log_*!` macros expand to `()` on that path, a compilation error in the
no-logging code path on wasm32 would not be caught by any CI check.

The `feature-matrix.yml` wasm32 leg runs `cargo check --target wasm32-unknown-unknown --lib`
but again uses default features. There is no test that checks `wasm32 + no-default-features`.

**Fix:** Add a step to `wasm-check.yml`:
```yaml
- name: cargo check (no-default-features, logging off)
  run: cargo check --target wasm32-unknown-unknown --lib --no-default-features
```

## Info

### IN-01: `MIGRATION.md` missing leading `/` in `Cargo.toml` include list

**File:** `Cargo.toml:29`
**Issue:** All other entries in the `include` list use root-anchored paths (`"/src"`,
`"/README.md"`, `"/CHANGELOG.md"`, etc.), but `MIGRATION.md` is listed without a leading
slash:
```toml
include = [
    "/src",
    "/README.md",
    "/CHANGELOG.md",
    ...
    "MIGRATION.md",   # <-- inconsistent, not anchored
]
```
Cargo treats a path without a leading `/` as a glob that can match anywhere in the directory
tree. For a root-level file this is harmless, but it is inconsistent and could silently match
unintended paths if a file named `MIGRATION.md` were ever nested in a subdirectory.

**Fix:**
```toml
    "/MIGRATION.md",
```

---

_Reviewed: 2026-06-15_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
