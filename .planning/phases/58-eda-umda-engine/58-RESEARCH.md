# Phase 58: EDA / UMDA Engine - Research

**Researched:** 2026-06-04
**Domain:** Estimation of Distribution Algorithms — UMDA variant, Rust engine pattern
**Confidence:** HIGH

## Summary

Phase 58 implements `EdaEngine<U>` — a Univariate Marginal Distribution Algorithm (UMDA) engine — as a new module under `src/engines/eda/`. The engine replaces crossover and mutation with a probabilistic model learned from the fittest parents each generation and then sampled to produce a new population.

All design decisions are locked in `58-CONTEXT.md`. The UMDA algorithm is well-established in the EDA literature; the implementation challenge is Rust-specific: dispatching between Bernoulli and Gaussian model estimation at compile time based on the `U::Gene: RealGene` bound, and sampling new DNA via `gene.with_real_value(v)` (Gaussian path) or constructing `Binary` genes with `id = rng.random() < p_i ? 1 : 0` (Bernoulli path).

The engine pattern to follow is `src/engines/pso/` (most recent, canonical reference). The PSO engine has already solved all structural problems this phase will face: `FitnessFn<U::Gene>` type alias, `Arc<dyn GaObserver<U>>` wiring with the `notify()` helper, `TerminationCause` import from `crate::ga`, `GenerationStats::from_fitness_values()` call, and the `#[cfg(not(target_arch = "wasm32"))]` gate pattern.

**Primary recommendation:** Clone the PSO engine structure verbatim for file layout and observer wiring; replace the update loop with the UMDA model estimation + sampling loop; dispatch between Bernoulli and Gaussian via two `impl` blocks gated on the `U::Gene: RealGene` bound.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `EdaEngine<U>` accepts any `U: LinearChromosome`. Model strategy selected at compile time:
  - `U::Gene: RealGene` → Gaussian univariate (`mean_i`, `std_i` per gene position)
  - Otherwise → Bernoulli UMDA: `p_i = count(gene_i.id() == 1) / num_parents`
  - Compile-time dispatch via two `impl` blocks or a helper trait — planner decides mechanism

- **D-02:** `selection_ratio: f64` (default 0.5) in `EdaConfiguration`. Top `floor(pop_size * selection_ratio)` by fitness feed model estimation. Minimum 1 parent (clamp). `ProblemSolving` controls sort direction.

- **D-03:** `EdaResult<U>` contains: `population: Vec<U>`, `best: U`, `best_fitness: f64`, `generations: usize`, `learned_model: EdaModel`

- **D-04:**
  ```rust
  pub enum EdaModel {
      Bernoulli(Vec<f64>),
      Gaussian { means: Vec<f64>, stds: Vec<f64> },
  }
  ```

- **D-05:** `Option<Arc<dyn GaObserver<U> + Send + Sync>>` with `with_observer()` builder. 5 hooks: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end`.

- **D-06:** Example is `eda_trap` — deceptive trap function on Binary chromosome.

### Claude's Discretion

- Probability clamping for Bernoulli model (suggested `[0.01, 0.99]`)
- Std deviation floor for Gaussian model
- Whether `EdaConfiguration` exposes `max_generations`, `fitness_target`, `population_size`, `problem_solving` as direct fields (mirrors CmaConfiguration — likely yes)
- Whether `Gaussian` variant is gated on `U::Gene: RealGene` at the type level or via internal helper
- `GenerationStats` field population — use existing fields or add model-diversity proxy
- Default `population_size` if 0 is passed (suggest `100`)
- Whether `EdaModel` derives `Debug`, `Clone`, `serde::Serialize` (if `serde` feature enabled)

### Deferred Ideas (OUT OF SCOPE)

- Multivariate EDA (BMDA, MIMIC, BOA)
- Population-Based Incremental Learning (PBIL)
- Adaptive selection_ratio
- Discrete PSO
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Population initialization | `EdaEngine` | user-provided `init_fn` | Engine calls `init_fn(pop_size)`, same pattern as PSO/CMA |
| Fitness evaluation | `EdaEngine` | user-provided `fitness_fn` | Engine calls fitness_fn per offspring; optionally parallel (rayon-gated) |
| Parent selection (selection_ratio) | `EdaEngine` | — | Sort population, slice top fraction — engine-internal, no operator dispatch |
| Model estimation (Bernoulli/Gaussian) | `EdaEngine` | trait bound dispatch | Compile-time via `impl` block with `U::Gene: RealGene` bound |
| Offspring sampling | `EdaEngine` | `crate::rng::make_rng()` | Engine samples from model; uses `gene.with_real_value()` for Gaussian, constructs `Binary` genes for Bernoulli |
| Observer hooks | `EdaEngine` | `GaObserver<U>` trait | 5 standard hooks, same wiring as PSO engine |
| Result packaging | `EdaResult<U>` + `EdaModel` | — | Struct + enum in `engine.rs` |
| Module registration | `src/engines/mod.rs` + `src/lib.rs` | — | Follow PSO #[path] re-export pattern |

---

## Standard Stack

### Core (no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` | existing | Uniform sampling for Bernoulli, Box-Muller for Gaussian | Already in Cargo.toml; `SmallRng` is WASM-safe |
| `crate::rng::make_rng()` | internal | Seedable RNG with reproducibility | Standard pattern across all engines |
| `crate::stats::GenerationStats` | internal | Per-generation stats for observer | Reused verbatim from PSO/CMA |
| `crate::ga::TerminationCause` | internal | Run termination reason enum | Same import path as PSO/CMA engines |
| `crate::configuration::ProblemSolving` | internal | Maximize/Minimize/FixedFitness | Same as all other engines |

