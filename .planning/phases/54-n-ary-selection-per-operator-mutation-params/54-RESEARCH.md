# Phase 54: N-ary Selection + Per-Operator Mutation Params — Research

**Researched:** 2026-05-28
**Domain:** Rust trait/enum API refactoring — operator layer (selection + mutation)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01 Execution Order:** Wave 1 = N-ary selection; Wave 2 = Mutation params. Independent changes, separated to reduce conflict surface.

**D-02 `SelectionOperator::select` new signature:**
```rust
fn select<U>(
    &self,
    chromosomes: &[U],
    number_of_couples: usize,
    number_of_threads: usize,
    num_parents: usize,   // NEW: 2 for standard, N for multi-parent
) -> Vec<Vec<usize>>
```
Breaking change for all custom implementations.

**D-03 `selection::factory` extends to include `num_parents`** read from the crossover variant embedded field (Undx/Spx/Pcx carry their own `num_parents`). If crossover has no `num_parents` field (standard variants), `num_parents = 2`.

**D-04 `parent_crossover` changes `parents` parameter from `&[(usize, usize)]` to `&[Vec<usize>]`.** Inside, `group.len()` drives dispatch: `len() == 2` → `crossover::factory`, `len() > 2` → `crossover::factory_multi_parent_dispatch`. Unified N-ary and multi-parent paths.

**D-05 `factory_multi_parent_dispatch` from Phase 51 kept** as internal helper, called from unified `parent_crossover`. No public API removal.

**D-06 `Mutation` enum variants gain inline params:**
- `Mutation::Gaussian { sigma: Option<f64> }` (None → default 0.1)
- `Mutation::Creep { step: Option<f64> }` (None → default 0.01)
- `Mutation::Polynomial { eta: Option<f64> }`
- `Mutation::Cauchy { scale: Option<f64> }`
- `Mutation::LevyFlight { alpha: Option<f64> }`
- `Mutation::SelfAdaptiveGaussian { tau: Option<f64>, tau_prime: Option<f64>, sigma_min: Option<f64>, sigma_max: Option<f64> }`
- `Mutation::Insertion` / `Mutation::Deletion` — no inline params
- Unit variants (Swap, Inversion, Scramble, BitFlip, Value, NonUniform, Differential, PermutationInsert, ListValue, Uniform) — no params, remain unit variants
- `Mutation` loses `Copy`; derives `Clone` instead. Serde derives remain.

**D-07 `MutationOperator` trait new signature:**
```rust
fn mutate<U>(&self, individual: &mut U, mutation: &Mutation) -> Result<(), GaError>
where U: LinearChromosome + ValueMutable + 'static;
```
Operators extract their own params from the `mutation` variant reference. Breaking change for custom implementations.

**D-08 `MutationConfiguration` retains only operator-agnostic fields:** `probability`, `probability_max`, `probability_min`, `dynamic_mutation`, `probability_step`, `target_cardinality`. All operator-specific fields (`step`, `sigma`, `polynomial_eta`, `non_uniform_b`, `differential_f`, `cauchy_scale`, `levy_alpha`, `self_adaptive_tau`, `self_adaptive_tau_prime`, `sigma_min`, `sigma_max`) are REMOVED. They move into enum variants.

**D-09 The GA loop's big if/else mutation dispatch chain** is replaced by a single `mutation_method.mutate(&mut child, &mutation_method)` call through the trait. The operator struct reads its own params from the variant.

### Claude's Discretion

None documented.

### Deferred Ideas (OUT OF SCOPE)

- GP-specific observer hooks (`on_bloat_detected`)
- `SelectionOperator` supporting non-uniform group sizes in a single call
- Making `Crossover` variants carry inline params
</user_constraints>

---

## Summary

Phase 54 is a pure API refactoring — no new algorithms, no new files beyond modifications. Both changes are mechanical and surgical: the selection change affects one trait, two factory functions, one GA loop function, and all call sites that destructure `(usize, usize)` tuples. The mutation change affects one trait, one enum, one configuration struct, the GA loop dispatch block, and all builders that set now-removed configuration fields.

The key challenge is scope: `Mutation` is `#[derive(Copy)]` today and is silently copied in many places. Once it becomes non-Copy, every place that assigns from a `Mutation` field (checkpoint restore, AOS portfolio index, method field save/restore) must use `.clone()`. The GA loop's AOS path at line 2619 does `portfolio[op_idx]` which produces a copy today — after the change it must become `portfolio[op_idx].clone()`. Similarly, `builder_mutation = self.configuration.mutation_configuration.method` at line 1425 must become `.clone()`.

`serde` serialization of the `Mutation` enum is gated behind `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`. Adding struct-variant fields to the enum is a compatible serde change for JSON deserialization as long as the fields carry `#[serde(default)]`. The checkpoint `GaConfiguration` struct embeds `MutationConfiguration`; removing fields from that struct is a serde-breaking change — old checkpoint files that contain `step`, `sigma`, etc. will produce unknown-field errors unless `#[serde(deny_unknown_fields)]` is absent (it is absent in the current code, so unknown fields are silently ignored).

