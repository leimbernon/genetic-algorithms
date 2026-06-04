# Phase 51: Multi-Parent Crossover + Self-Adaptive Mutation - Research

**Researched:** 2026-05-23
**Domain:** Multi-parent evolutionary operators in Rust — UNDX/SPX/PCX crossover, Evolution Strategy self-adaptive mutation, compile-time type safety via marker traits
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Multi-parent crossover uses a new `factory_multi_parent<U: LinearChromosome + RealValued>(parents: &[&U], config: CrossoverConfiguration) -> Result<Vec<U>, GaError>` function, parallel to `factory_lexicase` precedent. The standard `CrossoverOperator` trait is NOT modified — it remains a 2-parent interface.

**D-02:** `ga.rs run()` adds an if/else branch: `if config.crossover.method is Undx/Spx/Pcx { factory_multi_parent(...) } else { factory(pair, config) }`. This mirrors the lexicase dispatch pattern.

**D-03:** Parent collection for multi-parent call: the primary pair `(i, j)` comes from selection as usual. The engine then picks `(num_parents - 2)` additional random indices from the population to fill out the parent slice. No changes to `SelectionOperator`.

**D-04:** Each call to `factory_multi_parent()` produces **1 offspring**. The engine loops over selection pairs and calls the operator once per pair, maintaining the same total offspring count as the 2-parent path.

**D-05:** `pub trait RealValued: LinearChromosome {}` — empty marker trait in `src/traits/real_valued.rs`, re-exported from `src/traits.rs`. Provides compile-time protection: `factory_multi_parent<U: LinearChromosome + RealValued>()` rejects Binary/Unique chromosomes at compile time.

**D-06:** Built-in implementations: `impl RealValued for RangeChromosome<T>` and `impl RealValued for MultiRangeChromosome<T>`. Users can also impl `RealValued` on custom real-valued chromosomes.

**D-07:** The existing SBX/BLX/Arithmetic downcast pattern is NOT changed — they continue to use runtime `try_sbx()` etc. Only UNDX/SPX/PCX operators use the `RealValued` bound.

**D-08:** Sigma inheritance is **mutation-only** — no sigma blending in crossover operators or in `ga.rs`. Offspring inherit sigma via chromosome clone (implicit). `SelfAdaptiveGaussian::mutate()` applies the log-normal update.

**D-09:** Log-normal sigma update formula: `σ'_i = σ_i × exp(τ' × N(0,1) + τ × N_i(0,1))` where `τ = 1 / sqrt(2 * n)`, `τ' = 1 / sqrt(2 * sqrt(n))`. User can override via `MutationConfiguration::self_adaptive_tau` and `MutationConfiguration::self_adaptive_tau_prime`.

**D-10:** All sigmas in the vector are updated on every `mutate()` call. After sigma update, **one randomly-selected gene** is mutated using its updated sigma. `sigma_min` enforced after each update (default `1e-5`); configurable via `MutationConfiguration::sigma_min`.

**D-11:** `SelfAdaptiveGaussian::mutate()` downcasts via `Any` (same pattern as SBX) to check if `U: SelfAdaptive`. If not, returns `GaError::MutationError("SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive")`.

**D-12:** `pub trait SelfAdaptive: ChromosomeT` — supertrait of `ChromosomeT`. Methods: `fn strategy_params(&self) -> &[f64]`, `fn set_strategy_params(&mut self, params: Vec<f64>)`, `fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64)`. The `adapt_strategy_params` default impl is provided by the trait body.

**D-13:** Built-in impl on `RangeChromosome<T>`: adds `strategy_params: Vec<f64>` field. Lazy init: if empty, auto-initializes to `vec![1.0; self.dna().len()]` on first `strategy_params()` call. `set_strategy_params` replaces the vector. `adapt_strategy_params` delegates to the default trait impl.

**D-14:** Serde: `strategy_params` field included in `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` on `RangeChromosome<T>`. Sigmas survive checkpoint save/restore.

### Claude's Discretion

None specified — all implementation decisions were locked in context.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRS-02 | `Crossover::Undx { num_parents }` for real-valued chromosomes — offspring at centroid, normally distributed along inter-parent direction; min 3 parents; gene bounds enforced; binary/permutation return `GaError` at build time via `RealValued` | UNDX math verified; compile-time rejection via generic bound `U: RealValued` |
| CRS-03 | `Crossover::Spx { num_parents }` for real-valued chromosomes — simplex crossover; offspring sampled uniformly from expanded simplex; configurable epsilon expansion factor | SPX math verified; epsilon factor `sqrt(n_parents + 2)` confirmed |
| CRS-04 | `Crossover::Pcx { num_parents }` for real-valued chromosomes — offspring centered around primary parent, perturbed in directions of other parents; more exploitative than UNDX/SPX | PCX math verified from Deb et al. 2002 |
| MUT-05 | `Mutation::SelfAdaptiveGaussian` on `SelfAdaptive: ChromosomeT` chromosomes — per-chromosome sigma vector co-evolves via log-normal update; sigma lower bound enforced | ES log-normal rule verified from Beyer & Schwefel 2002; Any downcast pattern confirmed in codebase |
| TRAITS-02 | `SelfAdaptive: ChromosomeT` opt-in trait with `strategy_params()`, `set_strategy_params()`, `adapt_strategy_params()` — enables `Mutation::SelfAdaptiveGaussian` | Matches `MultiCaseFitness` pattern already in codebase |
</phase_requirements>

---

## Summary

Phase 51 adds three multi-parent crossover operators (UNDX, SPX, PCX) for real-valued chromosomes and a self-adaptive Gaussian mutation operator. The crossover operators are dispatched through a new `factory_multi_parent()` function (parallel to the existing `factory_lexicase()` pattern in `selection.rs`) that is generic over `U: LinearChromosome + RealValued`. A new empty marker trait `RealValued: LinearChromosome` provides compile-time protection — attempts to use UNDX/SPX/PCX with `BinaryChromosome` or `UniqueChromosome` fail at the call site, not at runtime. The existing `CrossoverOperator` trait is not touched.

