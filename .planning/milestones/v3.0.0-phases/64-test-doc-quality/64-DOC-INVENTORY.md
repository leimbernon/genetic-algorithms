# Phase 64-04: Rustdoc Examples Inventory

**Generated:** 2026-06-11
**Scope:** Module-root `pub fn`, `pub struct`, `pub trait`, `pub enum` in `src/`
**Exclusions:** trait impl items, enum variants, type aliases, `pub(crate)` items, items in `benchmarks/` (feature-gated, secondary scope)

## Classification Legend

- **complex** — Requires engine setup; use ` ```rust,no_run `
- **simple** — Self-contained; use runnable ` ```rust ` with at least one `assert!`

---

## src/error.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/error.rs:29 | `GaError` | simple | done |

## src/population.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/population.rs:38 | `Population<U>` | complex | done |

## src/rng.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/rng.rs:47 | `set_seed` | simple | done |
| src/rng.rs:68 | `make_rng` | simple | done |

## src/stats.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/stats.rs:26 | `GenerationStats` | simple | done |

## src/types/genotypes/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/types/genotypes/binary.rs:38 | `Binary` | simple | already_has |
| src/types/genotypes/range.rs:44 | `Range<T>` | simple | already_has |
| src/types/genotypes/list.rs:39 | `List<T>` | simple | already_has |
| src/types/genotypes/multi_range.rs:43 | `MultiRangeGenotype<T>` | simple | already_has |
| src/types/genotypes/unique.rs:45 | `UniqueGenotype<T>` | simple | already_has |

## src/types/chromosomes/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/types/chromosomes/binary.rs:24 | `chromosomes::Binary` | simple | done |
| src/types/chromosomes/range.rs:43 | `chromosomes::Range<T>` | simple | already_has |
| src/types/chromosomes/list.rs:44 | `chromosomes::ListChromosome<T>` | simple | already_has |
| src/types/chromosomes/length.rs:28 | `ChromosomeLength` | simple | already_has |
| src/types/chromosomes/multi_range.rs:54 | `MultiRangeChromosome<T>` | simple | already_has |
| src/types/chromosomes/multi_unique.rs:82 | `MultiUniqueChromosome<T>` | simple | already_has |
| src/types/chromosomes/unique.rs:60 | `UniqueChromosome<T>` | simple | already_has |

## src/traits/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/traits/chromosome.rs:27 | `ChromosomeT` | complex | done |
| src/traits/gene.rs:17 | `GeneT` | complex | done |
| src/traits/configuration.rs:14 | `SelectionConfig` | complex | done |
| src/traits/configuration.rs:29 | `CrossoverConfig` | complex | done |
| src/traits/configuration.rs:55 | `MutationConfig` | complex | done |
| src/traits/configuration.rs:79 | `SurvivorConfig` | complex | done |
| src/traits/configuration.rs:92 | `StoppingConfig` | complex | done |
| src/traits/configuration.rs:110 | `NichingConfig` | complex | done |
| src/traits/configuration.rs:120 | `ElitismConfig` | complex | done |
| src/traits/configuration.rs:126 | `ExtensionConfig` | complex | done |
| src/traits/configuration.rs:141 | `LocalSearchConfig` | complex | done |
| src/traits/configuration.rs:155 | `ConfigurationT` | complex | done |
| src/traits/operators.rs:37 | `SelectionOperator` | complex | done |
| src/traits/operators.rs:79 | `CrossoverOperator` | complex | done |
| src/traits/operators.rs:119 | `MutationOperator` | complex | done |
| src/traits/operators.rs:164 | `SurvivorOperator` | complex | done |
| src/traits/operators.rs:202 | `ExtensionOperator` | complex | done |
| src/traits/operators.rs:244 | `LocalSearchOperator` | complex | done |
| src/traits/linear_chromosome.rs:27 | `LinearChromosome` | complex | done |
| src/traits/real_gene.rs:23 | `RealGene` | complex | done |
| src/traits/real_valued.rs:65 | `RealValued` | complex | done |
| src/traits/self_adaptive.rs:49 | `SelfAdaptive` | complex | done |
| src/traits/strategy.rs:8 | `Strategy<U>` | complex | done |
| src/traits/vector_fitness.rs:36 | `VectorFitness` | complex | done |
| src/traits/group_aware.rs:47 | `GroupAware` | complex | done |
| src/traits/multi_case_fitness.rs:9 | `MultiCaseFitness` | complex | done |
| src/traits/operator_compat.rs:53 | `OperatorCompat` | complex | done |
| src/traits/common.rs:78 | `initialize_chromosomes` | complex | done |
| src/traits/common.rs:107 | `initialize_chromosomes_par` | complex | done |

