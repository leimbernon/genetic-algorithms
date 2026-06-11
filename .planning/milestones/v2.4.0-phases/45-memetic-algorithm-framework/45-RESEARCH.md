# Phase 45: Memetic Algorithm Framework - Research

**Researched:** 2026-05-14
**Domain:** Memetic algorithm / local search operator framework for GA
**Confidence:** HIGH

## Summary

This phase adds a `LocalSearchOperator` trait with enum+fatory dispatch (following the exact same pattern as the 5 existing operators) and integrates it into the Ga engine's generation loop after mutation+fitness+repair+constraints and before population merge. The framework ships with a `HillClimbing` variant extracted from ScatterEngine, four application strategies (AllOffspring, BestN, Probabilistic, EveryNGenerations), and Lamarckian/Baldwinian modes. Parallelism via rayon `par_iter()` with WASM fallback.

The ScatterEngine's existing `local_search_improve()` at `src/engines/scatter/engine.rs:238-258` provides the reference implementation for HillClimbing. The key extraction challenge is that it uses `DeGene` trait methods (`de_value()`, `with_de_value()`) for real-valued perturbation. The new operator must likewise constrain to chromosome types that support numeric value operations, either through the existing `ValueMutable` trait or a new trait requirement.

**Primary recommendation:** Follow the exact operator pattern (trait + enum + factory) established by Crossover/Mutation/Selection/Survivor/Extension. Use `src/operations/local_search.rs` as the new module. The `LocalSearch` enum carries `HillClimbing` as the first variant with config params (step_size, max_iterations). Application strategies dispatch via a `LocalSearchApplicationStrategy` enum. Lamarckian/Baldwinian via a `LocalSearchMode` enum flag on `LocalSearchConfiguration`.

## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** LocalSearchOperator uses the full trait + enum + factory pattern, consistent with existing operators
- **D-02:** The trait method receives `&mut U + &dyn Fn(&[U::Gene]) -> f64` as parameters
- **D-03:** Fitness function is shared via `Arc::clone()` at the call site
- **D-04:** Local search executes AFTER crossover+mutation+fitness, AFTER repair+constraint penalty, BEFORE survivor selection
- **D-05:** Local search is NOT applied to extension regrown individuals
- **D-06:** Both Lamarckian and Baldwinian modes via a config flag, default Lamarckian
- **D-08:** Parallelism via rayon `par_iter()` over selected individuals
- **D-09:** WASM: `#[cfg(target_arch = "wasm32")]` fallback to sequential `iter()`
- **D-10:** Ga engine only (consistent with Phases 40-43 pattern)

### Claude's Discretion
- LocalSearch enum variants (HillClimbing as first variant; others reserved)
- Application strategy implementation details (BestN selection, Probabilistic defaults, EveryNGenerations interval defaults)
- `LocalSearchOperator` trait method signature details (strategy params per-call vs stored in struct)
- `LocalSearchConfiguration` struct fields and builder methods
- HillClimbing specific config (step_size, max_iterations from ScatterEngine defaults)
- Factory function location: `src/operations/local_search.rs`
- Serde derives on `LocalSearchConfiguration` and operator state structs
- Whether to support user-supplied custom local search strategies via closures
- Ga struct field: `local_search: Option<Box<dyn LocalSearchOperator<U>>>`

### Deferred Ideas (OUT OF SCOPE)
- Local search for non-Ga engines (Nsga2Ga, De, Scatter, Cellular, Alps)
- New GaObserver hooks specific to memetic events
- Per-gene or per-individual local search tracking in observability
- Built-in complex local search strategies (simulated annealing, tabu search, gradient descent)

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| MEM-01 | Memetic algorithm with local search operator trait | Full research below: trait pattern, HillClimbing extraction, application strategies, GA loop integration, WASM compat |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Local search operator trait | API/Backend | — | Operator runs inside GA generation loop; receives chromosome refs and fitness fn |
| Application strategy dispatch | API/Backend | — | Decides WHICH offspring get refined; uses fitness values from GA loop |
| Lamarckian/Baldwinian mode | API/Backend | — | Controls whether DNA is updated (Lamarckian) or only fitness (Baldwinian) |
| HillClimbing implementation | API/Backend | — | Pure computation: random perturbation + fitness re-eval + accept/revert |
| Parallel execution | API/Backend | Database/Storage | Rayon par_iter over selected individuals; WASM fallback to sequential |
| WASM compatibility constraint | API/Backend | — | cfg-gate rayon par_iter, no Instant/std::time needed |