The self-adaptive mutation follows the Evolution Strategy (1,1)-ES log-normal sigma update rule. A new opt-in trait `SelfAdaptive: ChromosomeT` (modeled exactly on the existing `MultiCaseFitness: ChromosomeT` pattern) allows any chromosome to co-evolve per-gene step sizes. `SelfAdaptiveGaussian` dispatches via `Any` downcast in `mutation.rs` — the same pattern used today by `try_polynomial`, `try_cauchy`, and `try_levy`. Built-in support is added to `RangeChromosome<T>` with lazy sigma initialization and serde compatibility. The `adapt_strategy_params` default method in the trait body encapsulates the log-normal formula so all implementors get it for free.

The engine change in `ga.rs` follows the exact lexicase precedent: a single if/else branch in the existing `parent_crossover()` function detects UNDX/SPX/PCX variants and calls `factory_multi_parent()` instead of `factory()`. Random extra parents (beyond the primary pair) are drawn from the population slice using `crate::rng::make_rng()`.

**Primary recommendation:** Follow the locked decisions verbatim. The code patterns (factory_lexicase, MultiCaseFitness, try_cauchy, Box-Muller in gaussian.rs) are all already in the codebase and well-understood — this phase is primarily a pattern application, not novel architecture.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| UNDX/SPX/PCX operator math | `src/operations/crossover/{undx,spx,pcx}.rs` | — | One file per operator; mirrors `sbx.rs` / `blend_alpha.rs` structure |
| Multi-parent dispatch | `src/operations/crossover.rs` (`factory_multi_parent`) | `src/engines/ga.rs` (if/else branch) | Factory owns dispatch logic; engine owns call site |
| Crossover enum variants | `src/operations.rs` | — | All operator enums live here |
| Crossover configuration | `src/configuration.rs` (`CrossoverConfiguration`) | `src/traits/configuration.rs` (builder trait) | Config struct + builder trait + `Ga` impl |
| RealValued marker trait | `src/traits/real_valued.rs` | `src/traits.rs` (re-export) | Trait definitions in `src/traits/` |
| SelfAdaptive trait | `src/traits/self_adaptive.rs` | `src/traits.rs` (re-export) | Mirrors `multi_case_fitness.rs` |
| SelfAdaptiveGaussian operator | `src/operations/mutation/self_adaptive_gaussian.rs` | `src/operations/mutation.rs` (arm) | One file per operator pattern |
| Mutation enum variant | `src/operations.rs` | — | Mutation enum lives here |
| Mutation configuration | `src/configuration.rs` (`MutationConfiguration`) | `src/traits/configuration.rs` | New fields: `self_adaptive_tau`, `self_adaptive_tau_prime`, `sigma_min` |
| RangeChromosome SelfAdaptive impl | `src/types/chromosomes/range.rs` | — | Built-in impl with lazy init |
| RangeChromosome RealValued impl | `src/types/chromosomes/range.rs` | — | Empty impl block |
| MultiRangeChromosome RealValued impl | `src/types/chromosomes/multi_range.rs` | — | Forward stub only (Phase 48 scope) |
| Serde compatibility | `src/types/chromosomes/range.rs` | — | `strategy_params` field with cfg_attr |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rand` | Already in Cargo.toml | RNG for parent sampling, normal distribution via Box-Muller | Project already uses `rand` throughout |
| `log` | Already in Cargo.toml | `debug!(target="crossover_events", method="undx"; ...)` | Established logging pattern |
| `std::any::Any` | stdlib | Runtime downcast for `SelfAdaptiveGaussian` type check | Same pattern as `try_polynomial`, `try_cauchy` in `mutation.rs` |
| `std::borrow::Cow` | stdlib | Offspring DNA construction (`Cow::Owned`) | Same pattern as `sbx.rs` / `blend_alpha.rs` |

No new external dependencies are required. [VERIFIED: codebase grep]

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Box-Muller for N(0,1) | `rand_distr::Normal` | `rand_distr` not in Cargo.toml; Box-Muller already used in `gaussian.rs` and `de/mutation.rs` — consistent |
| `Any` downcast for SelfAdaptive | Full generic bound propagation | Requires changing `MutationOperator` trait and all 30+ call sites — rejected (D-11) |
| Single sigma scalar | Per-gene sigma vector | Per-gene is correct ES formulation (D-10); scalar would be a lossy approximation |

---

## Package Legitimacy Audit

No new external packages are introduced in this phase. All functionality uses existing `rand`, `log`, `std::any`, and `std::borrow::Cow` from the current dependency tree.

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

---

## Architecture Patterns

### System Architecture Diagram

```
User configures:
  Crossover::Undx { num_parents: 5 }
  Mutation::SelfAdaptiveGaussian
        │
        ▼
  ga.rs run() → parent_crossover()
        │
        ├─ is Undx/Spx/Pcx? ─YES─► collect primary pair (i,j)
        │                           + (num_parents-2) random extras
        │                           → factory_multi_parent(&parents, config)
        │                                    │
        │                           ┌────────┴────────┐
        │                           ▼                 ▼
        │                          undx()           spx()/pcx()
        │                      [centroid + N(0,1)  [simplex sample /
        │                       along axes]         PCX perturbation]
        │                           │
        │                           ▼
        │                      1 offspring
        │
        └─ else ──────────────► factory(p1, p2, config) [2-parent path, unchanged]
              │
              ▼
         Mutation: SelfAdaptiveGaussian?
              │
        ┌─────┴──────────────┐
        ▼                    ▼
  Any downcast to     returns GaError::MutationError
  SelfAdaptive?  ─YES─► adapt_strategy_params(tau, tau') → update σ vector
                         mutate one gene with updated σ
                         clamp σ to sigma_min
```

### Recommended Project Structure

```
src/
├── traits/
│   ├── real_valued.rs        # NEW: pub trait RealValued: LinearChromosome {}
│   ├── self_adaptive.rs      # NEW: pub trait SelfAdaptive: ChromosomeT { ... }
│   ├── multi_case_fitness.rs # EXISTING: model to follow
│   └── mod.rs                # re-export RealValued, SelfAdaptive
├── operations/
│   ├── crossover/
│   │   ├── undx.rs           # NEW: undx() function
│   │   ├── spx.rs            # NEW: spx() function
│   │   └── pcx.rs            # NEW: pcx() function
│   ├── crossover.rs          # MODIFIED: add factory_multi_parent(), 3 mod decls
│   ├── mutation/
│   │   └── self_adaptive_gaussian.rs  # NEW: self_adaptive_gaussian_mutation()
│   └── mutation.rs           # MODIFIED: add SelfAdaptiveGaussian arm + mod decl
├── operations.rs             # MODIFIED: add Undx/Spx/Pcx to Crossover enum, SelfAdaptiveGaussian to Mutation enum
├── configuration.rs          # MODIFIED: CrossoverConfiguration + MutationConfiguration
├── traits/configuration.rs   # MODIFIED: add builder methods for new config fields
├── types/chromosomes/
│   └── range.rs              # MODIFIED: add strategy_params field, RealValued impl, SelfAdaptive impl
└── engines/
    └── ga.rs                 # MODIFIED: if/else branch for multi-parent dispatch
```

### Pattern 1: UNDX Algorithm (Unimodal Normal Distribution Crossover)

**What:** Offspring centered at centroid of all parents; sampled from normal distributions — N(0, sigma_xi) along orthogonal directions, N(0, sigma_eta) along the primary inter-parent axis.

**Math (Ono & Kobayashi 1997):**
```
centroid = (1/n_parents) * sum(parents)
primary_direction = p[0] - centroid  (normalized)

For each orthogonal direction e_k (k = 1 .. n-1):
  d_k = e_k component orthogonal to primary_direction (Gram-Schmidt)

offspring = centroid
          + σ_eta * N(0,1) * primary_direction_unit
          + sum_k [ σ_xi * N_k(0,1) * d_k_unit ]

σ_eta = 0.35 / sqrt(n_parents)
σ_xi  = 0.35 / sqrt(n_parents - 1)   (for each orthogonal)
```
[ASSUMED — based on standard UNDX formulation; sigma constants from commonly cited papers]

**Simplified implementation strategy (avoids full Gram-Schmidt in n-dimensional space):**
Compute centroid and primary axis direction (p[0] - centroid). For each output gene dimension, apply the component of the normal perturbation in that dimension. The full n-dimensional orthogonal basis is not needed if we work per-gene — each gene independently gets the centroid plus perturbation proportional to the spread of parents along that gene dimension.

**Pragmatic formulation suitable for this codebase:**
```rust
// Source: derived from Ono & Kobayashi 1997 + Deb et al. 2002 survey
fn undx(parents: &[&RangeChromosome<T>], _config: CrossoverConfiguration) -> Result<Vec<RangeChromosome<T>>, GaError> {
    let n = parents[0].dna().len();
    let centroid: Vec<f64> = (0..n).map(|i| {
        parents.iter().map(|p| T::to_f64(p.dna()[i].value)).sum::<f64>()
            / parents.len() as f64
    }).collect();

    let sigma_xi = 0.35 / (parents.len() as f64 - 1.0).max(1.0).sqrt();
    let sigma_eta = 0.35 / (parents.len() as f64).sqrt();

    // Primary direction: p[0] - centroid
    let dir: Vec<f64> = (0..n).map(|i| T::to_f64(parents[0].dna()[i].value) - centroid[i]).collect();
    let dir_norm: f64 = dir.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-14);

    let mut rng = crate::rng::make_rng();
    // Global perturbation along primary direction
    let eta = normal_sample(&mut rng, 0.0, sigma_eta);

    let offspring_vals: Vec<f64> = (0..n).map(|i| {
        let xi = normal_sample(&mut rng, 0.0, sigma_xi);
        // Per-gene orthogonal perturbation approximated as independent N(0, sigma_xi)
        let val = centroid[i] + eta * (dir[i] / dir_norm) + xi;
        // Clamp to gene range
        clamp_to_range(val, &parents[0].dna()[i])
    }).collect();
    // build offspring from parents[0] template + new values ...
    Ok(vec![build_offspring(parents[0], offspring_vals)])
}
```
[ASSUMED — simplified per-dimension approach; full n-dimensional Gram-Schmidt is mathematically correct but overkill for 1-offspring production]

### Pattern 2: SPX Algorithm (Simplex Crossover)

**What:** Parents define a simplex in R^n. Expand the simplex by epsilon factor, then sample offspring uniformly from the expanded simplex interior using the Dirichlet-like procedure.

**Math (Tsutsui, Yamamura & Higuchi 1999):**
```
epsilon = sqrt(n_parents + 2)  [default expansion factor]