## src/operations.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations.rs:45 | `Selection` | complex | done |
| src/operations.rs:95 | `AlignmentStrategy` | simple | done |
| src/operations.rs:109 | `Crossover` | complex | done |
| src/operations.rs:208 | `Mutation` | complex | done |
| src/operations.rs:352 | `Survivor` | complex | done |
| src/operations.rs:374 | `Extension` | complex | done |

## src/operations/crossover.rs (factory fns)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/crossover.rs:538 | `factory` | complex | done |
| src/operations/crossover.rs:550 | `aga_probability` | complex | done |
| src/operations/crossover.rs:447 | `factory_multi_parent` | complex | done |
| src/operations/crossover.rs:496 | `factory_multi_parent_dispatch` | complex | done |

## src/operations/mutation.rs (factory fns)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/mutation.rs:178 | `ValueMutable` | complex | done |
| src/operations/mutation.rs:357 | `factory` | complex | done |
| src/operations/mutation.rs:377 | `factory_with_params` | complex | done |
| src/operations/mutation.rs:411 | `factory_with_chromosome_length` | complex | done |
| src/operations/mutation.rs:441 | `factory_self_adaptive` | complex | done |
| src/operations/mutation.rs:466 | `factory_non_value` | complex | done |
| src/operations/mutation.rs:570 | `aga_probability` | complex | done |
| src/operations/mutation.rs:593 | `compute_cardinality` | complex | done |
| src/operations/mutation.rs:609 | `dynamic_probability` | simple | done |

## src/operations/selection.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/selection.rs:92 | `factory` | complex | done |
| src/operations/selection.rs:164 | `factory_lexicase` | complex | done |

## src/operations/survivor.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/survivor.rs:53 | `factory` | complex | done |

## src/operations/local_search.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/local_search.rs:20 | `LocalSearch` | complex | done |
| src/operations/local_search.rs:53 | `HillClimbingConfig` | complex | done |
| src/operations/local_search.rs:75 | `LocalSearchApplicationStrategy` | complex | done |
| src/operations/local_search.rs:104 | `LocalSearchMode` | complex | done |
| src/operations/local_search.rs:119 | `factory` | complex | done |
| src/operations/local_search.rs:127 | `factory_with_config` | complex | done |

## src/configuration.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/configuration.rs:54 | `ProblemSolving` | complex | done |
| src/configuration.rs:77 | `LogLevel` | simple | done |
| src/configuration.rs:98 | `SelectionConfiguration` | complex | done |
| src/configuration.rs:133 | `CrossoverConfiguration` | complex | done |
| src/configuration.rs:195 | `MutationConfiguration` | complex | done |
| src/configuration.rs:229 | `LimitConfiguration` | complex | done |
| src/configuration.rs:255 | `SaveProgressConfiguration` | complex | done |
| src/configuration.rs:266 | `LocalSearchConfiguration` | complex | done |
| src/configuration.rs:295 | `GaConfiguration` | complex | done |

## src/initializers/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/initializers/binary_initializer.rs:29 | `binary_random_initialization` | simple | already_has |
| src/initializers/range_initializer.rs:36 | `range_random_initialization` | simple | already_has |
| src/initializers/list_initializer.rs:42 | `list_random_initialization` | simple | already_has |
| src/initializers/list_initializer.rs:98 | `list_random_initialization_without_repetitions` | simple | already_has |
| src/initializers/generic_initializer.rs:19 | `generic_random_initialization` | complex | done |
| src/initializers/generic_initializer.rs:51 | `generic_random_initialization_without_repetitions` | complex | done |
| src/initializers/multi_range_initializer.rs:56 | `multi_range_random_initialization` | simple | already_has |
| src/initializers/unique_initializer.rs:54 | `unique_random_initialization` | simple | already_has |