## Standard Stack

### Core
| Library | Purpose | Why Standard |
|---------|---------|--------------|
| `LocalSearch` enum (new) | Runtime variant dispatch for local search strategies | Matches Crossover/Mutation/Survivor/Extension enum pattern |
| `LocalSearchOperator` trait (new) | Trait interface for custom local search impls | Matches CrossoverOperator/MutationOperator etc. |
| `LocalSearchConfiguration` (new) | Config struct with strategy, mode, hill-climbing params | Matches CrossoverConfiguration, MutationConfiguration pattern |
| `Ga` struct field (new) | `local_search: Option<Box<dyn LocalSearchOperator<U>>>` | Zero-overhead optional operator pattern (D-10 discretion) |

### Supporting
| Type | Purpose | When to Use |
|------|---------|-------------|
| `LocalSearchApplicationStrategy` enum | AllOffspring, BestN {n}, Probabilistic {p}, EveryNGenerations {interval} | Always with local search configured |
| `LocalSearchMode` enum | Lamarckian, Baldwinian | Always with local search configured |
| `HillClimbing` struct | HillClimbing config: step_size, max_iterations | Default first variant |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Enum + trait + factory | Trait-only dispatch | Enum pattern is the established project convention; traits alone would break consistency |
| Strategy trait objects for application strategies | Enum on LocalSearch | Enum is simpler; strategy objects would be more extensible but add complexity for 4 fixed variants |
| Store fitness fn in operator | Pass per-call | D-02/D-03 decide this: per-call via Arc::clone makes operator stateless and avoids lifetime issues |

## Architecture Patterns

### System Architecture Diagram

```
Generation loop (ga.rs)
    │
    ├── Selection ─────────────────► parent pairs
    ├── Crossover+Mutation+Fitness ─► offspring Vec<U>
    ├── Repair operator (if configured) ─► fixed offspring
    ├── Constraint penalty (if configured) ─► penalized offspring
    │
    │   ┌── Local Search Block (NEW) ─────────────────────────────┐
    │   │                                                          │
    │   │  1. Strategy dispatch (should we apply this gen?)        │
    │   │     │  └─ EveryNGenerations: skip if generation % N != 0 │
    │   │     │                                                     │
    │   │  2. Select candidates from offspring                      │
    │   │     │  └─ AllOffspring: use all offspring                 │
    │   │     │  └─ BestN: sort by fitness, pick top N              │
    │   │     │  └─ Probabilistic: filter by probability p          │
    │   │     │                                                     │
    │   │  3. Local search in parallel (rayon par_iter / WASM iter) │
    │   │     │  for each candidate:                                │
    │   │     │    local_search_operator.improve(&mut ind, fitness) │
    │   │     │      └─ HillClimbing: random perturbation,          │
    │   │     │         re-evaluate, accept/revert, repeat          │
    │   │     │                                                     │
    │   │  4. Apply Lamarckian/Baldwinian to accepted candidates    │
    │   │     └─ Lamarckian: set_dna(modified) + set_fitness(new)   │
    │   │     └─ Baldwinian: set_fitness(new) only, DNA preserved   │
    │   └───────────────────────────────────────────────────────────┘
    │
    ├── Population merge (add_chromosomes)
    ├── Hall of Fame update
    ├── Elitism → Survivor selection → Elite reinsertion
    ├── Niching → Best update → Stats → Extension → Checkpoint
    └── Observer notification → Stop check
```

### Recommended Project Structure
```
src/
├── traits/
│   └── operators.rs              # ADD LocalSearchOperator trait definition
├── operations/
│   ├── mod.rs                    # ADD pub mod local_search
│   ├── local_search.rs (NEW)     # LocalSearch enum + factory + HillClimbing impl
│   │   └── hill_climbing.rs (optional, inline or submodule)
│   └── ...
├── configuration.rs              # ADD LocalSearchConfiguration struct
├── traits/configuration.rs       # ADD LocalSearchConfig trait + builder methods
├── engines/ga.rs                 # ADD field, builder method, integration in loop
├── error.rs                      # ADD LocalSearchError variant to GaError
└── lib.rs                        # ADD pub use re-exports
```

### Pattern 1: LocalSearchOperator Trait
**What:** The 6th operator trait, following the exact signature conventions of CrossoverOperator, MutationOperator, etc. The key difference: it receives the fitness function as a parameter per-call (D-02) so it can re-evaluate fitness during refinement.

