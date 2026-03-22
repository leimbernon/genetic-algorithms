# Codebase Structure

**Analysis Date:** 2026-03-20

## Directory Layout

```
src/
├── lib.rs                    # Library root: module re-exports, documentation
├── ga.rs                     # Single-population Ga orchestrator (1395+ lines)
├── error.rs                  # GaError enum (all failure types)
├── configuration.rs          # GaConfiguration and sub-configs
├── population.rs             # Population<U> container
├── stats.rs                  # GenerationStats per-generation metrics
├── rng.rs                    # Seedable random number generation
├── traits/                   # Core trait abstractions
│   ├── chromosome.rs         # ChromosomeT trait (DNA, fitness, age)
│   ├── gene.rs               # GeneT trait (minimal gene)
│   ├── operators.rs          # Operator traits (Selection, Crossover, Mutation, Survivor, Extension)
│   ├── configuration.rs      # Configuration builder traits
│   ├── common.rs             # Type aliases: InitializationFn, FitnessFn
│   └── mod.rs                # Re-exports all trait submodules
├── chromosomes/              # Concrete chromosome implementations
│   ├── binary.rs             # Binary chromosome (bool DNA)
│   ├── range.rs              # Range<T> chromosome (numeric DNA)
│   └── mod.rs                # Re-exports
├── genotypes/                # Concrete gene implementations
│   ├── binary.rs             # Binary gene (bool)
│   ├── range.rs              # Range<T> gene (numeric)
│   └── mod.rs                # Re-exports
├── operations/               # All genetic operators
│   ├── selection/            # Parent selection operators
│   │   ├── tournament.rs
│   │   ├── fitness_proportionate.rs
│   │   ├── rank.rs
│   │   ├── boltzmann.rs
│   │   ├── truncation.rs
│   │   ├── random.rs
│   │   └── mod.rs            # Factory function
│   ├── crossover/            # Recombination operators
│   │   ├── single_point.rs
│   │   ├── multipoint.rs
│   │   ├── uniform_crossover.rs
│   │   ├── cycle.rs
│   │   ├── order.rs
│   │   ├── pmx.rs
│   │   ├── sbx.rs
│   │   ├── blend_alpha.rs
│   │   ├── arithmetic.rs
│   │   ├── clone.rs
│   │   ├── rejuvenate.rs
│   │   └── mod.rs            # Factory function
│   ├── mutation/             # Perturbation operators
│   │   ├── swap.rs
│   │   ├── inversion.rs
│   │   ├── scramble.rs
│   │   ├── value.rs
│   │   ├── bit_flip.rs
│   │   ├── creep.rs
│   │   ├── gaussian.rs
│   │   ├── polynomial.rs
│   │   ├── non_uniform.rs
│   │   ├── insertion.rs
│   │   └── mod.rs            # Factory function, ValueMutable trait
│   ├── survivor/             # Survivor selection operators
│   │   ├── fitness.rs
│   │   ├── age.rs
│   │   ├── mu_plus_lambda.rs
│   │   ├── mu_comma_lambda.rs
│   │   └── mod.rs            # Factory function
│   ├── extension/            # Diversity rescue operators
│   │   ├── mass_extinction.rs
│   │   ├── mass_genesis.rs
│   │   ├── mass_degeneration.rs
│   │   ├── mass_deduplication.rs
│   │   └── mod.rs            # Factory function
│   └── mod.rs                # Enums (Selection, Crossover, Mutation, Survivor, Extension)
├── initializers/             # Population initialization
│   ├── binary_initializer.rs
│   ├── range_initializer.rs
│   ├── generic_initializer.rs
│   └── mod.rs
├── fitness/                  # Fitness function helpers
│   ├── cache.rs              # LRU fitness cache wrapper
│   ├── count_true.rs         # OneMax fitness
│   ├── fitness_fn_wrapper.rs # Fitness function Arc wrapper
│   └── mod.rs
├── island/                   # Island model (multi-population)
│   ├── mod.rs                # IslandGa orchestrator
│   ├── configuration.rs      # IslandConfiguration
│   ├── topology.rs           # MigrationTopology (Ring, FullyConnected, Star)
│   ├── migration.rs          # migrate() function
│   └── nsga2.rs              # Island-based NSGA-II
├── nsga2/                    # Multi-objective optimization (NSGA-II)
│   ├── mod.rs                # Nsga2Ga orchestrator
│   ├── configuration.rs      # Nsga2Configuration, ObjectiveDirection
│   ├── pareto.rs             # ParetoIndividual, ParetoFront
│   ├── non_dominated_sort.rs # Non-dominated sorting
│   └── crowding_distance.rs  # Crowding distance assignment
├── niching/                  # Fitness sharing for diversity
│   ├── mod.rs
│   ├── configuration.rs
│   ├── distance.rs           # Distance metrics
│   └── sharing.rs            # Fitness sharing function
├── extension/                # Extension (diversity threshold) configuration
│   ├── mod.rs
│   └── configuration.rs
├── validators/               # Configuration validation
│   ├── generic_validator.rs
│   └── validator_factory.rs  # Factory dispatch
└── checkpoint.rs             # Serialization/deserialization (serde feature)

tests/
├── test_ga.rs
├── test_operations.rs        # Module entry (imports sub-tests)
├── operations/               # Per-operator integration tests
│   ├── test_crossover*.rs
│   ├── test_mutation*.rs
│   ├── test_selection*.rs
│   └── test_survivor*.rs
├── test_chromosomes.rs
├── test_island.rs
├── test_nsga2.rs
├── test_niching.rs
├── test_fitness.rs
├── test_fitness_cache.rs
├── test_stats.rs
├── test_validators.rs
├── test_population.rs
├── test_rng.rs
├── test_checkpoint.rs
├── test_serde.rs             # Feature-gated (serde)
├── test_error.rs
├── test_extension*.rs
├── test_initializers.rs
└── structures.rs             # Shared test fixtures (Chromosome, Gene)

benches/
├── selection.rs
├── crossover.rs
├── mutation.rs
├── survivor.rs
├── ga_run.rs
├── nsga2.rs
└── island_ga.rs
```