## src/niching/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/niching/configuration.rs:21 | `NichingConfiguration` | complex | already_has |
| src/niching/distance.rs:29 | `hamming_distance` | simple | already_has |
| src/niching/distance.rs:76 | `euclidean_distance` | simple | already_has |
| src/niching/distance.rs:112 | `DistanceMetric` (niching) | complex | done |
| src/niching/distance.rs:118 | `HammingDistance` | simple | done |
| src/niching/distance.rs:127 | `EuclideanDistance` | simple | done |
| src/niching/sharing.rs:30 | `sharing_function` | simple | already_has |
| src/niching/sharing.rs:72 | `apply_fitness_sharing` | complex | already_has |
| src/niching/sharing.rs:120 | `compute_distance_matrix` | complex | done |
| src/niching/sharing.rs:151 | `apply_fitness_sharing_with_dna` | complex | done |

## src/extension/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/extension/configuration.rs:24 | `ExtensionConfiguration` | complex | already_has |

## src/fitness/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/fitness/batch.rs:50 | `BatchFitnessEvaluator<U>` | complex | done |
| src/fitness/cache.rs:21 | `FitnessCache` | complex | done |
| src/fitness/cache.rs:101 | `hash_dna` | simple | done |
| src/fitness/cache.rs:130 | `wrap_with_cache` | complex | done |
| src/fitness/count_true.rs:23 | `count_true` | simple | done |
| src/fitness/fitness_fn_wrapper.rs:17 | `FitnessFnWrapper<G>` | complex | done |
| src/fitness/surrogate.rs:76 | `SurrogateModel<U>` | complex | done |

## src/constraints.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/constraints.rs:32 | `PenaltyStrategy` | complex | done |
| src/constraints.rs:64 | `ConstraintHandling` | complex | done |
| src/constraints.rs:76 | `total_violation` | simple | done |
| src/constraints.rs:83 | `apply_static_penalty` | simple | done |
| src/constraints.rs:90 | `apply_dynamic_penalty` | simple | done |
| src/constraints.rs:107 | `validate_penalty_strategy` | simple | done |

## src/aos.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/aos.rs:25 | `AosStrategy` | complex | done |
| src/aos.rs:115 | `AosState` | complex | done |
| src/aos.rs:388 | `compute_normalized_reward` | simple | done |

## src/hall_of_fame.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/hall_of_fame.rs:33 | `DistanceMetric` | simple | done |
| src/hall_of_fame.rs:66 | `HallOfFameConfig` | complex | done |
| src/hall_of_fame.rs:90 | `Entry<U>` | simple | done |
| src/hall_of_fame.rs:110 | `HallOfFame<U>` | complex | done |
| src/hall_of_fame.rs:296 | `genotypic_distance` | simple | done |

## src/observe/observer/mod.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/observe/observer/mod.rs:39 | `ExtensionEvent` | simple | done |
| src/observe/observer/mod.rs:66 | `GaObserver<U>` | complex | done |
| src/observe/observer/mod.rs:132 | `NoopObserver` | simple | done |
| src/observe/observer/mod.rs:140 | `IslandGaObserver<U>` | complex | done |
| src/observe/observer/mod.rs:161 | `Nsga2Observer<U>` | complex | done |
| src/observe/observer/mod.rs:187 | `Nsga3Observer<U>` | complex | done |
| src/observe/observer/mod.rs:211 | `MoeaDObserver<U>` | complex | done |
| src/observe/observer/mod.rs:235 | `Spea2Observer<U>` | complex | done |
| src/observe/observer/mod.rs:268 | `SmsEmoaObserver<U>` | complex | done |
| src/observe/observer/mod.rs:292 | `IbeaObserver<U>` | complex | done |
| src/observe/observer/mod.rs:319 | `AllObserver<U>` | complex | done |
| src/observe/observer/composite.rs:45 | `CompositeObserver<U>` | complex | done |
| src/observe/observer/log.rs:46 | `LogObserver` | simple | done |

