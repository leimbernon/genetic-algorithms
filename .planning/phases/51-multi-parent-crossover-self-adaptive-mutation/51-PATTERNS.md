# Phase 51: Multi-Parent Crossover + Self-Adaptive Mutation - Pattern Map

**Mapped:** 2026-05-23
**Files analyzed:** 14 (6 new, 8 modified)
**Analogs found:** 14 / 14

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/traits/real_valued.rs` | trait | — | `src/traits/multi_case_fitness.rs` | exact |
| `src/traits/self_adaptive.rs` | trait | transform | `src/traits/multi_case_fitness.rs` | exact |
| `src/operations/crossover/undx.rs` | operator | transform | `src/operations/crossover/sbx.rs` | exact |
| `src/operations/crossover/spx.rs` | operator | transform | `src/operations/crossover/sbx.rs` | exact |
| `src/operations/crossover/pcx.rs` | operator | transform | `src/operations/crossover/blend_alpha.rs` | exact |
| `src/operations/mutation/self_adaptive_gaussian.rs` | operator | transform | `src/operations/mutation/cauchy.rs` | exact |
| `src/traits.rs` | module | — | `src/traits.rs` (self, add re-exports) | self-mod |
| `src/operations.rs` | module | — | `src/operations.rs` (self, extend enums) | self-mod |
| `src/operations/crossover.rs` | dispatcher | request-response | `src/operations/selection.rs` (factory_lexicase) | role-match |
| `src/operations/mutation.rs` | dispatcher | request-response | `src/operations/mutation.rs` (Cauchy/LevyFlight arms) | self-mod |
| `src/configuration.rs` | config | — | `src/configuration.rs` (self, MutationConfiguration) | self-mod |
| `src/types/chromosomes/range.rs` | model | CRUD | `src/types/chromosomes/range.rs` + MultiCaseFitness impl in `tests/structures.rs` | self-mod |
| `src/types/chromosomes/multi_range.rs` | model | — | `src/types/chromosomes/range.rs` | role-match |
| `src/engines/ga.rs` | engine | event-driven | `src/engines/ga.rs` (lexicase if/else branch) | self-mod |

---

## Pattern Assignments

### `src/traits/real_valued.rs` (new trait, marker)

**Analog:** `src/traits/multi_case_fitness.rs` (lines 1-15)

**Full file pattern** — the file is intentionally minimal:
```rust
//! RealValued marker trait for compile-time enforcement on multi-parent crossover.

use crate::traits::LinearChromosome;

/// Marker trait for real-valued chromosomes.
///
/// Implement alongside [`LinearChromosome`] to enable multi-parent crossover operators
/// (UNDX, SPX, PCX) which operate on real-valued gene spaces.
/// Binary and permutation chromosomes must NOT implement this trait.
pub trait RealValued: LinearChromosome {}
```

**Key points:**
- No methods — empty marker trait, identical pattern to how `MultiCaseFitness` is a data-carrying opt-in trait (but here with zero methods since it's purely compile-time).
- Supertrait is `LinearChromosome`, not `ChromosomeT`, because `factory_multi_parent` receives `LinearChromosome` instances.

---

### `src/traits/self_adaptive.rs` (new trait, transform)

**Analog:** `src/traits/multi_case_fitness.rs` (lines 1-15)

**Imports pattern** (from `multi_case_fitness.rs` lines 1-4):
```rust
//! Self-adaptive mutation trait for Evolution Strategy sigma co-evolution.

use crate::traits::ChromosomeT;
```

**Core trait pattern** (modeled on `multi_case_fitness.rs`, with default method added):
```rust
/// Opt-in trait enabling `Mutation::SelfAdaptiveGaussian`.
///
/// Implement alongside [`ChromosomeT`]. The `strategy_params` field co-evolves
/// via the ES log-normal update rule applied in `SelfAdaptiveGaussian::mutate()`.
pub trait SelfAdaptive: ChromosomeT {
    /// Returns the per-gene step size (sigma) vector.
    fn strategy_params(&self) -> &[f64];

    /// Replaces the sigma vector.
    fn set_strategy_params(&mut self, params: Vec<f64>);