```rust
/// Trait for local search refinement operators for memetic algorithms.
///
/// Implement this trait to define a local search strategy that refines
/// individual chromosomes after genetic operators. Built-in implementations
/// are provided for the [`LocalSearch`] enum variants.
///
/// The trait method receives the fitness function at each call site
/// rather than storing it, enabling parallel execution via Arc::clone().
pub trait LocalSearchOperator {
    /// Apply local search refinement to a single individual.
    ///
    /// # Arguments
    ///
    /// * `individual` - Mutable reference to the chromosome to refine.
    /// * `fitness_fn` - Fitness function for re-evaluation during refinement.
    /// * `mode` - Lamarckian (update DNA+fitness) or Baldwinian (fitness only).
    ///
    /// # Returns
    ///
    /// The number of successful improvements made during refinement.
    fn improve<U>(
        &self,
        individual: &mut U,
        fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
    ) -> Result<usize, GaError>
    where
        U: ChromosomeT + Send + Sync + 'static + Clone;
}
```

### Pattern 2: LocalSearch Enum + Factory Dispatch
**What:** Standard enum dispatch following Crossover/Mutation pattern. The enum holds config params for each variant.

```rust
/// Local search refinement strategies for memetic algorithms.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocalSearch {
    /// Hill-climbing with random perturbations (default strategy).
    /// Perturbs one gene at a time; accepts if improvement found.
    HillClimbing(HillClimbingConfig),
}

/// Configuration for HillClimbing local search.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HillClimbingConfig {
    pub step_size: f64,
    pub max_iterations: usize,
}

impl Default for HillClimbingConfig {
    fn default() -> Self {
        Self {
            step_size: 0.1,   // From ScatterEngine default
            max_iterations: 20, // From ScatterEngine default
        }
    }
}
```

### Pattern 3: Application Strategies as Separate Enum
**What:** Strategies that control WHICH offspring receive local search refinement.

```rust
/// Determines which offspring receive local search refinement.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocalSearchApplicationStrategy {
    /// Apply local search to every offspring in the generation.
    AllOffspring,
    /// Apply local search only to the top N offspring by fitness.
    BestN { n: usize },
    /// Apply local search to each offspring with the given probability.
    Probabilistic { probability: f64 },
    /// Apply local search every N generations.
    EveryNGenerations { interval: usize },
}
```

### Pattern 4: GA Loop Integration (Insertion Point)
**What:** The local search block inserts between constraint penalty (~line 1392) and population merge (~line 1395), wrapped in an `if let Some(ref local_search) = self.local_search` guard.

```rust
// After constraint penalty block (~line 1392)
// Apply local search refinement to selected offspring before merge
if let Some(ref local_search_op) = self.local_search {
    let local_search_config = &self.configuration.local_search_configuration;
    let strategy = local_search_config.application_strategy;
    let mode = local_search_config.mode;
    
    // Step 1: Strategy gate - should we apply this generation?
    let should_apply = match strategy {
        LocalSearchApplicationStrategy::EveryNGenerations { interval } => {
            (i + 1) % interval == 0
        }
        _ => true,
    };
    
    if should_apply && !offspring.is_empty() {
        // Step 2: Select candidates
        let candidates: Vec<usize> = match strategy {
            LocalSearchApplicationStrategy::AllOffspring => {
                (0..offspring.len()).collect()
            }
            LocalSearchApplicationStrategy::BestN { n } => {
                // Sort by fitness, pick top N indices
                // Reuse sort-by-fitness logic (similar to elite extraction)
                let mut indices: Vec<usize> = (0..offspring.len()).collect();
                let ps = self.configuration.limit_configuration.problem_solving;
                indices.sort_unstable_by(|&a, &b| {
                    let cmp = offspring[a].fitness().partial_cmp(&offspring[b].fitness());
                    match ps {
                        ProblemSolving::Minimization => cmp.unwrap_or(std::cmp::Ordering::Equal),
                        ProblemSolving::Maximization => cmp.map(|o| o.reverse()).unwrap_or(std::cmp::Ordering::Equal),
                        _ => cmp.unwrap_or(std::cmp::Ordering::Equal),
                    }
                });
                indices.truncate(n.min(indices.len()));
                indices
            }
            LocalSearchApplicationStrategy::Probabilistic { probability } => {
                (0..offspring.len())
                    .filter(|_| rand::thread_rng().gen::<f64>() < probability)
                    .collect()
            }
            LocalSearchApplicationStrategy::EveryNGenerations { .. } => {
                (0..offspring.len()).collect() // Already gated above
            }
        };
        
        // Step 3: Apply local search in parallel
        let ff = Arc::clone(&self.fitness_fn.as_ref().unwrap());
        #[cfg(not(target_arch = "wasm32"))]
        {
            candidates.into_par_iter().for_each(|idx| {
                let _ = local_search_op.improve(&mut offspring[idx], ff.as_ref(), mode);
            });
        }
        #[cfg(target_arch = "wasm32")]
        {
            candidates.into_iter().for_each(|idx| {
                let _ = local_search_op.improve(&mut offspring[idx], ff.as_ref(), mode);
            });
        }
    }
}
// Population merge (~line 1395)
```