**No new external crates required.** [VERIFIED: codebase grep] The PSO and CMA engines show the complete set of imports needed; EDA is a simpler algorithm than either and uses the same subset.

### Algorithms to Hand-Implement (UMDA core)

| Operation | Implementation Note |
|-----------|-------------------|
| Bernoulli estimation | `p_i = selected.iter().map(|c| c.dna()[i].id() as f64).sum() / n_selected` — uses `gene.id()` as binary indicator (0 or 1) |
| Bernoulli clamping | `p_i = p_i.clamp(0.01, 0.99)` per position |
| Bernoulli sampling | `rng.random::<f64>() < p_i` → `id = 1`, `value = true`; else `id = 0`, `value = false` |
| Gaussian estimation | `mean_i = selected.iter().map(|c| c.dna()[i].real_value()).sum() / n`; `std_i = sqrt(variance + floor)` |
| Gaussian std floor | `std_i = std_i.max(1e-6)` to prevent degenerate sampling |
| Gaussian sampling | Box-Muller: `z = sqrt(-2 ln u1) * cos(2π u2)`; `v = mean_i + std_i * z`; clamp to gene bounds if available |
| Parent sort | `pop.sort_unstable_by(|a, b| ...)` with `is_maximization` flag |

**Installation:** No new packages. All algorithm logic is pure Rust math.

---

## Package Legitimacy Audit

No new external packages are introduced in this phase. All dependencies are from the existing `Cargo.toml`.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User
  |
  |  EdaEngine::new(config, init_fn, fitness_fn)
  |  .with_observer(obs)
  |  .run()
  v
┌─────────────────────────────────────────────────────────────┐
│  EdaEngine<U: LinearChromosome>                             │
│                                                             │
│  on_run_start ──────────────────────────────────► Observer  │
│                                                             │
│  init_fn(pop_size) ─────────────────────────────► Vec<U>    │
│  fitness_fn(dna) ───── [for each individual] ──► f64        │
│                                                             │
│  identify best ─────────────────────────────────► best: U   │
│  on_new_best ───────────────────────────────────► Observer  │
│                                                             │
│  ┌──── Generation Loop (0..max_generations) ──────────────┐ │
│  │                                                        │ │
│  │  on_generation_start ──────────────────────► Observer  │ │
│  │                                                        │ │
│  │  Sort pop by fitness                                   │ │
│  │  Select top floor(pop_size * selection_ratio) parents  │ │
│  │                                                        │ │
│  │  Estimate model:                                       │ │
│  │    if U::Gene: RealGene → Gaussian(mean_i, std_i)      │ │
│  │    else                 → Bernoulli(p_i)               │ │
│  │                                                        │ │
│  │  Sample pop_size new individuals from model            │ │
│  │  [optionally parallel — rayon cfg gate]                │ │
│  │                                                        │ │
│  │  Evaluate fitness for new population                   │ │
│  │                                                        │ │
│  │  Update best if improved → on_new_best ──── Observer   │ │
│  │                                                        │ │
│  │  GenerationStats::from_fitness_values(...)             │ │
│  │  on_generation_end ────────────────────────► Observer  │ │
│  │                                                        │ │
│  │  Early stop if fitness_target reached                  │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                             │
│  on_run_end ────────────────────────────────────► Observer  │
│                                                             │
│  EdaResult { population, best, best_fitness,               │
│              generations, learned_model }                   │
└─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src/engines/eda/
├── mod.rs           # pub mod configuration; pub mod engine; pub use ...
├── configuration.rs # EdaConfiguration struct + builder methods
└── engine.rs        # EdaResult, EdaModel, EdaEngine<U>, run()
tests/engines/eda/
└── test_eda.rs      # All tests (project rule: tests never inline)
examples/
└── eda_trap.rs      # eda_trap deceptive trap function example
```

**lib.rs additions:**
```rust
#[path = "engines/eda/mod.rs"]
pub mod eda;
pub use eda::{EdaConfiguration, EdaEngine, EdaModel, EdaResult};
```

**engines/mod.rs is not a file** — engines are registered directly in `lib.rs` via `#[path]` re-exports (established pattern, confirmed by grep). [VERIFIED: codebase grep]

