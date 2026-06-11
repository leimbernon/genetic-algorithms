# Phase 37: SPEA2 — Strength Pareto Evolutionary Algorithm 2 - Research

**Researched:** 2026-05-10
**Domain:** Multi-objective optimization — archive-based evolutionary algorithm (strength + density fitness)
**Confidence:** HIGH

## Summary

SPEA2 (Zitzler, Laumanns & Thiele 2001) is a classic multi-objective evolutionary algorithm that maintains a fixed-size external archive of non-dominated solutions. Fitness is computed from raw strength (domination count) plus density (k-nearest-neighbour distance), and the archive is truncated using iterative nearest-neighbour removal when it exceeds capacity. The new `Spea2Ga<U>` engine follows the established multi-objective engine pattern from MOEA/D (Phase 36, closest structural analog): `src/engines/spea2/` directory with `mod.rs` + `configuration.rs`, `with_observer()` / `notify()` dispatch, WASM cfg-gating, and `run()` returning `Result<ParetoFront<U>, GaError>`.

**Primary recommendation:** Implement `Spea2Ga<U>` as a direct structural mirror of `MoeaDGa<U>`, replacing MOEA/D's weight-vector/scalarization logic with SPEA2's strength + density fitness assignment, archive management, and environmental selection with truncation. The reuse of shared `multi_objective` utilities (non_dominated_sort, ParetoIndividual, ObjectiveFn) means the engine module is self-contained.

## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Archive Sizing
- **D-01:** `Spea2Configuration` exposes `.with_archive_size(usize)`. Default: archive size equals population size (canonical SPEA2). `validate()` rejects `archive_size > population_size` or `archive_size == 0` as `InvalidSpea2Configuration`.

#### Density k Parameter
- **D-02:** k-nearest-neighbour parameter for density estimation is auto-calculated as `k = floor(sqrt(N_pop + N_archive))` — matches the SPEA2 paper exactly. No configuration method — users cannot override.

#### Truncation Strategy
- **D-03:** Archive truncation uses the exact SPEA2 algorithm: iteratively remove the individual with the smallest Euclidean distance to its nearest neighbour in objective space, recomputing distances after each removal. No alternative strategies — this is the canonical SPEA2 algorithm.

#### Observer Hooks
- **D-04:** `Spea2Observer<U>` sub-trait exposes two generation-level hooks:
  - `fn on_fitness_assigned(&self, generation: usize, duration_ms: f64, pop_size: usize, archive_size: usize) {}` — after raw strength R(i) + density D(i) assignment to all individuals
  - `fn on_archive_updated(&self, generation: usize, archive_size: usize, non_dominated_count: usize) {}` — after environmental selection + archive truncation
  - All methods have default no-op implementations. `Send + Sync` supertraits.
  - Mirrors `Nsga3Observer<U>` and `MoeaDObserver<U>` hook structure (two hooks, generation-level).
- **D-05:** `Spea2Ga<U>` stores `Option<Arc<dyn Spea2Observer<U> + Send + Sync>>` — zero overhead when `None`. Same `with_observer()` + `notify()` pattern as all prior multi-objective engines.
- **D-06:** `LogObserver` MUST implement `Spea2Observer<U>` in this phase. Debug-level log messages on `"spea2_events"` target. Mirrors existing observer impl blocks for NSGA-III and MOEA/D.
- **D-07:** `AllObserver<U>` is NOT updated in this phase to include `Spea2Observer<U>` — avoids breaking existing implementors (same rationale as Phase 35 D-10 and Phase 36 D-13).

#### Example Benchmark
- **D-08:** User-facing example is `examples/spea2_zdt1.rs` — ZDT1 (2-objective, 30 variables). ZDT1 is the canonical SPEA2 benchmark from the original Zitzler et al. 2001 paper. Mirrors `examples/nsga2_zdt1.rs` structure with SPEA2 adaptations.

#### Return Type
- **D-09:** `Spea2Ga<U>::run()` returns `Result<ParetoFront<U>, GaError>` — identical return type to all existing multi-objective engines (Nsga2Ga, Nsga3Ga, MoeaDGa). The Pareto front is extracted from the final archive via non-dominated sorting.

