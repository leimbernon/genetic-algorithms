# IBEA

> Indicator-Based Evolutionary Algorithm — uses the I_epsilon+ binary indicator with exponential fitness scaling.

## Description

IBEA (Zitzler & Kunzli 2004) is a multi-objective evolutionary algorithm that uses a **pairwise indicator function** (the additive epsilon indicator I_eps+) to assign fitness values, eliminating the need for Pareto dominance ranking or diversity metrics like crowding distance.

**Indicator fitness:**
- `I_eps+(a, b)` = minimum additive shift needed for `b` to dominate `a`. A positive value means `a` is better; 0.0 means `a` is dominated by or equal to `b`.
- Fitness `F(x) = sum_{y != x} -exp(-I_eps+(y, x) / c)` where `c` is the maximum absolute indicator value (adaptive scaling). **Higher fitness = better** — a positive I_eps+ (x is worse than y) reduces x's fitness.

Per generation, IBEA:
1. Compute the pairwise I_eps+ indicator matrix for the population.
2. Compute indicator-based fitness for each individual.
3. **Environmental selection:** iteratively remove the individual with the lowest indicator fitness, recalculating fitnesses after each removal, until the population reaches the target size.
4. Create offspring via binary tournament + crossover + mutation.
5. Merge offspring into the population.
6. Trim population back to pop_size to prevent unbounded growth.

IBEA does **not** require a reference point (unlike SMS-EMOA) and works with any binary quality indicator that can be paired with the exponential scaling scheme.

## When to Use

- **Problem type:** Multi-objective (2+ objectives)
- **Number of objectives:** 2+
- **Variable type:** Continuous, binary, permutation, or `List<T>`
- **Key strength:** No reference point needed. Flexible indicator framework — the I_eps+ indicator captures dominance relationships without expensive hypervolume computation.
- **Key weakness:** O(N^2) pairwise indicator computation per generation. Environmental selection iteratively removes one at a time and recalculates, making it O(K*N^2) where K is the number of removals per generation.

## Quick Reference

### Mandatory Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `num_objectives` | `usize` | `2` | Number of objectives (>= 2). |
| `population_size` | `usize` | `100` | Population size (>= 2). |
| `max_generations` | `usize` | `250` | Maximum generations. |
| `init_fn` | `Fn` | — | Chromosome initialization function. |
| `objective_fns` | `Vec<ObjectiveFn>` | — | One closure per objective. |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `objective_directions` | `Vec<ObjectiveDirection>` | All `Minimize` | Per-objective Min/Max. |
| `kappa` | `f64` | `0.05` | Exponential scaling factor for indicator fitness. |
| `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
| `observer` | `IbeaObserver<U>` | `None` | Lifecycle observer. |

## Complete Example

```rust,ignore
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::ibea::IbeaGa;
use genetic_algorithms::ibea::configuration::IbeaConfiguration;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation};

type MyChromosome = RangeChromosome<f64>;

let ibea_config = IbeaConfiguration::new()
    .with_num_objectives(2)
    .with_population_size(100)
    .with_max_generations(250);

let ga_config = GaConfiguration::default()
    .with_crossover_method(Crossover::Sbx)
    .with_mutation_method(Mutation::Polynomial);

let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut ibea = IbeaGa::<MyChromosome>::new(ibea_config, ga_config)
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
    .expect("Valid IBEA configuration");

let pareto_front = ibea.run().expect("IBEA run failed");
println!("Front size: {}", pareto_front.len());
```

## Configuration Tips

- The `kappa` scaling parameter controls selection pressure: smaller kappa means higher pressure (faster convergence); larger kappa means more relaxed (better diversity). Default is 0.05.
- IBEA's O(N^2) fitness computation is the main bottleneck. For large populations (> 200), consider using SMS-EMOA or NSGA-II instead.
- The iterative environmental selection removes one individual at a time and recalculates all fitnesses — this is the dominant cost. The algorithm removes enough to accommodate the incoming offspring.
- Unlike SMS-EMOA, IBEA does not require a reference point, making it easier to configure for arbitrary problems.

## When to Choose This vs SMS-EMOA

| Factor | IBEA | SMS-EMOA |
|--------|------|----------|
| Indicator | I_eps+ (additive epsilon) | Hypervolume (S-metric) |
| Reference point | Not needed | Required |
| Objectives | 2+ (pairwise O(N^2)) | 2-3 (HV cost O(N*2^M)) |
| Cost per generation | O(K*N^2) iterative removal | O(N*2^M) HV contribution |
| Selection mode | Batch offspring | Steady-state (mu+1, one offspring) |
| Diversity mechanism | Implicit via indicator fitness | Implicit via hypervolume |

## References

- Zitzler, E., & Kunzli, S. (2004). Indicator-based selection in multiobjective search. _Parallel Problem Solving from Nature - PPSN VIII_, LNCS 3242, 832-842.

## See Also

- [SMS-EMOA](sms_emoa.md) — Hypervolume-based alternative with reference point
- [NSGA-II](engines.md#nsga2gau--nsga-ii) — Fast Pareto ranking alternative
- [Multi-Objective Concepts](multi_objective.md) — Shared MO primitives and quality indicators
- [Engines Overview](engines.md) — Full engine decision matrix
- [docs.rs/genetic_algorithms::ibea](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/ibea/index.html) — Module API reference
