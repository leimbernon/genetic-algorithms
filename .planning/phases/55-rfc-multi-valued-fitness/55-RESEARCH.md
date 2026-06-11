# Phase 55: RFC Multi-Valued Fitness — Research

**Researched:** 2026-05-29
**Domain:** Rust trait system, multi-objective engine integration, internal API rename
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** `MultiCaseFitness` is renamed to `VectorFitness`. It remains a supertrait of `ChromosomeT` (opt-in). NOT auto-implemented via a blanket impl.

**D-02:** `VectorFitness` gains a **default implementation** for `fitness_values()` that wraps scalar `self.fitness()`. The planner must resolve the lifetime issue — see Pitfall 1 below.

**D-03:** Method rename: `case_fitness() -> &[f64]` → `fitness_values() -> &[f64]`; `set_case_fitness(Vec<f64>)` → `set_fitness_values(Vec<f64>)`.

**D-04:** One trait, two use cases (lexicase + MO). Semantic difference handled by engine behavior, not separate traits.

**D-05:** `VectorFitness` re-exported from `src/lib.rs`. No `MultiCaseFitness` alias.

**D-06:** All MO engines (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA) add `U: VectorFitness` bound. Engine reads `chromosome.fitness_values()` to get objective values.

**D-07:** `objective_fns: Vec<Arc<dyn Fn(&[Gene]) -> f64>>` **removed** from all MO engine structs. v3.0.0 breaking change. Users move objective evaluation into `calculate_fitness()`.

**D-08:** `ParetoIndividual.objectives` populated by `chromosome.fitness_values().to_vec()` during init and re-evaluation.

**D-09:** Hard rename in v3.0.0. No type alias bridge. `MultiCaseFitness` disappears.

**D-10:** `factory_lexicase<U: ChromosomeT + MultiCaseFitness>()` bound updates to `U: ChromosomeT + VectorFitness`.

**D-11:** `select_parents_lexicase()` on `Ga<U>` updates `U: MultiCaseFitness` to `U: VectorFitness`.

### Claude's Discretion

Not specified.

### Deferred Ideas (OUT OF SCOPE)

- `objective_fns` as a convenience shorthand helper (deferred for future usability phase).
- Blanket impl of `VectorFitness` for all `ChromosomeT`.
</user_constraints>

---

## Summary

Phase 55 is an internal API rename and integration phase. The existing `MultiCaseFitness` trait (15-line file at `src/traits/multi_case_fitness.rs`) is renamed to `VectorFitness` with method renames, and all MO engines are refactored to read objective values from the chromosome via `fitness_values()` instead of from external closure vectors stored on the engine struct.

The change touches four concern areas: (1) the trait file itself and its module plumbing, (2) lexicase selection call sites (mechanical bound/method name update), (3) all six MO engines plus island NSGA-II (remove `objective_fns` field + `with_objective_fns` builder + closure eval loop, add `VectorFitness` bound, populate `ParetoIndividual` from `fitness_values()`), and (4) all built-in chromosome types need to add a `fitness_values: Vec<f64>` storage field and implement `VectorFitness` so they satisfy the new engine bounds.

The most non-trivial design decision in this phase is resolving the lifetime issue for the default `fitness_values()` implementation. Because `ChromosomeT::fitness()` returns `f64` (a Copy value), `std::slice::from_ref` cannot construct a `&[f64]` with `'self` lifetime from it. The planner must choose a concrete resolution strategy. All test and example files using `objective_fns` or `MultiCaseFitness` must be updated.