**Primary recommendation:** Tackle this in two fully independent waves. Wave 1 (N-ary selection) has a smaller blast radius — only the selection trait, two factory functions, three GA loop call sites, the island/GP/cellular engine call sites, and test files that iterate over parent pairs. Wave 2 (mutation params) has the larger blast radius across 14 test files, 5 examples, the configuration struct, all builder methods, and the GA loop dispatch block.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| N-ary selection API change | Trait layer (`src/traits/operators.rs`) | Factory dispatch (`src/operations/selection.rs`) | Trait defines contract; factory implements dispatch |
| Selection call site update | Engine layer (`src/engines/ga.rs`) | Island/GP/Cellular engines | Each engine owns its selection invocation |
| Mutation enum variant params | Operation enum (`src/operations.rs`) | Trait layer | Enum carries data; trait defines behavior |
| MutationOperator dispatch | Trait impl (`src/operations/mutation.rs`) | — | `impl MutationOperator for Mutation` is the single dispatch point |
| MutationConfiguration cleanup | Configuration layer (`src/configuration.rs`) | Builder traits (`src/traits/configuration.rs`) | Struct definition + builder methods |
| GA loop dispatch simplification | Engine layer (`src/engines/ga.rs`) | — | `parent_crossover` owns the if/else mutation block |

---

## Standard Stack

No new external packages. This phase modifies existing Rust code only.

| Item | Current State | Post-Phase State |
|------|--------------|-----------------|
| `Mutation` enum derive | `#[derive(Copy, Clone, Debug, PartialEq)]` | `#[derive(Clone, Debug, PartialEq)]` (Copy removed) |
| `SelectionOperator::select` return | `Vec<(usize, usize)>` | `Vec<Vec<usize>>` |
| `MutationOperator::mutate` signature | `fn mutate<U>(&self, individual: &mut U, step: Option<f64>, sigma: Option<f64>) -> Result<(), GaError>` | `fn mutate<U>(&self, individual: &mut U, mutation: &Mutation) -> Result<(), GaError>` |
| `MutationConfiguration` fields | 14 fields (step, sigma, polynomial_eta, etc.) | 6 fields (probability, probability_max, probability_min, dynamic_mutation, probability_step, target_cardinality) |

## Package Legitimacy Audit

No packages installed. Section not applicable.

---

## Architecture Patterns

### System Architecture Diagram

```
Phase 54 change surfaces (two independent waves):

Wave 1 — N-ary Selection
  Configuration
    └── CrossoverConfiguration.method → Undx/Spx/Pcx carries num_parents
          ↓
  selection::factory(chromosomes, sel_config, threads)   [currently → Vec<(usize,usize)>]
          ↓ [CHANGE: → Vec<Vec<usize>>; factory reads num_parents from cx method]
  GA loop: parent_crossover(parents: &[Vec<usize>], ...)
          ↓
  group.len() == 2 → crossover::factory(parent_1, parent_2, cx_config)
  group.len() >  2 → crossover::factory_multi_parent_dispatch(&parent_refs, cx_config)
          ↑ (replaces ad-hoc Undx/Spx/Pcx match block in parent_crossover)

  Also touches:
  - SelectionOperator trait (add num_parents param)
  - Selection::select impl (use num_parents to build Vec<Vec<usize>>)
  - factory_lexicase (update return type)
  - Island engine loop (for &(idx_a, idx_b) → for group in &parent_pairs)
  - GP engine loop   (for (i, j) in &pairs → for group in &pairs)
  - Cellular engine  (.select → update destructure)

Wave 2 — Per-Operator Mutation Params
  Mutation enum [CHANGE: variants gain Option<f64> fields; lose Copy]
          ↓
  MutationConfiguration [CHANGE: remove step/sigma/…_eta/…_scale/etc.]
  Builder methods [CHANGE: with_mutation_step/sigma → removed; with_gaussian_sigma etc. added OR variant constructed inline]
          ↓
  GA loop: mutation if/else dispatch block
          ↓ [CHANGE: single call mutation_method.mutate(&mut child, &mutation_method)]
  MutationOperator::mutate(&self, individual, &Mutation) [CHANGE: extract params from variant]
          ↓
  operator implementations (gaussian.rs, creep.rs, etc.) — no signature change inside
```

### Recommended Project Structure

No structural changes. All changes are in-place modifications:

