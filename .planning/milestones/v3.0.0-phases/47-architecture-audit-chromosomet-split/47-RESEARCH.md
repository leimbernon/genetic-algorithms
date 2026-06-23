# Phase 47: Architecture Audit & ChromosomeT Split — Research

**Researched:** 2026-05-19
**Domain:** Rust trait design, breaking API refactors, CI pipeline
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**ChromosomeT / LinearChromosome Trait Split (ARCH-01, ARCH-02)**

- D-01: `ChromosomeT` retains only: `fitness()`, `set_fitness()`, `calculate_fitness()`, `age()`, `set_age()`, `fitness_distance()`. No flat-slice methods, no fitness function installation.
- D-02: `LinearChromosome: ChromosomeT` is the supertrait for flat-slice chromosomes. Adds: `dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()`, `new_gene()`. Provides default implementations of `set_gene()` and `reset()`.
- D-03: `default(mut self) -> Self` reset helper renamed to `reset() -> &mut Self` on `LinearChromosome`. Removes shadowing ambiguity with `Default` trait; builder-style `&mut Self` return.
- D-04: `ChromosomeT` does not have fitness function installation. `calculate_fitness()` is the only method. `set_fitness_fn<F>()` stays on `LinearChromosome` only.
- D-05: Mechanical bound change (`U: ChromosomeT` → `U: LinearChromosome`) across ~30 operator files via `sed -i 's/U: ChromosomeT/U: LinearChromosome/g'` + `cargo check` loop.

**Configuration Cleanup (ARCH-04, ARCH-05, ARCH-06)**

- D-06: `LimitConfiguration.needs_unique_ids` and `LimitConfiguration.alleles_can_be_repeated` removed without replacement in Phase 47. Initializers drop the uniqueness enforcement logic.
- D-07: `ChromosomeLength` is a standalone public type in a new file (e.g., `src/chromosomes/length.rs`). Variants: `ChromosomeLength::Fixed(usize)` and `ChromosomeLength::Variable { min: usize, max: usize }`. Re-exported from `lib.rs`.
- D-08: `StoppingCriteria` struct removed entirely. Its 3 fields become direct `pub(crate)` fields on `GaConfiguration`. Builder methods: `.with_stagnation_limit()`, `.with_convergence_threshold()`, `.with_max_duration_secs()`.
- D-09: `GaConfiguration` fields become `pub(crate)` with sub-struct level read-only public accessors.

**Reporter Removal (ARCH-03)**

- D-10: `Reporter<U>` trait removed entirely in v3.0.0. `with_reporter()` builder method removed from `Ga`.

**MIGRATION.md (ARCH-03 expanded)**

- D-11: Full v3.0.0 breaking changes guide at crate root (`MIGRATION.md`). Covers all Phase 47 breaking changes with before/after code examples. Include in `Cargo.toml` `include` list. Link from `README.md`.

**Examples CI (ARCH-07)**

- D-12: New CI workflow `.github/workflows/examples-smoke.yml`. Triggers on pushes/PRs to the milestone branch only. Compiles and runs each of the 10 examples with a short generation count.

**PR Execution Strategy**

- D-13: 3 staged PRs on the milestone branch:
  - PR 1 — ChromosomeT split: ARCH-01 + ARCH-02
  - PR 2 — Config cleanup: ARCH-04 + ARCH-05 + ARCH-06
  - PR 3 — Reporter removal + CI: ARCH-03 + ARCH-07

### Claude's Discretion