**Primary recommendation:** Implement `VectorFitness` in two layers — (a) a mandatory `set_fitness_values` / `fitness_values` with concrete storage (no default), and (b) provide the default via a stored `Vec<f64>` field added to every built-in chromosome type. This is consistent with how `MultiCaseChromosome` in tests already stores `case_scores: Vec<f64>`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Trait rename + module plumbing | Library trait layer | `src/lib.rs` re-export | Single file rename, module path update, re-export swap |
| Lexicase bound + method update | Selection operator layer | `src/engines/ga.rs` impl block | Mechanical rename — no behavioral change |
| MO engine `objective_fns` removal | Engine layer (6 engines + island NSGA-II) | `multi_objective` shared module | Core breaking change — each engine independently stores closures today |
| `ParetoIndividual` population source | Engine layer (init + re-eval fns) | `pareto.rs` struct (unchanged) | `ParetoIndividual::new(chrom, objectives)` contract unchanged; only the `objectives` source changes |
| Built-in chromosome `VectorFitness` impl | Chromosome types (`src/types/chromosomes/`) | `src/engines/gp/chromosome.rs` | Each concrete type must store `fitness_values: Vec<f64>` and implement the trait |
| Validation logic change | Engine `validate()` methods | — | `objective_fns.len() == num_objectives` check replaced by a runtime `fitness_values().len()` check |
| Test + example updates | Tests (`tests/`) + examples | `tests/structures.rs` | All test chromosomes using `objective_fns` need `VectorFitness` impl; all examples need migration |

---

## Standard Stack

No new crates. This phase uses only the Rust standard library and the existing codebase.

### Relevant Language Features

| Feature | Version | Purpose |
|---------|---------|---------|
| `std::slice::from_ref` | stable | Would allow `&[f64]` from `&f64` — requires a reference, NOT a value |
| Default trait method impls | stable | Mechanism for optional `fitness_values()` default — constrained by lifetime |
| `Vec<f64>` storage field | stable | Only practical solution for default `fitness_values()` across heterogeneous types |

---

## Package Legitimacy Audit

> Phase 55 installs zero new packages. This section is N/A.

---

## Architecture Patterns

### System Architecture Diagram

```
Before (Phase 55):
  User code                 MO Engine
  ──────────────            ──────────────────────────────────
  GaChromosome              Nsga2Ga<U: LinearChromosome>
    fitness: f64               objective_fns: Vec<Arc<Fn(&[Gene])→f64>>
    calculate_fitness()          ↓ called during init/re-eval
    → sets self.fitness          ParetoIndividual { objectives: Vec<f64> }

After (Phase 55):
  User code                 MO Engine
  ──────────────            ──────────────────────────────────
  GaChromosome              Nsga2Ga<U: LinearChromosome + VectorFitness>
    fitness: f64               (no objective_fns field)
    fitness_values: Vec<f64>     ↓ reads during init/re-eval
    calculate_fitness()          chromosome.fitness_values().to_vec()
    → sets self.fitness          → ParetoIndividual { objectives: Vec<f64> }
    → sets self.fitness_values
```

### Recommended Project Structure

No structural changes to module layout. Files change in-place:

```
src/
├── traits/
│   ├── multi_case_fitness.rs   → renamed to vector_fitness.rs
│   └── (traits.rs mod declaration updated)
├── engines/
│   ├── nsga2/mod.rs            → remove objective_fns, add VectorFitness bound
│   ├── nsga3/mod.rs            → same
│   ├── moead/mod.rs            → same
│   ├── spea2/mod.rs            → same
│   ├── sms_emoa/mod.rs         → same
│   ├── ibea/mod.rs             → same
│   └── island/nsga2.rs         → same
├── types/chromosomes/
│   ├── binary.rs               → add fitness_values: Vec<f64>, impl VectorFitness
│   ├── range.rs                → same
│   ├── list.rs                 → same
│   ├── unique.rs               → same
│   ├── multi_range.rs          → same
│   └── multi_unique.rs         → same
├── engines/gp/chromosome.rs    → add fitness_values: Vec<f64>, impl VectorFitness
└── lib.rs                      → swap MultiCaseFitness re-export → VectorFitness
```

### Pattern 1: Trait Rename (mechanical)

