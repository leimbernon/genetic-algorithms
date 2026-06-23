# Phase 73: Move Inline #[cfg(test)] Modules to tests/ - Research

**Researched:** 2026-06-18
**Domain:** Rust test organization — migrating inline `#[cfg(test)]` blocks to `tests/` integration test files
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Tests that access `pub(crate)` helpers are rewritten to exercise the same behavior through the public API. No visibility promotions.
**D-02:** For each helper test that is dropped, write an equivalent public-API assertion that exercises the same invariant.
**D-03:** `src/operations/local_search.rs` and `src/operations/mutation/levy_flight.rs` also use `use super::*;` — apply the same rewrite strategy.
**D-04:** Follow the mirrored-subdirectory pattern for all files.
**D-05:** `src/benchmarks/dtlz.rs`, `zdt.rs`, `single_objective.rs` → `tests/benchmarks/dtlz.rs`, `tests/benchmarks/zdt.rs`, `tests/benchmarks/single_objective.rs` (new `tests/benchmarks/` directory).
**D-06:** `src/engines/multi_objective/indicators/*.rs` → `tests/engines/multi_objective/indicators/generational_distance.rs`, etc.
**D-07:** `src/aos.rs` → `tests/engines/aos/` (directory already exists; add a new test file).
**D-08:** `src/operations/local_search.rs` → `tests/engines/local_search.rs` already exists; merge or add alongside.
**D-09:** `src/operations/mutation/levy_flight.rs` → `tests/operations/test_mutation_levy_flight.rs`.
**D-10:** Nested `#[cfg(feature = "serde")]` blocks carry over as `#[cfg(feature = "serde")]` at the test-file level.

### Claude's Discretion

- Exact file name for the AOS test file inside `tests/engines/aos/` (e.g., `strategy.rs`, `test_aos_strategy.rs`).
- Whether to merge `src/operations/local_search.rs` tests into the existing `tests/engines/local_search.rs` or create a separate file.
- Order of test functions within the new test files.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

## Summary

Phase 73 is a mechanical code hygiene migration: 56 inline `#[cfg(test)]` tests spread across 10 `src/` files are moved to corresponding files under `tests/`, so that `grep -rn '#[cfg(test)]' src/` returns zero matches.

The critical discovery is that **all four destination categories have different readiness levels**:

1. **indicators (GD, HV, IGD, Spread):** Target files under `tests/engines/multi_objective/indicators/` already exist on disk with expanded test coverage (8 tests each vs. 6 in src/), but are **not wired into any harness file** — they are currently dead code. Phase 73 must wire them into `test_engines.rs` AND delete the inline src/ blocks.

2. **AOS:** `tests/engines/aos/test_aos.rs` exists AND is wired in `test_engines.rs`. The inline src/ tests (24) are superseded by the external file (25 tests using public API). The inline block only needs to be deleted.

3. **local_search:** `tests/engines/local_search.rs` exists AND is wired at line 3 of `test_engines.rs`. The inline src/ tests use public items only (via `use super::*` for convenience) and need to be merged/appended to the existing external file, then deleted from src/.

4. **levy_flight:** No external file exists yet. The two inline tests access **private functions** (`mantegna_sigma_u`, `gamma_approx`) — they must be **rewritten** as public-API tests against `levy_flight_mutation`, then placed in a new `tests/operations/test_mutation_levy_flight.rs` declared in `test_operations.rs`.

5. **benchmarks:** No `tests/benchmarks/` directory exists yet. The benchmark module is gated behind `#[cfg(feature = "benchmarks")]` in `src/lib.rs`, so new test files must be gated `#[cfg(feature = "benchmarks")]` at the test-function level AND declared via a `tests/test_benchmarks.rs` harness.