None specified — all implementation decisions locked.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARCH-01 | User can implement `ChromosomeT` with only fitness/age/calculate_fitness — no flat-slice methods | ChromosomeT currently has 98 lines; new version is ~20 lines. Flat-slice methods move to `LinearChromosome`. |
| ARCH-02 | User can implement `LinearChromosome` to gain full operator compatibility — mechanical bound change across ~30 operator files | `grep -rn "U: ChromosomeT"` finds 213 occurrences; ~30 unique operator files call `dna()`/`set_dna()` and need `LinearChromosome`. Confirmed via file scan. |
| ARCH-03 | User can build without `Reporter<U>` — trait removed; MIGRATION.md published | `Reporter<U>` is fully deprecated since v2.2.0 (`#[deprecated(since = "2.2.0", ...)]`); 4 fire points in `ga.rs`; zero usage in examples. |
| ARCH-04 | `GaConfiguration` fields become `pub(crate)` with read-only accessors; `needs_unique_ids` and `alleles_can_be_repeated` removed | 6 examples directly access `ga_config.limit_configuration.*`; 8 multi-obj engine files read these fields internally. All must migrate to builder/accessor pattern. |
| ARCH-05 | `ChromosomeLength` replaces bare `genes_per_chromosome: usize` | `with_genes_per_chromosome` called in 7+ examples and numerous engine files; `genes_per_chromosome` field referenced in 26 places in `ga.rs` alone. |
| ARCH-06 | Flat builder methods replace `StoppingCriteria` struct; `LocalSearch` already is an enum (no Arc needed) | `StoppingCriteria` has 3 fields; currently set via `with_stopping_criteria(StoppingCriteria{...})`. `LocalSearch` is already a `Copy` enum — ARCH-06 scoped to `StoppingCriteria` flattening only. |
| ARCH-07 | All 10 examples compile and pass CI smoke tests after every PR | 19 examples exist; "10 runnable" are the subset listed in the success criteria. New `examples-smoke.yml` workflow needed. |
</phase_requirements>

---

## Summary

Phase 47 is a pure refactor: no new algorithms, only API cleanup. It delivers three types of changes: (1) a Rust trait split that changes the public contract for custom chromosome types, (2) configuration struct cleanup that encapsulates `GaConfiguration` and removes two legacy fields, and (3) removal of the already-deprecated `Reporter<U>` trait. The risk profile is high-breadth but low-depth — ~30 operator files need a mechanical bound change, and ~10 examples plus 8 multi-objective engine files need field access migrated to builder/accessor methods.

The `ChromosomeT` split is the most structurally significant change. The current `ChromosomeT` trait (98 lines, `src/traits/chromosome.rs`) mixes two concerns: the flat-slice contract (`dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()`, `new_gene()`) and the pure evaluation contract (`fitness()`, `set_fitness()`, `calculate_fitness()`, `age()`, `set_age()`, `fitness_distance()`). After the split, `ChromosomeT` becomes ~20 lines covering only the evaluation contract. `LinearChromosome: ChromosomeT` covers the flat-slice contract plus default impls of `set_gene()` and `reset()`.

The `sed` approach for the mechanical bound change is well-targeted: operators in `src/operations/` (all ~30 files that call `dna()`/`dna_mut()`/`set_dna()`), `src/validators/`, and orchestrators in `src/engines/`. A crucial distinction: survivor, extension, and selection operators that only use `fitness()` and `age()` correctly remain at `ChromosomeT` bounds — only those that touch flat-slice methods need `LinearChromosome`. `ValueMutable: ChromosomeT` also needs updating to `ValueMutable: LinearChromosome`.

**Primary recommendation:** Execute the 3 PR strategy in order (trait split → config cleanup → reporter/CI). Each PR must pass `cargo check --target wasm32-unknown-unknown` before merge. The `sed` pass saves the most time on PR 1; manual audit is needed for the 20+ locations where `genes_per_chromosome` is read and passed to initializers (PR 2).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ChromosomeT core trait | Library public API | — | Trait definition; downstream implementors are external users |
| LinearChromosome supertrait | Library public API | Operator layer | All existing operators become `U: LinearChromosome`; new tree types stay at `ChromosomeT` |
| Operator bound change | Operator layer | Engine orchestrators | Mechanical `sed` pass; affects `src/operations/`, `src/validators/`, `src/engines/` |
| GaConfiguration encapsulation | Configuration layer | Engine orchestrators | Fields become `pub(crate)`; public read via sub-struct accessors |
| ChromosomeLength enum | Configuration layer | Engine orchestrators + initializers | Replaces `genes_per_chromosome: usize`; engine reads it to call initializers |
| StoppingCriteria flattening | Configuration layer | Engine (ga.rs) | 3 fields inline on `GaConfiguration`; `ga.rs` already reads them via `self.configuration.stopping_criteria.X` — path changes to `self.configuration.X` |
| Reporter removal | Public API (removal) | Engine (ga.rs) | 4 `reporter` fire points in ga.rs; `src/observe/reporter/` module deleted from `lib.rs` pub mod |
| MIGRATION.md | Documentation | — | Crate root; added to `Cargo.toml` `include` array |
| examples-smoke.yml | CI pipeline | — | New GitHub Actions workflow; triggers on milestone/feat branch PRs |