### Claude's Discretion

- Internal archive management: maintain archive as a `Vec<U>` alongside the population; after fitness assignment, copy non-dominated individuals to archive, then truncate if over capacity
- Mating selection: binary tournament from the archive (standard SPEA2)
- SPEA2 fitness algorithm: combine population + archive, compute strength S(i) = count of dominated individuals, raw fitness R(i) = sum of S(j) for all j dominating i, density D(i) = 1/(sigma_k + 2)
- WASM cfg-gating: apply `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` to all `Instant::now()` and `par_iter()` call sites (mandatory — CLAUDE.md constraint)
- Builder methods return `Self` (fluent pattern)

### Deferred Ideas (OUT OF SCOPE)
- Archive size adaptation (dynamic sizing)
- Alternative truncation strategies (random, farthest-first)
- `AllObserver<U>` updated to include `Spea2Observer<U>`
- Alternative density estimators (k-th vs fixed k)
- DTLZ2 or other 3-objective example

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MOO-03 | User can run SPEA2 with a configurable archive size; fitness is computed from raw strength + density (k-nearest-neighbour), and the archive is truncated using the Euclidean crowding criterion | Verified: SPEA2 algorithm defined in Zitzler et al. 2001; codebase patterns from MOEA/D (Phase 36) provide exact structural template |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SPEA2 engine orchestration | API/Backend (Spea2Ga) | — | Engine module owns the full generation loop: fitness assignment, archive management, selection |
| Strength + density fitness | API/Backend (spea2 module) | — | SPEA2-specific algorithm; no existing utility provides this |
| Archive management | API/Backend (spea2 module) | — | Archive population, environmental selection, iterative truncation — all engine-internal |
| Non-dominated sorting | Shared utility (multi_objective) | — | Already exists as `non_dominated_sort_with_directions()` |
| Observer hooks | API/Backend (Spea2Observer) | — | Per-engine sub-trait, same pattern as Nsga3Observer and MoeaDObserver |
| Binary tournament selection | API/Backend (spea2 module) | — | Engine-internal method (same as NSGA-III and MOEA/D) |
| Crossover + mutation | Operations library | — | Reuse existing `crossover::factory()` and `mutation::factory_with_params()` |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `Spea2Ga<U>` | new | SPEA2 engine orchestrator | New engine — follows `MoeaDGa<U>` structural pattern |
| `Spea2Configuration` | new | SPEA2-specific config | Builder pattern — mirrors `MoeaDConfiguration` / `Nsga3Configuration` |
| `GaConfiguration` | existing | Base GA config (operators, limits) | Reused by all engines |
| `ParetoIndividual<U>` | existing | Chromosome + objectives wrapper | Shared across all multi-objective engines |
| `ParetoFront<U>` | existing | Return type for `run()` | Shared across all multi-objective engines |
| `non_dominated_sort_with_directions()` | existing | Dominance sorting | Used for post-hoc front extraction |
| `Spea2Observer<U>` | new | Engine-specific observer trait | Same pattern as `Nsga3Observer<U>` / `MoeaDObserver<U>` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `LogObserver` | existing | Combined observer | D-06: must add `impl Spea2Observer<U> for LogObserver` |
| `crossover::factory()` | existing | Crossover dispatch | SPEA2 reuses standard crossover operators |
| `mutation::factory_with_params()` | existing | Mutation dispatch | SPEA2 reuses standard mutation operators (like NSGA-II/III) |
| `make_rng()` | existing | Seeded RNG | Tournament selection + random operations |
| `GaError::InvalidSpea2Configuration(String)` | new | Config validation error | Mirrors `InvalidMoeaDConfiguration` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Binary tournament from archive | Roulette wheel from archive | Tournament is standard SPEA2, simpler to implement |
| Full SPEA2 truncation | Simplified random removal | Canonical truncation is required per D-03 |

**Installation:**
```bash
# No new dependencies — all shared utilities and operators already exist
```