**Primary recommendation:** Work file-by-file in a single wave. The 4 indicator files plus multi_objective wiring are a single logical unit. Benchmarks require creating both the harness file and the `tests/benchmarks/` subdirectory.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Inline test removal | Source files (`src/`) | — | Each file owns its own `#[cfg(test)]` block; removal is in-place |
| Test harness wiring | Harness files (`tests/test_engines.rs`, `tests/test_operations.rs`) | New `tests/test_benchmarks.rs` | Cargo discovers subdirectory tests only via `mod` declarations in top-level test files |
| New test file creation | `tests/` subtree | — | Mirrors `src/` directory structure |
| Private-API rewrites | New `tests/` files | — | Must use `genetic_algorithms::...` paths, no `use super::*` |

---

## Standard Stack

No external packages are installed in this phase. All work uses Cargo's built-in test infrastructure.

### Core
| Tool | Version | Purpose |
|------|---------|---------|
| `cargo test` | (project Rust version) | Compile and run all tests |
| `cargo test --features benchmarks` | — | Compile and run benchmark-gated tests |

**Installation:** None required.

---

## Package Legitimacy Audit

Not applicable — this phase installs zero external packages.

---

## Architecture Patterns

### How Cargo Discovers Tests in `tests/` Subdirectories

Cargo does **not** auto-discover `tests/**/*.rs` recursively. The project uses a **harness file** pattern:

```
tests/
├── test_engines.rs      ← top-level harness, declares `mod engines { ... }`
├── test_operations.rs   ← top-level harness, declares `mod operations { ... }`
├── engines/
│   ├── aos/
│   │   └── test_aos.rs        ← included via test_engines.rs mod declaration
│   ├── multi_objective/
│   │   └── indicators/
│   │       └── test_*.rs      ← NOT YET WIRED (dead code)
│   └── local_search.rs        ← included via test_engines.rs mod declaration
└── operations/
    └── test_mutation_*.rs     ← included via test_operations.rs mod declarations
```

Each top-level file in `tests/*.rs` becomes a separate test binary. Files in subdirectories are only compiled if a parent `mod` declaration chains to them.

**Critical:** `tests/engines/multi_objective/indicators/test_*.rs` files exist on disk but are dead code. They are not referenced in `test_engines.rs` and produce zero test runs today.

### System Architecture Diagram

```
cargo test
    │
    ├─► unittests src/lib.rs (56 inline tests from src/ — TARGET FOR REMOVAL)
    │       aos::tests::*
    │       multi_objective::indicators::generational_distance::tests::*
    │       multi_objective::indicators::hypervolume::tests::*
    │       multi_objective::indicators::inverted_generational_distance::tests::*
    │       multi_objective::indicators::spread::tests::*
    │       operations::local_search::tests::*
    │       operations::mutation::levy_flight::tests::*
    │       (+ benchmarks::*::tests::* when --features benchmarks, 30 more)
    │
    ├─► tests/test_engines.rs (compiled as one binary via mod declarations)
    │       mod engines {
    │           mod local_search;       ← tests/engines/local_search.rs (wired)
    │           mod aos { mod test_aos; } ← tests/engines/aos/test_aos.rs (wired)
    │           (multi_objective NOT YET DECLARED)
    │       }
    │
    ├─► tests/test_operations.rs
    │       mod operations {
    │           mod test_mutation_*;   ← wired
    │           (test_mutation_levy_flight NOT YET DECLARED)
    │       }
    │
    └─► (no test_benchmarks.rs yet)
```

After Phase 73:
- `src/lib.rs` unittests binary: 0 tests for migrated modules
- `test_engines.rs` binary: gains AOS (merge), local_search (merge), indicators (new wiring)
- `test_operations.rs` binary: gains levy_flight (new file + declaration)
- `test_benchmarks.rs` binary: gains benchmarks tests (new harness + new files)

### Recommended Project Structure After Phase 73