---

## Standard Stack

### Core (no new dependencies required)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| (none) | — | Phase 47 has zero new external dependencies | Pure refactor |

All work uses existing Rust stdlib, serde (existing feature), and the rayon/log crates already in `Cargo.toml`. [VERIFIED: codebase grep]

**Package Legitimacy Audit:** Not applicable — this phase installs no packages.

---

## Architecture Patterns

### Existing Patterns This Phase Uses

**Trait supertrait composition (Rust idiom)** [ASSUMED — standard Rust]
```rust
// Source: decision D-01 / D-02
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    type Gene: GeneT;
    fn fitness(&self) -> f64;
    fn set_fitness(&mut self, fitness: f64) -> &mut Self;
    fn calculate_fitness(&mut self);
    fn age(&self) -> usize;
    fn set_age(&mut self, age: usize) -> &mut Self;
    fn fitness_distance(&self, fitness_target: &f64) -> f64 {
        (fitness_target - self.fitness()).abs()
    }
}

pub trait LinearChromosome: ChromosomeT {
    fn dna(&self) -> &[Self::Gene];
    fn dna_mut(&mut self) -> &mut [Self::Gene];
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;
    fn new_gene() -> Self::Gene { Self::Gene::new() }
    // Default implementations (D-02, D-03):
    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self {
        let len = self.dna().len();
        if gene_index >= len {
            log::warn!("set_gene: index {} out of bounds (DNA length {})", gene_index, len);
            return self;
        }
        self.dna_mut()[gene_index] = gene;
        self
    }
    fn reset(&mut self) -> &mut Self {
        self.set_fitness(f64::NAN);
        self.set_age(0);
        self.set_dna(Cow::Borrowed(&[]));
        self
    }
}
```

**`pub(crate)` + public accessor pattern** [VERIFIED: codebase grep — already used in some sub-configs]
```rust
// After ARCH-04:
pub struct GaConfiguration {
    pub(crate) adaptive_ga: bool,
    pub(crate) limit_configuration: LimitConfiguration,
    // ...
}

impl GaConfiguration {
    pub fn limit(&self) -> &LimitConfiguration { &self.limit_configuration }
    pub fn selection(&self) -> &SelectionConfiguration { &self.selection_configuration }
    // sub-struct level only (D-09)
}
```

**Standalone enum type** [VERIFIED: codebase grep — `ProblemSolving`, `LogLevel` follow this pattern]
```rust
// src/chromosomes/length.rs (new file per D-07)
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}
```

**Flat config fields (StoppingCriteria dissolution)** [VERIFIED: codebase grep — fields go on `GaConfiguration`]
```rust
// GaConfiguration after D-08:
pub(crate) stagnation_generations: Option<usize>,
pub(crate) convergence_threshold: Option<f64>,
#[cfg(not(target_arch = "wasm32"))]
pub(crate) max_duration_secs: Option<f64>,

// New builder methods on StoppingConfig trait:
fn with_stagnation_limit(self, n: usize) -> Self;
fn with_convergence_threshold(self, threshold: f64) -> Self;
fn with_max_duration_secs(self, secs: f64) -> Self;
```

**GitHub Actions examples CI** [ASSUMED — standard Actions pattern]
```yaml
# .github/workflows/examples-smoke.yml
on:
  push:
    branches: ["milestone/**", "feat/**", "fix/**"]
  pull_request:
    branches: ["milestone/**"]
jobs:
  examples-smoke:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        example: [knapsack_binary, rastrigin, onemax_binary, ...]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo run --example ${{ matrix.example }} --release
```

