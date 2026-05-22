//! # genetic_algorithms
//!
//! A modular, performant Rust library for evolutionary computation and metaheuristic optimization.
//! Provides 11 optimization engines, 45+ composable operators, a full lifecycle observer system,
//! and framework extensions — all generic over chromosome and gene types via traits.
//!
//! Published on [crates.io](https://crates.io/crates/genetic_algorithms) as `genetic_algorithms`.
//! API reference on [docs.rs](https://docs.rs/genetic_algorithms/latest/genetic_algorithms).
//!
//! ## Quick Start
//!
//! Minimize the Rastrigin function using 5-dimensional [`Range<f64>`](crate::genotypes::Range)
//! chromosomes with the standard [`Ga`](crate::ga::Ga) engine:
//!
//! ```rust,ignore
//! use genetic_algorithms::chromosomes::Range as RangeChromosome;
//! use genetic_algorithms::configuration::ProblemSolving;
//! use genetic_algorithms::ga::Ga;
//! use genetic_algorithms::genotypes::Range as RangeGenotype;
//! use genetic_algorithms::initializers::range_random_initialization;
//! use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
//! use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
//! use genetic_algorithms::{CompositeObserver, LogObserver};
//! use std::sync::Arc;
//!
//! let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
//!     let a = 10.0;
//!     let n = dna.len() as f64;
//!     a * n + dna.iter()
//!         .map(|g| g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos())
//!         .sum::<f64>()
//! };
//!
//! let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
//! let alleles_clone = alleles.clone();
//!
//! let mut ga = Ga::new()
//!     .with_genes_per_chromosome(5_usize)
//!     .with_population_size(100)
//!     .with_initialization_fn(move |genes_per_chromosome, _, _| {
//!         range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
//!     })
//!     .with_fitness_fn(fitness_fn)
//!     .with_selection_method(Selection::Tournament)
//!     .with_crossover_method(Crossover::Uniform)
//!     .with_mutation_method(Mutation::Gaussian)
//!     .with_survivor_method(Survivor::Fitness)
//!     .with_problem_solving(ProblemSolving::Minimization)
//!     .with_max_generations(500)
//!     .with_observer(Arc::new(CompositeObserver::new().add(Arc::new(LogObserver))))
//!     .build()
//!     .expect("Invalid configuration");
//!
//! let population = ga.run().expect("GA run failed");
//! println!("Best fitness: {:.4}", population.best_chromosome.fitness);
//! ```
//!
//! ## Engines (11 total)
//!
//! This crate offers 11 optimization engines, each designed for specific problem types:
//!
//! | Engine | Module | Objectives | Problem Type | Key Strength |
//! |--------|--------|------------|--------------|--------------|
//! | [`Ga<U>`](crate::ga::Ga) | `ga` | Single | Continuous / Combinatorial | General-purpose evolutionary optimization |
//! | [`DeEngine<U>`](crate::de::DeEngine) | `de` | Single | Continuous | Fast convergence on real-valued problems via differential mutation |
//! | [`ScatterEngine<U>`](crate::scatter::ScatterEngine) | `scatter` | Single | Continuous / Combinatorial | Reference-set based diversification |
//! | [`CellularEngine<U>`](crate::cellular::CellularEngine) | `cellular` | Single | Continuous / Combinatorial | Spatial diversity via toroidal grid neighborhoods |
//! | [`AlpsEngine<U>`](crate::alps::AlpsEngine) | `alps` | Single | Continuous / Combinatorial | Age-layered population structure for sustained diversity |
//! | [`IslandGa<U>`](crate::island::IslandGa) | `island` | Single | Any | Parallel sub-populations with configurable migration topologies |
//! | [`Nsga2Ga<U>`](crate::nsga2::Nsga2Ga) | `nsga2` | 2 | Continuous / Combinatorial | Fast Pareto ranking with crowding distance |
//! | [`Nsga3Ga<U>`](crate::nsga3::Nsga3Ga) | `nsga3` | 3+ | Continuous | Reference-point based niching for many-objective problems |
//! | [`MoeaDGa<U>`](crate::moead::MoeaDGa) | `moead` | 2+ | Continuous | Decomposition-based scalarization (Tchebycheff, PBI, weighted sum) |
//! | [`Spea2Ga<U>`](crate::spea2::Spea2Ga) | `spea2` | 2+ | Continuous / Combinatorial | Strength-Pareto archive with k-nearest density estimation |
//! | [`SmsEmoaGa<U>`](crate::sms_emoa::SmsEmoaGa) / [`IbeaGa<U>`](crate::ibea::IbeaGa) | `sms_emoa` / `ibea` | 2+ | Continuous | Hypervolume contribution / indicator-based selection |
//!
//! ### Single-Objective Engines
//!
//! - [`Ga<U>`](crate::ga::Ga) — Standard single-population GA. The general-purpose engine with full
//!   operator support (all selection, crossover, mutation, survivor strategies), adaptive GA mode,
//!   elitism, extension strategies, niching/fitness sharing, and checkpointing.
//! - [`DeEngine<U>`](crate::de::DeEngine) — Differential Evolution engine with 5 mutation strategies
//!   (rand/1, best/1, current-to-best/1, best/2, rand/2) and two adaptive variants (JADE, L-SHADE)
//!   with binomial and exponential crossover. Best for continuous, real-valued optimization.
//! - [`ScatterEngine<U>`](crate::scatter::ScatterEngine) — Scatter Search metaheuristic that maintains
//!   a diverse reference set, combines solutions via linear combination, and applies improvement
//!   methods. Strong on combinatorial optimization with rugged landscapes.
//! - [`CellularEngine<U>`](crate::cellular::CellularEngine) — Cellular GA that arranges the population
//!   on a 2D toroidal grid. Supports 4 neighborhood topologies (Von Neumann, Moore, Compact R2,
//!   Linear) and synchronous or asynchronous update. Preserves spatial diversity.
//! - [`AlpsEngine<U>`](crate::alps::AlpsEngine) — Age-Layered Population Structure that organizes
//!   individuals into age layers with 3 age schemes (Linear, Fibonacci, Polynomial). Enables
//!   cross-layer mating and periodic injection for sustained exploration.
//! - [`IslandGa<U>`](crate::island::IslandGa) — Island model GA that evolves multiple sub-populations
//!   in parallel with configurable migration topology (ring, fully connected, random), frequency,
//!   and migrant selection. Works with any chromosome type and can wrap [`Nsga2Ga`](crate::nsga2::Nsga2Ga).
//!
//! ### Multi-Objective Engines
//!
//! - [`Nsga2Ga<U>`](crate::nsga2::Nsga2Ga) — NSGA-II: fast non-dominated sorting with crowding
//!   distance for diversity. Efficient O(MN²) ranking. Standard choice for 2-objective problems.
//! - [`Nsga3Ga<U>`](crate::nsga3::Nsga3Ga) — NSGA-III: reference-point based selection using
//!   Das-Dennis weights and ASF-based normalization. Scales to 3+ objectives where crowding
//!   distance loses effectiveness. Supports 2 to 15+ objectives.
//! - [`MoeaDGa<U>`](crate::moead::MoeaDGa) — MOEA/D: decomposes multi-objective problems into
//!   scalar sub-problems using Tchebycheff, PBI, or weighted sum aggregation. Neighbor-based
//!   mating preserves convergence along the Pareto front.
//! - [`Spea2Ga<U>`](crate::spea2::Spea2Ga) — SPEA2: strength-Pareto approach with an external
//!   archive, k-nearest neighbor density estimation, and truncation mechanism. Effective on
//!   problems with irregular Pareto fronts.
//! - [`SmsEmoaGa<U>`](crate::sms_emoa::SmsEmoaGa) — SMS-EMOA: steady-state (mu+1) MOEA that
//!   selects the individual with the smallest hypervolume contribution for removal. Precise
//!   hypervolume-based convergence.
//! - [`IbeaGa<U>`](crate::ibea::IbeaGa) — IBEA: indicator-based MOEA using the I_epsilon+
//!   binary indicator with exponential fitness scaling. No diversity preservation mechanism
//!   required — the indicator implicitly balances convergence and diversity.
//!
//! ## Feature Flags
//!
//! | Flag | Description | Default |
//! |------|-------------|---------|
//! | `serde` | Serialization/deserialization for checkpoint save/load via `serde_json` | Off |
//! | `benchmarks` | Standard benchmark functions (Sphere, Rastrigin, Rosenbrock, ZDT, DTLZ) and multi-objective quality indicators | Off |
//! | `visualization` | PNG/SVG fitness plots, diversity charts, and histogram generation via `plotters` | Off |
//! | `observer-tracing` | Structured `tracing`-crate spans and events in the observer system | Off |
//! | `observer-metrics` | Per-generation metrics (gauges, counters, histograms) via the `metrics` facade | Off |
//!
//! ## Key Concepts
//!
//! ### Genotypes & Chromosomes
//!
//! The library separates the concept of a **gene** (a single atomic unit of information) from a
//! **chromosome** (a sequence of genes that represents a candidate solution). Genes implement
//! [`GeneT`](crate::traits::GeneT) and chromosomes implement
//! [`ChromosomeT`](crate::traits::ChromosomeT).
//!
//! Built-in gene types: [`Binary`](crate::genotypes::Binary) (boolean), [`Range<T>`](crate::genotypes::Range)
//! (bounded real/integer values), [`List<T>`](crate::genotypes::List) (finite symbolic alphabets).
//!
//! Built-in chromosome types: [`chromosomes::Binary`],
//! [`chromosomes::Range<T>`], [`chromosomes::ListChromosome<T>`].
//!
//! Custom chromosomes can be created by implementing [`ChromosomeT`](crate::traits::ChromosomeT).
//!
//! ### Configuration
//!
//! All engines use a fluent builder pattern via the
//! [`ConfigurationT`](crate::traits::ConfigurationT) trait. Each engine has its own configuration
//! struct ([`GaConfiguration`](crate::configuration::GaConfiguration),
//! [`DeConfiguration`](crate::de::DeConfiguration), etc.) with builder methods for limits, operators,
//! stopping criteria, and engine-specific parameters.
//!
//! See the [`configuration`] module for all shared configuration types.
//!
//! ### Operators
//!
//! Five operator categories are available, dispatched via enums with factory functions:
//!
//! - **Selection** ([`Selection`](crate::operations::Selection)): Random, RouletteWheel, SUS, Tournament,
//!   Rank, Boltzmann, Truncation, Clearing
//! - **Crossover** ([`Crossover`](crate::operations::Crossover)): Cycle, MultiPoint, Uniform, SinglePoint,
//!   Order (OX), PMX, SBX, BlendAlpha (BLX-alpha), Arithmetic, Edge Recombination, Clone, Rejuvenate
//! - **Mutation** ([`Mutation`](crate::operations::Mutation)): Swap, Inversion, Scramble, Value, BitFlip,
//!   Creep, Gaussian, Polynomial, NonUniform, Insertion, Cauchy, LevyFlight, Uniform, ListValue
//! - **Survivor** ([`Survivor`](crate::operations::Survivor)): Fitness, Age, MuPlusLambda, MuCommaLambda,
//!   DeterministicCrowding
//! - **Extension** ([`Extension`](crate::operations::Extension)): Noop, MassExtinction, MassGenesis,
//!   MassDegeneration, MassDeduplication
//!
//! See the [`operations`] module for full documentation of each operator.
//!
//! ### Observer System
//!
//! The [`GaObserver<U>`] trait provides 11 lifecycle hooks that fire during engine
//! execution (selection, crossover, mutation, fitness evaluation, survivor selection, new best,
//! stagnation, extension trigger, generation end, run start/end). Built-in observers include
//! [`LogObserver`], [`CompositeObserver<U>`],
//! `MetricsObserver` (feature `observer-metrics`),
//! `TracingObserver` (feature `observer-tracing`),
//! and [`NoopObserver`]. Engine-specific sub-traits exist for
//! [`IslandGaObserver`], [`Nsga2Observer`],
//! [`Nsga3Observer`], [`Spea2Observer`],
//! [`MoeaDObserver`], [`SmsEmoaObserver`],
//! and [`IbeaObserver`].
//!
//! Zero overhead when no observer is attached (stored as `Option<Arc<dyn GaObserver<U>>>`).
//!
//! ### Constraints
//!
//! The [`ConstraintHandling`] system provides constraint enforcement
//! through penalty strategies ([`PenaltyStrategy`] — Static, Dynamic,
//! Adaptive, Death) and feasibility rules. See the [`constraints`] module.
//!
//! ### Hall of Fame
//!
//! The [`HallOfFame<U>`] archive stores elite solutions across generations,
//! with configurable capacity and optional diversity enforcement via
//! [`DistanceMetric`]. See the [`hall_of_fame`] module.
//!
//! ### Adaptive Operator Selection (AOS)
//!
//! The [`AosStrategy`] and [`AosState`] types implement
//! credit-assignment based operator selection. See the [`aos`] module.
//!
//! ### Initializers
//!
//! Population initialization functions in the [`initializers`] module:
//! `binary_random_initialization`, `range_random_initialization`,
//! `list_random_initialization`, `list_random_initialization_without_repetitions`,
//! `generic_random_initialization`, `generic_random_initialization_without_repetitions`.
//!
//! ## When to Use Which Engine
//!
//! | If your problem is... | Use this engine... | Because... |
//! |-----------------------|--------------------|------------|
//! | General single-objective, any variable type | [`Ga<U>`](crate::ga::Ga) | Full operator support, adaptive mode, niching, extensions |
//! | Continuous, real-valued, single-objective | [`DeEngine<U>`](crate::de::DeEngine) | 5 strategies + JADE/L-SHADE, fast convergence |
//! | Combinatorial with rugged landscape | [`ScatterEngine<U>`](crate::scatter::ScatterEngine) | Reference set diversification avoids local optima |
//! | Single-objective, needs spatial diversity | [`CellularEngine<U>`](crate::cellular::CellularEngine) | Grid-based neighborhoods preserve niche exploration |
//! | Single-objective, sustained exploration needed | [`AlpsEngine<U>`](crate::alps::AlpsEngine) | Age layers prevent premature convergence |
//! | Parallel / distributed single-objective | [`IslandGa<U>`](crate::island::IslandGa) | Independent sub-populations with periodic migration |
//! | Exactly 2 objectives | [`Nsga2Ga<U>`](crate::nsga2::Nsga2Ga) | Fast O(MN²) ranking, well-established baseline |
//! | 3+ objectives (many-objective) | [`Nsga3Ga<U>`](crate::nsga3::Nsga3Ga) | Reference points scale beyond crowding distance |
//! | 2+ objectives, continuous | [`MoeaDGa<U>`](crate::moead::MoeaDGa) | Decomposition is efficient and interpretable |
//! | 2+ objectives, irregular front | [`Spea2Ga<U>`](crate::spea2::Spea2Ga) | Archive + density estimation handles complex fronts |
//! | 2+ objectives, precise convergence | [`SmsEmoaGa<U>`](crate::sms_emoa::SmsEmoaGa) | Hypervolume contribution directly measures quality |
//! | 2+ objectives, flexible indicator | [`IbeaGa<U>`](crate::ibea::IbeaGa) | Indicator-based selection needs no extra diversity mechanism |
//!
//! ## Examples
//!
//! The `examples/` directory contains 19 runnable examples covering all engines and features.
//! Run any example with `cargo run --example <name> --features <features>`.
//!
//! See the [README](https://github.com/leimbernon/rust_genetic_algorithms#readme) for a full
//! examples catalog and the [docs/ directory](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/docs)
//! for per-engine guides with complete usage examples.
//!
//! ## Further Reading
//!
//! - [README](https://github.com/leimbernon/rust_genetic_algorithms#readme) — Installation, full examples catalog, engines overview
//! - [docs/ directory](https://github.com/leimbernon/rust_genetic_algorithms/tree/main/docs) — Per-engine algorithm guides and framework extension docs
//! - [docs.rs/genetic_algorithms](https://docs.rs/genetic_algorithms/latest/genetic_algorithms) — Full API reference with module-level documentation
//! - [crates.io](https://crates.io/crates/genetic_algorithms) — Package registry and version history