## src/engines/ga.rs

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/ga.rs:231 | `TerminationCause` | simple | done |
| src/engines/ga.rs:257 | `Ga<U>` | complex | already_has |

## src/engines/island/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/island/configuration.rs:6 | `MigrationPolicy` | complex | done |
| src/engines/island/configuration.rs:42 | `IslandConfiguration` | complex | already_has |
| src/engines/island/topology.rs:7 | `MigrationTopology` | complex | done |
| src/engines/island/topology.rs:41 | `neighbors` | complex | done |
| src/engines/island/mod.rs:146 | `IslandGa<U>` | complex | done |
| src/engines/island/migration.rs:30 | `migrate` | complex | done |
| src/engines/island/migration.rs:255 | `migrate_pareto` | complex | done |
| src/engines/island/nsga2.rs:69 | `IslandNsga2Ga<U>` | complex | done |
| src/engines/island/nsga2.rs:459 | `binary_tournament` | complex | done |

## src/engines/nsga2/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/nsga2/configuration.rs:30 | `Nsga2Configuration` | complex | already_has |
| src/engines/nsga2/mod.rs:136 | `Nsga2Ga<U>` | complex | done |
| src/engines/nsga2/crowding_distance.rs:12 | `assign_crowding_distance` | complex | done |
| src/engines/nsga2/non_dominated_sort.rs:16 | `non_dominated_sort` | complex | done |
| src/engines/nsga2/non_dominated_sort.rs:24 | `non_dominated_sort_with_directions` | complex | done |
| src/engines/nsga2/non_dominated_sort.rs:39 | `non_dominated_sort_constrained` | complex | done |
| src/engines/nsga2/non_dominated_sort.rs:139 | `assign_ranks` | complex | done |
| src/engines/nsga2/pareto.rs:9 | `ParetoIndividual<U>` | complex | done |
| src/engines/nsga2/pareto.rs:48 | `ParetoFront<U>` | complex | done |
| src/engines/nsga2/pareto.rs:79 | `dominates` | simple | done |
| src/engines/nsga2/pareto.rs:99 | `dominates_with_directions` | simple | done |
| src/engines/nsga2/pareto.rs:126 | `constrained_dominates` | simple | done |

## src/engines/nsga3/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/nsga3/configuration.rs:31 | `Nsga3Configuration` | complex | already_has |
| src/engines/nsga3/mod.rs:150 | `Nsga3Ga<U>` | complex | done |
| src/engines/nsga3/das_dennis.rs:21 | `generate_das_dennis` | simple | done |

## src/engines/moead/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/moead/configuration.rs:15 | `ScalarizationFn` | complex | done |
| src/engines/moead/configuration.rs:54 | `MoeaDConfiguration` | complex | already_has |
| src/engines/moead/mod.rs:147 | `MoeaDGa<U>` | complex | done |

## src/engines/spea2/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/spea2/configuration.rs:29 | `Spea2Configuration` | complex | already_has |
| src/engines/spea2/mod.rs:134 | `Spea2Ga<U>` | complex | done |

## src/engines/sms_emoa/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/sms_emoa/configuration.rs:27 | `SmsEmoaConfiguration` | complex | already_has |
| src/engines/sms_emoa/mod.rs:134 | `SmsEmoaGa<U>` | complex | done |

## src/engines/ibea/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/ibea/configuration.rs:27 | `IbeaConfiguration` | complex | already_has |
| src/engines/ibea/mod.rs:138 | `IbeaGa<U>` | complex | done |

## src/engines/alps/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/alps/configuration.rs:12 | `AlpsAgeScheme` | simple | done |
| src/engines/alps/configuration.rs:31 | `AlpsConfiguration` | complex | done |
| src/engines/alps/configuration.rs:146 | `fibonacci` | simple | done |
| src/engines/alps/engine.rs:35 | `AlpsResult<U>` | complex | done |
| src/engines/alps/engine.rs:77 | `AlpsEngine<U>` | complex | done |