### Pattern 1: EdaEngine Struct (mirrors PsoEngine exactly)

```rust
// Source: src/engines/pso/engine.rs (canonical template)
pub struct EdaEngine<U: LinearChromosome> {
    config: EdaConfiguration,
    init_fn: Arc<dyn Fn(usize) -> Vec<U> + Send + Sync>,
    fitness_fn: Arc<FitnessFn<U::Gene>>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> EdaEngine<U> {
    pub fn new(
        config: EdaConfiguration,
        init_fn: impl Fn(usize) -> Vec<U> + Send + Sync + 'static,
        fitness_fn: impl Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    ) -> Self { ... }

    pub fn with_observer(mut self, obs: Arc<dyn GaObserver<U> + Send + Sync>) -> Self { ... }

    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer { f(obs.as_ref()); }
    }
}
```

### Pattern 2: Bernoulli Dispatch via Additional Bound

The Bernoulli path needs no `RealGene` — it works for any `LinearChromosome`. The Gaussian path requires `U::Gene: RealGene`. Two strategies are available:

**Option A — Two separate `impl` blocks (recommended for simplicity):**
```rust
// Bernoulli run() — no RealGene bound
impl<U: LinearChromosome + Clone> EdaEngine<U> {
    pub fn run(&mut self) -> EdaResult<U> {
        // Bernoulli estimation using gene.id() == 1
    }
}

// Gaussian run() — separate function or renamed method, with RealGene bound
impl<U: LinearChromosome + Clone> EdaEngine<U>
where U::Gene: RealGene
{
    pub fn run_gaussian(&mut self) -> EdaResult<U> {
        // Gaussian estimation using gene.real_value()
    }
}
```

**Option B — Single `run()` with runtime type dispatch via a private helper trait** (more ergonomic for users — recommended by CONTEXT.md):
```rust
// Private sealed helper
trait EdaModelStrategy<U: LinearChromosome> {
    fn estimate_and_sample(selected: &[U], pop_size: usize, rng: &mut SmallRng) -> (Vec<U>, EdaModel);
}
```

**Planner should choose Option B** — a single `run()` method is cleaner user API. Internal dispatch via a private sealed trait or a private `fn estimate_model` that has two compile-time specializations.

**Practical simplest approach:** One `run()` in the base `impl<U: LinearChromosome + Clone>` block that calls a private `fn sample_population` which is _also_ in the same impl block but only compiles when `U::Gene: RealGene` is not present — achieved by duplicating via a private helper trait with two blanket impls:

```rust
// Private sealed dispatch trait
trait UmdaModelBuilder<U: LinearChromosome>: Sized {
    fn estimate(selected: &[&U]) -> EdaModel;
    fn sample_individual(model: &EdaModel, template: &U, rng: &mut impl Rng) -> U;
}

// Blanket impl 1: Any LinearChromosome without RealGene → Bernoulli
impl<U: LinearChromosome + Clone> UmdaModelBuilder<U> for BernoulliDispatch { ... }

// Blanket impl 2: LinearChromosome where Gene: RealGene → Gaussian
impl<U: LinearChromosome + Clone> UmdaModelBuilder<U> for GaussianDispatch
where U::Gene: RealGene { ... }
```

**Note:** Rust doesn't support specialization on stable. The cleanest stable solution is a private `EdaBackend` trait with two implementations selected by the caller at the `EdaEngine` construction site. The planner should resolve this — the CONTEXT.md states "planner decides mechanism."

