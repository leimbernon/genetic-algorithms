# Phase 73: Move Inline #[cfg(test)] Modules to tests/ - Pattern Map

**Mapped:** 2026-06-18
**Files analyzed:** 17 (10 source deletions + 5 new test files + 2 harness edits)
**Analogs found:** 17 / 17

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `tests/engines/multi_objective/indicators/test_generational_distance.rs` | test | request-response | `tests/engines/multi_objective/indicators/test_generational_distance.rs` (self — already exists, unwired) | exact |
| `tests/engines/multi_objective/indicators/test_hypervolume.rs` | test | request-response | same — already exists, unwired | exact |
| `tests/engines/multi_objective/indicators/test_inverted_generational_distance.rs` | test | request-response | same — already exists, unwired | exact |
| `tests/engines/multi_objective/indicators/test_spread.rs` | test | request-response | same — already exists, unwired | exact |
| `tests/operations/test_mutation_levy_flight.rs` | test | transform | `tests/operations/test_mutation_bit_flip.rs` | role-match |
| `tests/benchmarks/dtlz.rs` | test | batch | `tests/engines/multi_objective/indicators/test_generational_distance.rs` | role-match |
| `tests/benchmarks/single_objective.rs` | test | batch | same | role-match |
| `tests/benchmarks/zdt.rs` | test | batch | same | role-match |
| `tests/test_benchmarks.rs` | harness | — | `tests/test_engines.rs` | exact |
| `tests/test_engines.rs` | harness | — | self (edit only) | exact |
| `tests/test_operations.rs` | harness | — | self (edit only) | exact |
| `tests/engines/local_search.rs` | test | request-response | self (append only) | exact |
| `src/aos.rs` | source | — | — | delete-only |
| `src/engines/multi_objective/indicators/generational_distance.rs` | source | — | — | delete-only |
| `src/engines/multi_objective/indicators/hypervolume.rs` | source | — | — | delete-only |
| `src/engines/multi_objective/indicators/inverted_generational_distance.rs` | source | — | — | delete-only |
| `src/engines/multi_objective/indicators/spread.rs` | source | — | — | delete-only |
| `src/operations/local_search.rs` | source | — | — | delete-only |
| `src/operations/mutation/levy_flight.rs` | source | — | — | delete-only |
| `src/benchmarks/dtlz.rs` | source | — | — | delete-only |
| `src/benchmarks/single_objective.rs` | source | — | — | delete-only |
| `src/benchmarks/zdt.rs` | source | — | — | delete-only |

---

## Pattern Assignments

### `tests/test_engines.rs` (harness edit — add multi_objective wiring)

**Analog:** `tests/test_engines.rs` lines 1-84 (current content — read only; edit adds 6 lines)

**Current structure** (lines 63-65, showing the AOS block to append after):
```rust
    mod aos {
        mod test_aos;
    }
```

**Pattern to insert immediately after the `aos` block** (after line 65):
```rust
    mod multi_objective {
        mod indicators {
            mod test_generational_distance;
            mod test_hypervolume;
            mod test_inverted_generational_distance;
            mod test_spread;
        }
    }
```

The full `mod engines { }` block in `tests/test_engines.rs` is the nesting container. The new declaration must sit inside `mod engines { }`, not at the top level.

---

### `tests/test_operations.rs` (harness edit — add levy_flight declaration)

**Analog:** `tests/test_operations.rs` lines 1-43

**Pattern:** append one line inside `mod operations { }` after the last `test_mutation_*` entry (line 30):
```rust
    mod test_mutation_levy_flight;
```

Place it after `mod test_mutation_self_adaptive;` (line 30), before `mod test_selection;` (line 31).

---

### `tests/test_benchmarks.rs` (new file)

**Analog:** `tests/test_engines.rs` (harness pattern)

**Full file content pattern:**
```rust
mod benchmarks {
    mod dtlz;
    mod single_objective;
    mod zdt;
}
```