### Pattern 5: HillClimbing Implementation (Extracted from ScatterEngine)
**What:** The ScatterEngine's local search at `src/engines/scatter/engine.rs:238-258` uses random perturbation with accept/revert. The extracted version:
1. Needs RNG access (use `rand::thread_rng()` or seedable RNG)
2. Needs numeric gene operations (`de_value()` / `with_de_value()`) — requires DeGene trait or type downcasting
3. Evaluates fitness via the passed `fitness_fn` closure

```rust
impl LocalSearchOperator for HillClimbingConfig {
    fn improve<U>(
        &self,
        individual: &mut U,
        fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
    ) -> Result<usize, GaError>
    where
        U: ChromosomeT + Send + Sync + 'static + Clone,
    {
        let step = self.step_size;
        let mut current_fitness = individual.fitness();
        let dim = individual.dna().len();
        let mut improvements = 0;
        let mut rng = rand::thread_rng();
        
        for _ in 0..self.max_iterations {
            let j = rng.random_range(0..dim);
            // NOTE: This requires type-specific perturbation.
            // For generic implementation, we need DeGene trait or downcasting.
            // See "Understanding the DeGene constraint" section below.
            let delta: f64 = (rng.random::<f64>() * 2.0 - 1.0) * step;
            
            // Save old state
            let old_value = /* gene value at j */;
            // Set new perturbed value at j
            /* individual.dna_mut()[j] = perturbed */;
            
            let new_fitness = fitness_fn(individual.dna());
            if is_better(new_fitness, current_fitness, problem_solving) {
                current_fitness = new_fitness;
                improvements += 1;
            } else {
                // Revert: restore old value at j
                /* individual.dna_mut()[j] = old_value */;
            }
        }
        
        individual.set_fitness(current_fitness);
        Ok(improvements)
    }
}
```

**Critical constraint:** The ScatterEngine's implementation uses `DeGene` trait methods:
- `ind.dna()[j].de_value()` — get f64 value from the gene
- `ind.dna()[j].with_de_value(old_val + delta)` — create a perturbed copy

For the generic `LocalSearchOperator`, we need a mechanism to perturb genes. Options:
1. **Feasible for Phase 45:** Use runtime downcasting (like `try_polynomial` in mutation.rs) to `RangeChromosome<T>` where T supports f64 conversion
2. **Alternate approach:** Add a `GeneT::perturb(&mut self, step_size: f64)` default method (non-breaking)
3. **Simplest approach for HillClimbing on Range chromosomes:** Implement `HillClimbing` with downcasting similar to SBX crossover, with a graceful error for non-Range types

### Anti-Patterns to Avoid
- **Storing fitness function in the operator:** D-02/D-03 mandate passing fitness fn per-call. Storing it in the operator struct would prevent the Arc::clone sharing pattern needed for parallelism.
- **Applying local search to extension-regrown individuals:** D-05 explicitly excludes this. The local search block operates on `offspring` (the crossover result), not on the full merged population.
- **Modifying offspring in place without fitness recalc:** After local search changes DNA (Lamarckian), the individual must have `set_fitness()` called with the improved value. For Baldwinian, only fitness changes.
- **Attempting HillClimbing without numeric gene operations:** The HillClimbing variant must handle non-numeric chromosomes gracefully (return an error or no-op).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Strategy dispatch logic | Custom branching per strategy | `LocalSearchApplicationStrategy` enum | Enum is the project's established pattern; prevents combinatorial explosion when adding new strategies |
| HillClimbing with random perturbation | Domain-specific local search | The extracted ScatterEngine pattern | The perturbation + accept/revert loop is a general-purpose hill climber; specialization deferred |
| Serialization of config | Manual serde impl | `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` | Matches all existing config types exactly |
| Parallel execution management | Custom thread pools | `rayon::par_iter()` with WASM cfg | Established pattern used throughout the codebase |