**What:** Rename file, rename trait, rename methods, update all `use` sites.
**When to use:** Single find-replace across codebase, confirmed by compiler errors.

```rust
// Source: existing src/traits/multi_case_fitness.rs — rename to vector_fitness.rs
// BEFORE:
pub trait MultiCaseFitness: ChromosomeT {
    fn case_fitness(&self) -> &[f64];
    fn set_case_fitness(&mut self, scores: Vec<f64>);
}

// AFTER (src/traits/vector_fitness.rs):
pub trait VectorFitness: ChromosomeT {
    fn fitness_values(&self) -> &[f64];
    fn set_fitness_values(&mut self, values: Vec<f64>);
}
```

### Pattern 2: Default `fitness_values()` — Lifetime Resolution

**What:** The CONTEXT.md decision D-02 specifies a default impl that wraps `self.fitness()`. This is non-trivial because `fitness()` returns `f64` (Copy value), and `&[f64]` requires a reference with `'self` lifetime.

**The problem:**
```rust
// DOES NOT COMPILE — fitness() returns f64 (value), not &f64
fn fitness_values(&self) -> &[f64] {
    std::slice::from_ref(&self.fitness())  // temporary &f64 — does not live long enough
}
```

**Resolution strategies (planner must choose one):**

| Option | Mechanism | Consequence |
|--------|-----------|-------------|
| **A — No default impl** | Remove the default; every implementor provides both methods | Explicit, zero magic. All built-in chromosomes must implement, but they must add the field anyway (see below) |
| **B — Return `Vec<f64>` instead of `&[f64]`** | Change return type to `Vec<f64>` | Avoids the lifetime issue. Imposes a heap allocation on every engine read. Diverges from `case_fitness()` precedent. |
| **C — Store `fitness_values: Vec<f64>` on the trait's expected implementors** | Each concrete type adds the field; the default impl does not exist at trait level — `VectorFitness::fitness_values()` has no default | Same as A — each type provides its own impl referencing its own field |
| **D — Provide a helper macro** | `impl_vector_fitness_default!(MyChromosome)` expands `fitness_values` returning `&self.fitness_values` | Reduces boilerplate but adds a macro |

**Recommended:** Option A / C combined — no default impl at the trait level. Every concrete type (built-in + user-defined) must provide the impl. The documentation explains the standard pattern (add `fitness_values: Vec<f64>` field, implement both methods). This is the pattern already established by `MultiCaseChromosome` in `tests/structures.rs`.

If D-02 is revisited and a default impl is desired anyway, Option B (return `Vec<f64>`) is the cleanest Rust solution, but it allocates on every read in engine hot paths (population init, re-eval). This is a design tradeoff the planner should flag to the user.

### Pattern 3: Engine `objective_fns` Removal

**What:** Each MO engine struct has `objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>`. This field and its builder method, validation check, and all call sites within `initialize_population()` and `reevaluate_population()` must be removed.

**Pattern (same for all 7 engine locations):**
```rust
// BEFORE (in initialize_population):
let objective_fns = &self.objective_fns;
chromosomes.iter().map(|chrom| {
    let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
    ParetoIndividual::new(chrom.clone(), objectives)
}).collect()

// AFTER:
chromosomes.iter().map(|chrom| {
    let objectives = chrom.fitness_values().to_vec();
    ParetoIndividual::new(chrom.clone(), objectives)
}).collect()
```

**Validation change:** Replace `objective_fns.len() != num_objectives` with a runtime check. Since `fitness_values()` length is only known after a chromosome is evaluated, `validate()` at build time cannot check it. Options:
- Remove the `objective_fns`-count validation entirely, and add a runtime check at the start of `run()` verifying the first chromosome's `fitness_values().len() == num_objectives`.
- Alternatively, keep `num_objectives` as a documentation/direction-matching field only (not validated against chromosome output at build time).

### Anti-Patterns to Avoid