    /// Applies the ES log-normal sigma update.
    /// Default impl: σ'_i = σ_i * exp(τ' * N_global(0,1) + τ * N_i(0,1)), clamped to sigma_min.
    fn adapt_strategy_params(&mut self, tau: f64, tau_prime: f64, sigma_min: f64) {
        let n = self.strategy_params().len();
        if n == 0 {
            return;
        }
        let mut rng = crate::rng::make_rng();
        // Box-Muller for global noise (shared across all dimensions — τ' term)
        let u1: f64 = rng.random_range(f64::EPSILON..1.0);
        let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        let global_noise = (-2.0 * u1.ln()).sqrt() * u2.cos();

        let mut new_params: Vec<f64> = self.strategy_params().to_vec();
        for sigma in new_params.iter_mut() {
            // Box-Muller for per-dimension local noise (τ term)
            let u1: f64 = rng.random_range(f64::EPSILON..1.0);
            let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
            let local_noise = (-2.0 * u1.ln()).sqrt() * u2.cos();
            *sigma = (*sigma * (tau_prime * global_noise + tau * local_noise).exp()).max(sigma_min);
        }
        self.set_strategy_params(new_params);
    }
}
```

**Critical:** Box-Muller reuse pattern from `src/operations/mutation/gaussian.rs` lines 53-55:
```rust
let u1: f64 = rng.random_range(f64::EPSILON..1.0);
let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;
```

---

### `src/operations/crossover/undx.rs` (new operator, transform)

**Analog:** `src/operations/crossover/sbx.rs` (all 149 lines)

**Imports pattern** (from `sbx.rs` lines 1-9):
```rust
//! UNDX (Unimodal Normal Distribution Crossover) for real-valued chromosomes.

use crate::chromosomes::Range as RangeChromosome;
use crate::error::GaError;
use crate::traits::LinearChromosome;
use log::debug;
use rand::Rng;
use std::borrow::Cow;
use std::fmt::Debug;
```

**Function signature pattern** (from `sbx.rs` lines 30-37, adapted for multi-parent):
```rust
pub fn undx<T>(
    parents: &[&RangeChromosome<T>],
    _num_parents: usize,  // extracted from Crossover::Undx { num_parents } at call site
) -> Result<Vec<RangeChromosome<T>>, GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + SbxConvertible,
```

Note: Reuse `SbxConvertible` from `sbx.rs` — same `to_f64`/`from_f64` conversion trait.

**Validation pattern** (from `sbx.rs` lines 38-45):
```rust
if parents.len() < 3 {
    return Err(GaError::CrossoverError(
        "UNDX requires at least 3 parents".to_string(),
    ));
}
let len = parents[0].dna().len();
// Check all parents same length
for p in parents.iter().skip(1) {
    if p.dna().len() != len {
        return Err(GaError::CrossoverError(format!(
            "All parents must have the same DNA length. Expected {}, got {}",
            len, p.dna().len()
        )));
    }
}
```

**Core algorithm pattern** (centroid + normal perturbation, using Box-Muller from `gaussian.rs` lines 53-55):
```rust
debug!(target="crossover_events", method="undx"; "Starting UNDX crossover with {} parents", parents.len());

let n = parents[0].dna().len();
let n_par = parents.len() as f64;

// Centroid across all parents
let centroid: Vec<f64> = (0..n).map(|i| {
    parents.iter().map(|p| T::to_f64(p.dna()[i].value)).sum::<f64>() / n_par
}).collect();

let sigma_xi = 0.35 / (n_par - 1.0).max(1.0).sqrt();
let sigma_eta = 0.35 / n_par.sqrt();

// Primary direction: parents[0] - centroid
let dir: Vec<f64> = (0..n)
    .map(|i| T::to_f64(parents[0].dna()[i].value) - centroid[i])
    .collect();
let dir_norm: f64 = dir.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-14);

let mut rng = crate::rng::make_rng();
// Global perturbation along primary direction (Box-Muller)
let u1: f64 = rng.random_range(f64::EPSILON..1.0);
let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let eta: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_eta;

let mut child_dna = Vec::with_capacity(n);
for i in 0..n {
    // Per-dimension orthogonal perturbation (Box-Muller)
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let xi: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_xi;

    let raw = centroid[i] + eta * (dir[i] / dir_norm) + xi;
    // Clamp to gene range (from sbx.rs lines 80-86 pattern)
    let clamped = if !parents[0].dna()[i].ranges.is_empty() {
        let lo: f64 = T::to_f64(parents[0].dna()[i].ranges[0].0);
        let hi: f64 = T::to_f64(parents[0].dna()[i].ranges[0].1);
        raw.clamp(lo, hi)
    } else {
        raw
    };
    let mut gene = parents[0].dna()[i].clone();
    gene.value = T::from_f64(clamped);
    child_dna.push(gene);
}

// Offspring construction (from sbx.rs lines 97-103)
let mut child = RangeChromosome::<T>::new();
child.set_dna(Cow::Owned(child_dna));