**Simplest stable Rust approach (recommended):** Make `EdaEngine` itself generic over the model strategy:
```rust
pub struct EdaEngine<U: LinearChromosome, S: EdaSampler<U> = BernoulliSampler> { ... }
```
…but this changes the user-facing API. Better: use a `Box<dyn EdaSampler>` internally set at construction, or just provide two constructor functions: `EdaEngine::bernoulli(...)` and `EdaEngine::gaussian(...)` with the appropriate inner sampler.

**Final recommendation:** Two factory-style constructors or two concrete engine types aliased via type aliases — but given the CONTEXT.md says "planner decides", the planner's PLAN.md should address this explicitly. Research conclusion: there is no trivial single-`run()` approach without a private dispatch trait or marker type.

### Pattern 3: Bernoulli Gene Construction

```rust
// Source: src/types/genotypes/binary.rs — Binary gene has .id and .value fields
// gene.id() returns i32; Bernoulli indicator is id == 1

// Estimation:
let p_i = selected.iter()
    .map(|c| if c.dna()[i].id() == 1 { 1.0 } else { 0.0 })
    .sum::<f64>() / n_selected as f64;
let p_i = p_i.clamp(0.01, 0.99);

// Sampling new gene at position i:
// Cannot use gene.id() == 1 directly for non-Binary genes.
// For Binary: construct BinaryGenotype { id: 1 or 0, value: true or false }
// EdaEngine is generic over U — sampling must go through U::Gene::new() or
// through cloning a template gene and mutating id.
// Safest approach: clone gene from template individual, then set id:
let mut new_gene = template.dna()[i].clone();
// For Bernoulli: only Binary genes will have id=1 as the Bernoulli indicator.
// The gene.id() contract says id uniquely identifies the gene TYPE, not value.
// ISSUE: id() is POSITIONAL, not a binary indicator in the general case.
```

**IMPORTANT FINDING:** `gene.id()` in the general `GeneT` trait is a positional identifier (set at init time as `i as i32`), not a value indicator. The Bernoulli model using `gene.id() == 1` is only valid specifically for `Binary` genes where `id` is explicitly set to 0 or 1 based on value during initialization (confirmed in `binary_random_initialization`: `id: i as i32`, NOT 0/1). [VERIFIED: codebase read of `src/initializers/binary_initializer.rs`]

**Critical finding:** In `binary_random_initialization`, the `id` is set to `i as i32` (positional index), NOT 0 or 1 based on value. The Bernoulli indicator in CONTEXT.md says "uses `gene.id() == 1` as the '1' indicator for binary genes." This only holds if `id` is 0 or 1 — which is NOT how the existing initializer sets ids for chromosome length > 1.

**Resolution:** For the Bernoulli path, the indicator should be the `Binary.value` boolean field (true=1) rather than `gene.id() == 1`. However, `GeneT` has no `value()` method — only `id()`. The correct approach is:
- Use `gene.id()` as the Bernoulli indicator only when the gene was constructed with id=0/1 (user responsibility for non-Binary types)
- Or: for Binary chromosomes specifically, cast through `BinaryGenotype` where `.value` is accessible
- Or: require that for Bernoulli path, `gene.id()` encodes the value (user contract)

**The CONTEXT.md statement** "Uses `gene.id() == 1` as the '1' indicator for binary genes" is the stated API contract. The planner must document this as a user contract: in Bernoulli mode, `gene.id()` MUST be 0 or 1 (the standard `Binary` gene satisfies this only if constructed with `id = value as i32`). The `eda_trap` example must construct Binary genes with `id = if value { 1 } else { 0 }`, not `id = position`.

**Recommended pattern for `eda_trap` init_fn:**
```rust
fn init_population(n: usize, length: usize) -> Vec<BinaryChromosome> {
    let mut rng = make_rng();
    (0..n).map(|_| {
        let dna: Vec<BinaryGenotype> = (0..length).map(|_| {
            let v = rng.random_bool(0.5);
            BinaryGenotype { id: if v { 1 } else { 0 }, value: v }
        }).collect();
        let mut c = BinaryChromosome::new();
        c.set_dna(Cow::Owned(dna));
        c
    }).collect()
}
```

### Pattern 4: Gaussian Sampling (Box-Muller, WASM-safe)