- **Keep `with_objective_fns` as deprecated alias:** D-07 says hard removal. No deprecation shim.
- **Blanket impl `VectorFitness` for all `ChromosomeT`:** Explicitly rejected in D-01.
- **Accessing `objective_fns` after the field is removed:** Every reference to the field in `sms_emoa/mod.rs:426` (`.objective_fns` direct access in an inner lambda) must also be cleaned up.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| File rename from `multi_case_fitness.rs` to `vector_fitness.rs` | Manual copy-paste | `mv` + update `mod` declaration in `traits.rs` | Atomic rename preserves git history |
| Locating all `MultiCaseFitness` call sites | Manual grep | `cargo check` error list after renaming the trait | Compiler finds every usage |
| Serde field compatibility | Custom migration | `#[serde(rename = "case_scores")]` if field names change | Existing `serde` feature tests must not break |

---

## Runtime State Inventory

> Omit — not a rename/refactor of stored data. This is a library API rename with no persisted state.

---

## Common Pitfalls

### Pitfall 1: The Default `fitness_values()` Lifetime Trap

**What goes wrong:** Attempting `std::slice::from_ref(&self.fitness())` in a default impl fails to compile because `fitness()` returns `f64` by value (temporary). The borrow does not outlive the expression.

**Why it happens:** `ChromosomeT::fitness(&self) -> f64` is a Copy return. There is no stored `&f64` to borrow from.

**How to avoid:** Either (a) accept there is no default impl (each concrete type provides its own with a stored field), or (b) change the return type to `Vec<f64>` which avoids lifetime entirely. Option (a) is recommended — it is consistent with how `MultiCaseChromosome` already works.

**Warning signs:** Compiler error `temporary value does not live long enough` in `vector_fitness.rs`.

### Pitfall 2: Built-In Chromosomes Fail MO Engine Bounds

**What goes wrong:** After adding `U: VectorFitness` to `Nsga2Ga<U>`, the existing tests compile with `Nsga2Ga::<Binary>` — but `Binary` does not currently implement `VectorFitness`. All test and example uses of built-in chromosomes with MO engines break.

**Why it happens:** `Binary`, `Range<T>`, `List<T>`, `Unique<T>`, `MultiRange<T>`, `MultiUnique<T>`, and `GpChromosome<N>` have no `fitness_values` field today.

**How to avoid:** Add `fitness_values: Vec<f64>` field and `VectorFitness` impl to every built-in chromosome type in `src/types/chromosomes/` and `src/engines/gp/chromosome.rs`. This is non-negotiable — the wave containing engine bound additions must come after the wave adding `VectorFitness` impls to all built-in types.

**Warning signs:** Cascading trait bound errors across all MO engine tests after adding the `VectorFitness` bound.

### Pitfall 3: `serde` Feature Breaks Due to New Field

**What goes wrong:** Adding `fitness_values: Vec<f64>` to serde-derived chromosome structs requires the field to be included in serialization, or explicitly skipped. Existing serialized data (checkpoint files) will fail to deserialize if the new field has no `default` attribute.

**Why it happens:** `serde` derive with a new field and no `#[serde(default)]` rejects JSON that doesn't include the field.

**How to avoid:** Annotate `fitness_values: Vec<f64>` with `#[serde(default)]` on all chromosome types. `Vec::new()` is the correct default (empty fitness values before evaluation).

**Warning signs:** `cargo test --features serde` panics with `missing field 'fitness_values'` in serde tests.

### Pitfall 4: Island NSGA-II Missed

**What goes wrong:** `src/engines/island/nsga2.rs` is a separate file from `src/engines/nsga2/mod.rs` and also stores `objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>`. It has 11 references. It is easy to miss when working engine-by-engine.

**Why it happens:** Island NSGA-II is a separate engine struct from the standalone Nsga2Ga.

**How to avoid:** The island NSGA-II is one of 7 locations that must be updated (6 standalone engines + 1 island engine).

