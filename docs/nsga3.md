# NSGA-III

> Reference-point based Non-dominated Sorting Genetic Algorithm for many-objective optimization (3+ objectives).

## Description

NSGA-III (Deb & Jain 2014) extends NSGA-II for **many-objective** optimization by replacing crowding distance with **reference-point association** on the unit hyperplane. This solves the diversity-collapse problem that NSGA-II suffers at higher objective counts.

Reference points are auto-generated via the **Das-Dennis simplex lattice** with subdivision count `p`, or user-supplied as a custom set. The number of auto-generated points is `C(p + M - 1, M - 1)` where M is the number of objectives.

Per generation, NSGA-III:
1. **Non-dominated sort** the parent population (for tournament ranks).
2. **Create offspring** via binary tournament + crossover + mutation.
3. **Merge** parent + offspring (size 2N).
4. **Non-dominated sort** the combined population.
5. **Environmental selection:**
   a. Take all fronts that fit entirely into the next population.
   b. **Normalise** onto the unit hyperplane (translate by ideal, scale by ASF-derived intercepts).
   c. **Associate** each individual to its nearest reference point (perpendicular distance).
   d. **Niche preservation** — pick from under-populated niches, preferring the closest candidate.

## When to Use

- **Problem type:** Many-objective (3+ objectives)
- **Number of objectives:** 3+
- **Variable type:** Continuous (real-valued), binary
- **Key strength:** Maintains diversity at 3-15+ objectives where crowding distance collapses. Reference points distribute solutions evenly across the entire Pareto front.
- **Key weakness:** Reference point generation assumes the ideal and nadir are known/computable. Degenerate cases need epsilon clamping. The normalisation step (ASF-based intercepts) is O(M*|St|).

## Quick Reference

### Mandatory Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `num_objectives` | `usize` | `3` | Number of objectives (>= 3 typical). |
| `population_size` | `usize` | `100` | Population size (>= 2). |
| `max_generations` | `usize` | `200` | Maximum generations. |
| `init_fn` | `Fn` | — | Chromosome initialization function. |
| `objective_fns` | `Vec<ObjectiveFn>` | — | One closure per objective. |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `objective_directions` | `Vec<ObjectiveDirection>` | All `Minimize` | Per-objective Min/Max. |
| `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
| `observer` | `Nsga3Observer<U>` | `None` | Lifecycle observer. |

### Reference Points

Reference points are **mandatory** — configure either via:
- `with_reference_points_auto(p)` — Das-Dennis lattice with `p` divisions per objective. Typical `p = 12` for 3 objectives (91 points).
- `with_reference_points(points)` — custom `Vec<Vec<f64>>`. Each point must be non-negative and sum to approximately 1.0.

## Complete Example

```rust,ignore
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::nsga3::Nsga3Ga;
use genetic_algorithms::nsga3::configuration::Nsga3Configuration;
use genetic_algorithms::operations::{Crossover, Mutation};
use std::sync::Arc;

// Define a chromosome type
type MyChromosome = RangeChromosome<f64>;

// NSGA-III-specific configuration
let nsga3_config = Nsga3Configuration::new()
    .with_num_objectives(3)
    .with_population_size(91)
    .with_max_generations(300)
    .with_reference_points_auto(12);

// Base GA configuration (operators, crossover/mutation probabilities)
let ga_config = GaConfiguration::default()
    .with_crossover_method(Crossover::Sbx)
    .with_mutation_method(Mutation::Polynomial);

let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut nsga3 = Nsga3Ga::<MyChromosome>::new(nsga3_config, ga_config)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(false))
    })
    .with_objective_fns(vec![
        // DTLZ2 objective 1
        Box::new(|dna: &[RangeGenotype<f64>]| -> f64 {
            let n = dna.len();
            let g = dna[2..].iter()
                .map(|g| (g.value - 0.5).powi(2))
                .sum::<f64>();
            (1.0 + g) * (std::f64::consts::FRAC_PI_2 * dna[0].value).cos()
                * (std::f64::consts::FRAC_PI_2 * dna[1].value).cos()
        }),
        // DTLZ2 objective 2
        Box::new(|dna: &[RangeGenotype<f64>]| -> f64 {
            let n = dna.len();
            let g = dna[2..].iter()
                .map(|g| (g.value - 0.5).powi(2))
                .sum::<f64>();
            (1.0 + g) * (std::f64::consts::FRAC_PI_2 * dna[0].value).cos()
                * (std::f64::consts::FRAC_PI_2 * dna[1].value).sin()
        }),
        // DTLZ2 objective 3
        Box::new(|dna: &[RangeGenotype<f64>]| -> f64 {
            let n = dna.len();
            let g = dna[2..].iter()
                .map(|g| (g.value - 0.5).powi(2))
                .sum::<f64>();
            (1.0 + g) * (std::f64::consts::FRAC_PI_2 * dna[0].value).sin()
        }),
    ])
    .build()
    .expect("Valid NSGA-III configuration");

let pareto_front = nsga3.run().expect("NSGA-III run failed");
println!("Front size: {}", pareto_front.len());
```

## Configuration Tips

- Population size should roughly match the number of reference points. For Das-Dennis with 3 objectives and `p = 12`, you get 91 points, so set pop_size around 91.
- For 5+ objectives, decrease `p` to keep the reference-set size manageable (e.g., `p = 6` for 5 objectives yields 252 points).
- Binary tournament in NSGA-III uses rank only (no crowding distance). Ties are broken randomly.
- SBX crossover (eta=20) and Polynomial mutation (eta_m=20) are the recommended operators for real-coded NSGA-III.

## When to Choose This vs MOEA/D

| Factor | NSGA-III | MOEA/D |
|--------|----------|--------|
| Mechanism | Reference-point niche preservation | Weight-vector decomposition |
| Objectives | 3+ (many-objective) | 2+ |
| Selection | Non-dominated sort + niche preservation | Scalarization + neighborhood |
| Speed | Normalize + associate per generation | Sub-problem iteration per generation |
| Pareto front | Uniform reference coverage | Even coverage depends on weight distribution |
| Parallelism | Batch offspring per generation | Sequential sub-problem iteration |

## References

- Deb, K., & Jain, H. (2014). An evolutionary many-objective optimization algorithm using reference-point-based nondominated sorting approach, part I: solving problems with box constraints. _IEEE Trans. on Evolutionary Computation_, 18(4), 577-601.

## See Also

- [MOEA/D](moead.md) — Decomposition-based alternative for 2+ objectives
- [NSGA-II](engines.md#nsga2gau--nsga-ii) — Baseline for 2 objectives
- [Multi-Objective Concepts](multi_objective.md) — Shared MO primitives and quality indicators
- [Engines Overview](engines.md) — Full engine decision matrix
- [docs.rs/genetic_algorithms::nsga3](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/nsga3/index.html) — Module API reference
