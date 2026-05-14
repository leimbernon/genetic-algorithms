# Multi-Objective Concepts

> Shared utilities for all multi-objective engines: Pareto dominance, non-dominated sorting, crowding distance, reference point generation, and quality indicators.

## Overview

The `multi_objective` module hosts the building blocks shared by all six multi-objective engines (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA). It provides non-dominated sorting, Pareto individual and front types, dominance predicates, quality indicators, and the `ObjectiveFn` type alias.

Every multi-objective engine re-exports these symbols for backward compatibility — existing user code continues to work via paths like `genetic_algorithms::nsga2::pareto::ParetoIndividual`.

## Key Types

| Type | Description |
|------|-------------|
| `ObjectiveDirection` | `Minimize` or `Maximize` per objective. |
| `ObjectiveFn<G>` | Closure signature `fn(&[G]) -> f64` for one objective. |
| `ParetoIndividual<U>` | Chromosome + objective vector + rank + crowding distance + constraint violation. |
| `ParetoFront<U>` | Ordered collection of non-dominated individuals. |

## Non-Dominated Sorting

The `non_dominated_sort` module provides:

- **`non_dominated_sort(population)`** — Fast non-dominated sorting (Deb et al. 2002). Returns fronts, each front being a `Vec<usize>` of indices into the input.
- **`non_dominated_sort_with_directions(population, directions)`** — Direction-aware variant supporting per-objective Minimize/Maximize.
- **`assign_ranks(ranks, fronts)`** — Populates a rank vector from sorted fronts.

Non-dominated sorting is O(M*N^2) in the worst case (M objectives, N individuals).

## Quality Indicators

The `indicators` module provides pure functions for evaluating Pareto front quality. All functions return `Result<f64, GaError>`.

| Function | Description | Lower Bound | Direction |
|----------|-------------|-------------|-----------|
| `hypervolume(points, reference)` | Dominated hypervolume (S-metric, Lebesgue measure). | 0 | Higher is better |
| `generational_distance(points, true_front)` | GD — average distance from each point to the nearest true Pareto point. | 0 | Lower is better |
| `inverted_generational_distance(points, true_front)` | IGD — average distance from each true Pareto point to the nearest found point. | 0 | Lower is better |
| `spread(points)` | Spread (Delta) — measures the spread of the front. | 0 | Lower is better |

### Hypervolume

The dominated hypervolume (or S-metric) measures the volume of objective space that is dominated by a set of points with respect to a reference point. It is the only unary quality indicator that is strictly monotonic with respect to Pareto dominance: if front A dominates front B, A's hypervolume is larger.

Available via `genetic_algorithms::multi_objective::indicators::hypervolume`.

### Generational Distance (GD)

GD measures the convergence of an obtained Pareto front to the true Pareto front by computing the average Euclidean distance from each obtained point to the nearest point on the true front. Lower values indicate better convergence.

### Inverted Generational Distance (IGD)

IGD is the mirror of GD: for each point on the true Pareto front, compute the distance to the nearest point on the obtained front, then average. IGD captures both convergence and diversity — an obtained front must cover the entire true front to achieve a low IGD value.

### Spread (Delta)

Spread measures the diversity of the obtained front by computing the relative gap between consecutive solutions on the front. A spread of 0 means perfectly uniform spacing.

## Usage Example

```rust,ignore
use genetic_algorithms::multi_objective::indicators::{
    hypervolume, generational_distance, inverted_generational_distance, spread,
};

// Suppose we have an obtained Pareto front
let obtained_front: Vec<Vec<f64>> = vec![
    vec![0.1, 0.9],
    vec![0.3, 0.6],
    vec![0.5, 0.4],
    vec![0.8, 0.2],
];

// Reference point that strictly dominates all solutions
let reference = vec![1.0, 1.0];

// True Pareto front (known from theory)
let true_front: Vec<Vec<f64>> = vec![
    vec![0.0, 1.0],
    vec![0.25, 0.75],
    vec![0.5, 0.5],
    vec![0.75, 0.25],
    vec![1.0, 0.0],
];

let hv = hypervolume(&obtained_front, &reference).expect("HV computation failed");
let gd = generational_distance(&obtained_front, &true_front).expect("GD computation failed");
let igd = inverted_generational_distance(&obtained_front, &true_front).expect("IGD computation failed");
let s = spread(&obtained_front).expect("Spread computation failed");

println!("Hypervolume: {:.4}", hv);
println!("GD: {:.6}", gd);
println!("IGD: {:.6}", igd);
println!("Spread: {:.4}", s);
```

## Pareto Selection and Comparison

The `pareto` module provides:

- `ParetoIndividual<U>` — a wrapper type holding a chromosome, its objective vector (Vec<f64>), rank, crowding distance, and constraint violation.
- `ParetoFront<U>` — a container for non-dominated solutions, providing iteration, length, and access to the underlying individuals.
- `dominates_with_directions(a, b, directions)` — checks if individual a dominates individual b considering per-objective directions.

## See Also

- [NSGA-II](engines.md#nsga2gau--nsga-ii) — First MO engine, uses non-dominated sort + crowding distance
- [NSGA-III](nsga3.md) — Reference-point based many-objective extension
- [MOEA/D](moead.md) — Decomposition-based approach
- [SPEA2](spea2.md) — Archive-based strength Pareto approach
- [SMS-EMOA](sms_emoa.md) — Hypervolume contribution-based selection
- [IBEA](ibea.md) — Indicator-based selection
- [Engines Overview](engines.md) — Full engine decision matrix