## src/engines/cellular/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/cellular/configuration.rs:11 | `Neighborhood` | simple | done |
| src/engines/cellular/configuration.rs:35 | `UpdateMode` | simple | done |
| src/engines/cellular/configuration.rs:46 | `CellularConfiguration` | complex | done |
| src/engines/cellular/engine.rs:32 | `CellularResult<U>` | complex | done |
| src/engines/cellular/engine.rs:73 | `CellularEngine<U>` | complex | done |

## src/engines/de/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/de/configuration.rs:11 | `DeMutationStrategy` | complex | done |
| src/engines/de/configuration.rs:36 | `DeCrossoverMode` | complex | done |
| src/engines/de/configuration.rs:47 | `DeAdaptive` | complex | done |
| src/engines/de/configuration.rs:65 | `DeConfiguration` | complex | done |
| src/engines/de/engine.rs:16 | `DeResult<U>` | complex | done |
| src/engines/de/engine.rs:50 | `DeEngine<U>` | complex | done |
| src/engines/de/mutation.rs:51 | `DeMutationParams<'a>` | complex | done |
| src/engines/de/mutation.rs:202 | `JadeState` | complex | done |
| src/engines/de/mutation.rs:267 | `LShadeState` | complex | done |

## src/engines/gp/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/gp/chromosome.rs:30 | `GpGene` | simple | done |
| src/engines/gp/chromosome.rs:49 | `TreeChromosome` | complex | done |
| src/engines/gp/chromosome.rs:97 | `GpChromosome<N>` | complex | done |
| src/engines/gp/configuration.rs:36 | `GpConfiguration` | complex | done |
| src/engines/gp/crossover.rs:32 | `GpCrossover` | complex | done |
| src/engines/gp/engine.rs:61 | `GpResult<N>` | complex | done |
| src/engines/gp/engine.rs:96 | `GpGa<N>` | complex | done |
| src/engines/gp/init.rs:66 | `ramped_half_and_half` | complex | done |
| src/engines/gp/mutation.rs:34 | `GpMutation` | complex | done |
| src/engines/gp/node.rs:48 | `GpNode` | complex | done |
| src/engines/gp/node.rs:123 | `Node<N>` | complex | done |
| src/engines/gp/primitives.rs:31 | `MathNode` | complex | done |
| src/engines/gp/primitives.rs:140 | `BoolNode` | complex | done |

## src/engines/cma/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/cma/configuration.rs:8 | `CmaConfiguration` | complex | done |
| src/engines/cma/engine.rs:300 | `CmaResult<U>` | complex | done |
| src/engines/cma/engine.rs:341 | `CmaEngine<U>` | complex | done |
| src/engines/cma/restart.rs:33 | `RestartStrategy` | complex | done |
| src/engines/cma/restart.rs:99 | `RestartKind` | simple | done |
| src/engines/cma/restart.rs:120 | `RestartEvent` | simple | done |

## src/engines/scatter/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/scatter/configuration.rs:7 | `ScatterConfiguration` | complex | done |
| src/engines/scatter/engine.rs:19 | `ScatterResult<U>` | complex | done |
| src/engines/scatter/engine.rs:48 | `ScatterEngine<U>` | complex | done |

## src/engines/pso/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/pso/configuration.rs:10 | `PsoInertia` | simple | done |
| src/engines/pso/configuration.rs:34 | `PsoTopology` | simple | done |
| src/engines/pso/configuration.rs:62 | `PsoConfiguration` | complex | done |
| src/engines/pso/engine.rs:31 | `PsoResult<U>` | complex | done |
| src/engines/pso/engine.rs:155 | `PsoEngine<U>` | complex | done |

## src/engines/eda/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/eda/engine.rs:47 | `EdaModel` | simple | done |
| src/engines/eda/configuration.rs:17 | `EdaConfiguration` | complex | done |
| src/engines/eda/engine.rs:69 | `EdaResult<U>` | complex | done |
| src/engines/eda/engine.rs:107 | `EdaEngine<U>` | complex | done |
| src/engines/eda/engine.rs:420 | `EdaRealEngine<U>` | complex | done |