```rust
// Box-Muller: requires only rand::Rng — no std::f64::consts::PI WASM issue
use std::f64::consts::PI;
let u1: f64 = rng.random();
let u2: f64 = rng.random();
let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
let v = mean_i + std_i * z;
// Clamp to gene bounds if available:
let v = if let Some((lo, hi)) = gene_template.bounds() { v.clamp(lo, hi) } else { v };
let new_gene = gene_template.with_real_value(v);
```
`std::f64::consts::PI` is WASM-safe (compile-time constant, no `std::time`). [ASSUMED — no WASM-specific test for `f64::consts`; well-known it is safe]

### Pattern 5: EdaConfiguration (mirrors CmaConfiguration + PsoConfiguration)

```rust
// Source: src/engines/pso/configuration.rs
pub struct EdaConfiguration {
    pub population_size: usize,   // 0 → default 100 at run time
    pub max_generations: usize,
    pub problem_solving: ProblemSolving,
    pub fitness_target: Option<f64>,
    pub selection_ratio: f64,     // default 0.5; clamped to [1/pop_size, 1.0]
}

impl Default for EdaConfiguration {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 500,
            problem_solving: ProblemSolving::Maximization,
            fitness_target: None,
            selection_ratio: 0.5,
        }
    }
}
```

Builder methods: `with_population_size`, `with_max_generations`, `with_problem_solving`, `with_fitness_target`, `with_selection_ratio`.

### Anti-Patterns to Avoid

- **Using `gene.id()` as positional index for Bernoulli:** `id()` in existing built-in initializers stores the position, not the bit value. The Bernoulli model ONLY works when id is the binary value. The `eda_trap` example must initialize genes with `id = value as i32` pattern.
- **Sorting the full population in-place destructively:** Sort a clone or use `sort_unstable_by` on indices; keep original ordering for stats.
- **Using `par_iter` for model estimation:** Model estimation is O(n * L) sequential — do not parallelize. Fitness evaluation of offspring CAN use rayon (gate with `#[cfg(not(target_arch = "wasm32"))]`).
- **Calling `Instant::now()` unconditionally:** Gate all timing behind `#[cfg(not(target_arch = "wasm32"))]`. PSO and CMA engines have already solved this by simply not timing anything — EDA should follow the same approach (no timing = WASM safe by default).
- **`EdaModel::Gaussian` storing NaN stds:** Clamp `std_i` to floor `1e-6` before storing.
- **Sampling offspring with `init_fn` instead of model:** The EDA does NOT call `init_fn` each generation — only at initialization. Offspring come from model sampling.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| RNG / seeding | Custom RNG | `crate::rng::make_rng()` | Reproducibility across all engines |
| Per-generation stats | Custom stats struct | `GenerationStats::from_fitness_values()` | Already handles best/worst/avg/std/diversity |
| Observer wiring | Custom callback system | `GaObserver<U>` + `notify()` helper | Established contract, used by all engines |
| Module registration | Custom registry | `#[path]` re-export in `lib.rs` | Project-established pattern |
| Bernoulli sampling | Custom distribution | `rng.random::<f64>() < p_i` | One line, no dependency |
| Gaussian sampling | External `statrs` crate | Box-Muller inline | WASM-compatible, no dependency |

**Key insight:** EDA is algorithmically simpler than PSO and CMA-ES. The existing infrastructure (RNG, stats, observer, FitnessFn type alias) covers all needs — no new infrastructure is required.

---

## Common Pitfalls

### Pitfall 1: Binary gene `id()` encodes position, not bit value

**What goes wrong:** Bernoulli model estimates `p_i = count(gene.id() == 1) / n_selected` — but `binary_random_initialization` sets `id = i as i32` (0, 1, 2, 3...). Only gene at position 1 gets counted.
**Why it happens:** `GeneT::id()` is a positional identifier in all existing initializers.
**How to avoid:** The `eda_trap` example must construct genes with `id = if value { 1 } else { 0 }`. The engine's API contract must document this. Alternatively, for the Bernoulli path, use `BinaryGenotype.value as i32` — but this requires downcasting or a separate trait.
**Warning signs:** Bernoulli probabilities always converge to near-zero (because `id == 1` only for gene at index 1 in a long chromosome).

### Pitfall 2: Empty selected_parents slice

**What goes wrong:** If `pop_size * selection_ratio < 1.0`, `floor()` gives 0 parents — division by zero in estimation.
**How to avoid:** `let n_selected = (pop_size as f64 * selection_ratio).floor().max(1.0) as usize;` enforced before estimation.
**Warning signs:** Panic in model estimation step.