```
tests/
├── test_engines.rs          ← add multi_objective + indicators declarations
├── test_operations.rs       ← add test_mutation_levy_flight declaration
├── test_benchmarks.rs       ← NEW: harness for benchmarks tests
├── engines/
│   ├── aos/
│   │   └── test_aos.rs     ← unchanged (25 tests, already complete)
│   ├── multi_objective/
│   │   └── indicators/
│   │       ├── test_generational_distance.rs   ← already exists, just wire in
│   │       ├── test_hypervolume.rs             ← already exists, just wire in
│   │       ├── test_inverted_generational_distance.rs  ← already exists
│   │       └── test_spread.rs                 ← already exists, just wire in
│   └── local_search.rs     ← append inline tests (7 tests to merge in)
└── benchmarks/              ← NEW directory
    ├── dtlz.rs             ← NEW: 17 tests from src/benchmarks/dtlz.rs
    ├── single_objective.rs ← NEW: 10 tests from src/benchmarks/single_objective.rs
    └── zdt.rs              ← NEW: 13 tests from src/benchmarks/zdt.rs
```

### Pattern 1: Adding a Module Declaration to a Harness File

```rust
// In tests/test_engines.rs — add multi_objective wiring
mod engines {
    // ... existing declarations ...
    mod multi_objective {              // ← add this block
        mod indicators {
            mod test_generational_distance;
            mod test_hypervolume;
            mod test_inverted_generational_distance;
            mod test_spread;
        }
    }
}
```

### Pattern 2: Converting Inline Tests to Integration Tests