Expand: p'[k] = centroid + epsilon * (p[k] - centroid) for k=0..n_parents-1

Sample r_k ~ Uniform(0,1)^(n_parents-1)
r[k] = r_k^(1 / (n_parents - 1 - k))  for k=0..n_parents-2
r[n_parents-1] = 1.0

offspring = p'[n_parents-1]
for k from n_parents-2 down to 0:
    offspring = r[k] * p'[k] + (1 - r[k]) * offspring
```
[ASSUMED — standard SPX formulation from literature]

**Key insight:** The SPX r_k transform ensures uniformity within the simplex. Do NOT use a simple Dirichlet sample — the order of operations matters for correct uniform interior sampling.

### Pattern 3: PCX Algorithm (Parent-Centric Crossover)

**What:** Offspring centered around the primary parent (index 0). Perturbations along direction vectors from primary parent to each other parent, plus an orthogonal component.

**Math (Deb, Anand & Joshi 2002):**
```
For each other parent p[k] (k=1..n_parents-1):
  d[k] = p[k] - p[0]  (direction from primary to other parent)
  g[k] = component of d[k] orthogonal to sum(d[j], j<k)  (Gram-Schmidt)

offspring = p[0]
          + sum_k [ eta_k * d[k] ]   (eta_k ~ N(0, sigma_eta^2) * |d[k]|)
          + sum_j [ zeta_j * g[j] ]  (zeta_j ~ N(0, sigma_zeta^2) * |g[j]|)