### Pitfall 3: Std deviation = 0 in Gaussian model (converged population)

**What goes wrong:** If all selected parents have identical gene values, `std_i = 0` → sampling produces only `mean_i` → population never escapes local optimum.
**How to avoid:** `std_i = std_i.max(1e-6)` floor before sampling.
**Warning signs:** Population converges in 5-10 generations and never improves.

### Pitfall 4: Sampling offspring with wrong DNA length

**What goes wrong:** Model vectors (`p_i` or `mean_i`) have length = DNA length of selected parents, but a new chromosome's DNA is set via `set_dna()` — if model length != expected chromosome length, results are silently wrong.
**How to avoid:** Assert/verify model length == `pop[0].dna().len()` before sampling loop. Use the first parent's DNA length to determine model length.
**Warning signs:** Offspring have unexpected fitness values or incorrect DNA lengths.

### Pitfall 5: Observer hook `on_new_best` fires with stale best

**What goes wrong:** `best` is cloned from population BEFORE new population is sampled, so `on_new_best` fires with the old chromosome even though a better one was found.
**How to avoid:** Follow PSO pattern exactly: update `best` and `best_fitness`, THEN call `on_new_best` with the updated clone.
**Warning signs:** Observer logs "new best" but fitness value matches previous generation.

### Pitfall 6: `EdaModel` returned in result captures initial (random) state

**What goes wrong:** `learned_model` in `EdaResult` should be the model estimated at the FINAL generation, not the initial population.
**How to avoid:** Update `last_model` at the end of each generation loop iteration, return it in `EdaResult`.
**Warning signs:** `result.learned_model` probabilities look random even after convergence.

---

## Code Examples

### Full UMDA Bernoulli Generation Step

```rust
// Source: CONTEXT.md §Code Context + codebase analysis
// Assumes: pop is sorted best-first, n_selected >= 1, dna_len >= 1

fn estimate_bernoulli(selected: &[&U], dna_len: usize) -> Vec<f64> {
    let n = selected.len() as f64;
    (0..dna_len)
        .map(|i| {
            let count: f64 = selected.iter()
                .map(|c| if c.dna()[i].id() == 1 { 1.0 } else { 0.0 })
                .sum();
            (count / n).clamp(0.01, 0.99)
        })
        .collect()
}

fn sample_bernoulli<U, R>(probs: &[f64], template: &U, rng: &mut R) -> U
where
    U: LinearChromosome + Clone,
    R: Rng,
{
    let new_dna: Vec<U::Gene> = probs.iter().enumerate().map(|(i, &p)| {
        let one = rng.random::<f64>() < p;
        // Clone template gene to preserve type, then set id to 0 or 1
        // This only works correctly when the gene type uses id as the value indicator
        // (i.e., Binary genes constructed with id = value as i32)
        let mut g = template.dna()[i].clone();
        // Note: GeneT requires set_id(&mut self, id: i32) -> &mut Self
        g.set_id(if one { 1 } else { 0 });
        g
    }).collect();
    let mut offspring = template.clone();
    offspring.set_dna(Cow::Owned(new_dna));
    offspring
}
```

### Full Gaussian Generation Step

```rust
// Source: src/traits/real_gene.rs (with_real_value pattern) + CONTEXT.md
fn estimate_gaussian<U>(selected: &[&U], dna_len: usize) -> (Vec<f64>, Vec<f64>)
where
    U: LinearChromosome,
    U::Gene: RealGene,
{
    let n = selected.len() as f64;
    let means: Vec<f64> = (0..dna_len).map(|i| {
        selected.iter().map(|c| c.dna()[i].real_value()).sum::<f64>() / n
    }).collect();
    let stds: Vec<f64> = means.iter().enumerate().map(|(i, &mean)| {
        let variance = selected.iter()
            .map(|c| { let d = c.dna()[i].real_value() - mean; d * d })
            .sum::<f64>() / n;
        variance.sqrt().max(1e-6)
    }).collect();
    (means, stds)
}

fn sample_gaussian<U, R>(means: &[f64], stds: &[f64], template: &U, rng: &mut R) -> U
where
    U: LinearChromosome + Clone,
    U::Gene: RealGene,
    R: Rng,
{
    use std::f64::consts::PI;
    let new_dna: Vec<U::Gene> = template.dna().iter().enumerate().map(|(i, g)| {
        let u1: f64 = rng.random::<f64>().max(1e-300); // avoid ln(0)
        let u2: f64 = rng.random();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        let v = means[i] + stds[i] * z;
        let v = if let Some((lo, hi)) = g.bounds() { v.clamp(lo, hi) } else { v };
        g.with_real_value(v)
    }).collect();
    let mut offspring = template.clone();
    offspring.set_dna(Cow::Owned(new_dna));
    offspring
}
```

