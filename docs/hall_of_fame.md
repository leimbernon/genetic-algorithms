# Hall of Fame

> Bounded archive of top-N unique solutions across a GA run, with optional minimum-distance diversity filtering.

## Overview

The Hall of Fame maintains an ordered archive of the best solutions discovered during a GA run. It stores chromosomes sorted by fitness descending (best first), with configurable capacity and optional diversity enforcement. The archive persists across all generations, preserving elite solutions even if the population loses them.

Each entry includes the chromosome, the generation number when it was added, and the fitness value at addition.

## Key Concepts

### HallOfFame\<U\>

The main archive struct, generic over chromosome type `U: ChromosomeT`.

| Method | Description |
|--------|-------------|
| `new(config)` | Creates a new Hall of Fame with the given configuration. |
| `try_insert(&mut self, chromosome, generation)` | Attempts to insert a chromosome. Returns `true` if admitted. |
| `entries(&self)` | Returns `&[Entry<U>]` — ordered slice of archived solutions. |
| `into_entries(self)` | Consumes the archive and returns `Vec<Entry<U>>`. |
| `len(&self)` | Current number of archived entries. |
| `capacity(&self)` | Maximum number of entries. |
| `is_empty(&self)` | Whether the archive is empty. |
| `best(&self)` | Returns the best entry (front of the list). |

### Admission Criteria

A chromosome is admitted if **all** of the following are satisfied:
1. Its fitness is not NaN.
2. The archive is not full, or its fitness is at least as good as the worst entry.
3. Its DNA is not already in the archive (genotypic uniqueness via gene ID comparison).
4. If `DistanceMetric::Genotypic` with `min_distance > 0.0`, its genotypic distance from all existing entries is >= `min_distance`.

### HallOfFameConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `capacity` | `usize` | `100` | Maximum number of archived solutions. |
| `distance_metric` | `DistanceMetric` | `Fitness { min_distance: 0.0 }` | Diversity filtering strategy. |

### DistanceMetric

| Variant | Description |
|---------|-------------|
| `Fitness { min_distance }` | Euclidean distance in fitness space. `\|f1 - f2\|` for single-objective. |
| `Genotypic { min_distance }` | Fraction of differing gene positions (by `gene.id()`). Range [0, 1]. |

### Entry\<U\>

| Field | Type | Description |
|-------|------|-------------|
| `chromosome` | `U` | The archived chromosome (clone). |
| `generation_added` | `u64` | Generation number when added. |
| `fitness_at_addition` | `f64` | Fitness value at addition time. |

## Usage Example

```rust,ignore
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Binary;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::hall_of_fame::{HallOfFame, HallOfFameConfig, DistanceMetric};

// Configure Hall of Fame with genotypic diversity enforcement
let hof_config = HallOfFameConfig {
    capacity: 50,
    distance_metric: DistanceMetric::Genotypic { min_distance: 0.1 },
};
let mut hof = HallOfFame::new(hof_config);

let mut ga = Ga::new()
    .with_genes_per_chromosome(32)
    .with_population_size(100)
    .with_initialization_fn(binary_random_initialization)
    .with_fitness_fn(|dna| dna.iter().filter(|g| g.value).count() as f64)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::BitFlip)
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Maximization)
    .with_max_generations(500)
    .build()
    .expect("Valid configuration");

ga.run().expect("GA run failed");

// Access archive after the run
let entries = hof.entries();
for entry in entries.iter().take(5) {
    println!("Gen {} fitness {:.2}",
        entry.generation_added,
        entry.fitness_at_addition);
}
```

## Configuration

```rust,ignore
use genetic_algorithms::hall_of_fame::{
    HallOfFame, HallOfFameConfig, DistanceMetric,
};

// Pure top-N by fitness (no diversity filtering)
let config = HallOfFameConfig {
    capacity: 100,
    distance_metric: DistanceMetric::Fitness { min_distance: 0.0 },
};
let mut hof = HallOfFame::new(config);

// Genotypic diversity enforcement
let config = HallOfFameConfig {
    capacity: 50,
    distance_metric: DistanceMetric::Genotypic { min_distance: 0.15 },
};
let mut hof = HallOfFame::new(config);
```

## Performance Considerations

- Insertion uses binary search for O(log n) position finding with O(n) shift — acceptable for typical archive sizes (10-100 entries).
- Genotypic uniqueness check is O(n * m) where n is archive size and m is chromosome length.
- The Hall of Fame is a separate, user-managed component — it does NOT automatically integrate with the GA loop. Call `try_insert()` at appropriate points (e.g., in an observer's `on_new_best` or `on_generation_end` hook).

## See Also

- [Constraints](constraints.md) — Constraint handling subsystem
- [Observer System](observer.md) — Integrate HOF with observer hooks
- [docs.rs/genetic_algorithms::hall_of_fame](https://docs.rs/genetic_algorithms/latest/genetic_algorithms/hall_of_fame/index.html) — Module API reference
- [hall_of_fame_demo example](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/examples/hall_of_fame_demo.rs)