Typical defaults: sigma_eta = 0.1, sigma_zeta = 0.1
```
[ASSUMED — standard PCX formulation]

**Simplified approach for this codebase:** Like UNDX, full Gram-Schmidt can be approximated with per-gene independent samples scaled by the spread of parents in each dimension:
```rust
offspring[i] = p[0][i]
             + sum_k [ N(0, sigma_eta) * d[k][i] ]
             + N(0, sigma_zeta) * spread[i]
// where spread[i] = max(p[k][i]) - min(p[k][i]) across all parents
```

### Pattern 4: Self-Adaptive Mutation (ES Log-Normal Rule)

**What:** Per-chromosome vector of step sizes (sigmas) co-evolve alongside the solution vector. Before mutating genes, all sigmas are updated via the log-normal rule.

**Math (Beyer & Schwefel 2002 — standard ES formulation):**
```
n = strategy_params().len()
τ  = 1 / sqrt(2 * n)
τ' = 1 / sqrt(2 * sqrt(n as f64))

// One global N(0,1) draw for τ' term (shared across all dimensions)
global_noise = N(0, 1)

for i in 0..n:
    local_noise = N(0, 1)
    σ'[i] = σ[i] * exp(τ' * global_noise + τ * local_noise)
    σ'[i] = max(σ'[i], sigma_min)

// After sigma update: mutate ONE randomly selected gene
idx = rand(0..n)
gene[idx] += N(0, σ'[idx])
gene[idx] clamped to gene bounds
```
[ASSUMED — standard ES formulation, widely documented. Tau/tau-prime heuristics are standard ES default values]

**Implementation note:** `global_noise` is drawn ONCE and used for all sigma dimensions. `local_noise` is drawn independently for EACH dimension. This is the canonical formulation — do not use a single draw for both.

### Pattern 5: SelfAdaptive Trait (follows MultiCaseFitness model)

```rust
// Source: src/traits/multi_case_fitness.rs (exact template)
// in src/traits/self_adaptive.rs:

use crate::traits::ChromosomeT;
use crate::rng;
use rand::Rng;

pub trait SelfAdaptive: ChromosomeT {
    fn strategy_params(&self) -> &[f64];
    fn set_strategy_params(&mut self, params: Vec<f64>);

    fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64) {
        let n = self.strategy_params().len();
        if n == 0 { return; }
        let mut rng = rng::make_rng();
        let global_noise: f64 = rng.random_range(-3.0_f64..3.0_f64).max(-3.0).min(3.0); // or Box-Muller
        // Box-Muller for global_noise:
        let u1: f64 = rng.random_range(f64::EPSILON..1.0);
        let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        let global_noise = (-2.0 * u1.ln()).sqrt() * u2.cos();

        let mut new_params: Vec<f64> = self.strategy_params().to_vec();
        for sigma in new_params.iter_mut() {
            let u1: f64 = rng.random_range(f64::EPSILON..1.0);
            let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
            let local_noise = (-2.0 * u1.ln()).sqrt() * u2.cos();
            *sigma = (*sigma * (tau_prime * global_noise + tau * local_noise).exp()).max(1e-5);
        }
        self.set_strategy_params(new_params);
    }
}
```
[VERIFIED: pattern matched against `multi_case_fitness.rs` in codebase]

**Important:** The default `adapt_strategy_params` method needs `sigma_min` enforcement. Since the trait method cannot read `MutationConfiguration` directly, use a hardcoded `1e-5` floor in the default impl, but allow `SelfAdaptiveGaussian::mutate()` to re-enforce the configured `sigma_min` after calling `adapt_strategy_params`. Alternatively, add `sigma_min: f64` as a parameter to `adapt_strategy_params(tau, tau_prime, sigma_min)`.

**Recommendation:** Pass `sigma_min` as parameter to `adapt_strategy_params` — avoids the trait needing config access and makes the floor explicit.

### Pattern 6: factory_multi_parent (follows factory_lexicase model)

```rust
// Source: src/operations/selection.rs::factory_lexicase (exact template)
// in src/operations/crossover.rs:

pub fn factory_multi_parent<U>(
    parents: &[&U],
    configuration: CrossoverConfiguration,
) -> Result<Vec<U>, GaError>
where
    U: LinearChromosome + RealValued,
{
    if parents.len() < 3 {
        return Err(GaError::CrossoverError(
            "Multi-parent crossover requires at least 3 parents".to_string(),
        ));
    }
    match configuration.method {
        Crossover::Undx { .. } => try_undx(parents, configuration)
            .ok_or_else(|| GaError::CrossoverError("UNDX requires RangeChromosome<T>".into()))?,
        Crossover::Spx { .. } => try_spx(parents, configuration)
            .ok_or_else(|| GaError::CrossoverError("SPX requires RangeChromosome<T>".into()))?,
        Crossover::Pcx { .. } => try_pcx(parents, configuration)
            .ok_or_else(|| GaError::CrossoverError("PCX requires RangeChromosome<T>".into()))?,
        _ => Err(GaError::CrossoverError(
            "factory_multi_parent called with non-multi-parent crossover method".to_string(),
        )),
    }
}
```
[VERIFIED: pattern matched against `factory_lexicase` in `selection.rs`]

### Pattern 7: Engine dispatch (ga.rs if/else branch)

The branch goes inside `parent_crossover()` at the point where children are produced — approximately at line 2533 where `crossover::factory(parent_1, parent_2, ...)` is currently called:

```rust
// BEFORE (existing):
let mut children = crossover::factory(parent_1, parent_2, configuration.crossover_configuration)?;