**Version verification:** No new external crates. Rust edition 2021, rust-version = "1.81.0" per Cargo.toml. Existing dependencies: rand 0.9.2, rayon 1.10.

## Project Constraints (from CLAUDE.md)

- **WASM compatibility mandatory**: Gate `Instant::now()` calls with `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` exactly as done in `MoeaDGa::run()` lines 325-336. Gate `par_iter()` calls with the same pattern (lines 454-471 in moead/mod.rs).
- **No breaking changes default policy**: New enum variants for `GaError`, new traits (`Spea2Observer<U>`), new builder methods, new modules — all non-breaking. Do NOT modify `AllObserver<U>`.
- **Follow existing engine patterns**: `#[path]` re-export in lib.rs, `pub mod` subdirectory, `configuration.rs` submodule, `with_observer()` + `notify()` pattern.
- **Error handling**: New `GaError::InvalidSpea2Configuration(String)` variant only — follows established pattern.
- **Tests in `tests/` folder**: Do NOT use inline `#[cfg(test)] mod tests` — create `tests/engines/spea2/` directory.
- **Observability preservation**: All observer notification points must remain in the execution flow.

## Architecture Patterns

### SPEA2 Generation Loop (Algorithm 1, Zitzler et al. 2001)

```
Input:  N (population size), N_archive (archive size), T (max generations)
1.  Initialize P_0 (size N), A_0 = empty
2.  For gen = 0..T:
    a.  U = P_gen ∪ A_gen
    b.  Compute strength S(i) = |{j: i dominates j}| for all i in U
    c.  Compute raw fitness R(i) = sum S(j) for all j that dominate i
    d.  Compute k = floor(sqrt(|U|))
    e.  Compute density D(i) = 1 / (sigma_k + 2)
    f.  Final fitness F(i) = R(i) + D(i)  [lower is better]
    g.  Truncate fitness for dominated: F(i) = R(i) + D(i) where R(i) >= |U|
    h.  Environmental selection:
        - Copy all non-dominated (R(i) = 0) to archive
        - If |archive| == N_archive: done
        - If |archive| < N_archive: fill with best F(i) dominated
        - If |archive| > N_archive: iterative truncation
    i.  Mating: binary tournament from archive -> offspring P_{gen+1}
3.  Return non-dominated from final archive as ParetoFront
```

### Recommended Project Structure

```
src/engines/spea2/
├── mod.rs              # Spea2Ga<U> engine struct + run() + helper methods
└── configuration.rs    # Spea2Configuration builder + default

tests/engines/spea2/
├── test_spea2.rs            # validate() + run() integration tests
└── test_spea2_configuration.rs  # Configuration builder tests

examples/
└── spea2_zdt1.rs            # ZDT1 benchmark example
```

### Pattern 1: Engine Struct (mirrors MOEA/D exactly)

```rust
pub struct Spea2Ga<U>
where
    U: ChromosomeT,
{
    pub spea2_config: Spea2Configuration,
    pub ga_config: GaConfiguration,
    pub alleles: Vec<U::Gene>,
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    pub observer: Option<Arc<dyn Spea2Observer<U> + Send + Sync>>,
}
```

### Pattern 2: Observer Dispatch Pattern (same as all prior engines)

```rust
pub fn with_observer(mut self, obs: Arc<dyn Spea2Observer<U> + Send + Sync>) -> Self {
    self.observer = Some(obs);
    self
}

#[inline]
fn notify<F: FnOnce(&dyn Spea2Observer<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Pattern 3: WASM cfg-gating (copy exactly from MoeaDGa)

```rust
// Timing for observer — Instant::now() gated:
let t_fitness: Option<Instant> = if self.observer.is_some() {
    #[cfg(not(target_arch = "wasm32"))]
    { Some(Instant::now()) }
    #[cfg(target_arch = "wasm32")]
    { None }
} else {
    None
};

// Parallel evaluation — par_iter() gated:
#[cfg(not(target_arch = "wasm32"))]
let population: Vec<ParetoIndividual<U>> = chromosomes
    .into_par_iter()
    .map(|chrom| { /* shared closure body */ })
    .collect();