**Key insight:** The memetic framework is primarily about wiring — providing the right trait, enum, config, and integration point. The actual refinement logic (HillClimbing) is secondary and serves as the default implementation. Users will extend with domain-specific local search via the trait pattern.

## Common Pitfalls

### Pitfall 1: Fitness Function Lifetime in Parallel Tasks
**What goes wrong:** The fitness function (`Arc<dyn Fn(&[U::Gene]) -> f64>`) must outlive the parallel rayon tasks. If cloned incorrectly, the Arc may be dropped before all tasks complete.
**Why it happens:** Rayon tasks execute on a thread pool; the local scope may exit before all tasks finish.
**How to avoid:** Always `Arc::clone()` the fitness function before the generation loop, and clone again inside the parallel block. The AOS integration (lines 1199-1213) in ga.rs is the canonical reference.
**Warning signs:** Use-after-free or "cannot move out of borrowed content" compiler errors.

### Pitfall 2: Lamarckian Fitness Staleness
**What goes wrong:** After Lamarckian local search updates the DNA, the stored fitness no longer matches what the fitness function would return.
**Why it happens:** LocalSearchOperator updates both DNA and `set_fitness()` but the chromosome's stored fitness (set by `set_fitness()`) may diverge from `calculate_fitness()`.
**How to avoid:** Always call `set_fitness(new_fitness)` after DNA modification in the operator. For Baldwinian, only call `set_fitness()` without touching DNA. Validate that survivor selection and evolution use `individual.fitness()` consistently (they do — the loop reads fitness() not calculate_fitness()).

### Pitfall 3: EveryNGenerations Strategy and Generation 0
**What goes wrong:** `EveryNGenerations { interval: 1 }` means every generation, but the check `(i + 1) % interval == 0` may exclude generation 0 depending on framing.
**Why it happens:** The GA loop's `i` is 0-based (absolute generation counter from `start_gen`). The interval check's framing matters.
**How to avoid:** Test with `interval: 1` to ensure it applies to all generations. Use `start_gen == 0` framing clarity: `(i + 1 - start_gen) % interval == 0` for relative, or `(i + 1) % interval == 0` for absolute.

### Pitfall 4: BestN Selection on Unsorted Offspring
**What goes wrong:** BestN assumes offspring are in arbitrary order; selecting by index without sorting gives random results.
**Why it happens:** The offspring vector is produced by parent_crossover() which does not sort by fitness.
**How to avoid:** Explicitly sort offspring indices by fitness (reusing the sort logic from `extract_elite()` at lines 1405-1413). Use `select_nth_unstable_by()` for partial sort efficiency.

### Pitfall 5: WASM Rayon Gate
**What goes wrong:** `par_iter()` called on wasm32 target panics at runtime.
**Why it happens:** Rayon does not compile/function on wasm32-unknown-unknown.
**How to avoid:** Always apply the dual-path pattern (verified in extension regrowth, lines 1624-1662):
```rust
#[cfg(not(target_arch = "wasm32"))]
let result: Vec<_> = items.par_iter().map(|x| op(x)).collect();
#[cfg(target_arch = "wasm32")]
let result: Vec<_> = items.iter().map(|x| op(x)).collect();
```

## Understanding the DeGene/Value Constraint for HillClimbing

The ScatterEngine's `local_search_improve()` uses `DeGene` trait methods that provide f64 access to gene values. For the generic HillClimbing implementation, several approaches exist:

### Approach A: Use `ValueMutable` trait (Already exists)
The `ValueMutable` trait at `src/operations/mutation.rs:140` provides methods like `creep_mutate(step)` which perturb a chromosome by applying small uniform changes. However, this does not support the `accept/revert` pattern needed for hill-climbing.

### Approach B: Runtime Type Downcasting
Follow the pattern from SBX/Polynomial/BLX-alpha operators: downcast to `RangeChromosome<f64>` and work directly with f64 values:
```rust
fn try_hill_climb<U: ChromosomeT + 'static>(
    individual: &mut U,
    step_size: f64,
    steps: usize,
    fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
) -> Option<Result<usize, GaError>> {
    // Try downcast to RangeChromosome<f64>
    // Perturb f64 values, accept/revert based on fitness
}
```
This is the **recommended approach for Phase 45** — it matches the established pattern for Range-specific operators.