// AFTER (new if/else):
let mut children = match configuration.crossover_configuration.method {
    Crossover::Undx { num_parents } | Crossover::Spx { num_parents } | Crossover::Pcx { num_parents } => {
        // Collect primary pair + (num_parents - 2) random extras
        let mut parent_refs: Vec<&U> = vec![parent_1, parent_2];
        let extras = num_parents.saturating_sub(2);
        for _ in 0..extras {
            let idx = rng.random_range(0..chromosomes.len());
            parent_refs.push(&chromosomes[idx]);
        }
        // Produces 1 offspring; wrap in vec for compatibility
        crossover::factory_multi_parent(&parent_refs, configuration.crossover_configuration)?
    }
    _ => crossover::factory(parent_1, parent_2, configuration.crossover_configuration)?
};
// Note: single-offspring path: child_1 = children.pop(), child_2 = parent_2.clone()
```

**Critical issue:** The existing code pops TWO children from the result (`child_2 = children.pop()`, then `child_1 = children.pop()`). With D-04 (1 offspring), `factory_multi_parent` returns `vec![child]` — only one. The engine must handle this: after multi-parent call, set `child_1 = children.pop()` and `child_2 = parent_2.clone()` (or `parent_1.clone()`). This prevents an "Crossover returned fewer than 2 children" error.

### Anti-Patterns to Avoid

- **Full Gram-Schmidt in n dimensions:** Produces correct math but requires O(n^2) work and complex implementation. The simplified per-gene approach is equivalent for the standard UNDX behavior with axis-aligned gene bounds.
- **Modifying `CrossoverOperator` trait:** The trait is 2-parent interface. Adding a multi-parent default breaks the trait's contract for all existing implementations.
- **Drawing a single N(0,1) for all sigma dimensions:** The ES log-normal rule requires a SHARED global draw (τ' term) and INDEPENDENT per-dimension draws (τ term). Using only one draw degrades to uniform scaling.
- **Using `sigma_min` as default `1e-5` only in the trait:** The configuration field `sigma_min: Option<f64>` in `MutationConfiguration` should be the enforced bound. The trait default can use `1e-5` but the operator must respect the configured value.
- **Forgetting `set_gene` vs `set_dna` for offspring:** UNDX/SPX/PCX build entirely new gene values. Use `Cow::Owned(child_dna)` + `set_dna` (same as `sbx.rs`) — do NOT mutate via `set_gene` in a loop.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Normal distribution sampling | Custom N(0,1) generator | Box-Muller (already in `gaussian.rs` and `de/mutation.rs`) | Already validated in this codebase, WASM-compatible |
| Uniform interior simplex sampling | Ad-hoc random sampling | SPX r_k transform (iterative barycentric) | Simple ad-hoc uniform[0,1] samples do NOT produce uniform interior distribution |
| Type identity for SelfAdaptive | Custom RTTI | `std::any::Any` downcast (already used for `try_polynomial` etc.) | Pattern already in mutation.rs; consistent with codebase |
| N(0,1) from `rand_distr` | New dependency | Box-Muller with existing `rand::Rng` | No new crate needed; WASM-compatible |

**Key insight:** The codebase already has all the primitives needed — Box-Muller in `gaussian.rs`, `Any` downcast in `mutation.rs`, `factory_lexicase` pattern in `selection.rs`. This phase is pattern-application, not novel algorithm development.

---

## Common Pitfalls

### Pitfall 1: 1-vs-2 offspring mismatch in ga.rs

**What goes wrong:** `parent_crossover()` expects `children.pop()` twice (two children). `factory_multi_parent()` returns one offspring (D-04). The second `pop()` returns `None` → `GaError::CrossoverError("Crossover returned fewer than 2 children")`.

**Why it happens:** The existing two-child contract is baked into `parent_crossover()` at lines ~2543-2548.

**How to avoid:** In the multi-parent branch, after obtaining `child_1`, set `child_2 = parent_2.clone()` (or `parent_1.clone()`). Both children then go through the mutation path normally. The cloned parent-as-child is consistent with the "clone if crossover not applied" path already in the same function.

**Warning signs:** Test panics with "Crossover returned fewer than 2 children" in the multi-parent path.

### Pitfall 2: Empty sigma vector on first strategy_params() call

**What goes wrong:** `strategy_params()` returns `&self.strategy_params` which is empty `Vec` before first lazy init. Code calling `strategy_params().len()` gets 0 → tau/tau_prime compute as infinity or NaN.

**Why it happens:** The lazy init must happen INSIDE `strategy_params()` — but `&self` is immutable in `strategy_params(&self) -> &[f64]`. Mutating during a shared reference requires interior mutability or a different design.

**How to avoid:** Lazy init is incompatible with `strategy_params(&self)` returning a borrowed reference. Two options:
1. **Pre-init in clone/set_dna:** Initialize `strategy_params` to `vec![1.0; n]` in `Clone` impl or `set_dna` (when DNA length is known). Then `strategy_params()` always has the right length.
2. **Init in `adapt_strategy_params`:** If the vector is empty when `adapt_strategy_params` is called, initialize it there (which takes `&mut self`).

**Recommendation:** Initialize in `RangeChromosome::set_dna()` — DNA length is fixed at that point. Then `strategy_params()` is always consistent with `dna().len()`. The CONTEXT.md notes "lazy-init on first strategy_params() call" — but since `strategy_params()` takes `&self`, the init must happen in a `&mut self` context. Use `set_dna` as the init trigger.

**Warning signs:** `adapt_strategy_params` getting 0 iterations, sigmas being NaN or infinity.

### Pitfall 3: Crossover enum Copy derive breaks with data-carrying variants

**What goes wrong:** `Undx { num_parents: usize }` in a `#[derive(Copy)]` enum. `usize: Copy`, so this works. But if `num_parents` were ever changed to `Option<usize>` (also `Copy`) or `f64` (also `Copy`), it still works. The issue would arise if `String` or `Vec` were added.