### Observer Wiring (identical to PSO)

```rust
// Source: src/engines/pso/engine.rs lines 291, 322, 427-428, 431-434, 448-449
self.notify(|obs| obs.on_run_start());
// ... per generation:
self.notify(|obs| obs.on_generation_start(gen));
// ... when best improves:
let best_clone = best.clone();
self.notify(|obs| obs.on_new_best(gen, best_clone));
// ... end of generation:
self.notify(|obs| obs.on_generation_end(&stats));
// ... after loop:
self.notify(|obs| obs.on_run_end(termination_cause, all_stats_ref));
```

### lib.rs Registration Pattern

```rust
// Source: src/lib.rs lines 330-334, 366 (PSO registration pattern)
#[path = "engines/eda/mod.rs"]
pub mod eda;

pub use eda::{EdaConfiguration, EdaEngine, EdaModel, EdaResult};
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `DeGene` trait | `RealGene` trait | Phase 56 | Import path is `crate::traits::RealGene`, NOT `crate::traits::DeGene` |
| `src/observe/observer/mod.rs` | `crate::observer::GaObserver` | v2.4.0 | Import: `use crate::observer::GaObserver` (lib.rs #[path] alias) |
| `Reporter` trait | Removed v3.0.0 | Phase 47 | EDA should NOT use Reporter; use GaObserver |
| Engines registered in `src/engines/mod.rs` | Registered in `src/lib.rs` via `#[path]` | v2.3.0 | No `engines/mod.rs` file — confirmed by `find` output |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `std::f64::consts::PI` is WASM-safe (compile-time constant, no `std::time`) | Code Examples §Gaussian | Low — well-known Rust constant, but not explicitly tested |
| A2 | Box-Muller with `u1.max(1e-300)` guard sufficiently avoids `ln(0)` panic | Code Examples | Low — standard numerical guard, not codebase-specific |
| A3 | `BinaryGenotype.set_id(1)` (setting id to 1) correctly encodes "bit is 1" for Bernoulli path when `eda_trap` builds genes with `id = value as i32` | Pitfall 1, Code Examples | MEDIUM — this is the user contract the example must satisfy; wrong example construction breaks the demo |

---

## Open Questions (RESOLVED)

1. **Bernoulli dispatch mechanism** — **RESOLVED**
   - What we know: Rust stable has no specialization; two `impl` blocks with overlapping bounds don't compile; private helper trait works but adds boilerplate.
   - What's unclear: The planner must choose between (a) two named constructors `EdaEngine::bernoulli()`/`EdaEngine::gaussian()`, (b) a private sealed trait dispatch, or (c) a type parameter approach.
   - **Resolution (chosen by planner):** Two named constructors — `EdaEngine::bernoulli(config, init_fn, fitness_fn)` and `EdaEngine::gaussian(config, init_fn, fitness_fn)` — plus a `pub fn new(...)` alias that dispatches to the Bernoulli path (satisfies ROADMAP SC-1 literal API `EdaEngine::new(...).run()`). The Gaussian path is reached via a separately named `run_gaussian()` method on a `where U::Gene: RealGene` impl block to avoid duplicate-symbol conflict with the base `run()`. Users call `EdaEngine::bernoulli(...).run()` or `EdaEngine::new(...).run()` for Bernoulli; `EdaEngine::gaussian(...).run_gaussian()` for Gaussian. This is the cleanest stable-Rust solution and is documented in Plan 01 Task 1 and Plan 02 Task 1.

2. **Bernoulli indicator for non-Binary gene types** — **RESOLVED**
   - What we know: `gene.id()` is positional in all existing initializers; only Binary genes constructed with `id = value as i32` satisfy the Bernoulli contract.
   - What's unclear: Should `EdaEngine` document this as a user responsibility, or should it require a separate `BinaryGene` bound for the Bernoulli path?
   - **Resolution (chosen by planner):** Document the `gene.id() ∈ {0, 1}` requirement as a **user contract** in the engine's `///` doc comments (Plan 01 Task 1 action: doc comment on `bernoulli()` / `new()` constructors). The `eda_trap` example (Plan 03) demonstrates the correct `id = if value { 1 } else { 0 }` initialization pattern. No new trait bound is added — the existing `LinearChromosome` bound stays, with the value-encoding contract carried in documentation. Test helpers in Plan 01 Task 2 (`binary_init`) also demonstrate the pattern.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build + test | ✓ | Rust toolchain | — |