```
src/
├── operations.rs              — Mutation enum variant fields added; Copy removed
├── operations/
│   ├── selection.rs           — factory + factory_lexicase return type change; SelectionOperator impl
│   └── mutation.rs            — factory_with_params, factory_self_adaptive removed or simplified;
│                                  MutationOperator::mutate impl updated
├── traits/
│   └── operators.rs           — SelectionOperator::select, MutationOperator::mutate signatures
├── configuration.rs           — MutationConfiguration struct fields removed; CrossoverConfiguration unchanged
├── traits/
│   └── configuration.rs       — Builder methods for removed fields deleted
└── engines/
    ├── ga.rs                  — parent_crossover params + mutation dispatch block
    ├── gp/engine.rs           — pairs iteration update
    ├── island/mod.rs          — parent_pairs iteration update
    └── cellular/engine.rs     — select call update
```

### Pattern 1: N-ary Selection Group Construction

**What:** Each built-in selection operator needs to build `Vec<Vec<usize>>` groups of `num_parents` instead of `Vec<(usize, usize)>` pairs.

**When to use:** Inside `impl SelectionOperator for Selection` and inside each selection function (tournament, roulette, rank, etc.).

**Current pattern (all functions return `Vec<(usize, usize)>`):**
```rust
// Source: src/operations/selection/tournament.rs (inferred from existing code)
fn tournament(chromosomes, number_of_couples, number_of_threads) -> Vec<(usize, usize)> {
    // ... builds pairs
    pairs.push((winner_1, winner_2));
}
```

**New pattern:**
```rust
// num_parents = 2 for standard; N for multi-parent crossover
fn tournament(chromosomes, number_of_couples, number_of_threads, num_parents: usize) -> Vec<Vec<usize>> {
    let mut groups = Vec::with_capacity(number_of_couples);
    for _ in 0..number_of_couples {
        let group: Vec<usize> = (0..num_parents).map(|_| /* tournament pick */ ).collect();
        groups.push(group);
    }
    groups
}
```

**Backward compatibility:** When `num_parents == 2`, all groups are `[a, b]` — identical semantics to the old `(a, b)` tuple, just stored in a `Vec<usize>`. No behavior change for standard 2-parent selection. [ASSUMED — standard approach for this pattern]

### Pattern 2: GA Loop Group Dispatch

**What:** The `parent_crossover` function receives `&[Vec<usize>]` instead of `&[(usize, usize)]`. Group length drives crossover dispatch.

**Current pattern:**
```rust
// Source: src/engines/ga.rs:2581
let process_pair = |(key, value): &(usize, usize)| -> Result<Vec<U>, GaError> {
    let parent_1 = chromosomes.get(*key)...;
    let parent_2 = chromosomes.get(*value)...;
    // ...crossover dispatch match on effective_method variant...
}
```

**New pattern:**
```rust
let process_group = |group: &Vec<usize>| -> Result<Vec<U>, GaError> {
    // group[0] is always parent_1; group[1] is parent_2 for 2-parent path
    // For N-parent path: collect all group[i] refs
    let parents: Vec<&U> = group.iter().map(|&idx| &chromosomes[idx]).collect();
    let children = if group.len() == 2 {
        crossover::factory(parents[0], parents[1], cx_config)?
    } else {
        crossover::factory_multi_parent_dispatch(&parents, cx_config)?
    };
    // mutation + fitness unchanged
};
```

This eliminates the existing match block for Undx/Spx/Pcx variants (lines 2674-2697 of ga.rs). [ASSUMED — derived from D-04 and existing code structure]

### Pattern 3: Per-Variant Mutation Params

**What:** The `Mutation` enum carries its own parameters. The `MutationOperator::mutate` trait reads params from the passed `&Mutation` reference.

**Current pattern:**
```rust
// Source: src/operations/mutation.rs:232
impl MutationOperator for Mutation {
    fn mutate<U>(&self, individual: &mut U, step: Option<f64>, sigma: Option<f64>) -> Result<(), GaError> {
        match self {
            Mutation::Creep => { let s = step.unwrap_or(1.0); individual.creep_mutate(s); }
            Mutation::Gaussian => { let s = sigma.unwrap_or(1.0); individual.gaussian_mutate(s); }
            // ...
        }
    }
}
```

**New pattern:**
```rust
// Source: Derived from D-06/D-07 locked decisions
impl MutationOperator for Mutation {
    fn mutate<U>(&self, individual: &mut U, mutation: &Mutation) -> Result<(), GaError>
    where U: LinearChromosome + ValueMutable + 'static
    {
        match mutation {
            Mutation::Creep { step } => {
                let s = step.unwrap_or(0.01);
                individual.creep_mutate(s);
            }
            Mutation::Gaussian { sigma } => {
                let s = sigma.unwrap_or(0.1);
                individual.gaussian_mutate(s);
            }
            Mutation::Cauchy { scale } => {
                let scale = scale.unwrap_or(1.0);
                return try_cauchy(individual, scale).unwrap_or_else(|| Err(...));
            }
            // ...unit variants unchanged...
            Mutation::Swap => swap(individual),
        }
        Ok(())
    }
}
```

