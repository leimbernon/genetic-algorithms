# genetic_algorithms — Documentation

> Comprehensive guides for every algorithm, operator, and feature.
> Use this index to navigate the per-engine guides, operator documentation, core concepts, and framework extensions.

This directory contains 30+ guide files that document all 15 optimization engines, all 50+ operators (selection, crossover, mutation, survivor, extension), core abstractions (traits, chromosomes, genotypes, configuration, population), and framework extensions (constraints, hall of fame, AOS, benchmarks, memetic algorithms, niching, error handling, initializers, observer system).

All guides are written following the Single Source of Truth (SSOT) principle, where `src/lib.rs` provides the crate-level overview, this `docs/` directory provides per-topic deep dives, and the README provides the complete feature catalog.

## Navigation

### Getting Started
- [Getting Started](getting-started.md) — Installation, quickstart, and basic concepts for new users

### Engines (15 total)

Each engine has a dedicated guide covering algorithm description, when-to-use guidance, parameter tables, and complete examples:

- [Engines Overview](engines.md) — Decision matrix: which engine to choose for your problem
- [Standard GA](engines.md#gau--standard-genetic-algorithm) — Single-population genetic algorithm with full operator support
- [Differential Evolution](engines.md#deengine--differential-evolution) — DE with 5 mutation strategies and JADE/L-SHADE adaptive variants
- [Scatter Search](engines.md#scatterengineu--scatter-search) — Reference-set metaheuristic for combinatorial optimization
- [Cellular GA](engines.md#cellularengineu--cellular-genetic-algorithm) — Neighborhood-based evolution on a 2D toroidal grid
- [ALPS](engines.md#alpsengineu--age-layered-population-structure) — Age-layered diversity with 3 age schemes (Linear, Fibonacci, Polynomial)
- [Island Model](engines.md#islandgau--island-model) — Parallel sub-populations with configurable migration topologies
- [Genetic Programming](gp.md) — Tree chromosome evolution with `GpGa<N>`, subtree/point/hoist mutation, bloat control, and built-in primitive sets
- [CMA-ES](cma.md) — Covariance Matrix Adaptation Evolution Strategy for continuous optimization
- [PSO](pso.md) — Particle Swarm Optimization with configurable inertia and topology
- [EDA](eda.md) — Univariate Marginal Distribution Algorithm (Bernoulli and Gaussian models)
- [NSGA-II](engines.md#nsga2gau--nsga-ii) — Pareto-ranking multi-objective optimization (2 objectives)
- [NSGA-III](nsga3.md) — Reference-point based many-objective optimization (3+ objectives)
- [MOEA/D](moead.md) — Decomposition-based multi-objective optimization
- [SPEA2](spea2.md) — Strength Pareto evolutionary algorithm with archive
- [SMS-EMOA](sms_emoa.md) — Hypervolume contribution-based steady-state MOEA
- [IBEA](ibea.md) — Indicator-based evolutionary algorithm using I_epsilon+ indicator
- [Multi-Objective Concepts](multi_objective.md) — Shared Pareto dominance, sorting, quality indicators, and reference point utilities

### Operators

Five operator categories provide 50+ strategies, dispatched via enum + factory functions:

- [Selection](operators/selection.md) — Tournament, RouletteWheel, SUS, Rank, Boltzmann, Truncation, Random, Clearing
- [Crossover](operators/crossover.md) — Uniform, SinglePoint, MultiPoint, Order (OX), PMX, SBX, BlendAlpha, Arithmetic, Cycle, Edge Recombination, Clone, Rejuvenate, VariableLength
- [Mutation](operators/mutation.md) — Swap, Inversion, Scramble, Value, BitFlip, Creep, Gaussian, Polynomial, NonUniform, PermutationInsert, Insertion (variable-length grow), Deletion (variable-length shrink), Cauchy, LevyFlight, Uniform, ListValue, Differential
- [Survivor](operators/survivor.md) — Fitness, Age, MuPlusLambda, MuCommaLambda, DeterministicCrowding; parsimony pressure via `with_length_penalty`
- [Extension](operators/extension.md) — Noop, MassExtinction, MassGenesis, MassDegeneration, MassDeduplication

### Core Concepts

- [Configuration](configuration.md) — Builder-based configuration for all engines and operators
- [Traits](traits.md) — GeneT, ChromosomeT, TreeChromosome, ConfigurationT, and operator trait definitions
- [Chromosomes & Genotypes](chromosomes.md) — Built-in types: Binary, Range, List, ListChromosome, ChromosomeLength, GpChromosome
- [Fitness](fitness.md) — Fitness function wrapper and distance computation
- [Population](population.md) — Population container, best tracking, and management
- [Validators](validators.md) — Configuration validation across all engines

### Framework Extensions

- [Constraints](constraints.md) — Constraint handling with Static, Dynamic, Adaptive, and Death penalty strategies
- [Hall of Fame](hall_of_fame.md) — Elite solution archive with configurable capacity and diversity enforcement
- [Adaptive Operator Selection](aos.md) — Credit-assignment based operator selection (AOS)
- [Benchmarks](benchmarks.md) — Standard benchmark functions (Sphere, Rastrigin, Rosenbrock, ZDT, DTLZ) — requires `benchmarks` feature
- [Memetic Algorithms](memetic.md) — Local search integration with GA
- [Niching](niching.md) — Fitness sharing for multimodal optimization
- [Operations Overview](operations.md) — Enum + factory dispatch pattern overview
- [Error Handling](error.md) — GaError enum and error handling patterns
- [Initializers](initializers.md) — Population initialization functions for all chromosome types

### Observability
- [Observer System](observer.md) — GaObserver lifecycle hooks, engine-specific sub-traits, built-in observers

### Reference

Developer-oriented guides for contributing and testing:

- [API Reference](api-reference.md) — Module-level API documentation with example code
- [Examples](examples.md) — End-to-end usage examples for all 19 examples
- [Architecture](ARCHITECTURE.md) — Codebase architecture and module organization
- [Development](DEVELOPMENT.md) — Development setup, build, and contribution guidelines
- [Testing](TESTING.md) — Testing strategy, running tests, and writing new tests

### External Resources

- [docs.rs/genetic_algorithms](https://docs.rs/genetic_algorithms) — Full API documentation with crate-level overview
- [README](https://github.com/leimbernon/rust_genetic_algorithms#readme) — Project overview, installation, and examples catalog
- [Crates.io](https://crates.io/crates/genetic_algorithms) — Package registry and version history