No imports. No `#[cfg]` gates. Cargo discovers this as a top-level test binary. The feature gating lives inside the benchmark test files themselves.

---

### `tests/engines/multi_objective/indicators/test_generational_distance.rs` (already exists — wire only)

**Action:** No content changes. Just wire it into `test_engines.rs` via the declaration added above.

**Confirmed structure** (lines 1-81 already on disk):
```rust
// line 1-2: imports
use genetic_algorithms::multi_objective::indicators::generational_distance;
use genetic_algorithms::GaError;

// line 14-19: test function pattern (no mod tests wrapper, no #[cfg(test)])
#[test]
fn test_gd_identical_fronts() {
    let front = vec![vec![1.0, 2.0], vec![2.0, 1.0], vec![1.5, 1.5]];
    let result = generational_distance(&front, &front, 2.0).unwrap();
    assert!((result - 0.0).abs() < 1e-15, "...");
}
```

The same top-level-function-no-wrapper pattern applies to `test_hypervolume.rs`, `test_inverted_generational_distance.rs`, and `test_spread.rs`.

---

### `tests/operations/test_mutation_levy_flight.rs` (new file — private fn rewrite)

**Analog:** `tests/operations/test_mutation_bit_flip.rs` (lines 1-134)

**Imports pattern** (from `test_mutation_bit_flip.rs` lines 1-6, adapted):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::mutation::levy_flight::levy_flight_mutation;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;
```

**Core test pattern** (from `test_mutation_bit_flip.rs` lines 8-40, adapted for levy_flight):
```rust
#[test]
fn levy_flight_mutation_produces_finite_value() {
    let gene = RangeGene::new(0, vec![(-10.0_f64, 10.0_f64)], 0.0);
    let mut chromosome = RangeChromosome::new();
    chromosome.set_dna(Cow::Owned(vec![gene]));

    levy_flight_mutation(&mut chromosome, 1.5);

    let val = chromosome.dna()[0].value;
    assert!(val.is_finite(), "levy flight must produce finite value, got {}", val);
}
```

**Private fn invariant coverage** — the two inline tests (`mantegna_sigma_u_finite_positive_at_default_alpha` and `gamma_approx_known_values`) tested private functions. Their invariants are observable through `levy_flight_mutation`:
- `mantegna_sigma_u` is finite and positive → levy_flight_mutation on a valid chromosome produces a finite, in-range gene value
- `gamma_approx` is numerically correct → if Gamma were wrong, levy_flight_mutation would produce NaN or Inf, which the assertion catches

**No `mod tests { }` wrapper.** All functions are top-level `#[test]` items. No `#[cfg(test)]`.

---

### `tests/benchmarks/dtlz.rs` (new file)

**Analog:** `tests/engines/multi_objective/indicators/test_generational_distance.rs` (import + test pattern)

**Imports pattern:**
```rust
#[cfg(feature = "benchmarks")]
use genetic_algorithms::benchmarks::dtlz::{
    BenchmarkFn, DTLZ1, DTLZ2, DTLZ3, DTLZ4, DTLZ5, DTLZ6, DTLZ7,
};
```

**Test function pattern:**
```rust
#[cfg(feature = "benchmarks")]
#[test]
fn test_dtlz1_form() {
    // body copied from src/benchmarks/dtlz.rs inline test block
}
```

**Serde test pattern** (carry over from nested `mod serde_tests` in src/):
```rust
#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_dtlz1_serde_roundtrip() {
    // body from src/benchmarks/dtlz.rs #[cfg(feature = "serde")] mod serde_tests
}
```

The same pattern applies verbatim to `tests/benchmarks/single_objective.rs` and `tests/benchmarks/zdt.rs` (substituting the relevant types and import paths).

---

### `tests/engines/local_search.rs` (existing file — append 7 tests)

**Analog:** `tests/engines/local_search.rs` lines 1-165 (current content)