**GA loop call simplification:**
```rust
// Before: long if/else chain with config field reads (lines 2716-2776 of ga.rs)
// After: single call
mutation_method.mutate(&mut child_1, &mutation_method)?;
```

Note: `mutation_method` is both `&self` and the `mutation` argument here. This is intentional — `self` provides the dispatch key and `mutation` is the source of params. Since they are the same value, in the impl you can match on either. [ASSUMED]

### Pattern 4: `Mutation::Differential` Special Case

`Mutation::Differential` requires population context (the full chromosome slice + target index). It cannot be dispatched through the standard trait signature. The existing early-return pattern should be preserved:

```rust
// Before: checked as first branch in if/else chain
if mutation_method == Mutation::Differential { ... }

// After: still handled before the trait call
if matches!(mutation_method, Mutation::Differential { .. }) {
    // Differential stays special-case: needs population context
    differential::differential_mutation(&mut child_1, chromosomes, idx, f)?;
} else {
    mutation_method.mutate(&mut child_1, &mutation_method)?;
}
```

`Mutation::Differential` is a unit variant (no inline params) per D-06. Its `f` parameter moves OUT of `MutationConfiguration` and INTO the enum variant as `Mutation::Differential { f: Option<f64> }`. The GA loop reads `f` from the variant instead of from `configuration.mutation_configuration.differential_f`. [ASSUMED — consistent with D-06 pattern; `Differential` is listed as a unit variant in D-06 but has a param]

**RISK NOTE:** The CONTEXT.md D-06 lists `Mutation::Differential` as a unit variant with "no params needed." However, `differential_f` is currently in `MutationConfiguration` (line 207 of configuration.rs) and D-08 says all operator-specific fields are removed. If `Differential` remains unit, `differential_f` has nowhere to live. **This is an ambiguity the planner must resolve.** Either:
- Option A: `Mutation::Differential { f: Option<f64> }` — consistent with D-08 (all params move to variants)
- Option B: Keep `differential_f` in `MutationConfiguration` as a special case — inconsistent with D-08

Option A is the safer interpretation given the stated goal.

### Pattern 5: AOS Portfolio Copy → Clone

`portfolio[op_idx]` at line 2619 of `ga.rs` returns a `Mutation` copy today. After removing `Copy`, it must become `portfolio[op_idx].clone()`. The AOS `mutation_portfolio` field is `Option<Vec<Mutation>>` — the `Vec` contains owned values, so indexing still works with `.clone()`.

The checkpoint restore at line 1425 (`let builder_mutation = self.configuration.mutation_configuration.method`) copies a `Mutation`. After this change: `let builder_mutation = self.configuration.mutation_configuration.method.clone()`.

`GaConfiguration` is currently `Clone` but NOT `Copy` (it contains `Vec<Mutation>` and `Vec<Crossover>` portfolios and a `String` in `SaveProgressConfiguration`). The removal of `Copy` from `Mutation` doesn't change `GaConfiguration`'s derive status. [VERIFIED: src/configuration.rs line 338 — no `#[derive(Copy)]` on GaConfiguration]

### Anti-Patterns to Avoid

- **Changing internal selection function signatures one-by-one without a plan:** All 8 internal selection functions (random, roulette_wheel, stochastic_universal_sampling, tournament, rank, boltzmann, truncation, clearing, lexicase, epsilon_lexicase) return `Vec<(usize, usize)>` today. They all need updating. Changing one at a time without a migration plan will cause partial compile failures. Change all in the same wave.
- **Forgetting `factory_lexicase` return type:** `factory_lexicase` in `selection.rs` (line 148) also returns `Vec<(usize, usize)>`. It must be updated to `Vec<Vec<usize>>` even though lexicase never generates N-ary groups (groups of 2 always).
- **Forgetting the cellular engine:** `src/engines/cellular/engine.rs` line 173 calls `self.config.selection.select(&local, 1, 1)` through the trait — it uses the `SelectionOperator` trait directly, not `factory`. This must receive the new `num_parents` argument.
- **Using `.copy()` implicitly:** Rust's `Copy` trait is implicit — search for every place that reads a `Mutation` value out of a struct field or collection without `&`. The compiler will find them all after removing `Copy`, but plan for these locations: (1) `portfolio[op_idx]` in ga.rs:2619, (2) `mutation_configuration.method` in ga.rs:1425, (3) anywhere `Mutation` is pattern-matched by value rather than by reference.
- **Forgetting `non_uniform_b`:** `Mutation::NonUniform` currently has `non_uniform_b` in `MutationConfiguration` but `NonUniform` is listed as a unit variant in D-06. However, `non_uniform` mutation needs its `b` parameter at runtime. Resolution: either `Mutation::NonUniform { b: Option<f64> }` (consistent with D-06 spirit) or keep `non_uniform_b` in config (inconsistent with D-08). The planner should default to adding `b` to the enum variant for consistency.
- **Forgetting the serde checkpoint impact:** When `MutationConfiguration` fields are removed, old checkpoint JSON files with those fields will silently ignore them (no `deny_unknown_fields` in current code). New checkpoint files won't have them. This is safe for forward compatibility but users resuming old checkpoints lose their configured params — document in MIGRATION.md.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Backward-compatible serde for changed enum variants | Custom serializer | `#[serde(default)]` on new fields | Serde's built-in default handles None for optional fields |
| Finding all Copy-of-Mutation sites | Manual grep | `cargo check` after removing `Copy` from derive | Compiler emits errors at every move-without-clone site |
| Propagating trait signature changes | Manual update per call site | `cargo check` driven iteration | Compiler exhaustively finds all impl sites and callers |