**Warning signs:** `cargo test` passes on engine tests but `tests/engines/island/test_island_nsga2.rs` still compiles with `with_objective_fns` calls — which would indicate the island engine wasn't updated.

### Pitfall 5: `validate()` Check Becomes Invalid

**What goes wrong:** After removing `objective_fns`, the `validate()` methods across all engines currently check `objective_fns.len() != nsga2_config.num_objectives`. If this check is not replaced, validation no longer verifies that users provide the right number of objectives.

**Why it happens:** Objective count was previously enforced at build time by counting closures. After the change, it can only be enforced at runtime by inspecting `chromosome.fitness_values().len()`.

**How to avoid:** In each engine's `run()` method, after initializing the population (which calls `calculate_fitness()` and thus populates `fitness_values()`), add a guard:
```rust
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

**Warning signs:** No runtime error when user provides a chromosome with 1 fitness value in a 3-objective engine.

### Pitfall 6: Test Chromosomes Need `VectorFitness` Impls

**What goes wrong:** Tests in `tests/engines/nsga2/test_nsga2.rs` use `Nsga2Ga::<Binary>`. If only the built-in `Binary` gets `VectorFitness`, but the test doesn't call `set_fitness_values`, the `objectives` in every `ParetoIndividual` will be an empty `Vec<f64>`, causing the engine to silently produce wrong results.

**Why it happens:** Tests with `with_initialization_fn(|_, _| vec![])` never trigger `calculate_fitness()` — the init closure returns an empty population, so `fitness_values()` is never populated.

**How to avoid:** Validation-only tests (testing mismatched params, zero objectives, etc.) can use `Binary` without ever running the engine. Full run tests must use a chromosome whose `calculate_fitness()` populates `fitness_values` with the correct number of values.

---

## Code Examples

### Example 1: `VectorFitness` trait definition (resolved form)

```rust
// src/traits/vector_fitness.rs
// Source: existing multi_case_fitness.rs pattern (verified from codebase)
use crate::traits::ChromosomeT;

/// Opt-in trait enabling multi-valued fitness for lexicase selection and
/// multi-objective optimization engines.
///
/// Implement alongside [`ChromosomeT`]. Call `set_fitness_values` inside
/// your `calculate_fitness()` implementation.
pub trait VectorFitness: ChromosomeT {
    /// Returns the per-objective (or per-case) fitness values set during
    /// `calculate_fitness`.
    fn fitness_values(&self) -> &[f64];

    /// Sets the fitness values. Called inside `calculate_fitness`.
    fn set_fitness_values(&mut self, values: Vec<f64>);
}
```

### Example 2: Built-in chromosome `VectorFitness` impl pattern

```rust
// Pattern to apply to Binary, Range<T>, List<T>, Unique<T>, MultiRange<T>, MultiUnique<T>
// Source: MultiCaseChromosome pattern in tests/structures.rs (verified from codebase)

// 1. Add field to struct:
pub struct Binary {
    // ... existing fields ...
    pub fitness_values: Vec<f64>,   // NEW — annotated with serde(default) if serde feature active
}

// 2. Add serde annotation (in the serde cfg block):
#[cfg_attr(feature = "serde", serde(default))]
pub fitness_values: Vec<f64>,

// 3. Default() implementation (add to existing Default/new):
fitness_values: Vec::new(),

// 4. VectorFitness impl:
impl VectorFitness for Binary {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }
    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}
```

### Example 3: Engine population init after change

```rust
// Pattern for initialize_population in all MO engines
// Source: verified from src/engines/nsga2/mod.rs:477-496 (current code)

// BEFORE:
let objective_fns = &self.objective_fns;
let pop: Vec<ParetoIndividual<U>> = chromosomes
    .into_iter()
    .map(|chrom| {
        let objectives: Vec<f64> = objective_fns.iter().map(|f| f(chrom.dna())).collect();
        ParetoIndividual::new(chrom, objectives)
    })
    .collect();