## src/engines/hill_climb/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/hill_climb/configuration.rs:7 | `HillClimbMode` | simple | done |
| src/engines/hill_climb/configuration.rs:16 | `HillClimbConfiguration` | complex | done |
| src/engines/hill_climb/engine.rs:26 | `HillClimbEngine<U>` | complex | done |

## src/engines/permutate/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/permutate/configuration.rs:8 | `PermutateConfiguration` | complex | done |
| src/engines/permutate/engine.rs:21 | `PermutateEngine<U>` | complex | done |

## src/engines/multi_objective/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/engines/multi_objective/mod.rs:58 | `ObjectiveDirection` | simple | done |
| src/engines/multi_objective/pareto.rs:9 | `ParetoIndividual<U>` | complex | done |
| src/engines/multi_objective/pareto.rs:48 | `ParetoFront<U>` | complex | done |
| src/engines/multi_objective/pareto.rs:79 | `dominates` | simple | done |
| src/engines/multi_objective/pareto.rs:99 | `dominates_with_directions` | simple | done |
| src/engines/multi_objective/pareto.rs:126 | `constrained_dominates` | simple | done |
| src/engines/multi_objective/non_dominated_sort.rs:16 | `non_dominated_sort` | complex | done |
| src/engines/multi_objective/non_dominated_sort.rs:24 | `non_dominated_sort_with_directions` | complex | done |
| src/engines/multi_objective/non_dominated_sort.rs:39 | `non_dominated_sort_constrained` | complex | done |
| src/engines/multi_objective/non_dominated_sort.rs:139 | `assign_ranks` | complex | done |
| src/engines/multi_objective/indicators/hypervolume.rs:34 | `hypervolume` | simple | done |
| src/engines/multi_objective/indicators/generational_distance.rs:29 | `generational_distance` | simple | done |
| src/engines/multi_objective/indicators/inverted_generational_distance.rs:31 | `inverted_generational_distance` | simple | done |
| src/engines/multi_objective/indicators/spread.rs:39 | `spread` | simple | done |

## src/validators/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/validators/generic_validator.rs:32 | `validate` | complex | done |
| src/validators/generic_validator.rs:83 | `unique_gene_ids` | complex | done |
| src/validators/generic_validator.rs:102 | `fitness_target_is_some` | complex | done |
| src/validators/generic_validator.rs:118 | `same_dna_length` | complex | done |
| src/validators/generic_validator.rs:139 | `chromosome_length_not_bigger_than_alleles` | complex | done |
| src/validators/generic_validator.rs:154 | `aga_crossover_probabilities` | complex | done |
| src/validators/generic_validator.rs:176 | `number_of_couples_is_set` | complex | done |
| src/validators/generic_validator.rs:193 | `operator_compat_check` | complex | done |

## src/observe/visualization/

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/observe/visualization/mod.rs:42 | `VisualizationError` | simple | done |
| src/observe/visualization/mod.rs:315 | `plot_fitness` | complex | done |
| src/observe/visualization/mod.rs:379 | `plot_diversity` | complex | done |
| src/observe/visualization/mod.rs:445 | `plot_histogram` | complex | done |
| src/observe/visualization/mod.rs:585 | `plot_pareto_front_2d` | complex | done |
| src/observe/visualization/mod.rs:725 | `plot_pareto_front_3d` | complex | done |
| src/observe/visualization/mod.rs:845 | `plot_true_fitness_calls` | complex | done |

## src/observe/checkpoint.rs (serde feature)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/observe/checkpoint.rs:40 | `Checkpoint<U>` | complex | done |
| src/observe/checkpoint.rs:62 | `save_checkpoint` | complex | done |
| src/observe/checkpoint.rs:97 | `load_checkpoint` | complex | done |

