# CMA-ES

> Covariance Matrix Adaptation Evolution Strategy — self-adapting continuous optimization with Jacobi eigendecomposition (no LAPACK, WASM-compatible).

## Description

CMA-ES (Hansen & Ostermeier 2001) evolves a multivariate Gaussian search distribution over continuous real-valued space. Each generation it samples `λ` candidate solutions, evaluates their fitness, then updates:

- **Mean** — toward the best candidates.
- **Covariance matrix** — to learn the shape and orientation of the fitness landscape (rank-one and rank-µ updates).
- **Step size `σ`** — via a separate cumulation path to control overall scale.

The covariance matrix allows CMA-ES to adapt to correlated/non-separable landscapes that defeat separable optimizers. The implementation uses **Jacobi eigendecomposition** — no LAPACK dependency, fully WASM-compatible.

Module path: `genetic_algorithms::cma`.  
Public exports: `CmaEngine`, `CmaConfiguration`, `CmaResult`, `RestartStrategy`.

## When to Use

- **Problem type:** Continuous real-valued optimization (sphere, Rosenbrock, Rastrigin, etc.)
- **Variable type:** `RangeChromosome<f64>` with `RealGene`-bounded genes
- **Key strength:** Self-adapts the covariance matrix to handle non-separable and correlated fitness landscapes; most internal parameters auto-tune (Hansen's formulas)
- **Key weakness:** Covariance matrix is O(n²) memory and O(n²) per update — practical upper bound is roughly 40 dimensions; prefer PSO for higher dimensions

## Quick Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `sigma0` | `f64` | `0.3` | Initial step size σ₀. Heuristic: typically 1/5 to 1/3 of the expected search range per dimension. |
| `population_size` | `usize` | `0` (auto) | λ. If 0, auto-computes `4 + floor(3·ln(n))` (Hansen's default) at `run()`. |
| `max_generations` | `usize` | `1000` | Stop after this many generations. |
| `problem_solving` | `ProblemSolving` | `Minimization` | Minimize or maximize fitness. |
| `fitness_target` | `Option<f64>` | `None` | Early-stop when the best fitness reaches this target. |
| `cc` | `Option<f64>` | `None` | Covariance matrix cumulation rate. `None` = Hansen auto-formula. |
| `cs` | `Option<f64>` | `None` | Step-size control cumulation rate. `None` = Hansen auto-formula. |
| `c1` | `Option<f64>` | `None` | Rank-one update rate. `None` = Hansen auto-formula. |
| `cmu` | `Option<f64>` | `None` | Rank-µ update rate. `None` = Hansen auto-formula. Constraint: `c1 + cmu ≤ 1`. |
| `restart_strategy` | `Option<RestartStrategy>` | `None` | IPOP or BIPOP restart strategy; `None` = no automatic restarts. |
| `fitness_cache_size` | `Option<usize>` | `None` | LRU cache capacity for fitness evaluations (avoids re-evaluating duplicate DNA). |

### Restart Strategies

`RestartStrategy` has two variants:

- **`Ipop { population_scale, stagnation_threshold, max_restarts }`** — doubles (or scales by `population_scale`) the population on each stagnation event. Promotes global exploration on multimodal landscapes.
- **`Bipop { population_scale, small_population_size, stagnation_threshold, max_restarts }`** — alternates between a large restart (IPOP-style, odd restarts) and a small restart (cheap basin exploitation, even restarts).

## Complete Example

```rust,ignore
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine, RestartStrategy};
use genetic_algorithms::configuration::ProblemSolving;

const DIMENSIONS: usize = 10;

// default_for_dim sets population_size = 4 + floor(3·ln(n)) automatically
let config = CmaConfiguration::default_for_dim(DIMENSIONS)
    // Search domain [-5.12, 5.12] → range 10.24 → sigma0 ≈ 10.24 / 3 ≈ 3.4
    .with_sigma0(3.4)
    .with_max_generations(1000)
    .with_problem_solving(ProblemSolving::Minimization)
    // IPOP restarts for multimodal landscapes (e.g. Rastrigin)
    .with_restart_strategy(RestartStrategy::Ipop {
        population_scale: 2.0,
        stagnation_threshold: 50,
        max_restarts: 9,
    })
    // Optional: cache repeated DNA evaluations
    .with_fitness_cache(256);

let mut engine = CmaEngine::new(config, init_population, fitness_fn);
let result = engine.run().expect("CMA-ES run should succeed");

println!("Generations: {}", result.generations);
println!("Best fitness: {:.6}", result.best_fitness);
```

## Configuration Tips

- **sigma0 heuristic:** Set `sigma0` to roughly 1/5 to 1/3 of the expected search range per dimension. For a domain of `[-5.12, 5.12]` (range 10.24), a value of `3.4` is a good starting point. Too-small `sigma0` causes slow convergence; too-large causes noisy search.
- **population_size = 0 (auto):** `default_for_dim(n)` or leaving `population_size = 0` triggers Hansen's formula `λ = 4 + floor(3·ln(n))`. Increase explicitly for better exploration at the cost of more evaluations per generation.
- **IPOP restarts:** Use `RestartStrategy::Ipop` when the landscape is multimodal (e.g. Rastrigin). The population doubles on each stagnation event, helping escape local optima. `stagnation_threshold = 50` and `max_restarts = 9` are common starting values.
- **cc, cs, c1, cmu:** Leave as `None` unless you have strong domain knowledge — Hansen's auto-formulas are near-optimal for most problems.

## When to Choose This vs PSO

| Factor | CMA-ES | PSO |
|--------|--------|-----|
| Mechanism | Covariance matrix adaptation (learns landscape geometry) | Swarm velocity + personal/neighborhood best attraction |
| Hyperparameters | More internal parameters, but most auto-tune (Hansen formulas) | Few parameters (inertia, c1, c2) |
| Dimensionality | Practical limit ~40 dims (O(n²) covariance overhead) | Scales to higher dimensions (no covariance matrix) |
| Landscape | Non-separable, correlated, rugged — CMA adapts its distribution shape | Smooth, unimodal — PSO converges fast with Global topology |
| Multimodal | IPOP/BIPOP restarts add multimodal support | Ring topology helps but premature convergence is a risk |

## References

- Hansen, N., & Ostermeier, A. (2001). Completely derandomized self-adaptation in evolution strategies. _Evolutionary Computation_, 9(2), 159–195.
- Auger, A., & Hansen, N. (2005). A restart CMA evolution strategy with increasing population size. _Proceedings of the 2005 IEEE Congress on Evolutionary Computation_ (Vol. 2, pp. 1769–1776).

## See Also

- [Engines Overview](engines.md) — Full engine decision matrix and engine selection guide
- [PSO](pso.md) — Swarm-based alternative for smooth/higher-dimensional landscapes
- [Example: cma_es_rastrigin](../examples/cma_es_rastrigin.rs) — Runnable 5D Rastrigin minimization
- [docs.rs/genetic_algorithms::cma](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/cma/index.html) — Module API reference