| `wasm32-unknown-unknown` target | WASM CI gate | assumed ✓ | — | `rustup target add wasm32-unknown-unknown` |
| `rand` crate | RNG in engine | ✓ (in Cargo.toml) | existing | — |

---

## Validation Architecture

`workflow.nyquist_validation` key is absent from `.planning/config.json` → treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test` |
| Config file | none (cargo default) |
| Quick run command | `cargo test --test test_eda` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | `EdaEngine::new(config, init_fn, fitness_fn).run()` returns `EdaResult<U>` | unit | `cargo test --test test_eda test_eda_run_returns_result` | ❌ Wave 0 |
| SC-2 | Bernoulli: probabilities estimated from selected parents, offspring sampled | unit | `cargo test --test test_eda test_eda_bernoulli_estimation` | ❌ Wave 0 |
| SC-2 | Gaussian: mean/std estimated, offspring sampled | unit | `cargo test --test test_eda test_eda_gaussian_estimation` | ❌ Wave 0 |
| SC-3 | Observer receives all 5 hooks | unit | `cargo test --test test_eda test_eda_observer_hooks` | ❌ Wave 0 |
| SC-4 | WASM gate passes | CI | `cargo check --target wasm32-unknown-unknown` | ❌ Wave 0 (CI step) |
| SC-4 | `eda_trap` example converges | integration | `cargo run --example eda_trap` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --test test_eda`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green + `cargo check --target wasm32-unknown-unknown` before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/engines/eda/test_eda.rs` — covers SC-1 through SC-4 (observer, convergence, result shape)
- [ ] `examples/eda_trap.rs` — integration smoke test; trap function convergence

---

## Security Domain

This phase adds a pure computation engine with no I/O, no network, no authentication, and no user input beyond the fitness function closure. ASVS categories do not apply. `security_enforcement` config key is absent — defaulting to enabled, but no controls are applicable to a math library with no external surface.

---

## Sources

### Primary (HIGH confidence)

- `src/engines/pso/engine.rs` — canonical engine template: struct layout, observer wiring, `notify()` helper, `TerminationCause`, `GenerationStats`, run loop
- `src/engines/pso/configuration.rs` — canonical configuration pattern: field names, builder methods, `Default` impl
- `src/engines/pso/mod.rs` — canonical module wiring pattern
- `src/traits/real_gene.rs` — `RealGene` trait: `real_value()`, `with_real_value()`, `bounds()`
- `src/types/genotypes/binary.rs` — `Binary` gene: `id`, `value`, `GeneT` impl; confirmed `id` is positional not boolean
- `src/initializers/binary_initializer.rs` — confirms `id = i as i32` (positional), not 0/1
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait: 12 hooks, default no-op impls
- `src/lib.rs` — engine registration via `#[path]` re-export; PSO re-export as template
- `src/stats.rs` — `GenerationStats::from_fitness_values()` signature
- `src/rng.rs` — `make_rng()` API
- `.planning/phases/58-eda-umda-engine/58-CONTEXT.md` — all locked decisions

### Secondary (MEDIUM confidence)

- CONTEXT.md `## Canonical References` section — points to correct reference files
- `src/engines/cma/engine.rs` — secondary engine reference confirming WASM gate pattern (no `Instant`, no `par_iter`)

### Tertiary (LOW confidence)

- General UMDA/EDA algorithm knowledge [ASSUMED] — the algorithm structure in CONTEXT.md aligns with standard UMDA literature

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all imports verified by reading engine source files
- Architecture: HIGH — PSO engine is the canonical template; direct file reads confirm all patterns
- Pitfalls: HIGH — most pitfalls derived from direct code inspection (gene.id() issue confirmed by reading binary_initializer.rs)
- UMDA algorithm correctness: MEDIUM — algorithm is textbook; specific Rust implementation choices (dispatch mechanism) have tradeoffs the planner must resolve

**Research date:** 2026-06-04
**Valid until:** 2026-07-04 (stable library, no fast-moving dependencies)
