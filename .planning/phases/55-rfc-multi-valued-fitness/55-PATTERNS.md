# Phase 55: RFC Multi-Valued Fitness — Pattern Map

**Mapped:** 2026-05-29
**Files analyzed:** 18 new/modified files
**Analogs found:** 18 / 18

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/traits/vector_fitness.rs` *(rename from `multi_case_fitness.rs`)* | trait | transform | `src/traits/multi_case_fitness.rs` | exact |
| `src/traits.rs` | module | — | self (mod declaration update) | exact |
| `src/lib.rs` | re-export | — | self (re-export swap) | exact |
| `src/operations/selection/lexicase.rs` | operator | request-response | self (method rename only) | exact |
| `src/operations/selection.rs` | factory | request-response | self (bound rename only) | exact |
| `src/engines/ga.rs` | engine | request-response | self (bound rename only) | exact |
| `src/engines/nsga2/mod.rs` | engine | CRUD | self (objective_fns removal) | exact |
| `src/engines/nsga3/mod.rs` | engine | CRUD | `src/engines/nsga2/mod.rs` | exact |
| `src/engines/moead/mod.rs` | engine | CRUD | `src/engines/nsga2/mod.rs` | exact |
| `src/engines/spea2/mod.rs` | engine | CRUD | `src/engines/nsga2/mod.rs` | exact |
| `src/engines/sms_emoa/mod.rs` | engine | CRUD | `src/engines/nsga2/mod.rs` | exact |
| `src/engines/ibea/mod.rs` | engine | CRUD | `src/engines/nsga2/mod.rs` | exact |
| `src/engines/island/nsga2.rs` | engine | CRUD | `src/engines/nsga2/mod.rs` | exact |
| `src/types/chromosomes/binary.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `src/types/chromosomes/range.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `src/types/chromosomes/list.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `src/types/chromosomes/unique.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `src/types/chromosomes/multi_range.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `src/types/chromosomes/multi_unique.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `src/engines/gp/chromosome.rs` | model | transform | `tests/structures.rs` (`MultiCaseChromosome`) | role-match |
| `tests/traits/test_vector_fitness.rs` *(new)* | test | — | `tests/operations/test_selection_lexicase.rs` | role-match |
| `tests/operations/test_selection_lexicase.rs` | test | — | self (method rename + bound update) | exact |
| `tests/operations/test_selection_lexicase_diversity.rs` | test | — | self (method rename only) | exact |
| `tests/engines/*/` (all MO engine test files) | test | — | self (with_objective_fns removal) | exact |

---

## Pattern Assignments

### `src/traits/vector_fitness.rs` (trait rename — was `multi_case_fitness.rs`)

**Analog:** `src/traits/multi_case_fitness.rs`

**Current file content** (lines 1–15 — entire file):
```rust
//! Multi-case fitness trait for lexicase selection.

use crate::traits::ChromosomeT;

/// Opt-in trait enabling `Selection::Lexicase` and `Selection::EpsilonLexicase`.
///
/// Implement alongside [`ChromosomeT`]. Call `set_case_fitness` inside your
/// `calculate_fitness()` implementation.
pub trait MultiCaseFitness: ChromosomeT {
    /// Returns the per-case fitness scores set during `calculate_fitness`.
    fn case_fitness(&self) -> &[f64];

    /// Sets the per-case fitness scores. Called inside `calculate_fitness`.
    fn set_case_fitness(&mut self, scores: Vec<f64>);
}
```

**New file content pattern** (mechanical rename — copy this exactly):
```rust
//! Vector fitness trait for lexicase selection and multi-objective optimization.

use crate::traits::ChromosomeT;

/// Opt-in trait enabling multi-valued fitness for [`Selection::Lexicase`],
/// [`Selection::EpsilonLexicase`], and all multi-objective engines
/// (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA).
///
/// Implement alongside [`ChromosomeT`]. Call `set_fitness_values` inside your
/// `calculate_fitness()` implementation to store objective (or case) scores.
///
/// # No default impl
///
/// `fitness_values()` has no default implementation because `ChromosomeT::fitness()`
/// returns `f64` by value — there is no stored `&f64` to borrow for `&[f64]`.
/// Every implementor must add a `fitness_values: Vec<f64>` field and provide
/// both methods explicitly. See the built-in chromosome types for the pattern.
pub trait VectorFitness: ChromosomeT {
    /// Returns the per-objective (or per-case) fitness values set during
    /// `calculate_fitness`.
    fn fitness_values(&self) -> &[f64];

    /// Sets the fitness values. Called inside `calculate_fitness`.
    fn set_fitness_values(&mut self, values: Vec<f64>);
}
```

**Key change:** No default impl. Decision D-02 desired one but the lifetime constraint makes it impossible without `Vec<f64>` allocation. Use Option A (no default) as recommended by RESEARCH.md.

---

### `src/traits.rs` (mod declaration update)

**Analog:** `src/traits.rs` lines 44 and 55

**Current pattern** (lines 44, 55):
```rust
pub mod multi_case_fitness;
// ...
pub use multi_case_fitness::MultiCaseFitness;
```

**New pattern:**
```rust
pub mod vector_fitness;
// ...
pub use vector_fitness::VectorFitness;
```

---

### `src/lib.rs` (re-export swap)

**Analog:** `src/lib.rs` line 359

**Current pattern:**
```rust
pub use traits::{LinearChromosome, MultiCaseFitness, OperatorCompat, Strategy};
```

**New pattern:**
```rust
pub use traits::{LinearChromosome, VectorFitness, OperatorCompat, Strategy};
```

---

### `src/operations/selection/lexicase.rs` (method rename throughout)

**Analog:** `src/operations/selection/lexicase.rs` (self, current file)

**Import pattern** (line 14 — change bound name):
```rust
// BEFORE:
use crate::traits::{ChromosomeT, MultiCaseFitness};
// AFTER:
use crate::traits::{ChromosomeT, VectorFitness};
```

**Function signature pattern** (lines 26–27, 63–68, 118–125, 168–176 — rename bound and method):
```rust
// BEFORE (example from line 26-27):
fn compute_mad_epsilons<U: MultiCaseFitness>(chromosomes: &[U], num_cases: usize) -> Vec<f64> {
    // ...
    chromosomes.iter().map(|c| c.case_fitness()[case_i])
// AFTER:
fn compute_mad_epsilons<U: VectorFitness>(chromosomes: &[U], num_cases: usize) -> Vec<f64> {
    // ...
    chromosomes.iter().map(|c| c.fitness_values()[case_i])
```

**All `case_fitness()` call sites to rename** (lines 30, 82, 84, 87, 129, 133, 180, 184):
- `c.case_fitness()` → `c.fitness_values()`
- `chromosomes[0].case_fitness()` → `chromosomes[0].fitness_values()`
- `chromosomes[i].case_fitness()` → `chromosomes[i].fitness_values()`

**Where bound `U: ChromosomeT + MultiCaseFitness` appears** (lines 124, 175):
```rust
// BEFORE:
where
    U: ChromosomeT + MultiCaseFitness,
// AFTER:
where
    U: ChromosomeT + VectorFitness,
```

---

### `src/operations/selection.rs` (bound rename + error message update)

**Analog:** `src/operations/selection.rs` (self, current file)

**Import line 11:**
```rust
// BEFORE:
use crate::traits::{ChromosomeT, MultiCaseFitness, SelectionOperator};
// AFTER:
use crate::traits::{ChromosomeT, VectorFitness, SelectionOperator};
```

**`factory_lexicase` function** (lines 164–224): rename the bound and all method call sites:
```rust
// BEFORE (line 170):
U: ChromosomeT + MultiCaseFitness + Sync + Send + 'static + Clone,
// AFTER:
U: ChromosomeT + VectorFitness + Sync + Send + 'static + Clone,
```

**Method call sites** (lines 177–184, 216):
```rust
// BEFORE:
chromosomes[0].case_fitness().is_empty()
c.case_fitness().iter().any(...)
let scores = c.case_fitness().to_vec();
// AFTER:
chromosomes[0].fitness_values().is_empty()
c.fitness_values().iter().any(...)
let scores = c.fitness_values().to_vec();
```

**Error message strings** (lines 179, 185): update the string literals that mention `case_fitness`/`set_case_fitness`:
```rust
// BEFORE:
"case_fitness() is empty — call set_case_fitness in calculate_fitness"
"NaN in case_fitness at chromosome {}"
// AFTER:
"fitness_values() is empty — call set_fitness_values in calculate_fitness"
"NaN in fitness_values at chromosome {}"
```

**Panic message** (line 71): update the string mentioning `MultiCaseFitness`:
```rust
// BEFORE:
"Island-model and NSGA-II paths do not support MultiCaseFitness."
// AFTER:
"Island-model and NSGA-II paths do not support VectorFitness."
```

---

### `src/engines/ga.rs` (bound rename on `select_parents_lexicase` impl block)

**Analog:** `src/engines/ga.rs` lines 2924–2967

**Current impl block bounds** (lines 2924–2936):
```rust
impl<U> Ga<U>
where
    U: LinearChromosome
        + MultiCaseFitness
        + Send
        + Sync
        + 'static
        + Clone
        + Debug
        + mutation::ValueMutable
        + MaybeSerialize
        + MaybeDeserialize
        + OperatorCompat,
```

**New pattern:** Replace `MultiCaseFitness` with `VectorFitness` in the where clause. Update doc comments on the impl block (lines 2941, 2957) that mention `MultiCaseFitness`.

**Import** (line 152): Add `VectorFitness` alongside or replace `MultiCaseFitness`:
```rust
// Current line 152 imports from traits — update that import
use crate::traits::{..., VectorFitness, ...};
```

---

### `src/engines/nsga2/mod.rs` (objective_fns removal + VectorFitness bound)

**Analog:** `src/engines/nsga2/mod.rs` (self — modify in place)

**Struct field removal** (lines 154–157 — remove `objective_fns` field):
```rust
// REMOVE these lines from Nsga2Ga<U> struct:
/// Objective functions (one per objective).
pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
```

**`new()` init removal** (line 173 — remove from constructor):
```rust
// REMOVE:
objective_fns: Vec::new(),
```

**`with_objective_fns()` builder removal** (lines 213–219 — delete entire method):
```rust
// DELETE this entire method:
pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
    self.objective_fns = fns.into_iter().map(Arc::from).collect();
    self
}
```

**`validate()` check replacement** (lines 266–272): Replace objective_fns count check with a simpler validity check (no build-time count check — runtime check goes in `run()`):
```rust
// REMOVE:
if self.objective_fns.len() != self.nsga2_config.num_objectives {
    return Err(GaError::InvalidNsga2Configuration(format!(
        "Expected {} objective functions, got {}",
        self.nsga2_config.num_objectives,
        self.objective_fns.len()
    )));
}
```

**`VectorFitness` bound addition** — Add `U: LinearChromosome + VectorFitness` to the `impl` blocks that call `fitness_values()`. Specifically the `run()` impl block (line 288):
```rust
// BEFORE:
impl<U> Nsga2Ga<U>
where
    U: LinearChromosome + mutation::ValueMutable,
// AFTER:
impl<U> Nsga2Ga<U>
where
    U: LinearChromosome + VectorFitness + mutation::ValueMutable,
```

**Runtime validation in `run()`** — Add after first `initialize_population()` call (line 311):
```rust
// NEW — add after: let mut population = self.initialize_population()?;
if let Some(first) = population.first() {
    let got = first.chromosome.fitness_values().len();
    if got != self.nsga2_config.num_objectives {
        return Err(GaError::InvalidNsga2Configuration(format!(
            "Expected {} objectives from fitness_values(), got {}",
            self.nsga2_config.num_objectives, got
        )));
    }
}
```

**`initialize_population()` pattern** (lines 476–500 — the core change):
```rust
// BEFORE (wasm32 and non-wasm32 branches, lines 476-500):
let objective_fns = &self.objective_fns;
let constraint_fns = &self.constraint_fns;
#[cfg(not(target_arch = "wasm32"))]
let population = chromosomes
    .into_par_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
        let mut ind = ParetoIndividual::new(chrom, objectives);
        ind.constraint_violation = constraint_violation;
        ind
    })
    .collect();
#[cfg(target_arch = "wasm32")]
let population = chromosomes
    .into_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        // ...
    })
    .collect();

// AFTER:
let constraint_fns = &self.constraint_fns;
#[cfg(not(target_arch = "wasm32"))]
let population = chromosomes
    .into_par_iter()
    .map(|mut chrom| {
        chrom.calculate_fitness();
        let objectives = chrom.fitness_values().to_vec();
        let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
        let mut ind = ParetoIndividual::new(chrom, objectives);
        ind.constraint_violation = constraint_violation;
        ind
    })
    .collect();
#[cfg(target_arch = "wasm32")]
let population = chromosomes
    .into_iter()
    .map(|mut chrom| {
        chrom.calculate_fitness();
        let objectives = chrom.fitness_values().to_vec();
        let constraint_violation = evaluate_constraints(chrom.dna(), constraint_fns);
        let mut ind = ParetoIndividual::new(chrom, objectives);
        ind.constraint_violation = constraint_violation;
        ind
    })
    .collect();
```

**`create_offspring()` pattern** (lines 563–587 — same change, the second `objective_fns` usage):
Same transformation: remove `let objective_fns = &self.objective_fns;` and replace `objective_fns.iter().map(|f| f(chrom.dna())).collect()` with `chrom.fitness_values().to_vec()`.

**Note on `calculate_fitness()` in `initialize_population()`:** The chromosomes from `initialize_chromosomes()` call (line 467) are created WITHOUT a fitness function installed (the `None` argument at line 472). For MO engines, chromosomes must have their fitness function set by the user. The AFTER pattern above calls `calculate_fitness()` — but only works if the user's chromosome type self-contains the fitness logic (which it must, post-Phase-55). This is the intended migration.

**Import additions** — Add `VectorFitness` to the use statement for traits:
```rust
use crate::traits::{InitializationFn, LinearChromosome, MutationOperator, VectorFitness};
```

**Remove `ObjectiveFn` type alias re-export** (line 134): `pub use crate::multi_objective::ObjectiveFn;` — keep this re-export only if `constraint_fns` still uses `ObjectiveFn`. Since `constraint_fns: Vec<Arc<ObjectiveFn<U::Gene>>>` remains (constraints are not removed), this re-export stays.

---

### `src/engines/nsga3/mod.rs`, `moead/mod.rs`, `spea2/mod.rs`, `sms_emoa/mod.rs`, `ibea/mod.rs`, `island/nsga2.rs` (identical pattern to nsga2)

**Analog:** `src/engines/nsga2/mod.rs` — apply the exact same changes:

1. Remove `objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>` struct field
2. Remove `objective_fns: Vec::new()` from constructor
3. Delete `with_objective_fns()` builder method
4. Remove `objective_fns.len() != num_objectives` check from `validate()`
5. Add `U: ... + VectorFitness` to `impl` bounds where `run()` is defined
6. In `run()`, add runtime objective count check after first population init
7. In `initialize_population()` and offspring evaluation: remove `objective_fns` usage, replace with `chrom.fitness_values().to_vec()` (after calling `calculate_fitness()`)
8. Add `VectorFitness` to imports

**Verification for each engine:** `grep -n "objective_fns" src/engines/{nsga3,moead,spea2,sms_emoa,ibea}/mod.rs src/engines/island/nsga2.rs` confirms all 7 files have the field.

---

### Built-in chromosome types: `src/types/chromosomes/binary.rs` (and range, list, unique, multi_range, multi_unique)

**Analog:** `tests/structures.rs` — `MultiCaseChromosome` struct pattern (lines 95–178), which already stores `case_scores: Vec<f64>` and implements the trait explicitly.

**Struct field addition pattern** (copy from `Binary` current struct at lines 24–30, new field added):
```rust
// BEFORE (src/types/chromosomes/binary.rs lines 24-30):
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Binary {
    pub dna: Vec<BinaryGenotype>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<BinaryGenotype>,
}

// AFTER — add fitness_values field with serde(default):
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Binary {
    pub dna: Vec<BinaryGenotype>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<BinaryGenotype>,
}
```

**`Default` / `new()` addition** (lines 93–117 — add field initialization):
```rust
// In Default impl and Binary::new():
fitness_values: Vec::new(),
```

**`VectorFitness` impl** (new block after existing impls — copy pattern from `MultiCaseChromosome` lines 166–174):
```rust
impl VectorFitness for Binary {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}
```

**Import addition** — Add `VectorFitness` to the traits import at the top of each chromosome file:
```rust
// BEFORE (binary.rs line 11):
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat};
// AFTER:
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat, VectorFitness};
```

**Apply this same pattern to:** `range.rs`, `list.rs`, `unique.rs`, `multi_range.rs`, `multi_unique.rs` — each already has a similar struct with `fitness: f64`, `age: usize`, and a `fitness_fn` field. Add `fitness_values: Vec<f64>` with `#[cfg_attr(feature = "serde", serde(default))]`, update constructors, add `VectorFitness` impl.

---

### `src/engines/gp/chromosome.rs` (VectorFitness impl with private field style)

**Analog:** `tests/structures.rs` `MultiCaseChromosome` — but note `GpChromosome` uses **private** fields (unlike built-in chromosomes which use `pub`).

**Field addition** (after line 101 `fitness: f64`):
```rust
pub struct GpChromosome<N: GpNode> {
    pub root: Box<Node<N>>,
    fitness: f64,
    age: usize,
    fitness_values: Vec<f64>,   // NEW — private, matching existing field style
    #[cfg_attr(feature = "serde", serde(skip, default = "default_fitness_fn"))]
    fitness_fn: TreeFitnessFn<N>,
}
```

**`serde` bound annotation** (lines 89–96): The existing serde derive uses a custom bound. The new `fitness_values` field needs `#[serde(default)]` since it's serialized (not skipped):
```rust
// Add #[serde(default)] attribute on the field when serde feature active:
#[cfg_attr(feature = "serde", serde(default))]
fitness_values: Vec<f64>,
```

**`Clone` impl update** (lines 125–134 — manual Clone because `Arc<dyn Fn>` is Clone):
```rust
impl<N: GpNode> Clone for GpChromosome<N> {
    fn clone(&self) -> Self {
        GpChromosome {
            root: self.root.clone(),
            fitness: self.fitness,
            age: self.age,
            fitness_values: self.fitness_values.clone(),  // NEW
            fitness_fn: self.fitness_fn.clone(),
        }
    }
}
```

**`Default` impl update** (lines 136–149):
```rust
GpChromosome {
    root: Box::new(Node::Terminal(N::default())),
    fitness: 0.0,
    age: 0,
    fitness_values: Vec::new(),  // NEW
    fitness_fn: None,
}
```

**`with_root()` constructor update** (lines 151–160):
```rust
GpChromosome {
    root,
    fitness: 0.0,
    age: 0,
    fitness_values: Vec::new(),  // NEW
    fitness_fn: None,
}
```

**`VectorFitness` impl** (new block, after `ChromosomeT` impl):
```rust
impl<N: GpNode + Default> VectorFitness for GpChromosome<N> {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}
```

---

### `tests/traits/test_vector_fitness.rs` (new test file)

**Analog:** `tests/operations/test_selection_lexicase.rs` lines 40–43 — existing `test_multi_case_fitness_trait_roundtrip` test.

**Test structure pattern** (copy from `test_selection_lexicase.rs` — uses `MultiCaseChromosome` struct from `tests/structures.rs`):
```rust
use genetic_algorithms::traits::{ChromosomeT, VectorFitness};
// Use MultiCaseChromosome renamed/updated test struct (or a simple inline test struct)

#[test]
fn test_vector_fitness_trait_roundtrip() {
    let mut c = /* test chromosome implementing VectorFitness */;
    c.set_fitness_values(vec![1.0, 2.0, 3.0]);
    assert_eq!(c.fitness_values(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_vector_fitness_reexport() {
    // Verifies D-05: accessible at genetic_algorithms::VectorFitness
    fn accepts_vector_fitness<U: genetic_algorithms::VectorFitness>(_: &U) {}
    let c = /* test chromosome */;
    accepts_vector_fitness(&c);
}
```

---

### Test files with `with_objective_fns` / `MultiCaseFitness` references (mass update)

**Affected test files:**
- `tests/engines/test_ga.rs` — lines 1175, 1217, 1266, 1325, 1392 (5 occurrences)
- `tests/engines/island/test_island_nsga2.rs` — lines 17, 39, 51, 67, 83, 99 (6 occurrences)
- `tests/engines/sms_emoa/test_sms_emoa.rs` (multiple)
- `tests/engines/ibea/test_ibea.rs` (multiple)
- `tests/engines/moead/test_moead.rs` (multiple)
- `tests/engines/nsga2/test_nsga2.rs` (multiple)
- `tests/engines/nsga3/test_nsga3.rs` (multiple)
- `tests/engines/spea2/test_spea2.rs` (multiple)
- `tests/operations/test_selection_lexicase.rs` — lines 10, 27, 40, 42, 43, 249
- `tests/operations/test_selection_lexicase_diversity.rs` — lines 21, 44, 59, 93

**Pattern for MO engine tests:** Remove `.with_objective_fns(...)` call chain. The test chromosome must implement `VectorFitness` and populate `fitness_values` in `calculate_fitness()`. If the test uses `Binary` directly, either switch to a custom test chromosome or ensure `Binary::calculate_fitness()` sets `fitness_values`.

**Pattern for `test_nsga2_validate_mismatched_objective_fns`** (and equivalent in other engines): Replace the build-time validation test with a runtime test that verifies the engine returns `GaError::InvalidNsga2Configuration` when `fitness_values().len() != num_objectives` on first `run()`.

**Pattern for lexicase tests** (from `test_selection_lexicase.rs` lines 40–43):
```rust
// BEFORE:
fn test_multi_case_fitness_trait_roundtrip() {
    c.set_case_fitness(vec![1.0, 2.0, 3.0]);
    assert_eq!(c.case_fitness(), &[1.0, 2.0, 3.0]);
}
// AFTER:
fn test_vector_fitness_trait_roundtrip() {  // renamed test fn
    c.set_fitness_values(vec![1.0, 2.0, 3.0]);
    assert_eq!(c.fitness_values(), &[1.0, 2.0, 3.0]);
}
```

---

## Shared Patterns

### VectorFitness field + impl (apply to all 7 concrete chromosome types)

**Source:** `tests/structures.rs` `MultiCaseChromosome` lines 101, 166–174

**Apply to:** `Binary`, `Range<T>`, `List<T>`, `Unique<T>`, `MultiRange<T>`, `MultiUnique<T>`, `GpChromosome<N>`

```rust
// Field (public for flat chromosomes, private for GpChromosome):
#[cfg_attr(feature = "serde", serde(default))]
pub fitness_values: Vec<f64>,

// Default / constructor value:
fitness_values: Vec::new(),

// Impl block:
impl VectorFitness for <Type> {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }
    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}
```

### WASM-safe parallel eval (preserve in all MO engine population init)

**Source:** `src/engines/nsga2/mod.rs` lines 479–500

The `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` split on `par_iter` vs `iter` must be preserved in every engine. After removing `objective_fns`, the structure stays the same — only the closure body changes:

```rust
#[cfg(not(target_arch = "wasm32"))]
let population = chromosomes
    .into_par_iter()
    .map(|mut chrom| {
        chrom.calculate_fitness();
        let objectives = chrom.fitness_values().to_vec();
        // ...
    })
    .collect();
#[cfg(target_arch = "wasm32")]
let population = chromosomes
    .into_iter()
    .map(|mut chrom| {
        chrom.calculate_fitness();
        let objectives = chrom.fitness_values().to_vec();
        // ...
    })
    .collect();
```

### Serde compatibility for new fields

**Source:** `src/types/chromosomes/binary.rs` lines 23–30 (existing `serde(skip, default)` pattern)

**Apply to:** All chromosome types' new `fitness_values` field

```rust
// The new field must be serde(default) so existing serialized data deserializes correctly:
#[cfg_attr(feature = "serde", serde(default))]
pub fitness_values: Vec<f64>,

// NOTE: do NOT use serde(skip) — the field should be serialized for checkpoint restore.
// Use serde(default) only, so missing fields in old checkpoints default to Vec::new().
```

### Trait bound rename pattern (apply to all 3 locations)

**Source:** `src/engines/ga.rs` line 2927, `src/operations/selection.rs` line 170, `src/operations/selection/lexicase.rs` lines 124, 175

Everywhere `MultiCaseFitness` appears as a trait bound, replace with `VectorFitness`. No other change needed — the method calls are updated separately.

---

## No Analog Found

All files have clear analogs in the codebase. No files in this phase require patterns from RESEARCH.md alone.

---

## Implementation Wave Order

The planner must sequence waves to avoid cascading build failures:

1. **Wave 0 — Trait foundation:** Rename file, rename trait/methods, update `src/traits.rs` mod + re-export, update `src/lib.rs` re-export. After this wave, `cargo check` shows only "MultiCaseFitness not found" errors at call sites.

2. **Wave 1 — Built-in chromosome VectorFitness impls:** Add `fitness_values` field + `VectorFitness` impl to all 7 concrete chromosome types. After this wave, built-in types satisfy the future engine bounds.

3. **Wave 2 — Lexicase callers:** Update `src/operations/selection/lexicase.rs`, `src/operations/selection.rs`, `src/engines/ga.rs`. Pure rename — no behavioral change.

4. **Wave 3 — MO engines (objective_fns removal + VectorFitness bound):** Update all 7 engine files. After this wave, `cargo check` should be green.

5. **Wave 4 — Tests and examples:** Update all test files removing `with_objective_fns`, updating `MultiCaseFitness` bounds, renaming method calls.

---

## Metadata

**Analog search scope:** `src/traits/`, `src/engines/`, `src/types/chromosomes/`, `src/operations/selection/`, `tests/`
**Files scanned:** 24
**Pattern extraction date:** 2026-05-29