## src/operations/crossover/* (individual operators)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/crossover/arithmetic.rs:35 | `arithmetic` | complex | done |
| src/operations/crossover/arithmetic.rs:95 | `ArithmeticConvertible` | complex | done |
| src/operations/crossover/blend_alpha.rs:35 | `blend_alpha` | complex | done |
| src/operations/crossover/blend_alpha.rs:103 | `BlendConvertible` | complex | done |
| src/operations/crossover/clone.rs:20 | `clone_crossover` | complex | done |
| src/operations/crossover/cycle.rs:19 | `cycle` | complex | done |
| src/operations/crossover/edge_recombination.rs:26 | `erx` | complex | done |
| src/operations/crossover/multi_group_ox.rs:44 | `multi_group_ox` | complex | done |
| src/operations/crossover/multi_group_pmx.rs:33 | `multi_group_pmx` | complex | done |
| src/operations/crossover/multipoint.rs:14 | `multipoint` | complex | done |
| src/operations/crossover/order.rs:19 | `order` | complex | done |
| src/operations/crossover/pmx.rs:23 | `pmx` | complex | done |
| src/operations/crossover/rejuvenate.rs:23 | `rejuvenate` | complex | done |
| src/operations/crossover/sbx.rs:30 | `sbx` | complex | done |
| src/operations/crossover/sbx.rs:107 | `SbxConvertible` | complex | done |
| src/operations/crossover/single_point.rs:17 | `single_point` | complex | done |
| src/operations/crossover/uniform_crossover.rs:17 | `uniform` | complex | done |
| src/operations/crossover/variable_length.rs:34 | `variable_length_crossover` | complex | done |
| src/operations/crossover/pcx.rs:35 | `pcx` | complex | done |
| src/operations/crossover/spx.rs:28 | `spx` | complex | done |
| src/operations/crossover/undx.rs:34 | `undx` | complex | done |

## src/operations/mutation/* (individual operators)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/mutation/bit_flip.rs:16 | `bit_flip` | complex | done |
| src/operations/mutation/cauchy.rs:28 | `cauchy_mutation` | complex | done |
| src/operations/mutation/creep.rs:28 | `creep_mutation` | complex | done |
| src/operations/mutation/differential.rs:40 | `differential_mutation` | complex | done |
| src/operations/mutation/gaussian.rs:27 | `gaussian_mutation` | complex | done |
| src/operations/mutation/gaussian.rs:67 | `GaussianConvertible` | complex | done |
| src/operations/mutation/gaussian.rs:126 | `multi_range_gaussian_mutation` | complex | done |
| src/operations/mutation/insertion.rs:40 | `insertion_mutation` | complex | done |
| src/operations/mutation/inversion.rs:18 | `inversion` | complex | done |
| src/operations/mutation/length_mutation.rs:44 | `length_insertion_mutation` | complex | done |
| src/operations/mutation/length_mutation.rs:108 | `length_deletion_mutation` | complex | done |
| src/operations/mutation/levy_flight.rs:42 | `levy_flight_mutation` | complex | done |
| src/operations/mutation/list_value.rs:23 | `list_value_mutation` | complex | done |
| src/operations/mutation/non_uniform.rs:41 | `non_uniform_mutation` | complex | done |
| src/operations/mutation/non_uniform.rs:116 | `NonUniformConvertible` | complex | done |
| src/operations/mutation/polynomial.rs:40 | `polynomial_mutation` | complex | done |
| src/operations/mutation/polynomial.rs:104 | `PolynomialConvertible` | complex | done |
| src/operations/mutation/scramble.rs:17 | `scramble` | complex | done |
| src/operations/mutation/self_adaptive_gaussian.rs:50 | `self_adaptive_gaussian_mutation` | complex | done |
| src/operations/mutation/swap.rs:17 | `swap` | complex | done |
| src/operations/mutation/uniform.rs:21 | `uniform_mutation` | complex | done |
| src/operations/mutation/value.rs:23 | `value_mutation` | complex | done |

## src/operations/selection/* (individual operators)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/selection/boltzmann.rs:36 | `boltzmann_selection` | complex | done |
| src/operations/selection/clearing.rs:30 | `clearing_selection` | complex | done |
| src/operations/selection/fitness_proportionate.rs:30 | `roulette_wheel_selection` | complex | done |
| src/operations/selection/fitness_proportionate.rs:94 | `stochastic_universal_sampling` | complex | done |
| src/operations/selection/lexicase.rs:118 | `lexicase_selection` | complex | done |
| src/operations/selection/lexicase.rs:172 | `epsilon_lexicase_selection` | complex | done |
| src/operations/selection/random.rs:23 | `random` | complex | done |
| src/operations/selection/rank.rs:33 | `rank_selection` | complex | done |
| src/operations/selection/tournament.rs:25 | `tournament` | complex | done |
| src/operations/selection/truncation.rs:35 | `truncation_selection` | complex | done |

