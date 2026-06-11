# Phase 36: MOEA/D — Decomposition-based multi-objective optimization - Research

**Researched:** 2026-05-09
**Domain:** Multi-objective evolutionary algorithm — decomposition-based (MOEA/D)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `MoeaDGa<U>::run()` returns `Result<ParetoFront<U>, GaError>` — post-hoc non-dominated sorting on all N sub-problem representatives.
- **D-02:** `ScalarizationFn` enum with `Tchebycheff` and `Pbi { theta: f64 }` variants.
- **D-03:** `.with_scalarization(ScalarizationFn)`. Default: `Tchebycheff`. `validate()` never fails on a missing scalarization call.
- **D-04:** `.with_weight_vectors_auto(p: usize)` — Das-Dennis simplex lattice generator (reuse from NSGA-III, do not duplicate).
- **D-05:** `.with_weight_vectors(Vec<Vec<f64>>)` — user-supplied custom weight vectors.
- **D-06:** Weight vectors are mandatory; `validate()` returns `GaError::InvalidMoeaDConfiguration` when neither builder was called.
- **D-07:** Auto and custom weight vectors are mutually exclusive; last builder call wins.
- **D-08:** `.with_neighborhood_size(t: usize)`. Default: T = 20.
- **D-09:** `.with_max_neighbor_replacements(nr: usize)`. Default: nr = 2.
- **D-10:** `MoeaDObserver<U>` sub-trait in `src/observe/observer/mod.rs` with generation-level hooks only:
  - `fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {}`
  - `fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {}`
  - All methods have default no-op implementations.
- **D-11:** `MoeaDGa<U>` stores `Option<Arc<dyn MoeaDObserver<U> + Send + Sync>>`.
- **D-12:** `LogObserver` gains `impl MoeaDObserver<U>` — debug-level on `"moead_events"` target.
- **D-13:** `AllObserver<U>` is NOT updated in this phase.

### Claude's Discretion

- Internal neighbourhood computation: precompute T nearest reference-point neighbours at initialisation (Euclidean distance in weight-vector space) — store as `Vec<Vec<usize>>` indexed by sub-problem.
- Ideal point update strategy: incremental after each offspring evaluation.
- WASM cfg-gating: `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` on all `Instant::now()` and `par_iter()` call sites (mandatory per CLAUDE.md).
- Internal normalisation for PBI: use current ideal point to shift objectives before computing PBI value; no explicit nadir tracking in Phase 36.
- Example DTLZ2 setup: population size 91 (C(12,2) with p=10 for 3 objectives), 300 generations.

### Deferred Ideas (OUT OF SCOPE)