// AFTER:
let pop: Vec<ParetoIndividual<U>> = chromosomes
    .into_iter()
    .map(|chrom| {
        let objectives = chrom.fitness_values().to_vec();
        ParetoIndividual::new(chrom, objectives)
    })
    .collect();
```

### Example 4: User migration pattern for MO engines

```rust
// BEFORE: user provided objective closures to the engine
let nsga2 = Nsga2Ga::<MyChromosome>::new(config, ga_config)
    .with_objective_fns(vec![
        Box::new(|dna| dna[0].value),
        Box::new(|dna| 1.0 - dna[0].value),
    ]);

// AFTER (v3.0.0): user implements VectorFitness on their chromosome
impl ChromosomeT for MyChromosome {
    fn calculate_fitness(&mut self) {
        let f1 = self.dna()[0].value;
        let f2 = 1.0 - f1;
        self.set_fitness(f1);           // scalar fitness still required
        self.set_fitness_values(vec![f1, f2]);  // objective vector
    }
    // ... other methods ...
}

impl VectorFitness for MyChromosome {
    fn fitness_values(&self) -> &[f64] { &self.objectives }
    fn set_fitness_values(&mut self, v: Vec<f64>) { self.objectives = v; }
}

let nsga2 = Nsga2Ga::<MyChromosome>::new(config, ga_config);
// No with_objective_fns needed
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `MultiCaseFitness::case_fitness()` (Phase 50) | `VectorFitness::fitness_values()` | Phase 55 | Same semantics, better name |
| `objective_fns: Vec<Arc<Fn>>` stored on engine struct | Chromosome implements `VectorFitness` | Phase 55 | Objectives co-located with chromosome; no Arc overhead in hot path |
| Validation: closure count == num_objectives (build time) | Runtime check: `fitness_values().len() == num_objectives` (first run) | Phase 55 | Weaker build-time guarantee; stronger encapsulation |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `GpChromosome<N>` needs a `fitness_values: Vec<f64>` field added (it currently has no such field) | Pitfall 2, Standard Stack | Low risk — verified by reading `src/engines/gp/chromosome.rs` (no such field exists) [VERIFIED: codebase grep] |
| A2 | No built-in chromosome type implements `MultiCaseFitness` today | Pitfall 2 | Low risk — verified by grep showing no `impl MultiCaseFitness for Binary` etc. [VERIFIED: codebase grep] |
| A3 | `src/engines/island/nsga2.rs` must be updated (it is the 7th engine location) | Pitfall 4 | High impact if missed — verified by reading the file, which has `pub objective_fns` [VERIFIED: codebase grep] |

**All claims in this research are VERIFIED or CITED against the codebase. No ASSUMED claims.**

---

## Open Questions (RESOLVED)

1. **Should the default `fitness_values()` impl exist at the trait level?**
   - **RESOLVED 2026-05-30:** No default impl (Option A). User confirmed. D-02 amended to reflect this. Every concrete type provides an explicit impl backed by a `fitness_values: Vec<f64>` field.

2. **Should `GpChromosome` have `fitness_values` public or private?**
   - **RESOLVED:** Match `GpChromosome`'s existing encapsulation style — `fitness_values` field private, accessed only via `VectorFitness` trait methods. Plans implement this.

3. **How are validation-only MO engine tests updated?**
   - **RESOLVED:** Replace `test_nsga2_validate_mismatched_objective_fns` with a runtime test that verifies the engine errors when `fitness_values().len() != num_objectives` on first run. Plans 04/05 implement this pattern; Plan 06 updates the tests.

---

## Environment Availability