**Key insight:** The Rust compiler is the discovery tool for this phase. Remove `Copy` from `Mutation` first (in isolation), run `cargo check`, and fix every error. Then change the trait signatures, run `cargo check` again. The compiler will exhaustively locate every affected call site.

---

## Runtime State Inventory

Not applicable. This is a code-only refactoring phase. No stored data, live service config, OS-registered state, secrets, or build artifacts are renamed or affected.

---

## Common Pitfalls

### Pitfall 1: N-ary Groups Never Smaller Than 2

**What goes wrong:** A selection function returns a `Vec<Vec<usize>>` where some inner `Vec` has 0 or 1 elements.

**Why it happens:** The internal selection loop might emit fewer candidates if the population is small or the RNG hits degenerate cases.

**How to avoid:** Validate `group.len() >= 2` before dispatching crossover. The existing population-size guard in `factory` (line 91: "population size too small for selection") should remain.

**Warning signs:** `Index out of bounds` panic when accessing `group[0]` or `group[1]` in `parent_crossover`.

### Pitfall 2: `Mutation::NonUniform` Loses Generation Context

**What goes wrong:** The `non_uniform_mutation` function requires `generation` and `max_generations` at call time. If `NonUniform` becomes a unit variant, the dispatch through `MutationOperator::mutate` cannot pass generation context.

**Why it happens:** `NonUniform` is listed as a unit variant in D-06 ("no params needed, remain unit variants"). But the underlying function needs generation info.

**How to avoid:** Keep the existing special-case path for `NonUniform` in the GA loop (similar to `Differential`). The GA loop already has generation context; the trait call is bypassed for these special variants. Document this as a known exception.

**Warning signs:** `Mutation::NonUniform requires generation context (generation, max_generations)` error at runtime if dispatched through the trait.

### Pitfall 3: `GaConfiguration` Is `Clone`-Derived, Not Manually Implemented

**What goes wrong:** `GaConfiguration` clones in checkpoint restore (line 1435: `self.configuration = ckpt.configuration`). If `MutationConfiguration` loses `Copy` but `GaConfiguration` is still `Clone`-derived, the clone works fine.

**Why it happens:** `MutationConfiguration` is `#[derive(Copy, Clone)]` today. Removing `Copy` from `Mutation` means `MutationConfiguration` can no longer derive `Copy` (since one of its fields — `method: Mutation` — is no longer `Copy`). This cascades to `GaConfiguration`.

**How to avoid:** Remove `Copy` from `MutationConfiguration`'s derive list explicitly. Also check `GaConfiguration`'s derive — it does NOT have `#[derive(Copy)]` currently, so it's unaffected. `SelectionConfiguration` has `#[derive(Copy)]` and its `method: Selection` field is still `Copy` (Selection remains Copy). Nothing to fix there.

**Warning signs:** Compiler error `the trait Copy is not implemented for Mutation` cascading from `MutationConfiguration`.

### Pitfall 4: Island Engine Loops `for &(idx_a, idx_b) in &parent_pairs`

**What goes wrong:** After `selection::factory` returns `Vec<Vec<usize>>`, the island engine loop at `mod.rs:547` that destructures `&(idx_a, idx_b)` will fail to compile.

**Why it happens:** The island engine calls `selection::factory` and iterates pairs directly — it does not go through `parent_crossover`.

**How to avoid:** Update the island engine loop to: `for group in &parent_pairs { let idx_a = group[0]; let idx_b = group[1]; ... }`. The island engine only uses 2-parent crossover currently, so `group[0]` and `group[1]` are always valid.

**Warning signs:** Compile error in `src/engines/island/mod.rs:547`.

### Pitfall 5: GP Engine `for (i, j) in &pairs`

**What goes wrong:** Same as Pitfall 4 — GP engine iterates pairs by destructuring.

**Why it happens:** `src/engines/gp/engine.rs:285` uses `for (i, j) in &pairs`.

**How to avoid:** Update to `for group in &pairs { let (i, j) = (group[0], group[1]); ... }`.

**Warning signs:** Compile error in `src/engines/gp/engine.rs:285`.

### Pitfall 6: `Mutation::Deletion` / `Mutation::Insertion` Still Need `ChromosomeLength`