### Recommended Project Structure
```
src/
├── traits/
│   ├── chromosome.rs          # ChromosomeT (minimal ~20 lines)
│   ├── linear_chromosome.rs   # LinearChromosome (new file)
│   └── operators.rs           # All operator bounds → LinearChromosome
├── chromosomes/
│   ├── length.rs              # ChromosomeLength enum (new file, D-07)
│   ├── binary.rs              # impl LinearChromosome
│   ├── range.rs               # impl LinearChromosome
│   └── list.rs                # impl LinearChromosome
├── configuration.rs           # GaConfiguration: pub(crate) fields, stop criteria flattened
├── observe/
│   ├── reporter/              # DELETED (D-10)
│   └── observer/              # Unchanged
└── engines/
    └── ga.rs                  # reporter field removed, bound → LinearChromosome
```

### Anti-Patterns to Avoid

- **Broad sed scope:** The sed command should target `src/operations/`, `src/validators/`, and engine orchestrator files — NOT `src/traits/`, `src/observe/`, `src/population.rs`, `src/stats.rs`. Validators and population containers operate on fitness/age and must STAY at `ChromosomeT`. Survivor operators that sort by fitness only (e.g., `age_based`, `fitness_based`) also stay at `ChromosomeT`; only `deterministic_crowding` and `mass_deduplication` need `LinearChromosome`.
- **Removing `ValueMutable: ChromosomeT` supertrait:** After the split, `ValueMutable` should become `ValueMutable: LinearChromosome` — it calls `swap()` (which calls `dna_mut()`).
- **Forgetting WASM gates on `max_duration_secs`:** When flattening `StoppingCriteria` into `GaConfiguration`, the field annotation and the `.with_max_duration_secs()` builder method must preserve the existing `#[cfg(not(target_arch = "wasm32"))]` gate pattern from `ga.rs` lines 1433-1442.
- **Making `ChromosomeLength` inline in `LimitConfiguration`:** D-07 says standalone public type; embedded enums inside config structs are not first-class exports.
- **Partial reporter removal:** All 4 fire points in `ga.rs` must be removed (lines ~1447, ~1974-1975, ~2060, ~2125), plus the `reporter` field on `Ga<U>`, plus `#[path = "observe/reporter/mod.rs"] pub mod reporter` in `lib.rs`.
- **Forgetting the `new_gene()` move:** `new_gene() -> Self::Gene` is currently a method on `ChromosomeT` (`chromosome.rs:32`). Under D-02 it moves to `LinearChromosome`. Any code calling `U::new_gene()` where `U: ChromosomeT` (not `LinearChromosome`) will break — scan for such call sites before removing.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Mechanical bound replacement | Per-file edits | `sed -i 's/U: ChromosomeT/U: LinearChromosome/g'` + `cargo check` loop | 30+ files; sed is faster and auditable |
| WASM check | Manual compile | Existing `wasm-check.yml` CI workflow | Already targets both default and serde features |
| `#[deprecated]` compile error on `Reporter` | Custom error type | `#[deprecated]` attribute + compiler error when struct field removed | Compiler surfacing is the right UX per D-10 |

---

## Runtime State Inventory

Not applicable — this is a code/API refactor. No stored data, live service config, OS-registered state, secrets, or build artifacts are involved.

---

## Common Pitfalls

### Pitfall 1: Sed Over-Reach Into Observer/Population Code
**What goes wrong:** `sed` replaces `U: ChromosomeT` → `U: LinearChromosome` in files that only use `fitness()` and `age()`, breaking non-linear chromosome types (e.g., `TreeChromosome` in Phase 53).
**Why it happens:** `src/population.rs`, `src/observe/observer/`, `src/stats.rs`, and fitness-only survivor operators are correct at `ChromosomeT` bounds.
**How to avoid:** Scope `sed` strictly to `src/operations/`, then run `cargo check` and manually resolve any remaining compiler errors. Do not run sed on the entire `src/` tree.
**Warning signs:** `cargo check` passes but `TreeChromosome` (future type) would fail to compile against operators.