extern crate core;

#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;
#[path = "types/chromosomes/mod.rs"]
pub mod chromosomes;
pub mod configuration;
pub mod constraints;
pub mod error;
pub mod extension;
pub mod fitness;
#[path = "engines/ga.rs"]
pub mod ga;
#[path = "types/genotypes/mod.rs"]
pub mod genotypes;
pub mod initializers;
#[path = "observe/observer/mod.rs"]
pub mod observer;
pub mod operations;
pub mod population;
pub mod rng;
pub mod hall_of_fame;
pub mod aos;
#[cfg(feature = "benchmarks")]
pub mod benchmarks;
pub mod stats;
pub mod traits;
pub mod validators;
#[cfg(feature = "visualization")]
#[path = "observe/visualization/mod.rs"]
pub mod visualization;

#[path = "engines/alps/mod.rs"]
pub mod alps;
#[path = "engines/cellular/mod.rs"]
pub mod cellular;
#[path = "engines/de/mod.rs"]
pub mod de;
#[path = "engines/scatter/mod.rs"]
pub mod scatter;
#[path = "engines/island/mod.rs"]
pub mod island;
pub mod niching;
#[path = "engines/multi_objective/mod.rs"]
pub mod multi_objective;
#[path = "engines/nsga2/mod.rs"]
pub mod nsga2;
#[path = "engines/nsga3/mod.rs"]
pub mod nsga3;