**What goes wrong:** These variants have no inline params (D-06), but the existing `factory_with_chromosome_length` function passes `ChromosomeLength` to them. After the trait signature change, the trait no longer receives a chromosome length.

**Why it happens:** ChromosomeLength comes from `LimitConfiguration`, not from the `Mutation` enum. The trait has no access to configuration.

**How to avoid:** Keep the special-case dispatch for `Insertion` and `Deletion` in the GA loop (before the trait call), just as `Differential` and `NonUniform` are special-cased. The trait's `mutate` implementation for these variants should return a `MutationError` (as it does now) — the GA loop bypasses the trait for these. Alternatively, add `chromosome_length: Option<ChromosomeLength>` to the enum variant — but D-06 explicitly says no inline params for these. The planner should confirm the special-case dispatch approach.

**Warning signs:** `Mutation::Insertion requires ChromosomeLength::Variable configuration` error at runtime if the trait path is used.

### Pitfall 7: Builder Methods for Removed Fields Must Be Removed

**What goes wrong:** `MutationConfig` trait in `src/traits/configuration.rs` exposes `with_mutation_step`, `with_mutation_sigma`, `with_cauchy_scale`, `with_levy_alpha`, `with_polynomial_eta`, `with_self_adaptive_tau`, etc. After D-08, these fields no longer exist in `MutationConfiguration`. The builder methods must be removed or replaced.

**Why it happens:** Builder methods delegate to configuration struct field assignments. Removing the fields breaks the builders.

**How to avoid:** Remove the builder methods from the trait and their implementations in `configuration.rs`. Users who used these methods must migrate to constructing parameterized variants inline: `Mutation::Gaussian { sigma: Some(0.05) }` instead of `.with_mutation_sigma(0.05)`. Document in MIGRATION.md.

**Warning signs:** Compile error `no field sigma on type MutationConfiguration`.

---

## Code Examples

### Before/After: Selection Factory

```rust
// BEFORE (src/operations/selection.rs:83)
pub fn factory<U>(
    chromosomes: &[U],
    configuration: SelectionConfiguration,
    number_of_threads: usize,
) -> Result<Vec<(usize, usize)>, GaError>

// AFTER
pub fn factory<U>(
    chromosomes: &[U],
    configuration: SelectionConfiguration,
    number_of_threads: usize,
    num_parents: usize,  // 2 for standard; N for multi-parent
) -> Result<Vec<Vec<usize>>, GaError>
```

The `num_parents` value is derived by the GA loop before calling `factory`:
```rust
// In ga.rs, before calling selection::factory:
let num_parents = match self.configuration.crossover_configuration.method {
    Crossover::Undx { num_parents }
    | Crossover::Spx { num_parents }
    | Crossover::Pcx { num_parents } => num_parents,
    _ => 2,
};
```

### Before/After: Mutation Enum

```rust
// BEFORE (src/operations.rs:199-289)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mutation {
    Gaussian,
    Creep,
    Cauchy,
    // ...unit variants...
}

// AFTER
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mutation {
    Gaussian { sigma: Option<f64> },
    Creep { step: Option<f64> },
    Polynomial { eta: Option<f64> },
    Cauchy { scale: Option<f64> },
    LevyFlight { alpha: Option<f64> },
    SelfAdaptiveGaussian {
        tau: Option<f64>,
        tau_prime: Option<f64>,
        sigma_min: Option<f64>,
        sigma_max: Option<f64>,
    },
    // Unit variants remain unchanged:
    Swap, Inversion, Scramble, Value, BitFlip, NonUniform,
    PermutationInsert, Insertion, Deletion, ListValue, Differential, Uniform,
}
```

### Before/After: MutationConfiguration

```rust
// BEFORE: 14 fields including step, sigma, polynomial_eta, cauchy_scale, etc.

// AFTER: operator-agnostic fields only
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MutationConfiguration {
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Mutation,
    pub dynamic_mutation: bool,
    pub target_cardinality: Option<f64>,
    pub probability_step: Option<f64>,
    // REMOVED: step, sigma, polynomial_eta, non_uniform_b, differential_f,
    //          cauchy_scale, levy_alpha, self_adaptive_tau, self_adaptive_tau_prime,
    //          sigma_min, sigma_max
}
```

Note: `MutationConfiguration` can no longer derive `Copy` because `method: Mutation` is no longer `Copy`.

### Before/After: MutationOperator Trait Call in GA Loop

