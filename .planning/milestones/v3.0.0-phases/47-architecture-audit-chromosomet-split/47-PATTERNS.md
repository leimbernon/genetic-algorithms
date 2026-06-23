# Phase 47: Architecture Audit & ChromosomeT Split - Pattern Map

**Mapped:** 2026-05-19
**Files analyzed:** 11 new/modified files across 3 PRs
**Analogs found:** 10 / 11

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/traits/chromosome.rs` | trait | transform | `src/traits/chromosome.rs` (self — shrinks) | exact |
| `src/traits/linear_chromosome.rs` | trait | transform | `src/traits/chromosome.rs` (current full content) | exact |
| `src/traits/operators.rs` | trait | request-response | `src/traits/operators.rs` (self — bound changes) | exact |
| `src/chromosomes/length.rs` | type/enum | config | `src/operations.rs` (Selection/Crossover/Mutation enums) | role-match |
| `src/configuration.rs` | config | CRUD | `src/configuration.rs` (self — field changes) | exact |
| `src/traits/configuration.rs` | trait | request-response | `src/traits/configuration.rs` (self — method changes) | exact |
| `src/engines/ga.rs` | engine | event-driven | `src/engines/ga.rs` (self — reporter removal, bound change) | exact |
| `src/observe/reporter/` | module | event-driven | `src/observe/reporter/mod.rs` (to be deleted) | exact |
| `src/initializers/binary_initializer.rs` | utility | transform | `src/initializers/binary_initializer.rs` (self — field removal) | exact |
| `MIGRATION.md` | documentation | — | — | no analog |
| `.github/workflows/examples-smoke.yml` | CI | batch | `.github/workflows/wasm-check.yml` | role-match |

---

## Pattern Assignments

### `src/traits/chromosome.rs` (trait, shrinks to ~25 lines)

**Analog:** `src/traits/chromosome.rs` (current, lines 1–98 — all content already in context)

**Current structure — full file:**
```rust
// Lines 1-2: imports
use crate::traits::GeneT;
use std::borrow::Cow;

// Lines 14-98: ChromosomeT trait — all of this is the source to split from
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    type Gene: GeneT;
    // ... 11 methods including dna(), dna_mut(), set_dna(), set_fitness_fn()
}
```

**After ARCH-01 — keep only these methods (lines 80–98 area + supers):**
```rust
// Retained on ChromosomeT after split:
fn fitness(&self) -> f64;
fn set_fitness(&mut self, fitness: f64) -> &mut Self;
fn calculate_fitness(&mut self);
fn age(&self) -> usize;
fn set_age(&mut self, age: usize) -> &mut Self;
fn fitness_distance(&self, fitness_target: &f64) -> f64 { ... } // default impl kept
```

**Removed from ChromosomeT (move to LinearChromosome):**
```rust
// Lines 19-29: fn new(), fn default() — default() renamed to reset() on LinearChromosome
// Lines 32-34: fn new_gene() — moves to LinearChromosome
// Lines 37-53: fn dna(), fn dna_mut(), fn set_dna() — moves to LinearChromosome
// Lines 60-72: fn set_gene() default impl — moves to LinearChromosome
// Lines 75-77: fn set_fitness_fn() — moves to LinearChromosome
```

---

### `src/traits/linear_chromosome.rs` (new trait file, ~50 lines)

**Analog:** `src/traits/chromosome.rs` (current full content, lines 1–98) — the new file is carved from the existing trait

**Imports pattern** (copy from current `chromosome.rs` lines 1–2):
```rust
use crate::traits::{ChromosomeT, GeneT};
use std::borrow::Cow;
```

**Supertrait declaration pattern** — standard Rust supertrait:
```rust
pub trait LinearChromosome: ChromosomeT {
    // required methods from current ChromosomeT flat-slice group:
    fn dna(&self) -> &[Self::Gene];
    fn dna_mut(&mut self) -> &mut [Self::Gene];
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;
    fn new_gene() -> Self::Gene { Self::Gene::new() }