debug!(target="crossover_events", method="undx"; "UNDX crossover finished");
Ok(vec![child])
```

---

### `src/operations/crossover/spx.rs` (new operator, transform)

**Analog:** `src/operations/crossover/sbx.rs` (same structure as UNDX above)

**Imports pattern:** Identical to `undx.rs` above.

**Core algorithm pattern** (simplex expansion + uniform interior sampling):
```rust
debug!(target="crossover_events", method="spx"; "Starting SPX crossover with {} parents", parents.len());

let n = parents[0].dna().len();
let n_par = parents.len();
let epsilon = ((n_par + 2) as f64).sqrt();  // expansion factor

// Centroid
let centroid: Vec<f64> = (0..n).map(|i| {
    parents.iter().map(|p| T::to_f64(p.dna()[i].value)).sum::<f64>() / n_par as f64
}).collect();

// Expanded vertices: p'[k] = centroid + epsilon * (p[k] - centroid)
let expanded: Vec<Vec<f64>> = parents.iter().map(|p| {
    (0..n).map(|i| {
        let pv = T::to_f64(p.dna()[i].value);
        centroid[i] + epsilon * (pv - centroid[i])
    }).collect()
}).collect();

let mut rng = crate::rng::make_rng();
// SPX r_k transform for uniform interior sampling (Tsutsui et al. 1999)
let mut r: Vec<f64> = (0..n_par - 1).map(|k| {
    let rk: f64 = rng.random_range(0.0..1.0);
    rk.powf(1.0 / (n_par - 1 - k) as f64)
}).collect();
r.push(1.0);

// Iterative barycentric combination (from last vertex inward)
let mut offspring_vals = expanded[n_par - 1].clone();
for k in (0..n_par - 1).rev() {
    for i in 0..n {
        offspring_vals[i] = r[k] * expanded[k][i] + (1.0 - r[k]) * offspring_vals[i];
    }
}

// Clamp and build offspring (same as sbx.rs / undx.rs pattern)
let mut child_dna = Vec::with_capacity(n);
for i in 0..n {
    let clamped = if !parents[0].dna()[i].ranges.is_empty() {
        let lo = T::to_f64(parents[0].dna()[i].ranges[0].0);
        let hi = T::to_f64(parents[0].dna()[i].ranges[0].1);
        offspring_vals[i].clamp(lo, hi)
    } else {
        offspring_vals[i]
    };
    let mut gene = parents[0].dna()[i].clone();
    gene.value = T::from_f64(clamped);
    child_dna.push(gene);
}
let mut child = RangeChromosome::<T>::new();
child.set_dna(Cow::Owned(child_dna));
Ok(vec![child])
```

---

### `src/operations/crossover/pcx.rs` (new operator, transform)

**Analog:** `src/operations/crossover/blend_alpha.rs` (lines 35-100) — shows the "per-gene spread-based perturbation" pattern clearly.

**Core algorithm pattern** (primary-parent-centric, perturbation in parent directions):
```rust
debug!(target="crossover_events", method="pcx"; "Starting PCX crossover with {} parents", parents.len());

let n = parents[0].dna().len();
let sigma_eta = 0.1_f64;   // perturbation along each direction vector
let sigma_zeta = 0.1_f64;  // orthogonal perturbation (approximated via spread)

let mut rng = crate::rng::make_rng();

// Per-gene spread across all parents
let spread: Vec<f64> = (0..n).map(|i| {
    let vals: Vec<f64> = parents.iter().map(|p| T::to_f64(p.dna()[i].value)).collect();
    let max_v = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_v = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    max_v - min_v
}).collect();

let mut child_dna = Vec::with_capacity(n);
for i in 0..n {
    let p0 = T::to_f64(parents[0].dna()[i].value);

    // Sum of perturbations along direction vectors (p[k] - p[0]) for k=1..n_par-1
    let directional: f64 = parents.iter().skip(1).map(|p| {
        let d = T::to_f64(p.dna()[i].value) - p0;
        let u1: f64 = rng.random_range(f64::EPSILON..1.0);
        let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        let noise = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_eta;
        noise * d
    }).sum::<f64>();

    // Orthogonal perturbation proportional to spread
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let zeta = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma_zeta * spread[i];

    let raw = p0 + directional + zeta;
    let clamped = if !parents[0].dna()[i].ranges.is_empty() {
        let lo = T::to_f64(parents[0].dna()[i].ranges[0].0);
        let hi = T::to_f64(parents[0].dna()[i].ranges[0].1);
        raw.clamp(lo, hi)
    } else {
        raw
    };
    let mut gene = parents[0].dna()[i].clone();
    gene.value = T::from_f64(clamped);
    child_dna.push(gene);
}
let mut child = RangeChromosome::<T>::new();
child.set_dna(Cow::Owned(child_dna));
Ok(vec![child])
```

---

### `src/operations/mutation/self_adaptive_gaussian.rs` (new operator, transform)

**Analog:** `src/operations/mutation/cauchy.rs` (lines 1-65)

**Imports pattern** (from `cauchy.rs` lines 1-16):
```rust
//! Self-adaptive Gaussian mutation for chromosomes implementing `SelfAdaptive`.