```rust
// BEFORE (ga.rs:2716-2776): ~60-line if/else chain
if mutation_method == Mutation::Differential { ... }
else if mutation_method == Mutation::Cauchy {
    mutation::factory_with_params(mutation_method, &mut child_1,
        configuration.mutation_configuration.cauchy_scale, None)?;
}
// ... 7 more branches ...

// AFTER: special cases only, then single trait call
match &mutation_method {
    Mutation::Differential { f } => {
        let f_val = f.unwrap_or(0.5);
        differential::differential_mutation(&mut child_1, chromosomes, idx_a, f_val)?;
    }
    Mutation::NonUniform { .. } => {
        non_uniform::non_uniform_mutation(&mut child_1, generation, max_generations)?;
    }
    Mutation::Insertion | Mutation::Deletion => {
        mutation::factory_with_chromosome_length(
            mutation_method.clone(), &mut child_1,
            Some(configuration.limit_configuration.chromosome_length), None, None)?;
    }
    _ => {
        mutation_method.mutate(&mut child_1, &mutation_method)?;
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Global step/sigma in config struct | Params in enum variants | Phase 54 (this phase) | Breaking — migration required |
| Tuple-of-two for parent pairs | `Vec<usize>` groups | Phase 54 (this phase) | Breaking — migration required |
| `Mutation: Copy` | `Mutation: Clone` | Phase 54 (this phase) | Breaking — clone must be explicit |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Mutation::Differential` should become `Mutation::Differential { f: Option<f64> }` to enable D-08 field removal | Code Examples, Anti-Patterns | If kept unit, `differential_f` must stay in config (inconsistent with D-08) |
| A2 | `Mutation::NonUniform` should become `Mutation::NonUniform { b: Option<f64> }` for D-08 consistency | Common Pitfalls | If kept unit, `non_uniform_b` must stay in config |
| A3 | `Mutation::Insertion`/`Deletion` remain special-case in GA loop (not dispatched through trait) | Code Examples | If dispatched through trait, trait needs `ChromosomeLength` context — requires more invasive trait change |
| A4 | `num_parents` for `factory` is extracted by the GA loop from the crossover method variant, not stored in `CrossoverConfiguration` | Standard Stack, Code Examples | `CrossoverConfiguration` does not have a `num_parents` field; it lives in the enum variant (verified in `src/operations.rs:173`) |
| A5 | `factory_lexicase` return type changes to `Vec<Vec<usize>>` even though lexicase always produces groups of 2 | Architecture Patterns | If left as `Vec<(usize,usize)>`, the call site in `select_parents_lexicase` needs separate handling |
| A6 | Default for `Mutation::Gaussian { sigma }` when `None` is `0.1` (not `1.0` as current code uses) | Code Examples | Current code uses `1.0` as default sigma — D-06 says "None → default 0.1". Planner should confirm the intended default. |
| A7 | Default for `Mutation::Creep { step }` when `None` is `0.01` (not `1.0` as current code uses) | Code Examples | Current code uses `1.0` as default step. D-06 says "None → default 0.01". |

**If this table is empty:** All claims in this research were verified or cited — no user confirmation needed.

The planner should confirm A1, A2, A3 with the user via the plan if ambiguous, or make a decision and document it clearly.

---

## Open Questions (RESOLVED)

1. **`Mutation::Differential` and `Mutation::NonUniform` inline params**
   - What we know: D-06 lists them as unit variants ("no params needed"). D-08 removes `differential_f` and `non_uniform_b` from `MutationConfiguration`.
   - What's unclear: If they stay unit variants, where do their parameters live?
   - Recommendation: Treat them as `{ f: Option<f64> }` and `{ b: Option<f64> }` respectively, consistent with the spirit of D-06/D-08. If the user wants them truly unit with hardcoded defaults, that's also valid but should be stated.

2. **Default param values for parameterized variants**
   - What we know: D-06 says `Mutation::Gaussian { sigma: Option<f64> }` with "None → default 0.1" and `Mutation::Creep { step: Option<f64> }` with "None → default 0.01".
   - What's unclear: Current code defaults both to `1.0`. Whether 0.1/0.01 are intentional behavior changes or documentation errors.
   - Recommendation: Use the CONTEXT.md values (0.1 for Gaussian, 0.01 for Creep) and document the change in MIGRATION.md.

3. **`factory_with_params` and `factory_self_adaptive` retention**
   - What we know: These functions currently exist as convenience wrappers. After the trait change, they become redundant.
   - What's unclear: Should they be kept for external crate users who call them directly, or removed?
   - Recommendation: Mark as `#[deprecated]` in Phase 54, remove in Phase 65 (MIGRATION.md phase). This avoids a double breaking change.

---

## Environment Availability