### Pitfall 2: alleles_can_be_repeated Still Read at Runtime
**What goes wrong:** Removing `alleles_can_be_repeated` from `LimitConfiguration` (D-06) breaks 8 multi-objective engine files that read `self.ga_config.limit_configuration.alleles_can_be_repeated` at population initialization time.
**Why it happens:** `sms_emoa`, `ibea`, `moead`, `spea2`, `nsga3`, `nsga2`, island's nsga2 wrapper, and island's main `mod.rs` all read this field (verified via grep, 16 references total).
**How to avoid:** PR 2 must touch ALL engine population-init code blocks, not just `ga.rs`. After removing the field from `LimitConfiguration`, `cargo check` will surface every caller.
**Warning signs:** `cargo check` with `alleles_can_be_repeated` removed still passing would indicate a silent drop, not a clean migration.

### Pitfall 3: WASM Gate Missing on StoppingCriteria Flattening
**What goes wrong:** When `stagnation_generations`, `convergence_threshold`, and `max_duration_secs` move to direct fields on `GaConfiguration`, `max_duration_secs` loses its WASM gate.
**Why it happens:** The current `StoppingCriteria` struct has no wasm gate (it's just `Option<f64>`); the gate lives at the call site in `ga.rs:2103-2104`. When the field moves, the call site stays gated but the field annotation must also be correct for serde serialization on wasm32.
**How to avoid:** Keep the `#[cfg(not(target_arch = "wasm32"))]` annotation on the field if using conditional compilation, OR keep it as `Option<f64>` un-gated and gate only the usage (current pattern — preferred, simpler). Verify with `cargo check --target wasm32-unknown-unknown` after PR 2.
**Warning signs:** `wasm-check.yml` failing after PR 2 merge.

### Pitfall 4: Examples Using Direct Field Access (Multiple Patterns)
**What goes wrong:** 6 examples bypass the builder and set `ga_config.limit_configuration.genes_per_chromosome = N_VARS` directly. After ARCH-04 makes `limit_configuration` `pub(crate)`, these won't compile.
**Why it happens:** Multi-objective engines (nsga2, nsga3, moead, spea2, sms_emoa, ibea) expose `pub ga_config: GaConfiguration` on their engine struct, and examples use that to set population parameters outside the builder flow.
**How to avoid:** PR 2 must update all 6 examples (`sms_emoa_zdt1`, `nsga2_zdt1`, `nsga3_dtlz2`, `ibea_zdt1`, `spea2_zdt1`, `moead_dtlz2`, `island_model`) to use builder methods. Also check whether multi-obj engines themselves need builder methods added for parameters they previously set via direct struct access.
**Warning signs:** `cargo build --example nsga2_zdt1` failing after the `pub(crate)` change.

### Pitfall 5: tests/ References to Reporter Types
**What goes wrong:** Tests in `tests/test_observe.rs` or `tests/structures.rs` may instantiate `Reporter` trait objects, `SimpleReporter`, `DurationReporter`, or `NoopReporter`. Removing the `reporter` module from `lib.rs` will break these.
**Why it happens:** Reporter was a public API; test infrastructure may still reference it even if examples don't.
**How to avoid:** Before PR 3, grep for `Reporter\|with_reporter\|SimpleReporter\|DurationReporter\|NoopReporter` across `tests/` and `examples/` and remove/migrate all references.
**Warning signs:** `cargo test` failing after deleting `src/observe/reporter/`.

### Pitfall 6: default() Method Name Shadowing Default Trait
**What goes wrong:** The current `ChromosomeT::default(mut self) -> Self` method (line 24 of `chromosome.rs`) shadows the `Default` trait's `default()` method, causing confusing method resolution.
**Why it happens:** D-03 exists precisely to fix this — `reset() -> &mut Self` on `LinearChromosome` is the replacement. But any code calling `.default()` on a chromosome instance (not `Default::default()`) will break.
**How to avoid:** Search for `.default()` call sites (method syntax, not turbofish `Default::default()`) before removing the `ChromosomeT::default()` method. Likely rare; the method was internally-facing.

---

## Code Examples

### ARCH-01/02: Trait Split Files

Two new/modified files:

**`src/traits/chromosome.rs` (shrinks to ~25 lines):**
```rust
// After split — only evaluation contract
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    type Gene: GeneT;
    fn fitness(&self) -> f64;
    fn set_fitness(&mut self, fitness: f64) -> &mut Self;
    fn calculate_fitness(&mut self);
    fn age(&self) -> usize;
    fn set_age(&mut self, age: usize) -> &mut Self;
    fn fitness_distance(&self, fitness_target: &f64) -> f64 {
        (fitness_target - self.fitness()).abs()
    }
}
```

**`src/traits/linear_chromosome.rs` (new file ~45 lines):**
```rust
use std::borrow::Cow;
pub trait LinearChromosome: ChromosomeT {
    fn dna(&self) -> &[Self::Gene];
    fn dna_mut(&mut self) -> &mut [Self::Gene];
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;
    fn new_gene() -> Self::Gene { Self::Gene::new() }
    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self { /* ... */ }
    fn reset(&mut self) -> &mut Self { /* ... */ }
}
```

### ARCH-05: ChromosomeLength enum usage in LimitConfiguration

```rust
// src/chromosomes/length.rs (new file)
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}

// LimitConfiguration field change:
// Before: pub genes_per_chromosome: usize
// After:  pub(crate) chromosome_length: ChromosomeLength

// Builder method change:
// Before: fn with_genes_per_chromosome(self, n: usize) -> Self
// After:  fn with_chromosome_length(self, length: ChromosomeLength) -> Self
```

### ARCH-06: StoppingConfig flat builder methods

```rust
// New methods on StoppingConfig trait:
fn with_stagnation_limit(self, n: usize) -> Self;
fn with_convergence_threshold(self, threshold: f64) -> Self;
fn with_max_duration_secs(self, secs: f64) -> Self;  // no-op or omit on wasm32

// Old method (REMOVED):
fn with_stopping_criteria(self, criteria: StoppingCriteria) -> Self;
```

### ARCH-07: examples-smoke.yml trigger and run pattern

```yaml
# The 10 examples to test (from success criteria; some have short generation counts):
# knapsack_binary, onemax_binary, onemax_extension, rastrigin, nsga2_zdt1,
# island_model, job_scheduling, niching, hall_of_fame_demo, aos_demo
# Run with: cargo run --example <name> --release
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ChromosomeT` all-in-one (flat-slice + fitness) | `ChromosomeT` (fitness) + `LinearChromosome` (flat-slice) | Phase 47 (v3.0.0) | Users no longer need to fake `dna()` for tree/non-linear types |
| `with_stopping_criteria(StoppingCriteria{...})` | `.with_stagnation_limit(n).with_convergence_threshold(t)` | Phase 47 (v3.0.0) | Chainable builder; no struct import needed |
| `Reporter<U>` (deprecated v2.2.0) | `GaObserver<U>` (11 hooks, Arc-shareable) | Removal in v3.0.0 | Cleaner API; `Reporter` was `&mut self` and 4-hook only |
| `genes_per_chromosome: usize` | `ChromosomeLength::Fixed(n)` | Phase 47 (v3.0.0) | Enables `Variable { min, max }` for future phases (CHR-01) |

**Deprecated/outdated after Phase 47:**
- `StoppingCriteria` struct: removed from type system (no longer importable)
- `Reporter<U>`, `SimpleReporter`, `DurationReporter`, `NoopReporter`: removed
- `with_genes_per_chromosome()`, `with_needs_unique_ids()`, `with_alleles_can_be_repeated()`: removed from `ConfigurationT`
- `with_stopping_criteria()`: removed from `StoppingConfig`
- `pub mod reporter` in `lib.rs`: removed
- `LimitConfiguration.needs_unique_ids`, `.alleles_can_be_repeated`, `.genes_per_chromosome`: removed

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The 10 CI-targeted examples are: `knapsack_binary`, `onemax_binary`, `onemax_extension`, `rastrigin`, `nsga2_zdt1`, `island_model`, `job_scheduling`, `niching`, `hall_of_fame_demo`, `aos_demo` | ARCH-07 | Wrong subset — planner must confirm which 10 from the 19 total |
| A2 | `default()` method on `ChromosomeT` is not called anywhere outside `src/traits/chromosome.rs` itself (it delegates to the three setter methods) | ARCH-01/02 pitfall | If called externally, removing it creates hidden breakage not caught until integration |
| A3 | Multi-objective engine structs (`Nsga2Ga`, `Nsga3Ga`, etc.) have `pub ga_config: GaConfiguration` — making those fields `pub(crate)` will require adding builder pass-through methods to these engines | ARCH-04 | If engines use a separate builder that doesn't forward calls, migration is more complex |
| A4 | `src/chromosomes/length.rs` is the correct new file path (vs `src/types/length.rs`) | ARCH-05 | Wrong path means the re-export path in `lib.rs` differs from CONTEXT.md example |
| A5 | The `with_max_duration_secs()` builder method is gated `#[cfg(not(target_arch = "wasm32"))]` at the method signature level (not at the field level) to keep WASM-safe | ARCH-06 | If gated at field level, serde serialization on wasm32 omits the field, which is actually fine — but the approach differs from current `ga.rs` pattern |

---

## Open Questions (RESOLVED)

1. **Which 10 examples are targeted by ARCH-07?**
   - What we know: 19 examples exist; success criteria says "all 10 existing runnable examples"
   - What's unclear: The 10 were not enumerated in CONTEXT.md; the 19 include multi-obj examples that require `--features` flags
   - Recommendation: Planner should include a task to enumerate the 10 examples explicitly (probably: the 9 using `Ga<U>` + 1 island or nsga2)

2. **How does `with_chromosome_length(ChromosomeLength::Fixed(n))` replace the initializer closure interface?**
   - What we know: Current `with_initialization_fn` closure receives `(genes_per_chromosome, alleles, needs_unique_ids)` — all three args will change
   - What's unclear: The closure signature must change. D-06 removes `needs_unique_ids` and `alleles_can_be_repeated`. Does the closure signature change in PR 2 or does the planner defer it?
   - Recommendation: Plan a task to update `InitializationFn<U::Gene>` type alias and all callers in PR 2

3. **Do multi-obj engine structs need new builder methods to replace direct field access?**
   - What we know: `Nsga2Ga`, `Nsga3Ga`, `SmsEmoaGa`, etc. expose `pub ga_config: GaConfiguration`; examples set fields directly
   - What's unclear: After `pub(crate)` — do the multi-obj engines need a `.with_chromosome_length()` on their own builder, or does the `ga_config` stay fully accessible within the engine module?
   - Recommendation: `pub(crate)` means accessible within the crate — the engines themselves can still mutate `ga_config` internally. Examples must use the builder API.

---

## Environment Availability

Step 2.6: SKIPPED — Phase 47 has no external tool dependencies beyond the existing Rust toolchain and `wasm32-unknown-unknown` target already verified in CI.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `cargo test` (built-in) |
| Config file | None — standard Cargo test runner |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARCH-01 | Custom type implements `ChromosomeT` with only 5 methods (no dna) | unit | `cargo test -- test_chromosomet_core` | ❌ Wave 0 |
| ARCH-02 | `LinearChromosome` impl grants full operator access | unit | `cargo test -- test_linear_chromosome` | ❌ Wave 0 |
| ARCH-02 | `cargo check` with bound change passes | compile | `cargo check --all-features` | ✅ (implicit) |
| ARCH-03 | `with_reporter()` removed — compile error | compile | `cargo test` (compilation) | ✅ (once removed) |
| ARCH-04 | `GaConfiguration` field direct access errors | compile | `cargo check` | ✅ (once `pub(crate)`) |
| ARCH-05 | `ChromosomeLength::Fixed` accepted by builder | unit | `cargo test -- test_chromosome_length` | ❌ Wave 0 |
| ARCH-06 | Flat builder methods set stopping fields | unit | `cargo test -- test_stopping_config` | ❌ Wave 0 |
| ARCH-07 | All 10 examples compile and run | smoke | `examples-smoke.yml` CI workflow | ❌ Wave 0 |
| WASM | All changes pass wasm32 check | compile | `cargo check --target wasm32-unknown-unknown` | ✅ existing CI |

### Sampling Rate
- **Per task commit:** `cargo check && cargo check --target wasm32-unknown-unknown`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo check --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/traits/test_chromosomet_core.rs` — tests `ChromosomeT` minimal contract (custom type without dna methods)
- [ ] `tests/traits/test_linear_chromosome.rs` — tests `LinearChromosome` contract + default impls
- [ ] `tests/test_chromosome_length.rs` — tests `ChromosomeLength::Fixed` and `Variable` variants in config
- [ ] `tests/test_stopping_config.rs` — tests flat builder methods `.with_stagnation_limit()` etc.

---

## Security Domain

Not applicable — this phase is a pure Rust API refactor with no input validation changes, no authentication, no cryptography, and no new external dependencies. Security attack surface is unchanged.

---

## Project Constraints (from CLAUDE.md)

| Directive | Applies to Phase 47 |
|-----------|---------------------|
| WASM mandatory: every feature must compile for `wasm32-unknown-unknown` | YES — all 3 PRs must pass `cargo check --target wasm32-unknown-unknown` |
| No breaking changes default policy | OVERRIDDEN — Phase 47 is explicitly breaking (v3.0.0 milestone); MIGRATION.md is required |
| Enum + factory for new operators | N/A — no new operators |
| Tests in `tests/` folder, never inline | YES — all new tests go in `tests/` |
| Branch from milestone branch, not main | YES — `feat/<issue>-<description>` branches from `milestone/v3.0.0` |
| `pub(crate)` + public accessor pattern | YES — ARCH-04 expands this to all GaConfiguration fields |
| Observer hooks preserved through refactor | YES — `GaObserver` fire points in `ga.rs` are untouched; only `reporter` fire points are removed |

---

## Sources

### Primary (HIGH confidence)
- `src/traits/chromosome.rs` — Current `ChromosomeT` definition (98 lines, all methods verified)
- `src/traits/operators.rs` — All operator trait signatures use `U: ChromosomeT` (verified)
- `src/configuration.rs` — `GaConfiguration`, `LimitConfiguration`, `StoppingCriteria` (fully read)
- `src/engines/ga.rs` — Reporter fire points (~1447, ~1974, ~2060, ~2125), `with_reporter()` line 856-857, WASM gates on `max_duration_secs` lines 1433-1442, 2103-2104 (verified)
- `src/observe/reporter/mod.rs` — `Reporter<U>` trait definition, deprecation annotation (verified)
- `src/types/chromosomes/binary.rs` — Current `BinaryChromosome` impl (verified)

### Secondary (MEDIUM confidence)
- `grep` counts of `U: ChromosomeT` occurrences (213 total) and files that call `dna()` methods (30 operator files) — derived from codebase analysis
- `grep` of example direct field access — 6 examples use `ga_config.limit_configuration.*` directly (verified)
- Multi-obj engine file list for `alleles_can_be_repeated` access — 16 references in 8 files (verified)

### Tertiary (LOW confidence — marked [ASSUMED])
- List of 10 target examples for ARCH-07 (A1)
- Exact path for `ChromosomeLength` new file (A4)

---

## Metadata

**Confidence breakdown:**
- ChromosomeT split (ARCH-01/02): HIGH — trait contents fully read; split is mechanical
- Configuration cleanup (ARCH-04/05/06): HIGH — all fields and callers verified; complexity is breadth not depth
- Reporter removal (ARCH-03): HIGH — fully deprecated, 0 example usage, 4 fire points located
- Examples CI (ARCH-07): MEDIUM — 10 examples not enumerated; 19 total exist
- WASM safety: HIGH — existing patterns verified in ga.rs; `max_duration_secs` handling confirmed

**Research date:** 2026-05-19
**Valid until:** 2026-06-19 (stable codebase, no upstream churn expected)