#[path = "engines/moead/mod.rs"]
pub mod moead;

#[path = "engines/spea2/mod.rs"]
pub mod spea2;

#[path = "engines/sms_emoa/mod.rs"]
pub mod sms_emoa;

#[path = "engines/ibea/mod.rs"]
pub mod ibea;

pub use ga::TerminationCause;
pub use observer::AllObserver;
pub use observer::CompositeObserver;
pub use observer::ExtensionEvent;
pub use observer::GaObserver;
pub use observer::IslandGaObserver;
pub use observer::LogObserver;
pub use observer::MoeaDObserver;
pub use observer::IbeaObserver;
pub use observer::SmsEmoaObserver;
pub use observer::Spea2Observer;
#[cfg(feature = "observer-metrics")]
pub use observer::MetricsObserver;
pub use observer::NoopObserver;
pub use observer::Nsga2Observer;
pub use observer::Nsga3Observer;
#[cfg(feature = "observer-tracing")]
pub use observer::TracingObserver;
pub use constraints::ConstraintHandling;
pub use constraints::PenaltyStrategy;
pub use hall_of_fame::{DistanceMetric, HallOfFame, HallOfFameConfig};
pub use aos::{AosState, AosStrategy};
pub use traits::LinearChromosome;
pub use traits::OperatorCompat;
pub use traits::Strategy;
pub use chromosomes::ChromosomeLength;
pub use chromosomes::UniqueChromosome;
pub use genotypes::UniqueGenotype;
pub use initializers::unique_random_initialization;
pub use chromosomes::MultiRangeChromosome;
pub use genotypes::MultiRangeGenotype;
pub use initializers::multi_range_random_initialization;
pub use chromosomes::MultiUniqueChromosome;
