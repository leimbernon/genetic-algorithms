# SMS-EMOA

> S-Metric Selection Evolutionary Multi-Objective Algorithm — a steady-state (mu+1) MOEA using hypervolume contribution for selection.

## Description

SMS-EMOA (Beume, Naujoks & Emmerich 2007) is a **steady-state** (mu+1) multi-objective evolutionary algorithm that uses **hypervolume contribution** to decide which individual is removed each generation. At each step, one offspring is created via selection + crossover + mutation, then the individual with the smallest contribution to the hypervolume of the worst non-dominated front is removed.

Per generation, SMS-EMOA:
1. Create one offspring via binary tournament + crossover + mutation.
2. Evaluate offspring objectives.
3. Merge offspring into the population (mu+1).
4. Non-dominated sort the population.
5. Identify the **worst front** (highest rank).
6. For each member of the worst front, compute **hypervolume contribution** — the reduction in hypervolume that removing that individual would cause.
7. Remove the individual with the smallest hypervolume contribution (back to mu individuals).

The hypervolume indicator simultaneously captures **convergence** (closer to the true Pareto front implies larger hypervolume) and **diversity** (well-spread solutions produce larger hypervolume), making SMS-EMOA a powerful unified approach.

## When to Use

- **Problem type:** Multi-objective (2+ objectives, best at 2-3)
- **Number of objectives:** 2-3 (best), up to 4+
- **Variable type:** Continuous (real-valued), binary
- **Key strength:** Hypervolume-based selection naturally balances convergence and diversity without requiring additional diversity preservation mechanisms.
- **Key weakness:** Hypervolume computation is expensive (O(N*2^M) for M objectives). Practical for 2-3 objectives; becomes prohibitive at 4+ objectives. Requires a reference point that strictly dominates all solutions.

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
| `hypervolume_reference_point` | `Option<Vec<f64>>` | `None` (auto-computed) | Reference point for HV calculation. |
| `ga_config` | `GaConfiguration` | `Default` | GA operators, limits, RNG seed. |
| `observer` | `SmsEmoaObserver<U>` | `None` | Lifecycle observer. |

## Complete Example

```rust,ignore
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation};
use genetic_algorithms::sms_emoa::SmsEmoaGa;
use genetic_algorithms::sms_emoa::configuration::SmsEmoaConfiguration;

type MyChromosome = RangeChromosome<f64>;

let sms_config = SmsEmoaConfiguration::new()
    .with_num_objectives(2)
    .with_population_size(100)
    .with_max_generations(250);

let ga_config = GaConfiguration::default()
    .with_crossover_method(Crossover::Sbx)
    .with_mutation_method(Mutation::Polynomial);

let alleles = vec![RangeGenotype::new(0, vec![(0.0, 1.0)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut sms_emoa = SmsEmoaGa::<MyChromosome>::new(sms_config, ga_config)
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
    .expect("Valid SMS-EMOA configuration");

let pareto_front = sms_emoa.run().expect("SMS-EMOA run failed");
println!("Front size: {}", pareto_front.len());
```

## Configuration Tips

- If `hypervolume_reference_point` is not set, it is auto-computed from the initial population as `max(objective) + 1.0` per dimension. For better control, set it explicitly to a point that strictly dominates all expected solutions.
- SMS-EMOA produces one offspring per generation, so `max_generations` is the number of fitness evaluations. A typical setting is 10-20x the population_size.
- Hypervolume computation allocates O(2^M) space; for M > 4 the `indicators` module is more efficient than the naive per-individual contribution calculation.
- The reference point should be set so that all Pareto-optimal solutions are strictly dominated by it (each coordinate greater than the maximum expected value in that objective).

## When to Choose This vs IBEA

| Factor | SMS-EMOA | IBEA |
|--------|----------|------|
| Selection | Hypervolume contribution | I_eps+ indicator fitness |
| Indicator | Hypervolume (S-metric) | Additive epsilon (dominance shift) |
| Reference point | Required (auto or explicit) | Not needed |
| Objectives | 2-3 (HV cost) | 2+ (pairwise O(N^2)) |
| Environmental selection | One removal per generation (steady-state) | Iterative removal + recalculation |
| Diversity | Implicit via hypervolume | Implicit via indicator |

## References

- Beume, N., Naujoks, B., & Emmerich, M. (2007). SMS-EMOA: Multiobjective selection based on dominated hypervolume. _European Journal of Operational Research_, 181(3), 1653-1669.

## See Also

- [IBEA](ibea.md) — Indicator-based alternative without reference point
- [NSGA-II](engines.md#nsga2gau--nsga-ii) — Fast Pareto ranking alternative
- [Multi-Objective Concepts](multi_objective.md) — Shared MO primitives and quality indicators
- [Engines Overview](engines.md) — Full engine decision matrix
- [docs.rs/genetic_algorithms::sms_emoa](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/sms_emoa/index.html) — Module API reference