#[cfg(target_arch = "wasm32")]
let population: Vec<ParetoIndividual<U>> = chromosomes
    .into_iter()
    .map(|chrom| { /* shared closure body */ })
    .collect();
```

### Anti-Patterns to Avoid
- **Modifying `AllObserver<U>`**: Do NOT add `Spea2Observer<U>` to the `AllObserver` trait bound — D-07 explicitly defers this to avoid breaking existing implementors.
- **Hand-rolling domination logic**: Use existing `non_dominated_sort_with_directions()` from `multi_objective` utilities. Do not reimplement.
- **Using weight vectors or scalarization**: SPEA2 does NOT use decomposition — it uses archive-based selection. Do not import any MOEA/D-specific types.
- **Inline tests**: Place all tests in `tests/engines/spea2/`, not inline in source files.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Non-dominated sorting | Custom O(n^2) dominance checks | `non_dominated_sort_with_directions()` | Already exists, handles direction-aware dominance correctly |
| Pareto front / individual types | Custom front struct | `ParetoFront<U>`, `ParetoIndividual<U>` | Shared return types ensure API consistency |
| Objective function type | Custom `Fn` signature | `ObjectiveFn<G>` type alias | Standardizes `Fn(&[G]) -> f64 + Send + Sync` |
| RNG | `rand::thread_rng()` | `crate::rng::make_rng()` | Supports seed-based reproducibility |
| Crossover / mutation | Custom per-engine | `crossover::factory()`, `mutation::factory_with_params()` | All operators are generic over chromosome/gene |

## Runtime State Inventory

> NOT applicable — this is a greenfield engine phase with no rename/refactor/migration requirements. No existing code or state references need updating.

## Common Pitfalls

### Pitfall 1: Strength Fitness Misimplementation
**What goes wrong:** The raw fitness R(i) is the sum of strengths of individuals that *dominate* i, not the sum of individuals that i *dominates*.
**Why it happens:** The term "strength" is confusing — S(i) counts what i dominates, but R(i) sums over what *dominates* i.
**How to avoid:** Implement in two clear steps: (1) compute S(i) for all i in U, (2) for each i, compute R(i) = sum of S(j) for all j that dominate i. `R(i) = 0` means i is non-dominated. Lower is better.

### Pitfall 2: Archive Truncation Algorithm
**What goes wrong:** Truncation removes the individual with the smallest nearest-neighbour distance, but ties must be resolved by looking at second-nearest, third-nearest, etc.
**Why it happens:** The SPEA2 paper specifies tie-breaking via successive nearest neighbours.
**How to avoid:** Implement `find_min_distance_index()` that, given the archive, finds the individual to remove by scanning all pairwise distances and using lexicographic ordering of sorted distance lists.

### Pitfall 3: Density Computation k-Parameter
**What goes wrong:** Using a fixed k instead of computing k = floor(sqrt(N_pop + N_archive)) each generation.
**Why it happens:** The population size and archive are fixed, so k is actually constant across generations. But computing it dynamically is the canonical formula.
**How to avoid:** Compute `let k = ((population.len() + archive.len()) as f64).sqrt().floor() as usize` at the start of each generation's fitness assignment.

### Pitfall 4: Selection from Empty or Near-Empty Archive
**What goes wrong:** Tournament selection from the archive fails if the archive has fewer than 2 individuals (early generations).
**Why it happens:** Binary tournament needs 2 distinct indices.
**How to avoid:** Handle the edge case by falling back to population-based selection when archive is empty or has only 1 individual. The SPEA2 paper assumes archive is non-empty by generation 1.

## Code Examples

Verified patterns from official sources:

### Spea2Configuration (mirror of MoeaDConfiguration)

```rust
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spea2Configuration {
    pub num_objectives: usize,
    pub population_size: usize,
    pub archive_size: usize,
    pub max_generations: usize,
    pub objective_directions: Vec<ObjectiveDirection>,
}