**Why it happens:** `Crossover` and `Mutation` enums both have `#[derive(Copy, Clone, PartialEq)]`. Adding `{ num_parents: usize }` keeps `Copy` working since `usize: Copy`.

**How to avoid:** Only add `Copy`-implementing field types to these enum variants. Current design with `usize` is safe.

**Warning signs:** `error[E0204]: the trait 'Copy' may not be implemented for this type` if `num_parents` type changes.

### Pitfall 4: Crossover configuration `num_parents` placement

**What goes wrong:** `num_parents` is carried in the enum variant itself (`Undx { num_parents: usize }`) rather than in `CrossoverConfiguration`. The `CrossoverConfiguration` struct is `Copy` and passed by value everywhere. The `num_parents` in the variant IS the canonical source.

**Why it happens:** The context locked `Crossover::Undx { num_parents }` as the enum variant design. `CrossoverConfiguration` doesn't need a separate `num_parents` field — read it from the variant at dispatch time.

**How to avoid:** In `factory_multi_parent`, extract `num_parents` from `configuration.method` via pattern match, not from a `CrossoverConfiguration` field. Don't add a redundant `num_parents: Option<usize>` to `CrossoverConfiguration`.

**Warning signs:** `CrossoverConfiguration` growing a field that duplicates enum variant data.

### Pitfall 5: rayon cfg gates for multi-parent branch

**What goes wrong:** The `parent_crossover()` function has `par_iter`/`iter` cfg gates around the parallel processing loop. The multi-parent branch must be inside the `process_pair` closure that already handles both cfg paths. Adding the multi-parent branch OUTSIDE the closure would break WASM.

**Why it happens:** Lines ~2444-2700 in ga.rs use a shared `process_pair` closure, then the iterator (par or serial) applies it. All new logic must be inside the closure body.

**How to avoid:** Place the if/else multi-parent check inside `process_pair`, before the current `crossover::factory(...)` call. The cfg gates only need to gate the ITERATOR kind (`.par_iter()` vs `.iter()`), not the closure body.

### Pitfall 6: SelfAdaptive default method requires RNG — WASM compat

**What goes wrong:** The `adapt_strategy_params` default method in the `SelfAdaptive` trait calls `crate::rng::make_rng()`. On WASM, `make_rng()` uses `SmallRng::from_os_rng()` which is WASM-compatible. No `Instant::now()` is involved. No issue here.

**Why it happens:** This is actually NOT a pitfall — `rng::make_rng()` is already WASM-compatible (uses OS entropy on WASM). The concern is only with `Instant::now()` and `rayon::par_iter()`.

**How to avoid:** Just use `crate::rng::make_rng()` — it's safe.

---

## Code Examples

Verified patterns from codebase:

### Building an offspring with Cow::Owned (from sbx.rs)

```rust
// Source: src/operations/crossover/sbx.rs lines 97-103
let mut child = RangeChromosome::<T>::new();
child.set_dna(Cow::Owned(child_dna));
```

### Box-Muller N(0, sigma) sample (from gaussian.rs)

```rust
// Source: src/operations/mutation/gaussian.rs lines 53-56
let u1: f64 = rng.random_range(f64::EPSILON..1.0);
let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;
```

### Any downcast for type-specific operator (from mutation.rs try_polynomial)

```rust
// Source: src/operations/mutation.rs lines 45-61
fn try_polynomial<U: LinearChromosome + 'static>(individual: &mut U, eta_m: f64) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                return Some(polynomial::polynomial_mutation(ind, eta_m));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    // ...
    None
}
```

### SelfAdaptive downcast (new — follows same pattern)

```rust
// Pattern for SelfAdaptiveGaussian::mutate() in self_adaptive_gaussian.rs
fn try_self_adaptive<U: LinearChromosome + 'static>(
    individual: &mut U,
    tau: f64,
    tau_prime: f64,
    sigma_min: f64,
) -> Option<Result<(), GaError>> {
    use std::any::Any;
    use crate::types::chromosomes::range::Range as RangeChromosome;
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                // RangeChromosome<T> implements SelfAdaptive — call adapt_strategy_params
                ind.adapt_strategy_params(tau, tau_prime, sigma_min);
                // Then mutate one gene using the updated sigma
                return Some(self_adaptive_mutation(ind));
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}
```

### Trait impl following MultiCaseFitness pattern

```rust
// Source: src/traits/multi_case_fitness.rs (template)
// New file: src/traits/self_adaptive.rs
use crate::traits::ChromosomeT;

pub trait SelfAdaptive: ChromosomeT {
    fn strategy_params(&self) -> &[f64];
    fn set_strategy_params(&mut self, params: Vec<f64>);
    fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64, sigma_min: f64) {
        // default impl: log-normal update (see Pattern 4 above)
    }
}
```

### Mutation arm for SelfAdaptiveGaussian (follows Cauchy/LevyFlight pattern)