## Key File Locations

**Entry Points:**
- `src/lib.rs` — Library root, public API surface
- `src/ga.rs` — Single-population GA (main user entry point)
- `src/island/mod.rs` — Island model orchestrator
- `src/nsga2/mod.rs` — NSGA-II multi-objective orchestrator

**Configuration:**
- `src/configuration.rs` — `GaConfiguration` (aggregates all settings)
- `src/island/configuration.rs` — Island-specific settings
- `src/nsga2/configuration.rs` — Multi-objective settings

**Core Logic:**
- `src/traits/chromosome.rs` — DNA access interface
- `src/traits/operators.rs` — Operator trait contracts
- `src/population.rs` — Population management and fitness calculation
- `src/operations/mod.rs` — `Crossover`, `Selection`, `Mutation`, `Survivor` enums

## Where to Add New Code

**New operator:** `src/operations/{type}/{name}.rs` → add enum variant in `src/operations.rs` → add dispatch arm in `src/operations/{type}/mod.rs` → add tests in `tests/operations/test_{type}_{name}.rs`

**New chromosome type:** `src/chromosomes/{name}.rs` → implement `ChromosomeT` → export from `src/chromosomes/mod.rs`

**New gene type:** `src/genotypes/{name}.rs` → implement `GeneT` → export from `src/genotypes/mod.rs`

## Naming Conventions

| Construct | Convention | Example |
|-----------|-----------|---------|
| Operator files | snake_case, one per file | `single_point.rs`, `tournament.rs` |
| Trait files | snake_case, noun-based | `chromosome.rs`, `operators.rs` |
| Factory functions | `factory()` / `factory_with_params()` | consistent across all operators |
| Generics | Single letters | `U` for chromosome, `T` for numeric type |

---

*Structure analysis: 2026-03-20*