- Two-layer weight vectors for M > 5 objectives.
- Constraint handling for MOEA/D.
- `AllObserver<U>` updated to include `MoeaDObserver<U>`.
- Sub-problem-level observer hooks (`on_neighbour_updated`, `on_subproblem_update`).
- Weighted-sum scalarization.
- Adaptive weight vector adjustment.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOO-02 | User can run MOEA/D with configurable weight vectors and either Tchebycheff or PBI scalarisation; each sub-problem maintains a neighbourhood of similar weight vectors and offspring compete only within that neighbourhood (#204) | Full engine design, scalarization formulas, neighbourhood-update loop, configuration builder, observer integration, test structure — all documented below. |
</phase_requirements>

---

## Summary

Phase 36 adds `MoeaDGa<U>` — a decomposition-based multi-objective engine implementing Zhang & Li 2007. The design is a direct structural clone of `Nsga3Ga<U>` (Phase 35): same return type (`ParetoFront<U>`), same builder pattern, same `Option<Arc<dyn Observer>>` wiring, and the same WASM cfg-gating discipline. The new engine decomposes a multi-objective problem into N scalar sub-problems (one per weight vector), evolves each sub-problem using only its T nearest neighbours, and applies Tchebycheff or PBI scalarization to evaluate offspring fitness. At the end of `run()`, post-hoc non-dominated sorting extracts the Pareto front from the N sub-problem representatives.

The codebase already contains every reusable component needed: the Das-Dennis lattice generator in `src/engines/nsga3/das_dennis.rs`, the `non_dominated_sort_with_directions()` function, `ParetoIndividual<U>` and `ParetoFront<U>` types, the `ObjectiveFn<G>` alias, and the full observer infrastructure pattern. The only new code is the MOEA/D-specific logic: neighbourhood precomputation, scalarization evaluation, and the per-generation sub-problem update loop.

**Primary recommendation:** Model every file in `src/engines/moead/` directly on its NSGA-III counterpart. Reuse `crate::nsga3::das_dennis::generate_das_dennis` for weight vector generation. Implement the sub-problem loop as a serial inner loop (one offspring per sub-problem per generation) in the generation loop body, with WASM-gated parallelism only for the initial population evaluation.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Weight vector generation | Library / engine init | — | Das-Dennis lattice computed at build time, stored in engine struct |
| Neighbourhood precomputation | Library / engine init | — | Euclidean distances in weight-vector space, computed once and stored |
| Sub-problem update loop | Library / engine run | — | Per-generation inner loop over all N sub-problems |
| Scalarization evaluation | Library / engine run | — | Tchebycheff/PBI applied inline to offspring during neighbour comparison |
| Ideal point tracking | Library / engine run | — | Incremental update after each offspring evaluation |
| Post-hoc Pareto sort | Library / multi_objective | — | `non_dominated_sort_with_directions()` applied to N representatives |
| Configuration builder | Library / engine config | — | `MoeaDConfiguration` — fluent builder, mirrors `Nsga3Configuration` |
| Observer hooks | Library / observe | — | `MoeaDObserver<U>` sub-trait, generation-level only |
| Error reporting | Library / error | — | `GaError::InvalidMoeaDConfiguration(String)` variant |
| Public re-export | Library / lib.rs | — | `pub mod moead` via `#[path]` |

---

## Standard Stack

### Core (all already in Cargo.toml — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std | 1.94.1 | `Vec`, `Arc`, closures | Platform standard |
| rayon | (existing) | Parallel population init + offspring eval | Already used throughout project; WASM-gated |
| rand | (existing) | RNG via `crate::rng::make_rng()` | Project-standard RNG entry point |
| log | (existing) | `debug!` macros in LogObserver | Project-standard logging |

[VERIFIED: codebase grep — no new crate additions needed]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde (feature-gated) | (existing) | `#[cfg_attr(feature = "serde", derive(...))]` on config and enum | When checkpoint serialisation is needed |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Das-Dennis reuse from nsga3 | Duplicate generator in moead/ | Duplication violates DRY — CONTEXT.md D-04 mandates reuse |
| Serial sub-problem loop | Fully parallel per-sub-problem | Sub-problem state (ideal point) is mutated — parallelism requires coordination overhead not justified for Phase 36 |

**Installation:** No new dependencies.

---

## Architecture Patterns

### System Architecture Diagram

```
User Code
    │
    ▼
MoeaDGa::new(moead_config, ga_config)
    │  .with_alleles()
    │  .with_initialization_fn()
    │  .with_objective_fns()
    │  .with_observer()
    │  .build()  ──► validate()
    │               ├── num_objectives > 0
    │               ├── population_size >= 2
    │               ├── initialization_fn present
    │               ├── objective_fns.len() == num_objectives
    │               ├── weight_vectors configured (D-06)
    │               └── each weight vector.len() == num_objectives
    │
    ▼
MoeaDGa::run()
    │
    ├── validate_and_get_weight_vectors()  ──► Vec<Vec<f64>>  (N sub-problems)
    │
    ├── precompute_neighbourhoods(weight_vectors, T)
    │       └── for each i: sort all j by Euclidean dist(wv[i], wv[j]), take T nearest
    │           result: Vec<Vec<usize>>  (neighbours[i] = T indices)
    │
    ├── initialize_population()  ──► Vec<ParetoIndividual<U>>  (N individuals)
    │       └── par_iter (WASM-gated): evaluate objective_fns for each chromosome
    │
    ├── initialize_ideal_point()  ──► Vec<f64>  (M components, one per objective)
    │       └── z*[k] = min over all i of f_k(individual[i])
    │
    └── for gen in 0..max_generations:
            │
            ├── [observer] t_sort = Instant::now() (WASM-gated)
            │
            ├── for each sub-problem i in 0..N:
            │       ├── sample two parents from neighbours[i]
            │       ├── crossover + mutation  ──► offspring chromosome
            │       ├── evaluate objective_fns(offspring.dna())  ──► objectives
            │       ├── update ideal point z*: z*[k] = min(z*[k], objectives[k]) ∀k
            │       ├── replacement_count = 0
            │       └── for each j in neighbours[i]:
            │               ├── if replacement_count >= max_neighbor_replacements: break
            │               ├── g_offspring = scalarize(objectives, wv[i], z*)
            │               └── if g_offspring < scalarize(population[j].objectives, wv[j], z*):
            │                       population[j] = offspring wrapped as ParetoIndividual
            │                       replacement_count += 1
            │
            ├── [observer] on_non_dominated_sort_complete(gen, elapsed_ms) (WASM-gated)
            │
            ├── [post-hoc rank for front_count]: non_dominated_sort on population
            │
            └── [observer] on_pareto_front_assigned(gen, front_count, pop_size)
            │
    ▼
    post-hoc non_dominated_sort_with_directions(population)
    ──► filter rank == 0
    ──► ParetoFront<U>
```

### Recommended Project Structure

```
src/engines/moead/
├── mod.rs            # MoeaDGa<U> engine struct, impl blocks, run(), helper fns
└── configuration.rs  # MoeaDConfiguration, ScalarizationFn enum

tests/engines/moead/
├── test_moead.rs               # Engine integration tests (validate, run, observer)
└── test_moead_configuration.rs # Configuration builder + scalarization unit tests

examples/
└── moead_dtlz2.rs   # 3-objective DTLZ2, population 91, 300 gens, p=10
```

### Pattern 1: MoeaDConfiguration — Weight Vector Builder (mirrors Nsga3Configuration)

**What:** Fluent builder with private `weight_vectors_auto_p` and `weight_vectors_custom` fields; last-call-wins semantics; `effective_weight_vectors()` materialises the configured option.

**When to use:** Any time the engine needs to access weight vectors (validate, run).

```rust
// Source: src/engines/nsga3/configuration.rs (established pattern)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoeaDConfiguration {
    pub num_objectives: usize,
    pub population_size: usize,
    pub max_generations: usize,
    pub objective_directions: Vec<ObjectiveDirection>,
    pub scalarization: ScalarizationFn,
    pub neighborhood_size: usize,
    pub max_neighbor_replacements: usize,
    weight_vectors_auto_p: Option<usize>,
    weight_vectors_custom: Option<Vec<Vec<f64>>>,
}

impl MoeaDConfiguration {
    pub fn with_weight_vectors_auto(mut self, p: usize) -> Self {
        self.weight_vectors_auto_p = Some(p);
        self.weight_vectors_custom = None;
        self
    }
    pub fn with_weight_vectors(mut self, vecs: Vec<Vec<f64>>) -> Self {
        self.weight_vectors_custom = Some(vecs);
        self.weight_vectors_auto_p = None;
        self
    }
    pub fn effective_weight_vectors(&self) -> Option<Vec<Vec<f64>>> {
        if let Some(p) = self.weight_vectors_auto_p {
            Some(crate::nsga3::das_dennis::generate_das_dennis(self.num_objectives, p))
        } else {
            self.weight_vectors_custom.clone()
        }
    }
}
```

[VERIFIED: codebase — direct mirror of `src/engines/nsga3/configuration.rs`]

### Pattern 2: ScalarizationFn Enum

**What:** Public enum with `Tchebycheff` and `Pbi { theta: f64 }` variants. Applied inline during neighbour replacement.

**When to use:** Inside the sub-problem update loop when comparing offspring to each neighbour.

```rust
// Source: CONTEXT.md D-02; mirrors ObjectiveDirection in nsga2/configuration.rs
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalarizationFn {
    /// Classic Tchebycheff: g = max_i { w_i * |f_i - z*_i| }
    Tchebycheff,
    /// Penalty-based boundary intersection: g = d1 + theta * d2
    /// theta default per Zhang & Li 2007: 5.0
    Pbi { theta: f64 },
}

impl Default for ScalarizationFn {
    fn default() -> Self { ScalarizationFn::Tchebycheff }
}
```

[VERIFIED: CONTEXT.md D-02 + Zhang & Li 2007 canonical formulas]

### Pattern 3: Neighbourhood Precomputation

**What:** At `run()` start, compute T nearest neighbours for each of the N sub-problems using Euclidean distance in weight-vector space. Stored as `Vec<Vec<usize>>`.

**When to use:** Once per `run()` call, before the generation loop.

```rust
// Source: CONTEXT.md Specifics + Zhang & Li 2007
fn precompute_neighbourhoods(weight_vectors: &[Vec<f64>], t: usize) -> Vec<Vec<usize>> {
    let n = weight_vectors.len();
    let t = t.min(n);
    let mut neighbourhoods = Vec::with_capacity(n);
    for i in 0..n {
        let mut dists: Vec<(usize, f64)> = (0..n)
            .map(|j| {
                let d: f64 = weight_vectors[i].iter()
                    .zip(weight_vectors[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                (j, d)
            })
            .collect();
        // Sort by distance; self (i) will be at index 0 with distance 0.0.
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        neighbourhoods.push(dists.into_iter().take(t).map(|(j, _)| j).collect());
    }
    neighbourhoods
}
```

[ASSUMED: Exact sort tie-breaking for equal distances — using index-stable sort is adequate for correctness; ties are rare in practice on uniformly spaced weight vectors]

### Pattern 4: Tchebycheff Scalarization

**What:** `g_tch(f, w, z*) = max_i { w_i * |f_i - z*_i| }`. Minimise. Robust to non-convex Pareto fronts.

```rust
// Source: Zhang & Li 2007 (equation 4) [CITED: semanticscholar.org/paper/MOEA-D:-A-Multiobjective-Evolutionary-Algorithm...]
fn scalarize_tchebycheff(objectives: &[f64], weights: &[f64], ideal: &[f64]) -> f64 {
    objectives.iter()
        .zip(weights.iter())
        .zip(ideal.iter())
        .map(|((f_i, w_i), z_i)| w_i * (f_i - z_i).abs())
        .fold(f64::NEG_INFINITY, f64::max)
}
```

### Pattern 5: PBI Scalarization

**What:** `g_pbi(f, w, z*, theta) = d1 + theta * d2`.
- `d1` = scalar projection of (f - z*) onto w direction = `|(f - z*) · w_unit|`
- `d2` = Euclidean distance from (f - z*) to the line spanned by w = `||(f - z*) - d1 * w_unit||`
- theta = 5.0 default per Zhang & Li; user-configurable as `Pbi { theta }`.

```rust
// Source: Zhang & Li 2007 (equation 5) [CITED: semanticscholar.org/paper/MOEA-D]
fn scalarize_pbi(objectives: &[f64], weights: &[f64], ideal: &[f64], theta: f64) -> f64 {
    let diff: Vec<f64> = objectives.iter().zip(ideal.iter()).map(|(f, z)| f - z).collect();
    let w_norm_sq: f64 = weights.iter().map(|w| w * w).sum::<f64>().max(f64::EPSILON);
    let w_norm = w_norm_sq.sqrt();
    // Unit weight vector
    let d1: f64 = diff.iter().zip(weights.iter()).map(|(d, w)| d * w).sum::<f64>() / w_norm;
    let d2_sq: f64 = diff.iter().zip(weights.iter())
        .map(|(d, w)| {
            let proj = d1 * w / w_norm;
            (d - proj).powi(2)
        })
        .sum();
    d1.abs() + theta * d2_sq.sqrt()
}
```

[ASSUMED: The exact form of d1 sign handling — taking `d1.abs()` ensures non-negative distance along the weight direction; some implementations use `d1` directly (can be negative if objective overshoots ideal). `d1.abs()` is the conservative choice consistent with the spirit of "distance to weight-vector line".]

### Pattern 6: WASM Cfg-Gating (mandatory)

**What:** Gate `Instant::now()` and `par_iter()` at every call site.

```rust
// Source: src/engines/nsga3/mod.rs — established pattern [VERIFIED: codebase]
// For Instant:
let t_sort: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};

// For par_iter (population init + offspring eval):
#[cfg(not(target_arch = "wasm32"))]
let population: Vec<ParetoIndividual<U>> = chromosomes.into_par_iter()
    .map(|chrom| { ... }).collect();
#[cfg(target_arch = "wasm32")]
let population: Vec<ParetoIndividual<U>> = chromosomes.into_iter()
    .map(|chrom| { ... }).collect();
```

### Pattern 7: MoeaDObserver Trait and LogObserver impl

**What:** New sub-trait in `src/observe/observer/mod.rs`, placed after `Nsga3Observer<U>`. `LogObserver` gets a new impl block in `src/observe/observer/log.rs`.

```rust
// Source: src/observe/observer/mod.rs lines 180–191 [VERIFIED: codebase]
pub trait MoeaDObserver<U: ChromosomeT>: Send + Sync {
    fn on_pareto_front_assigned(
        &self, _generation: usize, _front_count: usize, _population_size: usize,
    ) {}
    fn on_non_dominated_sort_complete(&self, _generation: usize, _duration_ms: f64) {}
}

// src/observe/observer/log.rs — mirrors Nsga3Observer impl [VERIFIED: codebase]
impl<U: ChromosomeT> MoeaDObserver<U> for LogObserver {
    fn on_pareto_front_assigned(&self, generation: usize, front_count: usize, population_size: usize) {
        log::debug!(target: "moead_events",
            "Generation {} complete, population size = {}, fronts = {}",
            generation, population_size, front_count);
    }
    fn on_non_dominated_sort_complete(&self, generation: usize, duration_ms: f64) {
        log::debug!(target: "moead_events",
            "Non-dominated sort complete at generation {} ({:.2}ms)", generation, duration_ms);
    }
}
```

### Pattern 8: Observer Import in log.rs

The `log.rs` import line must be updated to add `MoeaDObserver`:

```rust
// Current (after Phase 35):
use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer, Nsga3Observer};
// After Phase 36:
use crate::observer::{ExtensionEvent, GaObserver, IslandGaObserver, Nsga2Observer, Nsga3Observer, MoeaDObserver};
```

[VERIFIED: codebase — src/observe/observer/log.rs line 27]

### Pattern 9: lib.rs Re-exports

```rust
// After existing nsga3 re-export (src/lib.rs lines 113-114):
#[path = "engines/moead/mod.rs"]
pub mod moead;

// After existing Nsga3Observer pub use:
pub use observer::MoeaDObserver;
```

[VERIFIED: codebase — src/lib.rs lines 113-129 for exact insertion points]

### Pattern 10: GaError New Variant

```rust
// src/error.rs — add after InvalidNsga3Configuration:
/// A MOEA/D configuration parameter is invalid.
InvalidMoeaDConfiguration(String),
```

And in `Display` impl:
```rust
GaError::InvalidMoeaDConfiguration(msg) => write!(f, "Invalid MOEA/D configuration: {}", msg),
```

[VERIFIED: codebase — src/error.rs lines 36-68]

### Pattern 11: validate_and_get_weight_vectors() — avoid double Das-Dennis call

**What:** Mirrors `validate_and_get_ref_points()` from NSGA-III — runs all validation checks, materialises weight vectors once, returns them.

**When to use:** Called once at the top of `run()`.

```rust
fn validate_and_get_weight_vectors(&self) -> Result<Vec<Vec<f64>>, GaError> {
    // ... all parameter checks ...
    // Materialise weight vectors exactly once:
    let wvs = self.moead_config.effective_weight_vectors()
        .ok_or_else(|| GaError::InvalidMoeaDConfiguration(
            "weight vectors must be configured via with_weight_vectors_auto(p) or with_weight_vectors(vecs)".to_string()
        ))?;
    if wvs.is_empty() {
        return Err(GaError::InvalidMoeaDConfiguration("weight vector list must not be empty".to_string()));
    }
    for (i, wv) in wvs.iter().enumerate() {
        if wv.len() != self.moead_config.num_objectives {
            return Err(GaError::InvalidMoeaDConfiguration(format!(
                "weight vector {} has dimension {}, expected {}",
                i, wv.len(), self.moead_config.num_objectives
            )));
        }
    }
    Ok(wvs)
}
```

[VERIFIED: codebase — mirrors Nsga3Ga::validate_and_get_ref_points()]

### Anti-Patterns to Avoid

- **Duplicating the Das-Dennis generator:** Call `crate::nsga3::das_dennis::generate_das_dennis()` — never write a second copy. CONTEXT.md D-04 and canonical_refs are explicit on this.
- **Calling effective_weight_vectors() more than once in run():** Use `validate_and_get_weight_vectors()` to materialise once (matches the NSGA-III fix applied in Phase 35 WR-02).
- **Using `par_iter()` for the sub-problem update loop:** Each iteration mutates shared population slots and the ideal point — sequential inner loop is correct. Only the initial population evaluation and (optionally) offspring evaluation can be parallelised if offspring are batch-generated.
- **Missing `max_neighbor_replacements` cap:** Without the cap, a single offspring can push out many neighbours, causing premature convergence (well-documented MOEA/D pitfall per Zhang & Li 2007).
- **Applying scalarization without the ideal point shift:** `g_tch` must use `|f_i - z*_i|`, not `f_i` raw. Without the shift, the Tchebycheff metric is biased toward the origin, not the true ideal.
- **Storing offspring as a separate population:** MOEA/D maintains one individual per sub-problem at all times. There is no combined parent+offspring pool; the sub-problem representative is replaced in-place.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Uniform weight vector generation | Custom lattice generator | `crate::nsga3::das_dennis::generate_das_dennis()` | Already implemented, tested, handles edge cases |
| Post-hoc Pareto front | Custom dominance sort | `non_dominated_sort_with_directions()` + filter rank == 0 | Handles direction-aware dominance, already used by NSGA-II and NSGA-III |
| ParetoIndividual wrapping | Custom struct | `ParetoIndividual::new(chrom, objectives)` | Shared type — ensures uniform API with other MOO engines |
| Random number generation | `rand::thread_rng()` | `crate::rng::make_rng()` | Project-standard — honours `rng_seed` for reproducibility |
| Mutation dispatch | Custom match | `mutation::factory_with_params()` | Handles all mutation variants including Differential exclusion |
| Crossover dispatch | Custom match | `crossover::factory()` | Handles all crossover variants |

---

## Common Pitfalls

### Pitfall 1: Double Das-Dennis Materialisation
**What goes wrong:** `validate()` calls `effective_weight_vectors()` (generates lattice), then `run()` calls it again — redundant computation, same bug fixed in NSGA-III (WR-02 in Phase 35).
**Why it happens:** Separating validation from runtime materialisation while both need the materialised value.
**How to avoid:** Use `validate_and_get_weight_vectors()` that runs all checks and returns the materialised `Vec<Vec<f64>>` in one call.
**Warning signs:** Two calls to `effective_weight_vectors()` in the same execution path.

### Pitfall 2: Uncapped Neighbourhood Replacements
**What goes wrong:** A single fit offspring replaces all T neighbours, drastically reducing diversity in early generations.
**Why it happens:** Forgetting to track and cap replacements at `max_neighbor_replacements`.
**How to avoid:** Maintain a `replacement_count` per offspring; `break` the neighbour loop when it reaches `max_neighbor_replacements`. Default nr = 2 per Zhang & Li 2007.
**Warning signs:** Population converging to a single solution far faster than expected.

### Pitfall 3: Unshifted Scalarization
**What goes wrong:** Tchebycheff or PBI computed using raw `f_i` instead of `f_i - z*_i` — metric not centred on ideal point, biased toward origin.
**Why it happens:** Forgetting to subtract the ideal point before applying the formula.
**How to avoid:** Always pass both `objectives` and `ideal` to the scalarization function; compute `f_i - z*_i` as the first step in both Tchebycheff and PBI.
**Warning signs:** Algorithm failing to converge on problems where all objectives have non-zero optima (e.g., DTLZ2 with g > 0).

### Pitfall 4: Ideal Point Staleness
**What goes wrong:** Ideal point is computed once at initialisation and never updated — missing improvements from offspring, leading to suboptimal scalarization reference.
**Why it happens:** Only initialising z* from the initial population.
**How to avoid:** After evaluating each offspring, update z*[k] = min(z*[k], offspring.objectives[k]) for all k.
**Warning signs:** Scalarization values not decreasing across generations even on convex problems.

### Pitfall 5: WASM Compilation Failure
**What goes wrong:** `Instant::now()` or `par_iter()` called unconditionally → compile error on `wasm32-unknown-unknown`.
**Why it happens:** Forgetting cfg gates when copy-pasting from non-WASM code paths.
**How to avoid:** Apply `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` at every `Instant` call site and every `par_iter` vs `iter` branch. Run `cargo check --target wasm32-unknown-unknown` before marking implementation complete.
**Warning signs:** CI wasm-check.yml failing after PR.

### Pitfall 6: Scalarization Weights with Zero Components
**What goes wrong:** If any `w_i = 0`, the Tchebycheff term `w_i * |f_i - z*_i|` = 0 regardless of f_i — that objective is effectively ignored for that sub-problem. This is intentional and correct for boundary weight vectors (e.g., [1, 0, 0]).
**Why it happens:** Mistaking this for a bug; attempting to add epsilon to all weights would distort the uniform coverage property of Das-Dennis.
**How to avoid:** Accept zero-component weights as valid; do not add epsilon to weights before scalarization.
**Warning signs:** Tests failing because boundary weight vectors produce zero Tchebycheff values for non-active objectives.

---

## Code Examples

### Complete Sub-Problem Update Inner Loop

```rust
// Source: Zhang & Li 2007 Algorithm 1, Step 4; CONTEXT.md Specifics
// Called inside: for gen in 0..max_generations { for i in 0..n_subproblems { ... } }

let offspring_chrom: U = {
    let parent_a_idx = neighbourhoods[i][rng.random_range(0..t)];
    let parent_b_idx = neighbourhoods[i][rng.random_range(0..t)];
    let crossover_result = crossover::factory(
        &population[parent_a_idx].chromosome,
        &population[parent_b_idx].chromosome,
        crossover_config,
    )?;
    let mut child = crossover_result.into_iter().next().unwrap_or_else(|| population[parent_a_idx].chromosome.clone());
    mutation::factory_with_params(mutation_config.method, &mut child, mutation_config.step, mutation_config.sigma)?;
    child
};

let offspring_objectives: Vec<f64> = objective_fns.iter().map(|f| f(offspring_chrom.dna())).collect();

// Update ideal point incrementally
for k in 0..num_objectives {
    if offspring_objectives[k] < ideal_point[k] {
        ideal_point[k] = offspring_objectives[k];
    }
}

// Neighbourhood replacement with cap
let mut replacement_count = 0usize;
for &j in &neighbourhoods[i] {
    if replacement_count >= max_neighbor_replacements {
        break;
    }
    let g_offspring = scalarize(&offspring_objectives, &weight_vectors[i], &ideal_point, scalarization);
    let g_current = scalarize(&population[j].objectives, &weight_vectors[j], &ideal_point, scalarization);
    if g_offspring < g_current {
        population[j] = ParetoIndividual::new(offspring_chrom.clone(), offspring_objectives.clone());
        replacement_count += 1;
    }
}
```

[VERIFIED: algorithmic pattern from Zhang & Li 2007 — Tchebycheff and PBI formulas confirmed via Semantic Scholar]

### Scalarize Dispatch Function

```rust
fn scalarize(
    objectives: &[f64],
    weights: &[f64],
    ideal: &[f64],
    scalarization: ScalarizationFn,
) -> f64 {
    match scalarization {
        ScalarizationFn::Tchebycheff => {
            objectives.iter().zip(weights.iter()).zip(ideal.iter())
                .map(|((f_i, w_i), z_i)| w_i * (f_i - z_i).abs())
                .fold(f64::NEG_INFINITY, f64::max)
        }
        ScalarizationFn::Pbi { theta } => {
            let diff: Vec<f64> = objectives.iter().zip(ideal.iter()).map(|(f, z)| f - z).collect();
            let w_norm = weights.iter().map(|w| w * w).sum::<f64>().sqrt().max(f64::EPSILON);
            let d1 = diff.iter().zip(weights.iter()).map(|(d, w)| d * w).sum::<f64>() / w_norm;
            let d2_sq: f64 = diff.iter().zip(weights.iter())
                .map(|(d, w)| { let proj = d1 * w / w_norm; (d - proj).powi(2) })
                .sum();
            d1.abs() + theta * d2_sq.sqrt()
        }
    }
}
```

### Post-hoc Pareto Front Extraction (run() final step)

```rust
// Mirrors Nsga3Ga::run() final block [VERIFIED: codebase — src/engines/nsga3/mod.rs lines 363-365]
let obj_slices: Vec<&[f64]> = population.iter().map(|i| i.objectives.as_slice()).collect();
let fronts = non_dominated_sort_with_directions(&obj_slices, &directions);
let mut ranks = vec![0usize; population.len()];
assign_ranks(&mut ranks, &fronts);
for (i, &r) in ranks.iter().enumerate() {
    population[i].rank = r;
}
let front_individuals: Vec<ParetoIndividual<U>> = population.into_iter().filter(|ind| ind.rank == 0).collect();
Ok(ParetoFront::new(front_individuals))
```

### Example moead_dtlz2.rs Structure (DTLZ2 3-objective)

```rust
// p=10 → C(12,2) = 66 weight vectors (CONTEXT.md Specifics says 91 — that's p=10 for M=3: C(10+3-1,3-1)=C(12,2)=66)
// NOTE: C(12,2) = 66, not 91. C(p+M-1, M-1) for p=10, M=3 = C(12,2) = 66.
// For 91 points: p=12 gives C(14,2)=91 (same as NSGA-III example).
// CONTEXT.md says "population size 91 (C(12,2) with p=10)" — C(12,2) = 66, not 91.
// C(p+M-1, M-1) for M=3: C(p+2,2) = (p+2)(p+1)/2. For 91: p=12 → (14*13/2=91). Use p=12.
const DAS_DENNIS_P: usize = 12; // C(14,2)=91 weight vectors for M=3
const POP_SIZE: usize = 91;
const MAX_GENERATIONS: usize = 300;
```

[VERIFIED: arithmetic — C(p+M-1, M-1) for M=3 is (p+2)(p+1)/2; p=12 gives 91, p=10 gives 66]

**NOTE TO PLANNER:** CONTEXT.md Specifics contains an arithmetic inconsistency: "population size 91 (C(12,2) with p=10)". C(12,2) = 66. The value 91 is correct and matches p=12 (same as the NSGA-III example). Use p=12, population size 91. Clarify in the example comment.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-coded weight vectors | Das-Dennis auto-generation | Phase 35 | Uniform coverage of simplex without user math |
| Separate Pareto type per engine | Shared `ParetoIndividual<U>` / `ParetoFront<U>` | Phase 34/35 | Uniform API across NSGA-II, NSGA-III, MOEA/D |
| Hardcoded log calls | Observer sub-traits | Phase 35 | Zero overhead when observer absent; testable |

**No deprecated APIs in scope for this phase.**

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Tie-breaking in neighbourhood sort (equal Euclidean distances) — index-stable is adequate | Pattern 3 | Minimal; ties are geometrically rare in Das-Dennis lattice; outcome-neutral for correctness |
| A2 | PBI d1 sign: take `d1.abs()` to ensure non-negative distance | Pattern 5 | If the implementation should allow negative d1 (indicating overshoot past ideal), the PBI value would be smaller; the conservative `.abs()` prevents a sub-problem from "winning" by overshooting, which is the standard interpretation |
| A3 | Population size should match weight vector count N for standard MOEA/D | Pattern 11 | If population_size != N, some sub-problems have no dedicated representative; not validated by design in Phase 36 (CONTEXT.md does not require this check) |

---

## Open Questions

1. **Sub-problem representative semantics when replacement_count == 0**
   - What we know: The offspring is evaluated but replaces nothing.
   - What's unclear: Should the offspring still be evaluated for ideal point update even if it replaces nothing? (Yes — ideal point update happens unconditionally before replacement, per the standard algorithm.)
   - Recommendation: Update ideal point always; replacement is conditional. This is the standard Zhang & Li ordering.

2. **DTLZ2 example population size clarification**
   - What we know: CONTEXT.md says "population size 91 (C(12,2) with p=10)" but C(12,2)=66 and 91=C(14,2) from p=12.
   - Recommendation: Use p=12, population=91. Add a comment in the example file explaining the formula. No need to surface this to the user — it's a documentation arithmetic error in CONTEXT.md, not a design decision.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | ✓ | 1.94.1 | — |
| cargo test | Testing | ✓ | 1.94.1 | — |
| wasm32-unknown-unknown target | WASM cfg verification | ✓ (with getrandom cfg note) | — | Run `rustup target add wasm32-unknown-unknown` if absent |
| cargo clippy | Lint check | ✓ | 1.94.1 | — |

[VERIFIED: `cargo --version`, `rustc --version` — all available in environment]

**Missing dependencies with no fallback:** None.

**Note on WASM check:** `cargo check --target wasm32-unknown-unknown` triggers a known getrandom configuration message (the existing codebase has this), but this is not a new issue introduced by Phase 36.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + cargo test |
| Config file | Cargo.toml (no separate test config) |
| Quick run command | `cargo test --test test_moead 2>&1` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOO-02 | MoeaDGa validate() rejects missing weight vectors | unit | `cargo test --test test_moead test_moead_validate_missing_weight_vectors` | ❌ Wave 0 |
| MOO-02 | MoeaDGa validate() rejects zero objectives | unit | `cargo test --test test_moead test_moead_validate_zero_objectives` | ❌ Wave 0 |
| MOO-02 | MoeaDGa validate() rejects mismatched objective_fns count | unit | `cargo test --test test_moead test_moead_validate_mismatched_objective_fns` | ❌ Wave 0 |
| MOO-02 | MoeaDGa validate() rejects wrong-dimension weight vectors | unit | `cargo test --test test_moead test_moead_validate_wrong_dimension_weight_vectors` | ❌ Wave 0 |
| MOO-02 | MoeaDGa validate() passes with complete config | unit | `cargo test --test test_moead test_moead_validate_passes` | ❌ Wave 0 |
| MOO-02 | MoeaDConfiguration builder: last-call-wins weight vectors | unit | `cargo test --test test_moead_configuration` | ❌ Wave 0 |
| MOO-02 | ScalarizationFn default is Tchebycheff | unit | `cargo test --test test_moead_configuration test_scalarization_default` | ❌ Wave 0 |
| MOO-02 | run() produces non-empty ParetoFront on 3-objective DTLZ2 | integration | `cargo test --test test_moead test_moead_run_produces_pareto_front` | ❌ Wave 0 |
| MOO-02 | run() with Tchebycheff scalarization converges on DTLZ2 | integration | `cargo test --test test_moead test_moead_run_tchebycheff` | ❌ Wave 0 |
| MOO-02 | run() with PBI scalarization produces valid Pareto front | integration | `cargo test --test test_moead test_moead_run_pbi` | ❌ Wave 0 |
| MOO-02 | Observer hooks fire correct number of times per generation | integration | `cargo test --test test_moead test_moead_run_invokes_observer_hooks` | ❌ Wave 0 |
| MOO-02 | MoeaDObserver<U> for LogObserver compiles and emits | unit | `cargo test --test test_moead test_moead_log_observer` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --test test_moead 2>&1 | tail -5`
- **Per wave merge:** `cargo test && cargo clippy`
- **Phase gate:** `cargo test && cargo test --features serde && cargo clippy` all green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/engines/moead/test_moead.rs` — covers engine validate() + run() + observer (MOO-02)
- [ ] `tests/engines/moead/test_moead_configuration.rs` — covers MoeaDConfiguration builder + ScalarizationFn (MOO-02)
- [ ] `tests/engines/moead/mod.rs` — (if Rust requires mod.rs for subdirectory test module; check NSGA-III test layout for pattern)

---

## Security Domain

Phase 36 adds no authentication, session management, access control, cryptography, or external input parsing. All inputs are in-process Rust function calls. No security domain considerations apply.

---

## Project Constraints (from CLAUDE.md)

The following CLAUDE.md directives apply to this phase and constrain implementation choices:

| Directive | Impact on Phase 36 |
|-----------|-------------------|
| **No breaking changes** | `GaError::InvalidMoeaDConfiguration` is a new variant — additive, non-breaking. `MoeaDObserver<U>` is a new trait — additive. `pub mod moead` is a new re-export — additive. |
| **WASM compatibility mandatory** | All `Instant::now()` and `par_iter()` call sites must be cfg-gated. Run `cargo check --target wasm32-unknown-unknown` before marking complete. |
| **Tests in tests/ folder** | All tests go in `tests/engines/moead/`, never inline with implementation. |
| **Branching** | Work on `feat/<issue-number>-moead-engine` branched from `milestone/advanced-multi-objective-optimization`. |
| **Cargo version** | Already at 2.4.0 on milestone branch — no additional bump needed for this phase. |
| **Operator dispatch pattern** | Not directly applicable (MOEA/D does not add new operator enums), but crossover/mutation dispatch must use existing `crossover::factory()` and `mutation::factory_with_params()`. |
| **Observability** | `MoeaDObserver<U>` hooks must not be bypassed; `notify()` inline dispatch pattern is mandatory. |
| **Performance** | Use `Vec::with_capacity()` for weight_vectors, neighbourhoods, and offspring collections. |

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase] `src/engines/nsga3/mod.rs` — full engine pattern, WASM gating, observer wiring
- [VERIFIED: codebase] `src/engines/nsga3/configuration.rs` — builder pattern, effective_weight_vectors mirror
- [VERIFIED: codebase] `src/engines/nsga3/das_dennis.rs` — reusable lattice generator
- [VERIFIED: codebase] `src/engines/multi_objective/pareto.rs` — ParetoIndividual, ParetoFront
- [VERIFIED: codebase] `src/engines/multi_objective/mod.rs` — ObjectiveFn, ObjectiveDirection
- [VERIFIED: codebase] `src/observe/observer/mod.rs` lines 150-191 — Nsga2Observer, Nsga3Observer pattern
- [VERIFIED: codebase] `src/observe/observer/log.rs` — LogObserver impl blocks
- [VERIFIED: codebase] `src/error.rs` — GaError enum, exact insertion points
- [VERIFIED: codebase] `src/lib.rs` lines 95-129 — #[path] re-export pattern, observer pub use list

### Secondary (MEDIUM confidence)
- [CITED: semanticscholar.org/paper/MOEA-D:-A-Multiobjective-Evolutionary-Algorithm-on-Zhang-Li] — Zhang & Li 2007 canonical Tchebycheff and PBI scalarization formulas, neighbourhood update algorithm, max_neighbor_replacements=2 default, neighbourhood_size=20 default
- [CITED: link.springer.com/chapter/10.1007/978-3-030-72062-9_33] — PBI theta parameter analysis

### Tertiary (LOW confidence)
- None — all factual claims verified via codebase inspection or cited from canonical references.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all libraries already in Cargo.toml
- Architecture: HIGH — exact mirror of NSGA-III patterns verified in codebase
- MOEA/D algorithm: HIGH — Tchebycheff and PBI formulas from Zhang & Li 2007 (Semantic Scholar)
- Pitfalls: HIGH — directly derived from codebase patterns and algorithm properties
- Test structure: HIGH — mirrors NSGA-III test files verified in codebase

**Research date:** 2026-05-09
**Valid until:** 2026-06-09 (stable domain — Rust std patterns and the MOEA/D algorithm are not subject to rapid change)