### Approach C: Generic Single-Gene Perturbation (Future)
A generic approach using `GeneT::id()` would work for any chromosome type but would be limited to swapping/cloning genes rather than perturbing numeric values. This is deferred as not useful for the primary use case.

## Code Examples

### Example 1: LocalSearch trait definition (in `src/traits/operators.rs`)
```rust
/// Trait for local search refinement operators used in memetic algorithms.
///
/// Implement this trait to define a custom local search strategy. Built-in
/// implementations are provided for the [`LocalSearch`] enum variants.
///
/// The fitness function is received as a parameter at each call site (D-02),
/// enabling it to be Arc::cloned across parallel refinement tasks (D-03).
///
/// # Example
///
/// ```rust,ignore
/// use genetic_algorithms::traits::LocalSearchOperator;
///
/// struct MyLocalSearch;
///
/// impl LocalSearchOperator for MyLocalSearch {
///     fn improve<U: ChromosomeT + Send + Sync + 'static + Clone>(
///         &self,
///         individual: &mut U,
///         fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
///     ) -> Result<usize, GaError> {
///         // Custom local search logic here
///         Ok(0)
///     }
/// }
/// ```
pub trait LocalSearchOperator {
    /// Apply local search refinement to a single individual.
    ///
    /// # Arguments
    /// * `individual` - The chromosome to refine in-place.
    /// * `fitness_fn` - Fitness function for re-evaluation during refinement.
    ///
    /// # Returns
    /// Number of successful improvements made, or an error.
    fn improve<U>(
        &self,
        individual: &mut U,
        fitness_fn: &dyn Fn(&[U::Gene]) -> f64,
    ) -> Result<usize, GaError>
    where
        U: ChromosomeT + Send + Sync + 'static + Clone;
}
```

### Example 2: GaConfiguration fields (in `src/configuration.rs`)
```rust
// Add to GaConfiguration struct (before closing brace):
    /// Optional local search configuration for memetic algorithm.
    /// When `None`, no local search is performed (zero overhead).
    pub local_search_configuration: Option<LocalSearchConfiguration>,

// New configuration struct:
/// Configuration for local search refinement in memetic algorithms.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalSearchConfiguration {
    /// The local search operator variant (e.g., HillClimbing).
    pub method: LocalSearch,
    /// Which offspring receive local search refinement.
    pub application_strategy: LocalSearchApplicationStrategy,
    /// Lamarckian (update DNA+fitness) or Baldwinian (fitness only).
    pub mode: LocalSearchMode,
}