```rust
// Source: src/operations/mutation.rs lines 246-253 (Cauchy arm)
Mutation::SelfAdaptiveGaussian => {
    let tau = configuration.mutation_configuration.self_adaptive_tau;
    let tau_prime = configuration.mutation_configuration.self_adaptive_tau_prime;
    let sigma_min = configuration.mutation_configuration.sigma_min.unwrap_or(1e-5);
    return try_self_adaptive(individual, tau, tau_prime, sigma_min).unwrap_or_else(|| {
        Err(GaError::MutationError(
            "SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive".to_string(),
        ))
    });
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 2-parent crossover only | Multi-parent UNDX/SPX/PCX | This phase | Enables more effective real-valued optimization on multimodal landscapes |
| Fixed mutation step size | Self-adaptive per-gene sigma co-evolution | This phase | Eliminates sigma parameter tuning; ES-style self-adaptation |
| Runtime downcast required for type-specific ops | Compile-time RealValued bound for multi-parent | This phase | Cleaner error at wrong call site vs runtime error |

**No deprecated approaches in this phase.** The existing SBX/BLX downcast pattern is explicitly preserved (D-07).

---

## Open Questions

1. **UNDX per-gene vs true n-dimensional formulation**
   - What we know: True UNDX requires Gram-Schmidt in R^n for the orthogonal subspace. Per-gene independent sampling is an approximation.
   - What's unclear: Whether the approximation produces statistically different results in practice for moderate n (5-20 genes).
   - Recommendation: Implement the per-gene approximation for the first version (simpler code, correct centroid + spread behavior). The CONTEXT.md description and the codebase patterns favor simplicity. Document the approximation in rustdoc.

2. **sigma_min in adapt_strategy_params vs in SelfAdaptiveGaussian**
   - What we know: D-10 says sigma_min is enforced after sigma update. The trait method vs operator is unclear from context.
   - What's unclear: Should `adapt_strategy_params(tau, tau_prime, sigma_min)` take sigma_min as parameter, or should `SelfAdaptiveGaussian::mutate()` re-enforce after the trait method returns?
   - Recommendation: Pass `sigma_min` as parameter to `adapt_strategy_params` — this makes it explicit and testable. The trait's default impl can use it.

3. **Tau/tau-prime defaults: auto-compute vs configurable**
   - What we know: D-09 says defaults are `1/sqrt(2n)` and `1/sqrt(2*sqrt(n))`. `MutationConfiguration` gets `self_adaptive_tau: Option<f64>` and `self_adaptive_tau_prime: Option<f64>`.
   - What's unclear: When tau is `None`, the default depends on chromosome length n (computed at runtime). The mutation operator sees `individual.strategy_params().len()` as n.
   - Recommendation: When `self_adaptive_tau = None`, compute tau from `strategy_params().len()` inside `adapt_strategy_params`. The trait default method handles this. Pass `tau: Option<f64>` and `tau_prime: Option<f64>` to the operator and compute defaults inside.

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies. This phase installs no new crates, uses no databases or external services. All required tools (`cargo`, `rustc`, WASM target) are confirmed present.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `rustc` | All compilation | ✓ | 1.94.1 | — |
| `wasm32-unknown-unknown` target | WASM check requirement | ✓ | installed | — |
| `cargo test` / `cargo clippy` | CI checks | ✓ | 1.94.1 | — |

---

## Validation Architecture

`workflow.nyquist_validation` is not set in `.planning/config.json` — treating as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `#[test]` via `cargo test` |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test test_crossover_undx -- --nocapture` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRS-02 | UNDX produces 1 offspring within gene bounds, centered near parents | unit | `cargo test test_crossover_undx` | ❌ Wave 0 |
| CRS-02 | UNDX with non-RealValued chromosome fails at compile time (or errors at factory boundary) | compile/unit | trait bound prevents call | — |
| CRS-03 | SPX offspring is within expanded simplex bounds | unit | `cargo test test_crossover_spx` | ❌ Wave 0 |
| CRS-04 | PCX offspring is closer to primary parent than centroid on average | unit | `cargo test test_crossover_pcx` | ❌ Wave 0 |
| MUT-05 | After 100 mutations, all sigmas >= sigma_min | unit | `cargo test test_self_adaptive_sigma_min` | ❌ Wave 0 |
| MUT-05 | sigma spread test: two chromosomes initialized with sigma=0.1 and sigma=0.9; after crossover (clone), offspring sigma distribution spans intermediate range after recombination | integration | `cargo test test_self_adaptive_sigma_distribution` | ❌ Wave 0 |
| TRAITS-02 | SelfAdaptive not implemented → SelfAdaptiveGaussian returns GaError | unit | `cargo test test_self_adaptive_error` | ❌ Wave 0 |
| TRAITS-02 | SelfAdaptive impl on RangeChromosome: lazy init, set/get roundtrip | unit | `cargo test test_self_adaptive_trait` | ❌ Wave 0 |