Step 2.6: SKIPPED — this phase is code/trait-refactoring with no external tool dependencies beyond the existing Rust toolchain.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + criterion (benchmarks) |
| Config file | `Cargo.toml` (test targets auto-discovered) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| N-ary-SEL-01 | `selection::factory` returns `Vec<Vec<usize>>` of correct length | unit | `cargo test --test operations test_selection` | Yes (tests/operations/test_selection.rs) |
| N-ary-SEL-02 | Standard 2-parent selection produces groups of exactly 2 indices | unit | `cargo test --test operations test_selection` | Yes |
| N-ary-SEL-03 | Multi-parent selection (N=3) produces groups of exactly 3 indices | unit | `cargo test --test test_multi_parent_integration` | Yes (tests/test_multi_parent_integration.rs) |
| N-ary-SEL-04 | Island engine compiles and runs with new return type | integration | `cargo test --test engines` | Partial (tests/engines/) |
| N-ary-SEL-05 | GP engine compiles and runs with new return type | integration | `cargo test --test gp` | Yes (tests/gp.rs) |
| MUT-PARAM-01 | `Mutation::Gaussian { sigma: Some(0.05) }` applies sigma=0.05 | unit | `cargo test --test operations test_mutation_creep_gaussian` | Yes |
| MUT-PARAM-02 | `Mutation::Creep { step: None }` applies default step | unit | `cargo test --test operations test_mutation_creep_gaussian` | Yes |
| MUT-PARAM-03 | `Mutation::Cauchy { scale: Some(2.0) }` applies scale=2.0 | unit | `cargo test --test operations test_mutation_cauchy_levy_uniform` | Yes |
| MUT-PARAM-04 | `Mutation::SelfAdaptiveGaussian { tau: None, .. }` uses ES defaults | unit | `cargo test --test operations test_mutation_self_adaptive` | Yes |
| MUT-PARAM-05 | `MutationConfiguration` no longer has `step`/`sigma` fields | compile-time | `cargo check` | Wave 0 gap |
| MUT-PARAM-06 | AOS portfolio `Vec<Mutation>` works with non-Copy Mutation | unit | `cargo test` (existing AOS tests) | Partial |
| MUT-PARAM-07 | Checkpoint round-trip works with new `MutationConfiguration` | integration | `cargo test --features serde --test observe test_serde` | Yes (tests/observe/test_serde.rs) |

### Sampling Rate

- **Per task commit:** `cargo check`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] No test currently validates that `MutationConfiguration` lacks `step`/`sigma` fields — this is a compile-time guarantee; Wave 0 should add a compile-fail test or simply rely on `cargo check` as part of CI.
- [ ] No explicit test for N=3 selection group size from `selection::factory` with a multi-parent crossover config — needs a new test in `tests/operations/test_selection.rs`.

---

## Security Domain

Not applicable. This phase changes internal trait signatures and enum representations. No authentication, session management, access control, input validation from external sources, or cryptography is involved.

---

## Sources

### Primary (HIGH confidence)

- `src/traits/operators.rs` — Current `SelectionOperator::select` signature (3 params, returns `Vec<(usize, usize)>`), `MutationOperator::mutate` signature (step + sigma params) [VERIFIED: read directly]
- `src/operations/selection.rs` — `factory`, `factory_lexicase` current signatures and return types [VERIFIED: read directly]
- `src/operations/mutation.rs` — `impl MutationOperator for Mutation`, `factory_with_params`, `factory_self_adaptive`, `factory_with_chromosome_length` function signatures [VERIFIED: read directly]
- `src/operations.rs` — Current `Mutation` enum variants and derive attributes (`Copy, Clone`) [VERIFIED: read directly]
- `src/configuration.rs` — `MutationConfiguration` struct (14 fields), `CrossoverConfiguration` struct (no `num_parents` field), `GaConfiguration` struct (not Copy) [VERIFIED: read directly]
- `src/engines/ga.rs:2513-2885` — `parent_crossover` function: current params `&[(usize, usize)]`, mutation if/else dispatch block, AOS `portfolio[op_idx]` copy site, checkpoint method save/restore [VERIFIED: read directly]
- `src/engines/island/mod.rs:539-547` — `for &(idx_a, idx_b) in &parent_pairs` iteration [VERIFIED: read directly]
- `src/engines/gp/engine.rs:262-285` — `selection::factory` call + `for (i, j) in &pairs` iteration [VERIFIED: read directly]
- `src/engines/cellular/engine.rs:173` — `.select(&local, 1, 1)` direct trait call [VERIFIED: read directly]
- `.planning/phases/54-n-ary-selection-per-operator-mutation-params/54-CONTEXT.md` — All locked decisions D-01 through D-09 [VERIFIED: read directly]

### Secondary (MEDIUM confidence)

- `tests/operations/` directory listing — 40+ test files covering all operators; confirmed which tests will need updates for mutation param changes [VERIFIED: read directly]
- `examples/` — 5 examples use parameterized mutation variants (`Gaussian`, `Cauchy`); confirmed builder method usage in `memetic_rastrigin.rs` [VERIFIED: read directly]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new packages; all changes are to existing Rust code verified in-session
- Architecture: HIGH — all key files read; call sites identified and confirmed
- Pitfalls: HIGH — derived from direct inspection of the affected code paths

**Research date:** 2026-05-28
**Valid until:** 2026-06-28 (stable codebase; only external changes would be upstream Rust edition changes, irrelevant here)