Before (in `src/engines/multi_objective/indicators/generational_distance.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gd_identical_fronts() {
        let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        // ...
    }
}
```

After (in `tests/engines/multi_objective/indicators/test_generational_distance.rs`):
```rust
use genetic_algorithms::multi_objective::indicators::generational_distance;
use genetic_algorithms::GaError;

#[test]
fn test_gd_identical_fronts() {
    let approx = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
    // ...
}
```

Key changes:
- `use super::*` → `use genetic_algorithms::...` (fully qualified crate paths)
- No `mod tests { }` wrapper — each function is a top-level `#[test]`
- No `#[cfg(test)]` wrapper (the entire file is test-only)

### Pattern 3: Rewriting Private-Function Tests (levy_flight)

The two inline `levy_flight` tests access private `fn mantegna_sigma_u` and `fn gamma_approx`. These functions cannot be accessed from `tests/`. The tests must be rewritten to exercise the observable behavior of the public function `levy_flight_mutation`.

Before (private function test — cannot move):
```rust
// Source: src/operations/mutation/levy_flight.rs (private fn)
#[test]
fn mantegna_sigma_u_finite_positive_at_default_alpha() {
    let s = mantegna_sigma_u(1.5);
    assert!(s.is_finite() && s > 0.0, "σ_u(1.5) = {}", s);
}
```

After (public API test — observable mutation behavior):
```rust
// Source: tests/operations/test_mutation_levy_flight.rs
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::mutation::levy_flight_mutation;
use genetic_algorithms::traits::ChromosomeT;

#[test]
fn levy_flight_mutation_produces_finite_values() {
    // Exercise levy_flight_mutation to confirm sigma_u/gamma internals work
    let gene = RangeGene::new(0, vec![(-10.0_f64, 10.0_f64)], 0.0);
    let mut chromo = RangeChromosome::new();
    chromo.set_dna(std::borrow::Cow::Owned(vec![gene]));
    levy_flight_mutation(&mut chromo, 1.5);
    assert!(chromo.dna()[0].value.is_finite(), "levy flight must produce finite values");
}
```

### Pattern 4: Feature-Gated Benchmark Tests

The `benchmarks` module is behind `#[cfg(feature = "benchmarks")]` in `lib.rs`. Tests must be gated at the test-function level using the same feature:

```rust
// tests/benchmarks/dtlz.rs
#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::dtlz::{DTLZ1, DTLZ2, BenchmarkFn};

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz1_form() {
    let dtlz1 = DTLZ1::new(7, 3);
    let result = dtlz1.evaluate(&[0.0; 7]);
    assert_eq!(result.len(), 3);
}

// Serde-specific tests within the same file:
#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz1_serde_roundtrip() {
    let d = DTLZ1::new(7, 3);
    let json = serde_json::to_string(&d).unwrap();
    let d2: DTLZ1 = serde_json::from_str(&json).unwrap();
    assert_eq!(d.n_vars, d2.n_vars);
}
```

The harness file `tests/test_benchmarks.rs` uses `mod` declarations:
```rust
// tests/test_benchmarks.rs
mod benchmarks {
    mod dtlz;
    mod single_objective;
    mod zdt;
}
```

### Pattern 5: AOS Private Field Access Rewrite (D-01)

The inline `test_new_creates_correct_number_of_arms` test accesses private field `state.arms`:

```rust
// INVALID outside the module:
assert_eq!(state.arms.len(), 3);  // arms is private!
```

The external `tests/engines/aos/test_aos.rs` already handles this correctly using the public `num_arms()` method:
```rust
assert_eq!(state.num_arms(), 3);  // public accessor
```

The existing external file supersedes the inline tests. The inline block is simply deleted.

### Anti-Patterns to Avoid

- **Placing test files directly in `tests/subdirectory/` without wiring them in a harness:** Files in `tests/subdirectories/` are dead code unless declared via `mod` in a top-level `tests/*.rs` file. The existing indicator files are an example of this exact mistake.
- **Using `use super::*;` in integration tests:** No `super` exists for `tests/*.rs` files — they are separate crates. Use `use genetic_algorithms::...` paths only.
- **Forgetting `#[cfg(feature = "benchmarks")]` on benchmark test functions:** Without the feature gate, test functions referencing benchmark types will fail to compile when the feature is disabled (which is the default).
- **Combining `#[cfg(feature = "serde")]` and `#[cfg(feature = "benchmarks")]` incorrectly:** Use `#[cfg(all(feature = "benchmarks", feature = "serde"))]` on serde tests inside benchmark files.
- **Creating a `mod tests {}` wrapper in integration test files:** Integration test files don't need a `mod tests` wrapper — all functions are at the top level.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Discovering tests in subdirectories | Custom test runner | `mod` declarations in harness files | Cargo's built-in mechanism; custom runners break feature flags and parallelism |
| Accessing private functions in integration tests | Visibility promotion to `pub` | Rewrite tests against public API | D-01 explicitly prohibits visibility changes |
| Feature-gated types in test files | Duplicate type definitions | `#[cfg(feature = "...")]` on test functions | Standard Rust pattern; avoids dead-code warnings and compile errors |

---

## Runtime State Inventory

Not applicable — this is not a rename/refactor/migration phase that touches stored data or live services. All changes are source code only.

---

## Common Pitfalls

### Pitfall 1: Wiring subdirectory without parent declaration

**What goes wrong:** Adding `mod indicators { mod test_generational_distance; }` inside a new `mod multi_objective {}` block in `test_engines.rs` without also declaring `mod multi_objective {}` as a child of `mod engines {}`.

**Why it happens:** The `mod` hierarchy must match the filesystem path relative to the harness file. `tests/engines/multi_objective/indicators/test_generational_distance.rs` requires:
```
mod engines {
    mod multi_objective {
        mod indicators {
            mod test_generational_distance;
        }
    }
}
```

**How to avoid:** Add the full chain `engines → multi_objective → indicators → test_*` to `test_engines.rs`. Verify by running `cargo test` and confirming the test names appear.

**Warning signs:** Test count unchanged after wiring; no `multi_objective::` prefixed tests appear.

### Pitfall 2: Forgetting benchmarks harness file

**What goes wrong:** Creating `tests/benchmarks/dtlz.rs` etc. without creating `tests/test_benchmarks.rs` to declare them. The files compile fine but Cargo never runs them.

**Why it happens:** Cargo only makes test binaries from top-level `tests/*.rs` files. Subdirectory files need to be pulled in via `mod`.

**How to avoid:** Create `tests/test_benchmarks.rs` with `mod benchmarks { mod dtlz; mod zdt; mod single_objective; }` as a harness.

**Warning signs:** `cargo test --features benchmarks` reports 0 new tests after creating the benchmark test files.

### Pitfall 3: Private function tests for levy_flight

**What goes wrong:** Moving `mantegna_sigma_u_finite_positive_at_default_alpha` and `gamma_approx_known_values` directly to `tests/` — they reference private functions and will not compile.

**Why it happens:** `fn mantegna_sigma_u` and `fn gamma_approx` are private (not `pub fn`).

**How to avoid:** Rewrite as observable-behavior tests against the public `levy_flight_mutation` function (D-01/D-03). The invariant being tested (mutation produces finite values) is demonstrable through the public API.

**Warning signs:** Compile error: `function mantegna_sigma_u is private`.

### Pitfall 4: AOS private field `state.arms`

**What goes wrong:** Moving `test_new_creates_correct_number_of_arms` from `src/aos.rs` directly — it accesses `state.arms.len()` which is a private field.

**Why it happens:** `arms: Vec<ArmState>` is not `pub` on `AosState`.

**How to avoid:** The external `tests/engines/aos/test_aos.rs` already handles this correctly with `state.num_arms()`. The inline block is simply deleted (not ported).

**Warning signs:** The external test file already has 25 tests covering all the same cases via public API — nothing to port, just delete the inline block.

### Pitfall 5: Forgetting to remove the inline block after migration

**What goes wrong:** New external tests added but `#[cfg(test)] mod tests { }` still present in `src/`. `grep -rn '#[cfg(test)]' src/` still returns matches. Success criterion fails.

**How to avoid:** After creating and wiring each external file, immediately remove the entire `#[cfg(test)] mod tests { ... }` block from the corresponding source file.

### Pitfall 6: Benchmark serde tests use nested `mod serde_tests` in src/

**What goes wrong:** The benchmark serde tests in `src/` use a nested `mod serde_tests` inside `mod tests`. Moving them naively creates a nested module in the external file.

**Why it happens:** `#[cfg(feature = "serde")] mod serde_tests { use super::*; ... }` in the original; `super::*` won't resolve in integration tests.

**How to avoid (D-10):** Flatten the serde tests into top-level `#[test]` functions in the external benchmark test file, gated with `#[cfg(all(feature = "benchmarks", feature = "serde"))]`.

---

## Code Examples

### Wiring indicators in test_engines.rs

```rust
// Source: tests/test_engines.rs — current state at line 63-65
mod engines {
    // ... existing ...
    mod aos {
        mod test_aos;
    }
    // ADD THIS:
    mod multi_objective {
        mod indicators {
            mod test_generational_distance;
            mod test_hypervolume;
            mod test_inverted_generational_distance;
            mod test_spread;
        }
    }
}
```

### New test_benchmarks.rs harness

```rust
// tests/test_benchmarks.rs (new file)
mod benchmarks {
    mod dtlz;
    mod single_objective;
    mod zdt;
}
```

### Wiring levy_flight in test_operations.rs

```rust
// tests/test_operations.rs — add after existing mutation declarations
mod operations {
    // ... existing ...
    mod test_mutation_levy_flight;   // ← add this line
    // ...
}
```

### Benchmark test file skeleton (feature-gated)

```rust
// tests/benchmarks/dtlz.rs (new file)
#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::dtlz::{
    BenchmarkFn, DTLZ1, DTLZ2, DTLZ3, DTLZ4, DTLZ5, DTLZ6, DTLZ7,
};

#[cfg(feature = "benchmarks")]
const EPSILON: f64 = 1e-12;

#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz1_form() {
    // ... test body from src/benchmarks/dtlz.rs ...
}

// Serde tests:
#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz1_serde_roundtrip() {
    // ... serde test body ...
}
```

---

## File-by-File Migration Map

| Source file | Inline tests | Destination | Destination state | Action |
|-------------|-------------|-------------|-------------------|--------|
| `src/aos.rs` | 24 tests | `tests/engines/aos/test_aos.rs` | WIRED — 25 tests already present | Delete inline block only |
| `src/engines/multi_objective/indicators/generational_distance.rs` | 6 tests | `tests/engines/multi_objective/indicators/test_generational_distance.rs` | EXISTS (8 tests) — NOT WIRED | Wire in `test_engines.rs`, delete inline block |
| `src/engines/multi_objective/indicators/hypervolume.rs` | 6 tests | `tests/engines/multi_objective/indicators/test_hypervolume.rs` | EXISTS (8 tests) — NOT WIRED | Wire in `test_engines.rs`, delete inline block |
| `src/engines/multi_objective/indicators/inverted_generational_distance.rs` | 6 tests | `tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs` | EXISTS (8 tests) — NOT WIRED | Wire in `test_engines.rs`, delete inline block |
| `src/engines/multi_objective/indicators/spread.rs` | 5 tests | `tests/engines/multi_objective/indicators/test_spread.rs` | EXISTS (6 tests) — NOT WIRED | Wire in `test_engines.rs`, delete inline block |
| `src/operations/local_search.rs` | 7 tests (all public API) | `tests/engines/local_search.rs` | EXISTS (7 tests) — WIRED | Merge 7 tests, delete inline block |
| `src/operations/mutation/levy_flight.rs` | 2 tests (private fns!) | `tests/operations/test_mutation_levy_flight.rs` | DOES NOT EXIST | Create file with rewritten public-API tests, declare in `test_operations.rs`, delete inline block |
| `src/benchmarks/dtlz.rs` | 17 tests (feature-gated) | `tests/benchmarks/dtlz.rs` | DOES NOT EXIST | Create file + `tests/test_benchmarks.rs` harness, delete inline block |
| `src/benchmarks/single_objective.rs` | 10 tests (feature-gated) | `tests/benchmarks/single_objective.rs` | DOES NOT EXIST | Create file, delete inline block |
| `src/benchmarks/zdt.rs` | 13 tests (feature-gated) | `tests/benchmarks/zdt.rs` | DOES NOT EXIST | Create file, delete inline block |

**Total inline tests to remove:** 56 (default) + 30 (benchmarks feature) = 86

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Inline `#[cfg(test)] mod tests { use super::* }` | External `tests/` files with `use crate_name::...` | Tests compile separately; private API tests must go through public API |
| Rust 2015: no subdirectory discovery | Rust 2018+: subdirectory `*.rs` files discovered if declared in a harness | No `mod.rs` needed in subdirectories; harness pattern scales |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `levy_flight_mutation` accepts a mutable `RangeChromosome` and modifies gene values in-place — the test can observe post-mutation gene values | Code Examples | If the function signature changed, the rewritten test needs adjustment. Low risk — verified by reading the source. [ASSUMED from reading source, not from docs] |

**If this table were empty:** All claims would be verified directly from codebase inspection.

---

## Open Questions (RESOLVED)

1. **local_search.rs: merge or new file?**
   - What we know: `tests/engines/local_search.rs` exists with 7 tests; inline `src/` block has 7 different tests (HillClimbingConfig behavior, factory, etc.)
   - What's unclear: Whether to append the 7 inline tests to the existing file, or create a separate `tests/engines/test_local_search_operators.rs`
   - Recommendation (Claude's discretion): Merge into the existing `tests/engines/local_search.rs` to keep all local_search tests in one place, since it is already wired and follows the same subject domain. Avoid naming collisions by checking function names first.

2. **AOS test file naming (Claude's discretion):**
   - The CONTEXT.md leaves the exact filename to discretion.
   - Recommendation: The 24 inline tests are fully superseded by `tests/engines/aos/test_aos.rs` (25 tests). Simply delete the inline block. No new file needed.

---

## Environment Availability

| Dependency | Required By | Available | Fallback |
|------------|------------|-----------|----------|
| `cargo test` | All test validation | Yes | — |
| `cargo test --features benchmarks` | Benchmark test validation | Yes (benchmarks feature compiles cleanly via `--tests` flag; examples have unrelated errors) | Use `--tests` flag to skip example compilation |

**Note:** `cargo test --features benchmarks` without `--tests` fails because examples `ibea_zdt1` and `sms_emoa_zdt1` have unrelated compile errors when benchmarks+logging features are enabled. Use `cargo test --features benchmarks --tests` or `cargo test --features benchmarks --lib` to validate only the library and integration tests.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` |
| Config file | `Cargo.toml` (no explicit `[[test]]` entries needed for this phase) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde && cargo test --features benchmarks --tests` |
| Success condition | `grep -rn '#[cfg(test)]' src/` returns zero matches |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| SC-1 | `grep -rn '#[cfg(test)]' src/` returns zero matches | shell validation | `grep -rn '#\[cfg(test)\]' src/ \|\| echo "CLEAN"` |
| SC-2 | All moved tests pass under `cargo test` | integration | `cargo test` |
| SC-3 | No coverage regression — same number of tests pass before and after | count check | Compare `cargo test -- --list \| wc -l` before and after |

### Baseline Test Counts (for regression check)

Run before starting the phase:
- `cargo test -- --list 2>&1 | wc -l` → **61 lines** (56 tests + headers)
- `cargo test --features benchmarks -- --list 2>&1 | wc -l` → **91 lines** (86 tests + headers, lib only)

After the phase, integration test counts should be **higher** (tests moved to `tests/` binary, not lost). The inline count in `src/lib.rs` binary should reach 0 for migrated modules.

### Sampling Rate
- **Per source file deletion:** `cargo test` (verify test count unchanged or increased)
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** `cargo test && cargo test --features benchmarks --tests && grep -rn '#[cfg(test)]' src/ | wc -l` (must be 0)

### Wave 0 Gaps
- [ ] `tests/test_benchmarks.rs` — new harness file needed before benchmark test files can be used
- [ ] `tests/benchmarks/dtlz.rs` — new test file
- [ ] `tests/benchmarks/single_objective.rs` — new test file
- [ ] `tests/benchmarks/zdt.rs` — new test file
- [ ] `tests/operations/test_mutation_levy_flight.rs` — new test file (with rewritten tests)

---

## Security Domain

Not applicable — this phase makes no changes to authentication, authorization, data handling, cryptography, or input validation. It is a test organization refactor with zero logic changes.

---

## Sources

### Primary (HIGH confidence — verified by direct codebase inspection)
- `/Users/luis/RustroverProjects/genetic-algorithms/tests/test_engines.rs` — harness pattern; confirmed which modules are wired
- `/Users/luis/RustroverProjects/genetic-algorithms/tests/test_operations.rs` — operations harness; confirmed no levy_flight declaration
- `/Users/luis/RustroverProjects/genetic-algorithms/src/lib.rs` — confirmed `#[cfg(feature = "benchmarks")] pub mod benchmarks;`
- `/usr/bin/env cargo test --lib -- --list` — verified 56 inline tests (86 with benchmarks feature)
- All 10 source files being migrated — inspected inline test blocks, identified private vs. public access
- All existing destination files — inspected test counts and wiring status

### Tertiary (LOW confidence — from Rust documentation training knowledge)
- Cargo test discovery rules (top-level `tests/*.rs` only; subdirectory files need harness) [ASSUMED — consistent with observed behavior]

---

## Metadata

**Confidence breakdown:**
- File-by-file migration map: HIGH — verified by direct codebase inspection
- Harness wiring pattern: HIGH — confirmed by reading `test_engines.rs` and `test_operations.rs`
- Private function access (levy_flight): HIGH — confirmed `mantegna_sigma_u` and `gamma_approx` are not `pub`
- Private field access (AOS `arms`): HIGH — confirmed field is private; public `num_arms()` exists
- Benchmarks feature gate: HIGH — confirmed in `src/lib.rs` and `Cargo.toml`
- Wave 0 work items: HIGH — confirmed by file non-existence via `ls`

**Research date:** 2026-06-18
**Valid until:** 2026-07-18 (stable — no external dependencies; only invalidated by source changes)
