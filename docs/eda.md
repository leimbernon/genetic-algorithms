# EDA (Estimation of Distribution)

> Univariate Marginal Distribution Algorithm (UMDA) — probabilistic model-building optimization for binary (Bernoulli) and continuous (Gaussian) chromosomes.

## Description

EDA in this library ships as two distinct engine types driven by a single `EdaConfiguration`:

- **`EdaEngine<U>`** — Bernoulli model for binary chromosomes (classic UMDA). Use this when your gene type does **not** implement `RealGene`.
- **`EdaRealEngine<U>`** — Gaussian univariate model for continuous chromosomes. Use this when your gene type implements `RealGene` (e.g., `RangeGenotype<f64>`).

The model is selected by the engine type you instantiate, **not** by a configuration flag. Both engines share the same `EdaConfiguration` struct.

**How UMDA works:** At each generation, the top `selection_ratio` fraction of the population (by fitness) feeds the model estimation step. For the Bernoulli model, the engine estimates `p_i` = fraction of selected individuals with gene `i` equal to 1 at each position, then samples Bernoulli(`p_i`) to produce a completely new population. For the Gaussian model, the engine estimates `(mean_i, std_i)` per position and samples from N(`mean_i`, `std_i`) clamped to gene bounds. There are no crossover or mutation operators — offspring come entirely from the learned model.

The final learned model is returned in `EdaResult::learned_model` as an `EdaModel` enum:

- `EdaModel::Bernoulli(Vec<f64>)` — one probability per position (approaches 0 or 1 at convergence)
- `EdaModel::Gaussian { means: Vec<f64>, stds: Vec<f64> }` — per-position mean and standard deviation

## When to Use

- **Problem type (Bernoulli):** Binary combinatorial problems with epistasis or deception — trap functions, feature selection, deceptive benchmark landscapes.
- **Problem type (Gaussian):** Continuous real-valued optimization where variable distributions can be captured by independent univariate Gaussians.
- **Variable type:** Binary genes for `EdaEngine`; `RealGene`-bounded genes for `EdaRealEngine`.
- **Key strength:** Model-building avoids crossover disruption of building-block structure. The Bernoulli model learns per-position marginals, which is sufficient for separable or mildly-deceptive problems. EDA naturally handles deceptive binary landscapes that confound crossover-based GAs.
- **Key weakness:** UMDA is a **univariate** model — it captures only marginal distributions and does **not** model inter-variable dependencies (linkage). For problems with strong epistasis requiring dependency capture, multivariate EDAs (BOA, MIMIC) would be needed. Also, EDA typically requires larger populations than standard GAs to accurately estimate the probabilistic model.

## Quick Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `population_size` | `usize` | `100` | Population size. EDA typically needs larger populations than standard GA to estimate the probability model accurately. |
| `max_generations` | `usize` | `500` | Stop after this many generations. |
| `problem_solving` | `ProblemSolving` | `Maximization` | **Note:** EDA defaults to `Maximization`, unlike CMA-ES and PSO which default to `Minimization`. Set this explicitly for minimization problems. |
| `fitness_target` | `Option<f64>` | `None` | Early-stop when best fitness reaches this target. `None` runs until `max_generations`. |
| `selection_ratio` | `f64` | `0.5` | Fraction of population used as selected parents for model estimation. Must be in `(0.0, 1.0]`; a minimum of 1 parent is enforced. Default: top 50% (truncation selection). |
| `fitness_cache_size` | `Option<usize>` | `None` | LRU fitness cache capacity in entries. Avoids redundant evaluations for duplicate DNA. |

## Complete Example

### Binary Chromosomes → Bernoulli UMDA

```rust,ignore
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine};

// Configuration — note: Maximization is the default, shown explicitly for clarity
let config = EdaConfiguration::default()
    .with_population_size(300)
    .with_max_generations(500)
    .with_selection_ratio(0.3)
    .with_problem_solving(ProblemSolving::Maximization);

// EdaEngine<BinaryChromosome> selects the Bernoulli UMDA model
let mut engine = EdaEngine::<BinaryChromosome>::new(
    config,
    |n| init_population(n),   // fn(usize) -> Vec<BinaryChromosome>
    trap_fitness,              // fn(&[BinaryGene]) -> f64
);

let result = engine.run().expect("EDA run failed");
println!("Best fitness: {:.1}", result.best_fitness);
println!("Generations: {}", result.generations);
```

### Continuous Chromosomes → Gaussian Univariate Model

```rust,ignore
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::eda::{EdaConfiguration, EdaRealEngine};

// For minimization (e.g., sphere function), override the Maximization default explicitly
let config = EdaConfiguration::default()
    .with_population_size(100)
    .with_max_generations(300)
    .with_selection_ratio(0.5)
    .with_problem_solving(ProblemSolving::Minimization);

// EdaRealEngine<RangeChromosome<f64>> selects the Gaussian univariate model
let mut engine = EdaRealEngine::<RangeChromosome<f64>>::new(
    config,
    |n| init_population(n),   // fn(usize) -> Vec<RangeChromosome<f64>>
    sphere_fitness,            // fn(&[RangeGenotype<f64>]) -> f64
);

let result = engine.run().expect("EDA run failed");
println!("Best fitness: {:.6}", result.best_fitness);
```

## Configuration Tips

- **Population size:** Use larger populations than you would for a standard GA. Model estimation quality depends on sample size — 100 to 500 is common for binary problems. The example (`eda_trap`) uses 300.
- **Selection ratio:** Values between 0.3 and 0.5 (truncation) are typical. Lower ratios (e.g., 0.3) apply tighter selection pressure; higher ratios (e.g., 0.5) preserve more diversity.
- **Problem solving direction:** `EdaConfiguration` defaults to `Maximization`. If you are solving a minimization problem, call `.with_problem_solving(ProblemSolving::Minimization)` explicitly — this is a common source of confusion since CMA-ES and PSO default to `Minimization`.
- **Fitness cache:** Enable via `.with_fitness_cache_size(n)` when your fitness function is expensive and duplicate chromosomes are common (e.g., binary problems late in convergence).
- **Early stopping:** Use `.with_fitness_target(t)` to halt when the optimum is known (e.g., `30.0` for a trap function of length 30).

## When to Choose This vs Crossover-Based GAs

| Factor | EDA (UMDA) | Crossover-Based GA |
|--------|------------|-------------------|
| Mechanism | Model-building + sampling | Crossover + mutation |
| Building-block preservation | Preserved: marginal model learns per-position frequencies | Disrupted: crossover splits blocks |
| Variable dependencies | Univariate marginals only (no linkage capture) | Implicit via linkage in crossover |
| Typical use | Deceptive binary problems; separable continuous problems | General-purpose; well-studied landscapes |
| Population requirement | Larger (model estimation quality) | Smaller (direct recombination) |
| Operators needed | None (no crossover or mutation) | Crossover + mutation required |

## References

- Mühlenbein, H., & Paaß, G. (1996). From recombination of genes to the estimation of distributions I. Binary parameters. _Parallel Problem Solving from Nature — PPSN IV_, LNCS 1141, 178–187.
- Larrañaga, P., & Lozano, J. A. (Eds.). (2002). _Estimation of Distribution Algorithms: A New Tool for Evolutionary Computation_. Kluwer Academic Publishers.

## See Also

- [Engines Overview](engines.md) — Full engine decision matrix and EDA entry
- [Example: eda_trap](../examples/eda_trap.rs) — Runnable UMDA on a deceptive trap function
- [docs.rs/genetic_algorithms::eda](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/eda/index.html) — Module API reference