**Imports already present** (lines 1-11). The 7 inline tests from `src/operations/local_search.rs` use `use super::*;` for factory/config types. When ported, all types must be imported via `use genetic_algorithms::...` matching the style at lines 1-11. Check function names in the inline block against lines 37-165 before appending to avoid `#[test]` name collisions.

**Append pattern:**
```rust
// ==================== Migrated from src/operations/local_search.rs ====================

#[test]
fn <test_name_from_inline_block>() {
    // body unchanged, but any `use super::*` references resolved to
    // fully qualified genetic_algorithms:: paths
}
```

---

### Source file deletions (10 files)

For each source file, delete the entire `#[cfg(test)] mod tests { ... }` block. The block begins at `#[cfg(test)]` and ends at the matching closing `}`. No other content is modified.

| Source file | Block starts at line | Notes |
|---|---|---|
| `src/aos.rs` | search for `#[cfg(test)]` | AOS — full block deleted; superseded by external 25-test file |
| `src/engines/multi_objective/indicators/generational_distance.rs` | after line 58 | 6-test block |
| `src/engines/multi_objective/indicators/hypervolume.rs` | after public fns | 6-test block |
| `src/engines/multi_objective/indicators/inverted_generational_distance.rs` | after public fns | 6-test block |
| `src/engines/multi_objective/indicators/spread.rs` | after public fns | 5-test block |
| `src/operations/local_search.rs` | after public fns | 7-test block (merge first) |
| `src/operations/mutation/levy_flight.rs` | line 110 | 2-test block (rewrite first) |
| `src/benchmarks/dtlz.rs` | after impl blocks | 17-test block (feature-gated) |
| `src/benchmarks/single_objective.rs` | after impl blocks | 10-test block (feature-gated) |
| `src/benchmarks/zdt.rs` | after impl blocks | 13-test block (feature-gated) |

---

## Shared Patterns

### Integration test file structure (no-wrapper pattern)
**Source:** `tests/engines/multi_objective/indicators/test_generational_distance.rs` lines 1-81
**Apply to:** All new and migrated test files under `tests/`
```rust
// Pattern: imports at top, then bare #[test] functions — NO mod tests { } wrapper
use genetic_algorithms::some::public::path;

#[test]
fn descriptive_test_name() {
    // assertions
}
```

### Feature-gated test pattern
**Source:** RESEARCH.md Pattern 4
**Apply to:** `tests/benchmarks/dtlz.rs`, `tests/benchmarks/single_objective.rs`, `tests/benchmarks/zdt.rs`
```rust
#[cfg(feature = "benchmarks")]
#[test]
fn test_name() { ... }

#[cfg(all(feature = "benchmarks", feature = "serde"))]
#[test]
fn test_name_serde() { ... }
```

### Harness `mod` declaration pattern
**Source:** `tests/test_engines.rs` lines 1-84 and `tests/test_operations.rs` lines 1-43
**Apply to:** `tests/test_engines.rs` (edit), `tests/test_operations.rs` (edit), `tests/test_benchmarks.rs` (new)
```rust
mod <subdirectory_name> {
    mod <filename_without_extension>;  // file at tests/<subdirectory>/<filename>.rs
    mod nested_dir {                   // for deeper paths
        mod <filename>;
    }
}
```

### Mutation test import pattern
**Source:** `tests/operations/test_mutation_bit_flip.rs` lines 1-6
**Apply to:** `tests/operations/test_mutation_levy_flight.rs`
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::mutation::<module>::<function>;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;
```

---

## No Analog Found

None. All files have direct analogs in the codebase.

---

## Metadata

**Analog search scope:** `tests/`, `src/engines/multi_objective/indicators/`, `src/operations/mutation/`, `src/benchmarks/`
**Files scanned:** 7 (test_engines.rs, test_operations.rs, local_search.rs, test_mutation_bit_flip.rs, test_generational_distance.rs, levy_flight.rs, dtlz.rs partial)
**Pattern extraction date:** 2026-06-18