> Step 2.6: SKIPPED — Phase 55 is a pure code/API change with no external dependencies. All work is in-tree Rust.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `cargo test` |
| Config file | None (standard cargo) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRAITS-01 (updated) | `VectorFitness` trait roundtrip — `set_fitness_values` / `fitness_values` | unit | `cargo test test_vector_fitness` | ❌ Wave 0 (replaces `test_multi_case_fitness_trait_roundtrip`) |
| D-06 | `Nsga2Ga<U: VectorFitness>` — objectives populated from `fitness_values()` | integration | `cargo test engines::nsga2` | ✅ (must be updated) |
| D-06 | Same for NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA, Island NSGA-II | integration | `cargo test engines` | ✅ (must be updated) |
| D-07 | `with_objective_fns` removed — call site should not compile | compile-only | `cargo check` (no `with_objective_fns` in source) | N/A |
| D-10 | `factory_lexicase` bound works with `VectorFitness` | integration | `cargo test operations::selection` | ✅ (must be updated) |
| D-05 | `VectorFitness` accessible at `genetic_algorithms::VectorFitness` | unit | `cargo test test_vector_fitness_reexport` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo check`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/traits/test_vector_fitness.rs` — covers TRAITS-01 trait roundtrip and re-export (D-05)
- [ ] No new framework install needed — existing cargo test infrastructure applies

---

## Security Domain

> This phase is a pure API rename and internal refactor. No authentication, session management, access control, input from external sources, or cryptographic operations are involved. Security domain does not apply.

---

## Sources

### Primary (HIGH confidence — verified by reading source files)

- `src/traits/multi_case_fitness.rs` — Current 15-line trait definition; confirms exact method signatures
- `src/engines/nsga2/mod.rs` — Confirms `objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>` is on the engine struct (not a config struct), with `with_objective_fns()` builder and validation at lines 266-272
- `src/engines/nsga3/mod.rs`, `spea2/mod.rs`, `moead/mod.rs`, `sms_emoa/mod.rs`, `ibea/mod.rs` — Confirm identical `objective_fns` pattern in all 5 remaining standalone engines
- `src/engines/island/nsga2.rs` — Confirms 7th engine location with `objective_fns` field (line 87)
- `src/engines/nsga2/pareto.rs` — Confirms `ParetoIndividual::new(chromosome, objectives)` contract is unchanged; only data source changes
- `src/engines/nsga2/configuration.rs` — Confirms `Nsga2Configuration` does NOT contain `objective_fns` (field is on the engine struct directly)
- `src/traits/chromosome.rs` — Confirms `fitness(&self) -> f64` (Copy return, not `&f64`); explains default impl limitation
- `tests/structures.rs` — Confirms the established storage pattern: `case_scores: Vec<f64>` field with explicit `VectorFitness` impl
- `src/types/chromosomes/binary.rs` — Confirms `pub fitness: f64` pattern; no `fitness_values` field exists
- `src/engines/gp/chromosome.rs` — Confirms `GpChromosome` has no `fitness_values` field; uses private field style

### Secondary (MEDIUM confidence)

- `tests/engines/nsga2/test_nsga2.rs`, `tests/engines/spea2/test_spea2.rs` — Confirm test patterns and scope of test updates needed
- `examples/nsga2_zdt1.rs` — Confirms example migration scope: `with_objective_fns` + inline closures → `VectorFitness` impl on chromosome

---

## Metadata

**Confidence breakdown:**

- Trait rename mechanics: HIGH — single file rename, confirmed by reading source
- Engine `objective_fns` removal scope: HIGH — verified all 7 locations (6 engines + island NSGA-II)
- Default impl lifetime issue: HIGH — fundamental Rust lifetime constraint, not ambiguous
- Built-in chromosome `VectorFitness` scope: HIGH — grep confirms no existing impls; all 6 types + GpChromosome confirmed by file inspection
- Serde `#[default]` requirement: HIGH — serde behavior with new fields is well-understood

**Research date:** 2026-05-29
**Valid until:** 2026-06-28 (stable Rust codebase, 30-day window)