use crate::chromosomes::Range as RangeChromosome;
use crate::operations::mutation::gaussian::GaussianConvertible;
use crate::traits::{LinearChromosome, SelfAdaptive};
use log::debug;
use rand::Rng;
use std::fmt::Debug;
```

**Core function pattern** (from `cauchy.rs` lines 28-65, with SelfAdaptive logic added):
```rust
pub fn self_adaptive_gaussian_mutation<T>(
    individual: &mut RangeChromosome<T>,
    tau: f64,
    tau_prime: f64,
    sigma_min: f64,
) -> Result<(), GaError>
where
    T: Sync + Send + Clone + Default + Debug + PartialOrd + Copy + 'static + GaussianConvertible,
    RangeChromosome<T>: SelfAdaptive,
{
    let len = individual.dna().len();
    if len == 0 {
        return Ok(());
    }

    // Step 1: update all sigmas via log-normal rule (modifies self via trait default method)
    individual.adapt_strategy_params(tau, tau_prime, sigma_min);

    // Step 2: pick one random gene and mutate using its updated sigma
    let mut rng = crate::rng::make_rng();
    let idx = rng.random_range(0..len);
    let sigmas = individual.strategy_params();
    let sigma = sigmas.get(idx).copied().unwrap_or(1.0);

    let mut gene = individual.dna()[idx].clone();
    if gene.ranges.is_empty() {
        return Ok(());
    }
    let range_idx = rng.random_range(0..gene.ranges.len());
    let (lo, hi) = gene.ranges[range_idx];
    let current: f64 = T::to_f64(gene.value);
    let lo_f64 = T::to_f64(lo);
    let hi_f64 = T::to_f64(hi);

    // Box-Muller N(0, sigma) — from gaussian.rs lines 53-55
    let u1: f64 = rng.random_range(f64::EPSILON..1.0);
    let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
    let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;
    let new_val = (current + noise).clamp(lo_f64, hi_f64);

    gene.value = T::from_f64(new_val);
    individual.set_gene(idx, gene);

    debug!(
        target: "mutation_events",
        "SelfAdaptiveGaussian mutation applied at idx={} sigma={}",
        idx, sigma
    );
    Ok(())
}
```

---

### `src/operations/crossover.rs` (modified — add factory_multi_parent + mod decls)

**Analog:** `src/operations/selection.rs` lines 148-206 (`factory_lexicase`)

**New mod declarations** (add after existing `pub mod sbx;` at lines 36-38):
```rust
pub mod undx;
pub mod spx;
pub mod pcx;
```

**New try_undx dispatcher** (follows `try_sbx` pattern from `crossover.rs` lines 47-79, but with `&[&U]` slice):
```rust
fn try_undx<U: LinearChromosome>(
    parents: &[&U],
    _config: CrossoverConfiguration,
) -> Option<Result<Vec<U>, GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            // Downcast all parent refs to RangeChromosome<$t>
            let mut typed: Vec<&RangeChromosome<$t>> = Vec::with_capacity(parents.len());
            let mut all_match = true;
            for p in parents.iter() {
                if let Some(tp) = (*p as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    typed.push(tp);
                } else {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                let result = undx::undx(&typed, typed.len());
                return Some(result.map(|children| {
                    children.into_iter().map(|c| {
                        let boxed: Box<dyn Any> = Box::new(c);
                        *boxed.downcast::<U>().expect("type confirmed by downcast_ref")
                    }).collect()
                }));
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

**factory_multi_parent** (exact model: `factory_lexicase` from `selection.rs` lines 148-206):
```rust
/// Dispatch function for multi-parent crossover operators (UNDX, SPX, PCX).
///
/// Called from `ga.rs` when `CrossoverConfiguration::method` is one of the
/// multi-parent variants. Requires chromosomes implementing both
/// [`LinearChromosome`] and [`RealValued`].
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
        Crossover::Undx { .. } => {
            try_undx(parents, configuration).ok_or_else(|| {
                GaError::CrossoverError(
                    "UNDX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".into(),
                )
            })?
        }
        Crossover::Spx { .. } => {
            try_spx(parents, configuration).ok_or_else(|| {
                GaError::CrossoverError(
                    "SPX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".into(),
                )
            })?
        }
        Crossover::Pcx { .. } => {
            try_pcx(parents, configuration).ok_or_else(|| {
                GaError::CrossoverError(
                    "PCX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".into(),
                )
            })?
        }
        _ => Err(GaError::CrossoverError(
            "factory_multi_parent called with non-multi-parent crossover method".to_string(),
        )),
    }
}
```

Also add `Undx { .. } | Spx { .. } | Pcx { .. }` arms in `CrossoverOperator for Crossover` impl (lines 160-199) that return `GaError::CrossoverError("use factory_multi_parent")`.

---

### `src/operations/mutation.rs` (modified — add SelfAdaptiveGaussian arm + mod decl)

**Analog:** `src/operations/mutation.rs` lines 22-36 (mod decls) and lines 245-261 (Cauchy arm)

**New mod decl** (add after `pub mod swap;` at line 36):
```rust
pub mod self_adaptive_gaussian;
```

**New try_self_adaptive dispatcher** (follows `try_cauchy` from lines 63-84):
```rust
fn try_self_adaptive<U: LinearChromosome + 'static>(
    individual: &mut U,
    tau: f64,
    tau_prime: f64,
    sigma_min: f64,
) -> Option<Result<(), GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
                return Some(self_adaptive_gaussian::self_adaptive_gaussian_mutation(
                    ind, tau, tau_prime, sigma_min,
                ));
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

**New match arm** (add after LevyFlight arm at lines 254-261, following identical structure):
```rust
Mutation::SelfAdaptiveGaussian => {
    let n_hint = 1_usize;  // tau computed in adapt_strategy_params from params.len()
    let tau = step;         // step repurposed as tau when Some; None = compute from n in trait
    let tau_prime = sigma;  // sigma repurposed as tau_prime when Some
    let sigma_min_val = 1e-5_f64;  // hardcoded default; configuration field added separately
    // Note: actual tau/tau_prime/sigma_min come from MutationConfiguration fields
    // (self_adaptive_tau, self_adaptive_tau_prime, sigma_min) passed via factory_with_params
    return try_self_adaptive(individual, tau.unwrap_or(0.0), tau_prime.unwrap_or(0.0), sigma_min_val)
        .unwrap_or_else(|| {
            Err(GaError::MutationError(
                "SelfAdaptiveGaussian requires a chromosome implementing SelfAdaptive (RangeChromosome<T>).".to_string(),
            ))
        });
}
```

Note: The exact tau/tau_prime/sigma_min plumbing from `MutationConfiguration` new fields should go through an updated `factory_with_params` signature or via the `step`/`sigma` parameters. Follow the `Cauchy` precedent exactly — `step` carries `scale` for Cauchy, so `step`/`sigma` can carry `tau`/`tau_prime` here. Add a dedicated `factory_self_adaptive(individual, config)` helper if the 3-param needs are cleaner.

---

### `src/operations.rs` (modified — extend Crossover and Mutation enums)

**Analog:** `src/operations.rs` lines 84-130 (Crossover enum) and 136-191 (Mutation enum)

**Crossover enum additions** (after `MultiGroupOx` at line 130, same `#[derive(Copy, Clone, Debug, PartialEq)]` carries through since `usize: Copy`):
```rust
/// UNDX (Unimodal Normal Distribution Crossover) for real-valued chromosomes.
/// Requires chromosomes implementing [`RealValued`]. Uses `num_parents` parent
/// chromosomes; minimum 3. Dispatched via `crossover::factory_multi_parent()`.
Undx { num_parents: usize },
/// SPX (Simplex Crossover) for real-valued chromosomes.
/// Requires chromosomes implementing [`RealValued`]. Uses `num_parents` parent
/// chromosomes; minimum 3. Offspring sampled uniformly from expanded simplex interior.
Spx { num_parents: usize },
/// PCX (Parent-Centric Crossover) for real-valued chromosomes.
/// Requires chromosomes implementing [`RealValued`]. Uses `num_parents` parent
/// chromosomes; minimum 3. More exploitative than UNDX/SPX.
Pcx { num_parents: usize },
```

**Mutation enum addition** (after `Uniform` at line 191):
```rust
/// Self-adaptive Gaussian mutation for chromosomes implementing [`SelfAdaptive`].
/// Per-chromosome sigma vector co-evolves via the ES log-normal update rule before
/// each gene mutation. Configure τ, τ', and σ_min via `MutationConfiguration`.
SelfAdaptiveGaussian,
```

**Serde note:** Both enums have `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` at lines 84 and 136. `usize` is serde-safe, so struct variants serialize fine.

---

### `src/configuration.rs` (modified — add new config fields)

**Analog:** `src/configuration.rs` lines 133-222 (`CrossoverConfiguration` and `MutationConfiguration`)

**MutationConfiguration new fields** (add after `levy_alpha: Option<f64>` at line 193, following identical doc + Option<f64> pattern):
```rust
/// Global learning rate τ' for `Mutation::SelfAdaptiveGaussian`.
/// Default (when `None`): `1.0 / sqrt(2.0 * sqrt(n))` where n = `strategy_params().len()`.
/// Only consulted when `method == Mutation::SelfAdaptiveGaussian`.
pub self_adaptive_tau_prime: Option<f64>,
/// Per-dimension learning rate τ for `Mutation::SelfAdaptiveGaussian`.
/// Default (when `None`): `1.0 / sqrt(2.0 * n)` where n = `strategy_params().len()`.
/// Only consulted when `method == Mutation::SelfAdaptiveGaussian`.
pub self_adaptive_tau: Option<f64>,
/// Sigma lower bound for `Mutation::SelfAdaptiveGaussian`.
/// Sigmas are clamped to this value after each log-normal update.
/// Default (when `None`): `1e-5`.
pub sigma_min: Option<f64>,
```

Add `None` defaults for each in `impl Default for MutationConfiguration` (lines 204-222).

**CrossoverConfiguration:** No new fields needed — `num_parents` lives in the enum variants themselves per D-04 (Pitfall 4 in RESEARCH.md).

---

### `src/traits.rs` (modified — add re-exports)

**Analog:** `src/traits.rs` lines 48-64 (existing re-export block)

**Additions** (add alongside `MultiCaseFitness` re-export at line 53):
```rust
pub mod real_valued;
pub mod self_adaptive;
pub use multi_case_fitness::MultiCaseFitness;
pub use real_valued::RealValued;
pub use self_adaptive::SelfAdaptive;
```

Pattern to follow: `pub use multi_case_fitness::MultiCaseFitness;` at line 53 — same one-liner.

---

### `src/types/chromosomes/range.rs` (modified — add strategy_params field + trait impls)

**Analog:** `src/types/chromosomes/range.rs` lines 34-177 (existing struct + impls)
**Trait impl pattern:** `tests/structures.rs` lines 166-174 (MultiCaseFitness impl as minimal two-method pattern)

**Struct change** (add field after `fitness_fn` at line 48):
```rust
/// Per-gene step sizes for `Mutation::SelfAdaptiveGaussian`.
/// Initialized to `vec![1.0; dna.len()]` in `set_dna()` when not yet populated.
/// Included in serde serialization so evolved sigmas survive checkpoint save/restore.
pub strategy_params: Vec<f64>,
```

Add `strategy_params: Vec::new()` to both `Default` (line 59) and `new()` (line 83).

**Serde gate for new field** (follows `#[cfg_attr(feature = "serde", serde(skip, default))]` pattern at line 47 for `fitness_fn`):
- The `strategy_params` field IS included in serde (D-14) — no `serde(skip)` needed.
- The struct-level `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` at lines 35-42 already covers new fields automatically if `Vec<f64>` (which is serde-compatible).

**set_dna sigma init** (add to `set_dna` at line 156 — trigger sigma init here to avoid `&self` mut issue):
```rust
fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
    self.dna = match dna {
        Cow::Borrowed(slice) => slice.to_vec(),
        Cow::Owned(vec) => vec,
    };
    // Initialize strategy_params to 1.0 per gene if not already set
    if self.strategy_params.is_empty() && !self.dna.is_empty() {
        self.strategy_params = vec![1.0; self.dna.len()];
    }
    self
}
```

**RealValued impl** (add after `OperatorCompat` impl at line 54):
```rust
impl<T: Sync + Send + Copy + Default + Debug + 'static> RealValued for Range<T> {}
```

**SelfAdaptive impl** (add as new impl block, modeled on `tests/structures.rs` lines 166-174):
```rust
impl<T: Sync + Send + Copy + Default + Debug + 'static> SelfAdaptive for Range<T> {
    fn strategy_params(&self) -> &[f64] {
        &self.strategy_params
    }

    fn set_strategy_params(&mut self, params: Vec<f64>) {
        self.strategy_params = params;
    }
    // adapt_strategy_params: use default impl from trait (log-normal update)
}
```

---

### `src/types/chromosomes/multi_range.rs` (modified — add RealValued impl stub)

**Analog:** `src/types/chromosomes/range.rs` `OperatorCompat` impl at line 54

**Addition** (empty impl block, same pattern as `impl<T> OperatorCompat for Range<T> {}`):
```rust
impl<T: Sync + Send + Copy + Default + Debug + 'static> RealValued for MultiRangeChromosome<T> {}
```

Note: Phase 48 implements `SelfAdaptive` for `MultiRangeChromosome`. This phase adds only the `RealValued` marker — no body needed.

---

### `src/engines/ga.rs` (modified — multi-parent if/else dispatch branch)

**Analog:** `src/engines/ga.rs` lines 2533-2552 (existing crossover if/else in `process_pair` closure)

**Insertion point:** Inside the `process_pair` closure at line 2533, replace the current `crossover::factory(...)` call with an if/else:

```rust
// BEFORE (line 2534-2541 in standard path):
let mut children = if let Some((_op_idx, cx_op)) = selected_crossover {
    let mut cx_config = configuration.crossover_configuration;
    cx_config.method = cx_op;
    crossover::factory(parent_1, parent_2, cx_config)?
} else {
    crossover::factory(parent_1, parent_2, configuration.crossover_configuration)?
};

// AFTER — wrap with multi-parent check (mirror of ga.rs lexicase if/else):
let effective_method = selected_crossover.map(|(_, op)| op)
    .unwrap_or(configuration.crossover_configuration.method);

let mut children = match effective_method {
    Crossover::Undx { num_parents } | Crossover::Spx { num_parents } | Crossover::Pcx { num_parents } => {
        // Collect primary pair + (num_parents - 2) random extra parents
        let mut parent_refs: Vec<&U> = vec![parent_1, parent_2];
        let extras = num_parents.saturating_sub(2);
        for _ in 0..extras {
            let idx = rng.random_range(0..chromosomes.len());
            parent_refs.push(&chromosomes[idx]);
        }
        let mut cx_config = configuration.crossover_configuration;
        cx_config.method = effective_method;
        // Returns 1 offspring — handle 1-vs-2 mismatch below
        crossover::factory_multi_parent(&parent_refs, cx_config)?
    }
    _ => {
        if let Some((_op_idx, cx_op)) = selected_crossover {
            let mut cx_config = configuration.crossover_configuration;
            cx_config.method = cx_op;
            crossover::factory(parent_1, parent_2, cx_config)?
        } else {
            crossover::factory(parent_1, parent_2, configuration.crossover_configuration)?
        }
    }
};

// Handle 1-vs-2 offspring mismatch (D-04 / Pitfall 1 in RESEARCH.md)
// multi-parent path returns 1 child; standard path returns 2
child_2 = children.pop().ok_or_else(|| {
    GaError::CrossoverError("Crossover returned fewer than 2 children".to_string())
})?;
child_1 = children.pop().unwrap_or_else(|| parent_1.clone());
// ^ if only 1 child (multi-parent), child_1 = that child, child_2 = parent_1 clone
// Swap assignment: pop order gives child_2 first when 2 exist (sbx.rs convention)
// For single offspring: pop() → child, then pop() → None → parent_1.clone()
```

Note: The existing pop order (lines 2543-2548) pops `child_2` first then `child_1`. For multi-parent single offspring, invert: first pop is `child_1`, then `child_2 = parent_1.clone()`.

**WASM cfg gate:** The insertion is inside the `process_pair` closure body — this is correct (Pitfall 5). Only the iterator `.par_iter()` vs `.iter()` is cfg-gated, not the closure body. No changes to cfg gates needed.

---

## Test Patterns

### `tests/operations/test_crossover_undx.rs` (new test file)

**Analog:** `tests/operations/test_crossover_sbx.rs` (lines 1-143)

**Imports pattern** (from `test_crossover_sbx.rs` lines 1-5):
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::undx::undx;
use genetic_algorithms::traits::LinearChromosome;
use std::borrow::Cow;
```

**Builder helper pattern** (from `test_crossover_sbx.rs` lines 7-23, extended to 3+ parents):
```rust
fn build_parents(n: usize) -> Vec<RangeChromosome<f64>> {
    (0..n).map(|k| {
        let mut p = RangeChromosome::<f64>::new();
        let dna = vec![
            RangeGenotype::new(0, vec![(0.0, 100.0)], 10.0 + 20.0 * k as f64),
            RangeGenotype::new(1, vec![(0.0, 100.0)], 80.0 - 10.0 * k as f64),
        ];
        p.set_dna(Cow::Owned(dna));
        p
    }).collect()
}
```

**Test pattern** (from `test_crossover_sbx.rs` lines 34-51 — bounds check loop):
```rust
#[test]
fn undx_produces_one_offspring_within_bounds() {
    let parents = build_parents(3);
    let refs: Vec<&RangeChromosome<f64>> = parents.iter().collect();
    for _ in 0..100 {
        let children = undx(&refs, 3).unwrap();
        assert_eq!(children.len(), 1);
        for gene in children[0].dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(gene.value >= lo && gene.value <= hi);
        }
    }
}
```

Same pattern for `test_crossover_spx.rs` and `test_crossover_pcx.rs`.

### `tests/operations/test_mutation_self_adaptive.rs` (new test file)

**Analog:** `tests/operations/test_mutation_cauchy_levy_uniform.rs` (lines 1-60)

**Imports pattern**:
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::mutation::self_adaptive_gaussian::self_adaptive_gaussian_mutation;
use genetic_algorithms::traits::{LinearChromosome, SelfAdaptive};
use std::borrow::Cow;
```

**Test pattern** (from `test_mutation_cauchy_levy_uniform.rs` lines 31-44 — value changes loop):
```rust
#[test]
fn self_adaptive_sigma_min_enforced() {
    let mut c = build_f64_chromosome(4);
    // Initialize to very small sigmas
    c.set_strategy_params(vec![1e-8; 4]);
    for _ in 0..100 {
        self_adaptive_gaussian_mutation(&mut c, 0.0, 0.0, 1e-5).unwrap();
        for &s in c.strategy_params() {
            assert!(s >= 1e-5, "sigma {} below sigma_min 1e-5", s);
        }
    }
}
```

---

## Shared Patterns

### Box-Muller N(0,1) sampling
**Source:** `src/operations/mutation/gaussian.rs` lines 53-55
**Apply to:** `undx.rs`, `spx.rs` (not needed), `pcx.rs`, `self_adaptive.rs`, `self_adaptive_gaussian.rs`
```rust
let u1: f64 = rng.random_range(f64::EPSILON..1.0);
let u2: f64 = rng.random_range(0.0..std::f64::consts::TAU);
let noise: f64 = (-2.0 * u1.ln()).sqrt() * u2.cos() * sigma;
```

### Gene range clamping
**Source:** `src/operations/crossover/sbx.rs` lines 80-86
**Apply to:** `undx.rs`, `spx.rs`, `pcx.rs`
```rust
let clamped = if !dna[i].ranges.is_empty() {
    let lo: f64 = T::to_f64(dna[i].ranges[0].0);
    let hi: f64 = T::to_f64(dna[i].ranges[0].1);
    raw_val.clamp(lo, hi)
} else {
    raw_val
};
```

### Offspring construction via Cow::Owned
**Source:** `src/operations/crossover/sbx.rs` lines 97-103
**Apply to:** `undx.rs`, `spx.rs`, `pcx.rs`
```rust
let mut child = RangeChromosome::<T>::new();
child.set_dna(Cow::Owned(child_dna));
```

### Any downcast macro for type-specific operators
**Source:** `src/operations/mutation.rs` lines 67-84 (`try_cauchy`)
**Apply to:** `src/operations/mutation.rs` (`try_self_adaptive`), `src/operations/crossover.rs` (`try_undx`, `try_spx`, `try_pcx`)
```rust
macro_rules! try_type {
    ($t:ty) => {
        if let Some(ind) = (individual as &mut dyn Any).downcast_mut::<RangeChromosome<$t>>() {
            // ... call typed function
            return Some(result);
        }
    };
}
try_type!(f64);
try_type!(f32);
try_type!(i32);
try_type!(i64);
```

### Debug logging convention
**Source:** `src/operations/crossover/sbx.rs` line 47 and line 102
**Apply to:** All new operator files
```rust
debug!(target="crossover_events", method="undx"; "Starting UNDX crossover with {} parents", parents.len());
// ... at end:
debug!(target="crossover_events", method="undx"; "UNDX crossover finished");
```

### Option<f64> config field pattern
**Source:** `src/configuration.rs` lines 188-193 (`cauchy_scale`, `levy_alpha`)
**Apply to:** `MutationConfiguration` new fields (`self_adaptive_tau`, `self_adaptive_tau_prime`, `sigma_min`)
```rust
pub cauchy_scale: Option<f64>,  // → same pattern for self_adaptive_tau, etc.
```

---

## No Analog Found

All files have close analogs. No entries.

---

## Metadata

**Analog search scope:** `src/traits/`, `src/operations/crossover/`, `src/operations/mutation/`, `src/operations/selection.rs`, `src/types/chromosomes/`, `src/engines/ga.rs`, `src/configuration.rs`, `tests/operations/`, `tests/structures.rs`
**Files scanned:** 18
**Pattern extraction date:** 2026-05-23