    // Default impls (D-02, D-03):
    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self {
        // Copy verbatim from chromosome.rs lines 60-72
    }
    fn reset(&mut self) -> &mut Self {
        // Replaces fn default(mut self) -> Self at chromosome.rs lines 24-29
        // Return type changes from Self to &mut Self (builder-style)
        self.set_fitness(f64::NAN);
        self.set_age(0);
        self.set_dna(Cow::Borrowed(&[]));
        self
    }
}
```

**Module registration pattern** — add to `src/traits/` alongside `chromosome.rs`. Update `src/lib.rs` re-export block to include `LinearChromosome` beside `ChromosomeT`.

---

### `src/traits/operators.rs` (trait, bound changes only)

**Analog:** `src/traits/operators.rs` (current, lines 1–255 — all in context)

**Bound change pattern** — mechanical sed, then manual audit:

`SelectionOperator::select<U>` (line 47) — stays `U: ChromosomeT` (only uses `fitness()`, `age()`).

`CrossoverOperator::crossover<U: ChromosomeT>` (line 83) — changes to `U: LinearChromosome` (accesses `dna()`, `set_dna()`).

`MutationOperator::mutate<U>` (line 131) — stays `U: ChromosomeT + ValueMutable` (ValueMutable gains its own supertrait change; see below).

`SurvivorOperator::select_survivors<U: ChromosomeT>` (line 163) — stays `ChromosomeT` (fitness/age only).

`ExtensionOperator::apply_extension<U: ChromosomeT>` (line 203) — stays `ChromosomeT` for most impls; `mass_deduplication` and `deterministic_crowding` need `LinearChromosome`.

`LocalSearchOperator::improve<U>` (line 248) — changes to `U: LinearChromosome` (accesses `dna()` via fitness fn).

**ValueMutable bound** (defined in `src/operations/mutation.rs` — not shown above but referenced):
```rust
// Before:
pub trait ValueMutable: ChromosomeT { ... }
// After:
pub trait ValueMutable: LinearChromosome { ... }
```

---

### `src/chromosomes/length.rs` (new standalone enum type)

**Analog:** `src/operations.rs` (Selection enum, lines 43–70) — exact structural pattern

**Standalone enum pattern** (copy from `src/operations.rs` lines 43–44):
```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength {
    Fixed(usize),
    Variable { min: usize, max: usize },
}
```

**No factory function needed** — `ChromosomeLength` is a plain data enum with no dispatch.

**Re-export pattern** — add to `src/lib.rs` alongside the `ChromosomeT` / `GeneT` public items:
```rust
// In lib.rs, near line 273 (pub mod traits):
pub use chromosomes::length::ChromosomeLength;
```

**File placement** — `src/chromosomes/length.rs` (D-07). Add `pub mod length;` to `src/chromosomes/mod.rs`.

---

### `src/configuration.rs` (modified — field removals + encapsulation)

**Analog:** `src/configuration.rs` (self — current full file, lines 1–651, all in context)

**`pub(crate)` field + accessor pattern** (ARCH-04):
```rust
// Pattern: convert public fields to pub(crate), add read-only accessors
// Currently (line 312-352): all GaConfiguration fields are `pub`
// After ARCH-04:
pub struct GaConfiguration {
    pub(crate) adaptive_ga: bool,
    pub(crate) limit_configuration: LimitConfiguration,
    pub(crate) selection_configuration: SelectionConfiguration,
    // ... all sub-struct fields become pub(crate) ...
}
impl GaConfiguration {
    // Sub-struct level read-only accessors (D-09):
    pub fn limit(&self) -> &LimitConfiguration { &self.limit_configuration }
    pub fn selection(&self) -> &SelectionConfiguration { &self.selection_configuration }
    pub fn crossover(&self) -> &CrossoverConfiguration { &self.crossover_configuration }
    pub fn mutation(&self) -> &MutationConfiguration { &self.mutation_configuration }
    // ... one accessor per sub-struct field ...
}
```

**StoppingCriteria dissolution** (ARCH-06) — remove `StoppingCriteria` struct (lines 263–275) and `stopping_criteria` field (line 326). Inline the 3 fields directly onto `GaConfiguration`:
```rust
// New fields on GaConfiguration (replaces stopping_criteria: StoppingCriteria):
pub(crate) stagnation_generations: Option<usize>,
pub(crate) convergence_threshold: Option<f64>,
pub(crate) max_duration_secs: Option<f64>,   // keep as Option<f64> un-gated; gate at USAGE site
```

**LimitConfiguration field removals** (ARCH-04/D-06) — remove from `LimitConfiguration` (lines 229–231):
```rust
// Remove these fields entirely:
pub needs_unique_ids: bool,       // line 230
pub alleles_can_be_repeated: bool, // line 231
// Also remove genes_per_chromosome: usize (line 229); replace with:
pub(crate) chromosome_length: ChromosomeLength,
```

**Default impl update pattern** (line 233–245): Update `LimitConfiguration::default()` to remove the removed fields; set `chromosome_length: ChromosomeLength::Fixed(0)`.

**WASM gate pattern** — preserved from current `ga.rs` lines 2102–2104: the field itself is un-gated `Option<f64>`, usage is gated:
```rust
// Call site in ga.rs (DO NOT gate the field, only the usage):
#[cfg(not(target_arch = "wasm32"))]
if let Some(max_secs) = self.configuration.max_duration_secs {  // path changes
    if start_time.elapsed().as_secs_f64() >= max_secs { ... }
}
```

---

### `src/traits/configuration.rs` (modified — StoppingConfig trait changes)

**Analog:** `src/traits/configuration.rs` (self — current content, lines 1–100+, all in context)

**Builder method pattern** (copy existing methods at lines 75–83):
```rust
// Before: one method wrapping a struct (line 82):
fn with_stopping_criteria(self, criteria: StoppingCriteria) -> Self;