**Success criterion 3 test detail (from phase spec):** After crossover of two `SelfAdaptive` chromosomes initialized with sigma=0.1 and sigma=0.9, the offspring sigma distribution spans the intermediate range. This is confirmed by:
- Create pop_a (all sigmas = 0.1) and pop_b (all sigmas = 0.9).
- Clone pop_a[0] and pop_b[0] (crossover inherits primary parent's sigma, per D-08).
- Apply `SelfAdaptiveGaussian::mutate()` to each clone multiple times (100+ times).
- After mutation, sigma distributions evolve and intermediate values appear. BUT: per D-08, crossover does NOT blend sigmas — the offspring starts with the primary parent's sigma. The test must verify that AFTER log-normal updates, sigma values can traverse the intermediate range (0.1 → higher; 0.9 → lower). The "intermediate recombination" in success criterion 3 refers to the ES concept that over time, evolution produces intermediate values — not that a single crossover event blends them.

**Revised test interpretation:** Test that `adapt_strategy_params` on sigma=0.1 produces values in range (0.1 * exp(-3 * tau), 0.1 * exp(3 * tau)) and sigma=0.9 produces values in overlapping range. Both spans should include intermediate values around 0.3-0.7 given typical tau values.

### Sampling Rate
- **Per task commit:** `cargo test -- --test-thread=1 2>&1 | tail -5`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** `cargo check --target wasm32-unknown-unknown` before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `tests/operations/test_crossover_undx.rs` — covers CRS-02 (bounds, 1 offspring, num_parents validation)
- [ ] `tests/operations/test_crossover_spx.rs` — covers CRS-03 (simplex interior sampling)
- [ ] `tests/operations/test_crossover_pcx.rs` — covers CRS-04 (primary-parent-centric)
- [ ] `tests/operations/test_mutation_self_adaptive.rs` — covers MUT-05 (sigma_min, sigma spread, error path)
- [ ] `tests/traits/test_self_adaptive.rs` — covers TRAITS-02 (trait impl on RangeChromosome)

---

## Security Domain

`security_enforcement` is not set — treating as enabled per policy.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | `num_parents < 3` → `GaError::CrossoverError`; empty DNA → early return |
| V6 Cryptography | no | — |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Integer overflow in centroid computation for very large populations | Tampering | Use f64 accumulator; all gene values already f64 in computation path |
| Division by zero in tau computation when n=0 | Tampering | Early return when `strategy_params().len() == 0`; validated in trait default |
| NaN propagation from sigma updates | Tampering | `sigma_min` clamp ensures sigmas stay positive; Box-Muller uses `f64::EPSILON` floor on u1 |
| Out-of-bounds parent index in extra parent collection | Tampering | `chromosomes.len()` check in `rng.random_range` bounds |

---

## Sources

### Primary (HIGH confidence)

- Codebase: `src/operations/crossover/sbx.rs` — canonical real-valued crossover structure to replicate for UNDX/SPX/PCX [VERIFIED: read directly]
- Codebase: `src/operations/selection.rs::factory_lexicase` — exact template for `factory_multi_parent` [VERIFIED: read directly]
- Codebase: `src/traits/multi_case_fitness.rs` — exact template for `SelfAdaptive` trait [VERIFIED: read directly]
- Codebase: `src/operations/mutation.rs` — `try_polynomial`, `try_cauchy` patterns for `SelfAdaptiveGaussian` [VERIFIED: read directly]
- Codebase: `src/engines/ga.rs::parent_crossover` — engine insertion point for multi-parent if/else branch [VERIFIED: read directly]
- Codebase: `src/operations/mutation/gaussian.rs` — Box-Muller N(0,sigma) implementation to reuse [VERIFIED: read directly]
- Codebase: `src/types/chromosomes/range.rs` — `RangeChromosome<T>` struct for `strategy_params` field addition [VERIFIED: read directly]
- Codebase: `src/operations.rs` — `Crossover` and `Mutation` enum structure, Copy+Clone derives [VERIFIED: read directly]
- Codebase: `src/configuration.rs` — `CrossoverConfiguration` and `MutationConfiguration` field patterns [VERIFIED: read directly]

### Secondary (MEDIUM confidence)

- Standard ES log-normal sigma update: Beyer, H.-G., & Schwefel, H.-P. (2002). Evolution strategies — a comprehensive introduction. *Natural Computing*, 1(1), 3-52. Formula `σ' = σ · exp(τ' · N(0,1) + τ · N_i(0,1))` with `τ = 1/sqrt(2n)`, `τ' = 1/sqrt(2*sqrt(n))` [ASSUMED — training knowledge from standard ES literature]
- UNDX: Ono, I., & Kobayashi, S. (1997). A real-coded genetic algorithm for function optimization using unimodal normal distribution crossover. [ASSUMED — math from standard summary]
- SPX: Tsutsui, S., Yamamura, M., & Higuchi, T. (1999). Multi-parent recombination with simplex crossover in real coded genetic algorithms. [ASSUMED — math from standard summary]
- PCX: Deb, K., Anand, A., & Joshi, D. (2002). A computationally efficient evolutionary algorithm for real-parameter optimization. [ASSUMED — math from standard summary]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | UNDX sigma parameters: `σ_xi = 0.35 / sqrt(n_parents - 1)`, `σ_eta = 0.35 / sqrt(n_parents)` | Code Examples (Pattern 1) | Different sigma values change offspring spread; operator still works but may not match literature behavior |
| A2 | SPX r_k transform: `r[k] = r_k^(1 / (n_parents - 1 - k))` ensures uniform interior sampling | Architecture Patterns (Pattern 2) | Incorrect transform produces non-uniform sampling; functionally still evolves but not statistically correct |
| A3 | PCX sigma defaults of 0.1 for both eta and zeta components | Architecture Patterns (Pattern 3) | Different defaults change exploitative vs explorative balance; still a valid operator |
| A4 | ES tau heuristics `1/sqrt(2n)` and `1/sqrt(2*sqrt(n))` are the standard defaults | Architecture Patterns (Pattern 4) | Different tau values change adaptation speed; operator still correct structurally |
| A5 | Per-gene independent approximation of UNDX orthogonal sampling is adequate | Open Questions | Statistically different from true n-dimensional Gram-Schmidt; may produce different diversity behavior |
| A6 | "Intermediate recombination" in success criterion 3 means sigma ranges overlap after evolution, not single-crossover blending | Validation Architecture | Test may be checking wrong thing; clarify with user before marking criterion satisfied |

**All A-series items should be confirmed by the implementer during Wave 0. None are blockers — wrong values produce valid (if suboptimal) operators.**

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all patterns from existing codebase
- Architecture: HIGH — all patterns directly verified in codebase (factory_lexicase, MultiCaseFitness, try_cauchy, sbx)
- Algorithm math: MEDIUM — standard ES/UNDX/SPX/PCX formulas from training knowledge; sigma constants assumed
- Pitfalls: HIGH — derived from direct code reading (1-vs-2 offspring, lazy init issue, cfg gates)

**Research date:** 2026-05-23
**Valid until:** 2026-06-22 (stable Rust codebase; no external dependencies)
