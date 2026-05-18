# SPEA2

> Strength Pareto Evolutionary Algorithm 2 — an archive-based multi-objective EA with k-nearest neighbour density estimation.

## Description

SPEA2 (Zitzler, Laumanns & Thiele 2001) is a multi-objective evolutionary algorithm that maintains a **fixed-size external archive** of non-dominated solutions alongside the main population.

Fitness assignment combines two components:
- **Raw strength** `R(i)` — sum of strength values of all individuals that dominate `i`, where `strength(j) = count(j dominates all others)`.
- **Density** `D(i)` — estimated via k-nearest-neighbour distance in objective space: `D(i) = 1 / (sigma_k + 2)`, where `k = floor(sqrt(N))` and `sigma_k` is the distance to the k-th nearest neighbour.

**Final fitness** `F(i) = R(i) + D(i)` — lower is better.

Per generation, SPEA2:
1. Compute fitness on the combined population + archive set.
2. **Environmental selection:** copy all non-dominated solutions (fitness < 1.0) to the new archive. If under capacity, fill with best-dominated by fitness. If over capacity, truncate via iterative nearest-neighbour Euclidean removal.
3. **Binary tournament** from the archive — create new offspring population via crossover + mutation.
4. Replace the population with the offspring. The archive persists across generations.

## When to Use

- **Problem type:** Multi-objective (2+ objectives)
- **Number of objectives:** 2+
- **Variable type:** Continuous, binary, permutation, or `List<T>`
- **Key strength:** The external archive preserves non-dominated solutions even if the population loses them. The k-NN density provides fine-grained diversity preservation.
- **Key weakness:** The fitness computation is O(N^2) per generation (pairwise domination checks). Archive truncation via iterative nearest-neighbour removal is also O(N^2). Slower than NSGA-II for large populations.

## Quick Reference

### Mandatory Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `num_objectives` | `usize` | `2` | Number of objectives. |
| `population_size` | `usize` | `100` | Main population size (>= 2). |
| `archive_size` | `usize` | `100` | External archive size (<= pop_size). |
| `max_generations` | `usize` | `250` | Maximum generations. |
| `init_fn` | `Fn` | — | Chromosome initialization function. |
| `objective_fns` | `Vec<ObjectiveFn>` | — | One closure per objective. |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `objective_directions` | `Vec<ObjectiveDirection>` | All `Minimize` | Per-objective Min/Max. |
| `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
| `observer` | `Spea2Observer<U>` | `None` | Lifecycle observer. |

## Complete Example

```rust,ignore
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation};
use genetic_algorithms::spea2::Spea2Ga;
use genetic_algorithms::spea2::configuration::Spea2Configuration;

type MyChromosome = RangeChromosome<f64>;

let spea2_config = Spea2Configuration::new()
    .with_num_objectives(2)
    .with_population_size(100)
    .with_archive_size(100)
    .with_max_generations(250);

let ga_config = GaConfiguration::default()
    .with_crossover_method(Crossover::Sbx)
    .with_mutation_method(Mutation::Polynomial);

let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut spea2 = Spea2Ga::<MyChromosome>::new(spea2_config, ga_config)
    .with_initialization_fn(move |n, _, _| {
        range_random_initialization(n, Some(&alleles_clone), Some(false))
    })
    .with_objective_fns(vec![
        // ZDT1 f1
        Box::new(|dna: &[RangeGenotype<f64>]| -> f64 {
            dna[0].value
        }),
        // ZDT1 f2
        Box::new(|dna: &[RangeGenotype<f64>]| -> f64 {
            let n = dna.len();
            let g = 1.0 + 9.0 * dna[1..].iter().map(|g| g.value).sum::<f64>() / (n - 1) as f64;
            g * (1.0 - (dna[0].value / g).sqrt())
        }),
    ])
    .build()
    .expect("Valid SPEA2 configuration");

let pareto_front = spea2.run().expect("SPEA2 run failed");
println!("Front size: {}", pareto_front.len());
```

## Configuration Tips

- `archive_size` must be <= `population_size`. The canonical SPEA2 uses `archive_size = population_size`. Smaller archives converge faster but may lose diversity.
- SPEA2 works well with moderate population sizes (50-200). The O(N^2) fitness computation becomes noticeable beyond typical sizes.
- The archive truncation uses lexicographic nearest-neighbour removal, which preserves boundary points (they have the largest distances).
- For many-objective problems (3+ objectives), consider NSGA-III or MOEA/D instead, as SPEA2's density estimation degrades with higher-dimensional objective spaces.

## When to Choose This vs NSGA-II

| Factor | SPEA2 | NSGA-II |
|--------|-------|---------|
| Archive | External fixed-size archive | None (elitism from sorting) |
| Diversity | k-NN density estimation | Crowding distance |
| Fitness cost | O(N^2) domination + O(N^2) NN | O(M*N^2) sort + O(N log N) crowding |
| Archive quality | Preserves non-dominated across generations | Best front from combined set |
| Parameter | `archive_size` added | Simpler configuration |
| Front quality | Better for irregular fronts | Better for regular fronts |

## References

- Zitzler, E., Laumanns, M., & Thiele, L. (2001). SPEA2: Improving the strength Pareto evolutionary algorithm. _TIK-Report 103_, ETH Zurich.

## See Also

- [NSGA-II](engines.md#nsga2gau--nsga-ii) — Fast Pareto ranking alternative
- [Multi-Objective Concepts](multi_objective.md) — Shared MO primitives and quality indicators
- [Engines Overview](engines.md) — Full engine decision matrix
- [docs.rs/genetic_algorithms::spea2](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/spea2/index.html) — Module API reference