impl Default for LocalSearchConfiguration {
    fn default() -> Self {
        Self {
            method: LocalSearch::HillClimbing(HillClimbingConfig::default()),
            application_strategy: LocalSearchApplicationStrategy::AllOffspring,
            mode: LocalSearchMode::Lamarckian,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LocalSearchMode {
    Lamarckian,
    Baldwinian,
}

impl Default for LocalSearchMode {
    fn default() -> Self { Self::Lamarckian }
}
```

### Example 3: Ga struct field + builder (in `src/engines/ga.rs`)
```rust
// Add to Ga<U> struct fields after the AOS fields (~line 206):
    /// Optional local search operator for memetic refinement. (Phase 45)
    /// When `None` (default), no local search is performed.
    /// When `Some(Box<dyn LocalSearchOperator<U>>)`, the operator is applied
    /// to selected offspring each generation after mutation and before merge.
    local_search: Option<Box<dyn LocalSearchOperator<U>>>,

// Add to Default impl (~line 240):
    local_search: None,

// Add builder method:
    /// Configures a local search operator for memetic algorithm refinement.
    ///
    /// When configured, the local search operator is applied to offspring
    /// after crossover+mutation+fitness and after repair/constraints,
    /// before population merge and survivor selection.
    pub fn with_local_search(mut self, operator: Box<dyn LocalSearchOperator<U>>) -> Self {
        self.local_search = Some(operator);
        self
    }
```

### Example 4: BestN candidate selection pattern
```rust
// In the local search block within the generation loop:
let candidates: Vec<usize> = match strategy {
    LocalSearchApplicationStrategy::BestN { n } => {
        let mut indices: Vec<usize> = (0..offspring.len()).collect();
        let ps = self.configuration.limit_configuration.problem_solving;
        // Partial sort for efficiency (select_nth_unstable_by)
        indices.select_nth_unstable_by(
            n.min(indices.len()).saturating_sub(1),
            |&a, &b| {
                let fa = offspring[a].fitness();
                let fb = offspring[b].fitness();
                match ps {
                    ProblemSolving::Minimization => fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal),
                    ProblemSolving::Maximization => fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal),
                    _ => fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal),
                }
            },
        );
        indices.truncate(n.min(indices.len()));
        indices
    }
    // ... other strategy branches
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| ScatterEngine's private `local_search_improve()` | Generic `LocalSearchOperator` trait | Phase 45 | Extracted from engine-specific code to reusable framework component |
| Manual local search per engine | Shared trait + factory dispatch | Phase 45 | Consistent with all other operators; enables user-defined strategies |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `GaError::LocalSearchError` variant is the right way to represent local search failures | Architecture | Low — matches existing error pattern; alternatives are generic `GaError::ConfigurationError` |
| A2 | Default HillClimbingConfig values (step_size=0.1, max_iterations=20) from ScatterEngine are suitable | Standard Stack | Low — users can override via builder; ScatterEngine defaults are already battle-tested |
| A3 | BestN should re-sort offspring and truncate, not use elite extraction | Code Examples | Low — both approaches produce correct results; sort+truncate avoids extracting (and potentially dropping) individuals from the offspring vector |
| A4 | Application strategies are modeled as a separate enum (not LocalSearch variants) | Architecture | Low — the strategy is orthogonal to the refinement method; separating them avoids combinatorial explosion |
| A5 | The `improve()` method returns `usize` (count of improvements) | Trait Design | Low — the return value is informational; could also return `()` if unused |

## Open Questions

1. **Does the `improve()` trait method need per-call strategy parameters?**
   - What we know: D-02 says trait method receives `&mut U + &dyn Fn(&[U::Gene]) -> f64`. Claude's discretion allows deciding whether to pass strategy params per-call or store in struct.
   - What's unclear: Should `improve()` receive `&LocalSearchApplicationStrategy` or `&LocalSearchMode` as additional parameters?
   - Recommendation: Store strategy and mode in `LocalSearchConfiguration` (not passed per-call). The trait method should only receive the individual and fitness_fn — the application strategy decides WHICH individuals to refine, not HOW to refine them. The mode (Lamarckian/Baldwinian) is applied at the call site (in the GA loop), not inside the operator. This keeps the trait minimal.

2. **How does HillClimbing handle non-Range chromosomes types?**
   - What we know: The ScatterEngine's implementation uses `DeGene` trait which is specific to engines requiring f64 arithmetic. Range chromosomes are the only type with numeric perturbable values.
   - What's unclear: Should HillClimbing fail gracefully for non-Range types, or should we always require type downcasting?
   - Recommendation: Follow the SBX/BLX-alpha pattern — downcast to `RangeChromosome<f64>` and return a clear error message for unsupported types. This keeps the framework generic while giving users who use non-Range chromosomes actionable guidance.

3. **Should there be a `ValueMutable`-like trait bound on the operator for HillClimbing?**
   - What we know: The operator trait is generic over U: ChromosomeT. HillClimbing needs to read/write individual gene values.
   - What's unclear: Adding a trait bound to the `improve()` method limits what chromosome types can use local search. The downcasting approach is the established pattern.
   - Recommendation: Use the downcasting approach (like SBX). No additional trait bound on the `improve()` method.

4. **Is the `src/operations/mod.rs` file at `src/operations.rs` (not a directory)?**
   - What we know: The operations module uses `mod.rs` in each subdirectory (e.g., `src/operations/crossover/mod.rs`) but the top-level enum file is `src/operations.rs` (flat file, not a directory mod.rs).
   - What's unclear: Where exactly should `pub mod local_search;` be added?
   - Recommendation: Add `pub mod local_search;` to `src/operations.rs` alongside the existing 5 module declarations.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rayon | Parallel local search | Yes | — | Sequential iter() for WASM |
| rand | HillClimbing perturbation | Yes | — | — |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | Cargo.toml |
| Quick run command | `cargo test --test engines test_ga` |
| Full suite command | `cargo test && cargo test --features serde` |

### Validation Dimensions

The plan should validate these behaviors:

1. **Trait contract:** `LocalSearchOperator::improve()` is called with `&mut U` and the fitness function. Verified by a unit test that provides a mock fitness fn and checks it is called.

2. **HillClimbing correctness:** On a simple fitness function (sphere), HillClimbing with `step_size=0.1, max_iterations=20` improves fitness. Verified by comparing pre- and post-refinement fitness.

3. **Lamarckian mode:** After local search, `individual.dna()` differs from the original (DNA was modified). Verify by cloning before, comparing after.

4. **Baldwinian mode:** After local search, `individual.dna()` matches the original (preserved). `individual.fitness()` is updated. Verify both conditions.

5. **Application strategies:**
   - `AllOffspring`: Verify ALL offspring indices are refined.
   - `BestN { n=2 }`: Verify exactly 2 offspring are refined (prefer the best ones).
   - `Probabilistic { probability=1.0 }`: Verify ALL offspring are refined.
   - `EveryNGenerations { interval=2 }`: Verify refinement at gen 0, 2, 4... but NOT at gen 1, 3, 5...

6. **GA loop integration:** Smoke test: configure Ga with local search, run on a standard benchmark, verify the run completes and produces a non-NaN best fitness.

7. **WASM compatibility:** `cargo check --target wasm32-unknown-unknown` passes.

8. **Serde roundtrip:** `LocalSearchConfiguration` serializes and deserializes correctly.

### Wave 0 Gaps
- [ ] `tests/engines/local_search.rs` (NEW) — covers all validation dimensions above
- [ ] `tests/engines/test_ga.rs` — add a test case with local search configured

## Security Domain

> Security enforcement is not applicable for this phase (local search operator framework, no network/IO/user input involved beyond standard configuration).

### Applicable ASVS Categories
| ASVS Category | Applies | Reason |
|---------------|---------|--------|
| All ASVS categories | No | Memetic algorithm framework is pure computation with no external input, authentication, or data storage |

## Sources

### Primary (HIGH confidence)
- `src/traits/operators.rs` [VERIFIED: codebase] — All 5 existing operator trait definitions. Pattern to follow for LocalSearchOperator
- `src/engines/scatter/engine.rs:238-258` [VERIFIED: codebase] — Reference hill-climbing implementation with DeGene constraints
- `src/engines/ga.rs:1089-1784` [VERIFIED: codebase] — Full generation loop with AOS integration as reference for complex feature insertion
- `src/configuration.rs` [VERIFIED: codebase] — GaConfiguration, CrossoverConfiguration, MutationConfiguration patterns
- `src/operations.rs` [VERIFIED: codebase] — All operator enums (Selection, Crossover, Mutation, Survivor, Extension) and module declarations
- `src/operations/crossover.rs` [VERIFIED: codebase] — Enum + factory dispatch pattern including type downcasting
- `src/operations/mutation.rs` [VERIFIED: codebase] — factory_with_params pattern, ValueMutable trait, type downcasting pattern
- `src/operations/selection.rs` [VERIFIED: codebase] — Enum impl + factory pattern
- `src/operations/survivor.rs` [VERIFIED: codebase] — Factory dispatch pattern
- `src/operations/extension/mod.rs` [VERIFIED: codebase] — Extension operator factory pattern
- `src/engines/scatter/configuration.rs` [VERIFIED: codebase] — Defaults: step_size=0.1, steps=20
- `src/traits/configuration.rs` [VERIFIED: codebase] — ConfigurationT supertrait pattern
- `src/traits/chromosome.rs` [VERIFIED: codebase] — ChromosomeT trait (fitness(), set_fitness(), dna(), dna_mut(), set_dna(), set_gene())
- `src/error.rs` [VERIFIED: codebase] — GaError enum for new error variant
- `src/lib.rs` [VERIFIED: codebase] — Re-export patterns
- `CLAUDE.md` [VERIFIED: codebase] — WASM compatibility rules, project conventions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Verified by reading all 5 existing operator traits, their enum definitions, factory patterns, and configuration patterns
- Architecture: HIGH — Generation loop insertion point confirmed at precise line locations; AOS integration pattern provides reference for field+init+builder+loop integration
- Pitfalls: MEDIUM — DeGene constraint for HillClimbing is a real concern; ScatterEngine uses it but the generic approach (type downcasting) mirrors the established SBX/BLX-alpha pattern

**Research date:** 2026-05-14
**Valid until:** 7 days (Rust/rayon ecosystem is stable)