// After: three flat builder methods (D-08):
fn with_stagnation_limit(self, n: usize) -> Self;
fn with_convergence_threshold(self, threshold: f64) -> Self;
#[cfg(not(target_arch = "wasm32"))]
fn with_max_duration_secs(self, secs: f64) -> Self;
```

**Trait import cleanup** — remove `StoppingCriteria` from `use crate::configuration::StoppingCriteria` (line 8) after the struct is removed.

---

### `src/engines/ga.rs` (modified — reporter removal + bound changes)

**Analog:** `src/engines/ga.rs` (self — key line ranges already in context)

**Reporter field removal pattern** (lines 278–280):
```rust
// Remove this field entirely:
reporter: Option<Box<dyn Reporter<U> + Send>>,

// Remove the import (lines 140–141):
#[allow(deprecated)]
use crate::reporter::Reporter;
```

**Reporter fire point removal pattern** — 4 locations to delete:
```rust
// Line 1447-1449 (on_start):
if let Some(ref mut r) = self.reporter { r.on_start(); }

// Lines 1974-1975 (on_generation_complete):
// Reporter (legacy) — fires after extension, matching pre-v2.2.0 order
if let Some(ref mut r) = self.reporter { r.on_generation_complete(&stats); }

// Line 2060 (on_new_best):
if let Some(ref mut r) = self.reporter { r.on_new_best(i, best.clone()); }