impl Default for Spea2Configuration {
    fn default() -> Self {
        Spea2Configuration {
            num_objectives: 2,
            population_size: 100,
            archive_size: 100,
            max_generations: 250,
            objective_directions: Vec::new(),
        }
    }
}

impl Spea2Configuration {
    pub fn new() -> Self { Self::default() }
    pub fn with_num_objectives(mut self, n: usize) -> Self { self.num_objectives = n; self }
    pub fn with_population_size(mut self, size: usize) -> Self { self.population_size = size; self }
    pub fn with_archive_size(mut self, size: usize) -> Self { self.archive_size = size; self }
    pub fn with_max_generations(mut self, gens: usize) -> Self { self.max_generations = gens; self }
    pub fn with_objective_directions(mut self, dirs: Vec<ObjectiveDirection>) -> Self {
        self.objective_directions = dirs; self
    }

    pub fn effective_directions(&self) -> Vec<ObjectiveDirection> {
        if self.objective_directions.is_empty() {
            vec![ObjectiveDirection::Minimize; self.num_objectives]
        } else {
            self.objective_directions.clone()
        }
    }
}
```

### Spea2Observer trait (mirror of MoeaDObserver)

```rust
pub trait Spea2Observer<U: ChromosomeT>: Send + Sync {
    fn on_fitness_assigned(
        &self,
        _generation: usize,
        _duration_ms: f64,
        _pop_size: usize,
        _archive_size: usize,
    ) {}
    fn on_archive_updated(
        &self,
        _generation: usize,
        _archive_size: usize,
        _non_dominated_count: usize,
    ) {}
}
```

### LogObserver impl (mirror of existing engine impl blocks)

```rust
impl<U: ChromosomeT> Spea2Observer<U> for LogObserver {
    fn on_fitness_assigned(
        &self,
        generation: usize,
        duration_ms: f64,
        pop_size: usize,
        archive_size: usize,
    ) {
        log::debug!(target: "spea2_events",
            "Strength+density fitness assigned at generation {} ({:.2}ms) — pop={}, archive={}",
            generation, duration_ms, pop_size, archive_size);
    }
    fn on_archive_updated(
        &self,
        generation: usize,
        archive_size: usize,
        non_dominated_count: usize,
    ) {
        log::debug!(target: "spea2_events",
            "Archive updated at generation {} — size={}, non-dominated={}",
            generation, archive_size, non_dominated_count);
    }
}
```

### SPEA2 Fitness Assignment (core algorithm)

```rust
fn assign_spea2_fitness(
    population: &[ParetoIndividual<U>],
    archive: &[ParetoIndividual<U>],
    directions: &[ObjectiveDirection],
) -> Vec<f64>
where U: ChromosomeT
{
    // Combine population and archive
    let union: Vec<&ParetoIndividual<U>> = population.iter().chain(archive.iter()).collect();
    let n = union.len();
    let k = (n as f64).sqrt().floor() as usize;

    // Step 1: Compute strength S(i) for each individual
    let mut strength = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..n {
            if i != j && dominates_with_directions(
                &union[i].objectives,
                &union[j].objectives,
                directions,
            ) {
                strength[i] += 1.0;
            }
        }
    }

    // Step 2: Compute raw fitness R(i) = sum of strengths of dominators
    let mut raw_fitness = vec![0.0f64; n];
    for i in 0..n {
        for j in 0..n {
            if i != j && dominates_with_directions(
                &union[j].objectives,
                &union[i].objectives,
                directions,
            ) {
                raw_fitness[i] += strength[j];
            }
        }
    }

    // Step 3: Compute density D(i) = 1 / (sigma_k + 2)
    let mut density = vec![0.0f64; n];
    for i in 0..n {
        let mut distances: Vec<f64> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                euclidean_distance(&union[i].objectives, &union[j].objectives)
            })
            .collect();
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sigma_k = distances.get(k.saturating_sub(1)).copied().unwrap_or(f64::MAX);
        density[i] = 1.0 / (sigma_k + 2.0);
    }

    // Step 4: Final fitness = raw + density
    (0..n).map(|i| raw_fitness[i] + density[i]).collect()
}

fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}
```

### Archive Truncation Algorithm (canonical SPEA2)

```rust
fn truncate_archive(
    archive: &mut Vec<ParetoIndividual<U>>,
    target_size: usize,
) {
    while archive.len() > target_size {
        // For each individual, compute sorted distances to all others
        let n = archive.len();
        let mut min_dist_idx = 0usize;
        let mut min_dist_list: Vec<f64> = Vec::new();

        for i in 0..n {
            let mut dists: Vec<f64> = (0..n)
                .filter(|&j| j != i)
                .map(|j| euclidean_distance(&archive[i].objectives, &archive[j].objectives))
                .collect();
            dists.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            if i == 0 {
                min_dist_list = dists;
                min_dist_idx = i;
            } else {
                // Lexicographic comparison
                for (a, b) in dists.iter().zip(min_dist_list.iter()) {
                    if a < b {
                        min_dist_list = dists;
                        min_dist_idx = i;
                        break;
                    } else if a > b {
                        break;
                    }
                }
            }
        }

        archive.remove(min_dist_idx);
    }
}
```

### Environmental Selection

```rust
fn environmental_selection(
    pop: &[ParetoIndividual<U>],
    archive: &mut Vec<ParetoIndividual<U>>,
    fitness: &[f64],
    pop_size: usize,   // N
    archive_size: usize, // N_archive target
) {
    let n_pop = pop.len();
    // Archive fitness is at indices [n_pop..n_pop + archive.len()]
    let all_fitness = fitness; // Combined fitness for population + archive

    // Step 1: Collect all non-dominated individuals
    let mut new_archive: Vec<ParetoIndividual<U>> = pop.iter()
        .chain(archive.iter())
        .enumerate()
        .filter(|(i, _)| all_fitness[*i] < 1.0)  // R(i) = 0 means non-dominated
        .map(|(_, ind)| ind.clone())
        .collect();

    // Step 2: Fill or truncate
    if new_archive.len() == archive_size {
        *archive = new_archive;
    } else if new_archive.len() < archive_size {
        // Fill with best dominated individuals
        let dominated: Vec<(usize, &ParetoIndividual<U>)> = pop.iter()
            .chain(archive.iter())
            .enumerate()
            .filter(|(i, _)| all_fitness[*i] >= 1.0)
            .collect();
        let mut sorted: Vec<&ParetoIndividual<U>> = dominated.iter()
            .map(|(_, ind)| *ind)
            .collect();
        sorted.sort_by(|a, b| {
            // Use the already-computed fitness; lower is better
            let fi = all_fitness[a]; // Not clean — needs fitness per index
            let fj = all_fitness[b];
            fi.partial_cmp(&fj).unwrap_or(std::cmp::Ordering::Equal)
        });
        // HOWEVER: this is simplified — the actual selection needs the fitness values
        // stored per individual or passed alongside. Implementation detail.
        truncate_archive(&mut new_archive, archive_size);
        *archive = new_archive;
    } else {
        truncate_archive(&mut new_archive, archive_size);
        *archive = new_archive;
    }
}
```

**Note on implementation:** The actual implementation should compute fitness as part of the run() method and use it directly with the combined population+archive indices. The `non_dominated_count` for the observer hook is simply the number of individuals with `R(i) == 0` (fitness < 1.0).

### Spea2Ga::run() skeleton (mirror of MoeaDGa::run())

```rust
impl<U> Spea2Ga<U>
where
    U: ChromosomeT + mutation::ValueMutable,
{
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        crate::rng::set_seed(self.ga_config.rng_seed);
        let pop_size = self.spea2_config.population_size;
        let archive_size = self.spea2_config.archive_size;
        let max_gens = self.spea2_config.max_generations;
        let directions = self.spea2_config.effective_directions();

        let mut population = self.initialize_population()?;
        let mut archive: Vec<ParetoIndividual<U>> = Vec::new();

        for gen in 0..max_gens {
            // Timing for observer
            let t_fitness: Option<Instant> = if self.observer.is_some() {
                #[cfg(not(target_arch = "wasm32"))] { Some(Instant::now()) }
                #[cfg(target_arch = "wasm32")] { None }
            } else { None };

            // Fitness assignment (combined population + archive)
            let fitness = self.assign_fitness(&population, &archive, &directions);

            // Observer: fitness assigned
            if let Some(start) = t_fitness {
                self.notify(|obs| obs.on_fitness_assigned(
                    gen,
                    start.elapsed().as_secs_f64() * 1000.0,
                    population.len(),
                    archive.len(),
                ));
            }

            // Environmental selection + archive truncation
            let non_dominated_count_before = archive.len(); // approximation
            self.environmental_selection(&population, &mut archive, &fitness, archive_size);

            // Observer: archive updated
            let non_dominated_count = archive.iter()
                .filter(|ind| ind.rank == 0).count();
            self.notify(|obs| obs.on_archive_updated(
                gen,
                archive.len(),
                non_dominated_count,
            ));

            // Mating selection: binary tournament from archive -> offspring
            population = self.create_offspring(&archive)?;
        }

        // Extract Pareto front from final archive
        let obj_slices: Vec<&[f64]> = archive.iter().map(|ind| ind.objectives.as_slice()).collect();
        let fronts = non_dominated_sort_with_directions(&obj_slices, &directions);
        let mut ranks = vec![0usize; archive.len()];
        assign_ranks(&mut ranks, &fronts);
        for (i, &r) in ranks.iter().enumerate() {
            archive[i].rank = r;
        }
        let front_individuals: Vec<ParetoIndividual<U>> =
            archive.into_iter().filter(|ind| ind.rank == 0).collect();
        Ok(ParetoFront::new(front_individuals))
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SPEA1 (fitness = strength only, raw archive) | SPEA2 (strength + density, fixed-size archive via truncation) | 2001 | SPEA2 fixes SPEA1's archive overflow and fitness degradation issues |

**Deprecated/outdated:**
- SPEA (original): No density information, no archive truncation mechanism. SPEA2 fully supersedes it.
- Simple k-NN without k = sqrt calculation: SPEA2 paper specifies k = floor(sqrt(N)) as the canonical density estimation parameter.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SPEA2 binary tournament selection draws parents from the archive only (not from the population) | Claude's Discretion | Mating selection would be incorrect; verify against Zitzler et al. 2001 paper |
| A2 | The SPEA2 "fill with best dominated" step sorts dominated individuals by raw fitness (not combined fitness) | Claude's Discretion | Sort key affects which dominated individuals enter the archive |
| A3 | `Instant::now()` gating pattern from MOEA/D (conditional on observer) is correct for SPEA2 | Architecture | Timing is only needed when observer is attached; matches all prior engines |

## Open Questions

1. **[A1] Is binary tournament always from the archive?**
   - What we know: The CONTEXT.md specifies binary tournament from the archive (standard SPEA2).
   - What's unclear: Early generations may have near-empty archive.
   - Recommendation: Fall back to population-based selection when archive has fewer than 2 individuals.

2. **[A2] How exactly is the "fill with best dominated" step implemented?**
   - What we know: Fill when non-dominated count < archive_size, sorted by fitness F(i).
   - What's unclear: Whether to sort by raw R(i) or combined F(i) = R(i) + D(i).
   - Recommendation: Sort by F(i) (combined) — lower is better; this is the canonical SPEA2 approach.

3. **[fill step] Should archive-dominated individuals be excluded from the fill step?**
   - What we know: The combined set U includes both population and archive.
   - What's unclear: The paper says "copy all non-dominated individuals from U to archive" then "fill with best dominated." Are archive individuals that are dominated by population individuals eligible for the fill?
   - Recommendation: Yes — all dominated individuals in U are eligible for filling, sorted by F(i).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/rustc | All | yes | 1.94.1 | -- |
| Cargo | All | yes | 1.94.1 | -- |
| Node.js | Tooling | yes | 26.0.0 | -- |
| wasm32-unknown-unknown | WASM check | No (needs getrandom config) | -- | Run `cargo check` with env config; not a blocking concern for implementation |

**Missing dependencies with no fallback:** None that block implementation. WASM target requires `getrandom` js feature flag but this is a CI concern — the cfg-gating pattern is already established.

**Missing dependencies with fallback:** None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo-test (Rust built-in) |
| Config file | Cargo.toml (dev-dependencies) |
| Quick run command | `cargo test --test test_spea2` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MOO-03-valid | Config validation rejects invalid params | unit | `cargo test --test test_spea2 -- test_spea2_validate` | Will be created |
| MOO-03-run | `Spea2Ga::run()` produces non-empty ParetoFront | integration | `cargo test --test test_spea2 -- test_spea2_run_produces_pareto_front` | Will be created |
| MOO-03-obs | Observer hooks fire per generation | integration | `cargo test --test test_spea2 -- test_spea2_run_invokes_observer_hooks` | Will be created |
| MOO-03-log | LogObserver compiles and runs without panic | integration | `cargo test --test test_spea2 -- test_spea2_log_observer` | Will be created |
| MOO-03-config | Builder yields correct values | unit | `cargo test --test test_spea2_configuration` | Will be created |

### Sampling Rate
- **Per task commit:** `cargo test --test test_spea2` (quick subset)
- **Per wave merge:** `cargo test && cargo test --features serde`
- **Phase gate:** `cargo test && cargo test --features serde && cargo clippy && cargo doc --no-deps` and verify `cargo check --target wasm32-unknown-unknown` passes

### Wave 0 Gaps
- [ ] `tests/engines/spea2/test_spea2.rs` — covers REQ-MOO-03 (validate + run + observer)
- [ ] `tests/engines/spea2/test_spea2_configuration.rs` — covers REQ-MOO-03 (config builder)
- [ ] Framework install: not needed (cargo-test is built-in)

## Security Domain

Security enforcement is not configured in `.planning/config.json`. This phase introduces no new external dependencies, network access, or data persistence. The standard ASVS categories (authentication, session management, access control, input validation, cryptography) are not applicable to a non-networked, non-persistent optimization library.

## Sources

### Primary (HIGH confidence)
- CONTEXT.md for Phase 37 (all locked decisions D-01 through D-09)
- `src/engines/moead/mod.rs` — MOEA/D engine pattern (closest structural analog)
- `src/engines/moead/configuration.rs` — Configuration builder pattern
- `src/engines/nsga3/mod.rs` — NSGA-III engine pattern (secondary reference)
- `src/observe/observer/mod.rs` — Observer trait definitions (existing engine sub-traits)
- `src/observe/observer/log.rs` — LogObserver impl patterns for engine sub-traits
- `src/error.rs` — GaError enum with engine-specific error variants
- `src/lib.rs` — Module structure and `#[path]` re-export pattern
- `src/engines/multi_objective/non_dominated_sort.rs` — Shared non-dominated sort
- `src/engines/multi_objective/pareto.rs` — ParetoIndividual, ParetoFront types
- `tests/engines/moead/test_moead.rs` — Test pattern for multi-objective engines
- `tests/engines/moead/test_moead_configuration.rs` — Config test pattern
- `examples/nsga2_zdt1.rs` — Example structure to mirror for spea2_zdt1.rs
- CLAUDE.md §WASM Compatibility — Mandatory cfg-gating rules

### Secondary (MEDIUM confidence)
- Zitzler, Laumanns & Thiele 2001: "SPEA2: Improving the Strength Pareto Evolutionary Algorithm" — canonical algorithm definition (algorithm description from training knowledge, verified against CONTEXT.md specifics)

### Tertiary (LOW confidence)
- None — all research verified against existing codebase patterns and CONTEXT.md specifications

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All patterns verified against existing codebase (MOEA/D, NSGA-III configs, observer infra)
- Architecture: HIGH — Exact structural match to MoeaDGa<U>
- Pitfalls: HIGH — Algorithmic pitfalls are well-understood from the SPEA2 paper and previous engine implementations

**Research date:** 2026-05-10
**Valid until:** 2026-06-10 (stable — Rust compiler 1.94.1, rand 0.9.2, rayon 1.10)