## src/operations/survivor/* (individual operators)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/survivor/age.rs:19 | `age_based` | complex | done |
| src/operations/survivor/deterministic_crowding.rs:44 | `deterministic_crowding` | complex | done |
| src/operations/survivor/fitness.rs:28 | `fitness_based` | complex | done |
| src/operations/survivor/mu_comma_lambda.rs:26 | `mu_comma_lambda` | complex | done |
| src/operations/survivor/mu_plus_lambda.rs:27 | `mu_plus_lambda` | complex | done |
| src/operations/survivor/parsimony.rs:63 | `apply_parsimony_pressure` | complex | done |

## src/operations/extension/* (individual operators)

| file:line | item | classification | needs_examples |
|-----------|------|---------------|----------------|
| src/operations/extension/mass_deduplication.rs:17 | `mass_deduplication` | complex | done |
| src/operations/extension/mass_degeneration.rs:13 | `mass_degeneration` | complex | done |
| src/operations/extension/mass_extinction.rs:14 | `mass_extinction` | complex | done |
| src/operations/extension/mass_genesis.rs:13 | `mass_genesis` | complex | done |
| src/operations/extension/mod.rs:62 | `factory` (extension) | complex | done |

---

## Summary

| Category | Total | needs_examples | already_has |
|----------|-------|---------------|-------------|
| genotypes/ | 5 | 0 | 5 |
| chromosomes/ | 7 | 1 | 6 |
| traits/ | 16 | 15 | 1 |  
| operations enums | 6 | 6 | 0 |
| operations/crossover/* | 22 | 22 | 0 |
| operations/mutation/* | 22 | 22 | 0 |
| operations/selection/* | 10 | 10 | 0 |
| operations/survivor/* | 6 | 6 | 0 |
| operations/extension/* | 5 | 5 | 0 |
| operations/local_search | 6 | 6 | 0 |
| operations factories | 6 | 6 | 0 |
| configuration.rs | 9 | 9 | 0 |
| initializers/ | 8 | 2 | 6 |
| niching/ | 9 | 5 | 4 |
| extension/ | 1 | 0 | 1 |
| fitness/ | 7 | 7 | 0 |
| constraints.rs | 6 | 6 | 0 |
| aos.rs | 3 | 3 | 0 |
| hall_of_fame.rs | 5 | 5 | 0 |
| observer/* | 13 | 13 | 0 |
| visualization/ | 7 | 7 | 0 |
| checkpoint.rs | 3 | 3 | 0 |
| engines/ga.rs | 2 | 1 | 1 |
| engines/island/ | 9 | 8 | 1 |
| engines/nsga2/ | 12 | 11 | 1 |
| engines/nsga3/ | 3 | 2 | 1 |
| engines/moead/ | 3 | 2 | 1 |
| engines/spea2/ | 2 | 1 | 1 |
| engines/sms_emoa/ | 2 | 1 | 1 |
| engines/ibea/ | 2 | 1 | 1 |
| engines/alps/ | 5 | 5 | 0 |
| engines/cellular/ | 5 | 5 | 0 |
| engines/de/ | 9 | 9 | 0 |
| engines/gp/ | 13 | 13 | 0 |
| engines/cma/ | 6 | 6 | 0 |
| engines/scatter/ | 3 | 3 | 0 |
| engines/pso/ | 5 | 5 | 0 |
| engines/eda/ | 5 | 5 | 0 |
| engines/hill_climb/ | 3 | 3 | 0 |
| engines/permutate/ | 2 | 2 | 0 |
| engines/multi_objective/ | 14 | 14 | 0 |
| error.rs | 1 | 1 | 0 |
| population.rs | 1 | 1 | 0 |
| rng.rs | 2 | 2 | 0 |
| stats.rs | 1 | 1 | 0 |
| validators/ | 8 | 8 | 0 |
| **TOTAL** | **~320** | **~283** | **~37** |