// Lines 2125-2126 (on_finish):
if let Some(ref mut r) = self.reporter { r.on_finish(self.termination_cause, &self.stats); }
```

**`with_reporter()` builder removal pattern** (lines 848–858):
```rust
// Remove entire method including #[allow(deprecated)] and #[deprecated(...)] attributes
```

**Bound change pattern** — mechanical sed on import and struct bounds:
```rust
// In imports (line 153): ChromosomeT → LinearChromosome (where applicable)
// In struct definition and impl blocks: U: ChromosomeT → U: LinearChromosome
// ga.rs is an orchestrator — ALL bounds change since it calls dna() everywhere
```

**StoppingCriteria path change pattern** (current path at line 2104):
```rust
// Before:
self.configuration.stopping_criteria.max_duration_secs
// After (flat field):
self.configuration.max_duration_secs
```

---

### `src/observe/reporter/` (deleted module)

**Analog:** `src/observe/reporter/mod.rs` (full content in context, lines 1–56)

**Deletion checklist — files to remove:**
- `src/observe/reporter/mod.rs`
- `src/observe/reporter/duration.rs`
- `src/observe/reporter/noop.rs`
- `src/observe/reporter/simple.rs`

**lib.rs cleanup pattern** (current line 266):
```rust
// Remove:
pub mod reporter;
```

**Test file cleanup** — `tests/test_observe.rs` lines 12–14 reference `mod reporter { mod test_reporter; }`. The test file itself lives somewhere under `tests/observe/reporter/` — grep first, then delete.

---

### `src/initializers/binary_initializer.rs` (modified — signature change)

**Analog:** `src/initializers/binary_initializer.rs` (full content in context, lines 1–45)

**Current signature** (line 30–34):
```rust
pub fn binary_random_initialization(
    genes_per_chromosome: usize,
    _alleles: Option<&[BinaryGenotype]>,
    _needs_unique_ids: Option<bool>,        // ← remove this parameter (D-06)
) -> Vec<BinaryGenotype>
```

**After D-06** — remove `_needs_unique_ids` parameter. The `genes_per_chromosome: usize` parameter will also change once the `InitializationFn<U::Gene>` type alias is updated in PR 2.

**Same pattern applies to `src/initializers/range_initializer.rs`** — read that file before implementing to verify the parameter names match.

---

### `MIGRATION.md` (new documentation file)

**No close analog** — this is the first migration guide in the project. Create at crate root alongside `README.md` and `Cargo.toml`. Content must cover the before/after patterns extracted from all ARCH changes above.

**Cargo.toml `include` pattern** — add `"MIGRATION.md"` to the `include` array. Check existing `include` array in `Cargo.toml` before editing.

---

### `.github/workflows/examples-smoke.yml` (new CI workflow)

**Analog:** `.github/workflows/wasm-check.yml` (full content in context, lines 1–33)

**Branch trigger pattern** (copy from `wasm-check.yml` lines 3–7):
```yaml
on:
  push:
    branches: [main, "milestone/**", "feat/**", "fix/**"]
  pull_request:
    branches: [main, "milestone/**"]
```

**Toolchain + cache steps** (copy from `wasm-check.yml` lines 16–23):
```yaml
- uses: actions/checkout@v4
- name: Install stable toolchain
  uses: dtolnay/rust-toolchain@stable
- name: Cache cargo registry and target
  uses: Swatinem/rust-cache@v2
  with:
    key: examples-smoke
```

**Matrix strategy pattern** (standard Actions, no existing analog — use research pattern):
```yaml
jobs:
  examples-smoke:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        example:
          - knapsack_binary
          - onemax_binary
          - onemax_extension
          - rastrigin
          - nsga2_zdt1
          - island_model
          - job_scheduling
          - niching
          - hall_of_fame_demo
          - aos_demo
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          key: "examples-smoke-${{ matrix.example }}"
      - name: Run example (smoke)
        run: cargo run --example ${{ matrix.example }} --release
