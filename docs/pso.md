# PSO

> Particle Swarm Optimization — swarm-based continuous optimization with configurable inertia strategies and neighborhood topologies.

## Description

PSO (Kennedy & Eberhart 1995) evolves a swarm of particles across a continuous real-valued search space. Each particle maintains:

- **Position** — the current candidate solution.
- **Velocity** — direction and magnitude of movement.
- **Personal best** — the best position this particle has ever visited.

On each generation, velocity is updated from three components:

1. **Inertia** — retains a fraction of the current velocity (controls exploration vs exploitation balance).
2. **Cognitive term** — pulls the particle toward its personal best (weighted by `c1`).
3. **Social term** — pulls the particle toward the best position found by its neighborhood (weighted by `c2`).

PSO has few hyperparameters, no covariance matrix overhead, and converges fast on smooth landscapes.

Module path: `genetic_algorithms::pso`.  
Public exports: `PsoEngine`, `PsoConfiguration`, `PsoInertia`, `PsoTopology`.

## When to Use

- **Problem type:** Continuous real-valued optimization where gradients are unavailable
- **Variable type:** `RangeChromosome<f64>` with `RealGene`-bounded genes
- **Key strength:** Few hyperparameters, fast convergence on smooth/unimodal landscapes; scales to higher dimensions than CMA-ES (no covariance matrix)
- **Key weakness:** Prone to premature convergence on multimodal problems with Global topology; no correlation modeling

## Quick Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `population_size` | `usize` | `30` | Number of particles in the swarm. |
| `max_generations` | `usize` | `1000` | Stop after this many generations. |
| `problem_solving` | `ProblemSolving` | `Minimization` | Minimize or maximize fitness. |
| `fitness_target` | `Option<f64>` | `None` | Early-stop when the best fitness reaches this target. |
| `inertia` | `PsoInertia` | `LinearDecay { w_start: 0.9, w_end: 0.4 }` | Velocity retention strategy (see below). |
| `c1` | `f64` | `2.0` | Cognitive coefficient — personal-best attraction strength. |
| `c2` | `f64` | `2.0` | Social coefficient — neighborhood-best attraction strength. |
| `topology` | `PsoTopology` | `Global` | Neighborhood topology (see below). |
| `fitness_cache_size` | `Option<usize>` | `None` | LRU cache capacity for fitness evaluations. |

### Inertia Strategies

`PsoInertia` has two variants:

- **`Constant(f64)`** — fixed inertia weight every generation. Common value: `0.729` (Clerc's constriction coefficient) when paired with `c1 = c2 = 1.49445`.
- **`LinearDecay { w_start: f64, w_end: f64 }`** — linearly decreases from `w_start` to `w_end` over the run. Recommended defaults (Shi & Eberhart 1998): `w_start = 0.9, w_end = 0.4`. High initial inertia promotes broad exploration; low final inertia promotes fine-grained exploitation.

### Topologies

`PsoTopology` has two variants:

- **`Global`** — gbest topology: every particle is attracted toward the single global best position ever found. Fast convergence but prone to premature convergence on multimodal landscapes.
- **`Ring { neighborhood_size: usize }`** — lbest topology: each particle is attracted toward the best position within its `neighborhood_size` nearest neighbors by index (ring-wrapped). For `neighborhood_size = 2`, the neighborhood of particle `i` is `{ i-1, i, i+1 }`. Slower convergence but better exploration on multimodal problems.

## Complete Example

```rust,ignore
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
use genetic_algorithms::configuration::ProblemSolving;

// Typical recommended config: LinearDecay + Ring for multimodal landscapes.
// Clerc's constriction: Constant(0.729) with c1=c2=1.49445 is a stable alternative.
let config = PsoConfiguration::default()
    .with_population_size(50)
    .with_max_generations(1000)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_inertia(PsoInertia::LinearDecay { w_start: 0.9, w_end: 0.4 })
    .with_topology(PsoTopology::Ring { neighborhood_size: 2 })
    .with_c1(1.49445)
    .with_c2(1.49445)
    // Optional: cache repeated DNA evaluations
    .with_fitness_cache_size(256);

let mut engine = PsoEngine::new(config, init_population, fitness_fn);
let result = engine.run().expect("PSO run should succeed");

println!("Generations: {}", result.generations);
println!("Best fitness: {:.6}", result.best_fitness);
```

## Configuration Tips

- **Global topology** converges fastest on unimodal or smooth landscapes — use when the search space has a single basin.
- **Ring topology** trades convergence speed for multimodal exploration. `neighborhood_size = 2` (3-particle neighborhood) is a conservative starting point; larger values approximate Global.
- **Clerc's constriction:** Use `PsoInertia::Constant(0.729)` with `c1 = c2 = 1.49445`. This is a stable alternative to `LinearDecay` and avoids manual tuning of the inertia schedule.
- **LinearDecay defaults** (`w_start = 0.9, w_end = 0.4`) from Shi & Eberhart (1998) are a good starting point for most problems and are the library default.
- **population_size:** PSO literature standard is 30 particles (dimension-independent). Increase for higher-dimensional or multimodal problems.

## When to Choose This vs CMA-ES

| Factor | PSO | CMA-ES |
|--------|-----|--------|
| Mechanism | Swarm velocity + personal/neighborhood best | Covariance matrix adaptation (learns landscape geometry) |
| Hyperparameters | Few (inertia, c1, c2) | More internal parameters, but most auto-tune (Hansen formulas) |
| Dimensionality | Scales to higher dimensions — no covariance overhead | Practical limit ~40 dims (O(n²) covariance update) |
| Landscape correlation | No correlation modeling | Adapts to non-separable, correlated landscapes |
| Convergence on smooth | Fast with Global topology | Competitive but heavier per-generation update |

## References

- Kennedy, J., & Eberhart, R. (1995). Particle swarm optimization. _Proceedings of the 1995 IEEE International Conference on Neural Networks_ (Vol. 4, pp. 1942–1948).
- Shi, Y., & Eberhart, R. (1998). A modified particle swarm optimizer. _Proceedings of the 1998 IEEE International Conference on Evolutionary Computation_ (pp. 69–73).

## See Also

- [Engines Overview](engines.md) — Full engine decision matrix and engine selection guide
- [CMA-ES](cma.md) — Covariance-adapting alternative for correlated/rugged landscapes
- [Example: pso_rastrigin](../examples/pso_rastrigin.rs) — Runnable 10D Rastrigin minimization
- [docs.rs/genetic_algorithms::pso](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/pso/index.html) — Module API reference