```

**Note:** The 10 examples above match the full examples/ listing minus multi-obj examples that require special setup. Planner should confirm this list; see RESEARCH.md Assumption A1.

---

## Shared Patterns

### Trait Bound Change (applies to ~30 operator files)

**Source:** `src/operations/crossover/uniform_crossover.rs` (lines 4, 17)
**Apply to:** All files in `src/operations/crossover/`, `src/operations/mutation/` (those calling `dna()`/`set_dna()`)

```rust
// Before pattern (uniform_crossover.rs line 4, 17):
use crate::traits::ChromosomeT;
pub fn uniform<U: ChromosomeT>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {

// After pattern:
use crate::traits::LinearChromosome;
pub fn uniform<U: LinearChromosome>(parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
```

**Sed command** (scope: `src/operations/crossover/`, `src/operations/mutation/`, `src/engines/`):
```bash
sed -i 's/U: ChromosomeT/U: LinearChromosome/g' <file>
# Then update the import line in each file: ChromosomeT → LinearChromosome
```

**Stay at ChromosomeT** (do NOT change these — fitness/age only):
- `src/operations/survivor/age.rs` (line 17)
- `src/operations/survivor/fitness.rs` (line 26)
- `src/operations/survivor/mu_comma_lambda.rs` (line 24)
- `src/operations/survivor/mu_plus_lambda.rs` (line 25)
- `src/operations/extension/mass_genesis.rs` (line 13)
- `src/population.rs`, `src/stats.rs`, `src/observe/observer/`

**Must change to LinearChromosome** (call `dna()`):
- `src/operations/survivor/deterministic_crowding.rs` (lines 23, 44)
- `src/operations/extension/mass_deduplication.rs` (line 17)
- All crossover operator files (all call `dna()` / `set_dna()`)
- All mutation operator files except fitness-only ones

### WASM Gate Pattern

**Source:** `src/engines/ga.rs` (lines 1433–1442, 2102–2104)
**Apply to:** `max_duration_secs` field migration, builder method in `traits/configuration.rs`

```rust
// Pattern 1: warn at run start when feature not available (lines 1435-1442):
#[cfg(target_arch = "wasm32")]
if self.configuration.max_duration_secs.is_some() {
    log::warn!(target: "ga_events",
        "max_duration_secs is not supported on wasm32 — time limit will be ignored");
}

// Pattern 2: gate the usage call (lines 2102-2104):
#[cfg(not(target_arch = "wasm32"))]
if let Some(max_secs) = self.configuration.max_duration_secs {
    if start_time.elapsed().as_secs_f64() >= max_secs { ... }
}
```

### serde Feature Gate Pattern

**Source:** `src/operations.rs` (lines 43–44), `src/configuration.rs` (lines 51–52)
**Apply to:** `ChromosomeLength` enum in new `src/chromosomes/length.rs`

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChromosomeLength { ... }
```

### Deprecated Attribute + Removal Pattern

**Source:** `src/observe/reporter/mod.rs` (lines 36–39), `src/engines/ga.rs` (lines 851–855)

The `#[deprecated(since = "2.2.0", note = "...")]` attributes already exist on `Reporter<U>` and `with_reporter()`. Phase 47 completes the deprecation lifecycle by **removing** these items entirely — no new `#[deprecated]` annotation needed. The compiler error on removal IS the user-facing migration signal.

### ChromosomeT Implementor Update Pattern

**Source:** `src/types/chromosomes/binary.rs` (lines 32–84), `tests/structures.rs` (lines 32–76)

```rust
// Before: impl ChromosomeT for Binary { ... all methods ... }
// After: two impl blocks:

impl ChromosomeT for Binary {
    type Gene = BinaryGenotype;
    fn fitness(&self) -> f64 { ... }
    fn set_fitness(&mut self, fitness: f64) -> &mut Self { ... }
    fn calculate_fitness(&mut self) { ... }
    fn age(&self) -> usize { ... }
    fn set_age(&mut self, age: usize) -> &mut Self { ... }
    // fitness_distance() — default impl inherited, no override needed
}

impl LinearChromosome for Binary {
    fn dna(&self) -> &[Self::Gene] { ... }
    fn dna_mut(&mut self) -> &mut [Self::Gene] { ... }
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self { ... }
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self where ... { ... }
    // new_gene(), set_gene(), reset() — all use default impls from LinearChromosome
}
```

Apply this two-impl split to: `src/types/chromosomes/binary.rs`, `src/types/chromosomes/range.rs`, `src/types/chromosomes/list.rs`, `tests/structures.rs`.

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `MIGRATION.md` | documentation | — | First migration guide in this project; no existing analog. Use RESEARCH.md before/after tables as content source. |

---

## Critical Anti-Patterns (do not copy)

1. **Do not** run sed on `src/traits/`, `src/population.rs`, `src/stats.rs`, `src/observe/`. These must stay at `ChromosomeT` bounds.
2. **Do not** gate the `max_duration_secs` field itself with `#[cfg]` — only gate the call site (see WASM gate pattern above).
3. **Do not** make `ChromosomeLength` inline in `LimitConfiguration` — it must be a standalone public type in its own file.
4. **Do not** forget `new_gene()` call sites — it currently lives on `ChromosomeT` (line 32); after the split it lives on `LinearChromosome`. Any code calling `U::new_gene()` where `U: ChromosomeT` (not `LinearChromosome`) will fail to compile.
5. **Do not** remove only some reporter fire points — all 4 in `ga.rs` (lines ~1447, ~1975, ~2060, ~2125) plus the struct field and builder method must be removed together.

---

## Metadata

**Analog search scope:** `src/traits/`, `src/operations/`, `src/engines/`, `src/configuration.rs`, `src/observe/reporter/`, `.github/workflows/`, `tests/`
**Files scanned:** 18
**Pattern extraction date:** 2026-05-19
