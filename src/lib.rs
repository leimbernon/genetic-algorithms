//! Genetic Algorithms library with flexible genotypes, chromosomes, and operators.
//!
//! This crate provides a modular and efficient framework for building Genetic Algorithms (GAs).
//! It focuses on:
//! - Clear abstractions (traits for genes and chromosomes).
//! - Efficient data handling using Cow to avoid unnecessary clones.
//! - Composable operators (selection, crossover, mutation, survivor).
//! - Parallelism for fitness evaluation and reproduction.
//!
//! Key concepts:
//! - Genotypes: reusable gene types (e.g., Binary, Range) that implement `GeneT`.
//! - Chromosomes: sequences of genes that implement `ChromosomeT` and hold fitness and age.
//! - GA Orchestrator: `ga::Ga` coordinates the lifecycle (init, selection, crossover, mutation, survivor, evaluation).
//! - Configuration: a cohesive configuration model to tune operators and limits.
//!
//! Quickstart (example):
//! ```ignore
//! use genetic_algorithms::configuration::ProblemSolving;
//! use genetic_algorithms::genotypes::Range as RangeGene;
//! use genetic_algorithms::chromosomes::Range as RangeChromosome;
//! use genetic_algorithms::ga::Ga;
//! use genetic_algorithms::initializers::range_random_initialization;
//! use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
//! use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
//!
//! const N: i32 = 8;
//!
//! fn fitness_fn(dna: &[RangeGene<i32>]) -> f64 {
//!     // ... compute fitness ...
//!     0.0
//! }
//!
//! let alleles = vec![RangeGene::new(0, vec![(0, N - 1)], 0)];
//! let alleles_clone = alleles.clone();
//! let mut ga = Ga::new()
//!     .with_genes_per_chromosome(N)
//!     .with_population_size(100)
//!     .with_initialization_fn(move |genes_per_chromosome, _, _| {
//!         range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
//!     })
//!     .with_fitness_fn(fitness_fn)
//!     .with_selection_method(Selection::Tournament)
//!     .with_crossover_method(Crossover::Uniform)
//!     .with_mutation_method(Mutation::Swap) // Swap or value-mutation for Range<i32> chromosomes
//!     .with_problem_solving(ProblemSolving::Minimization)
//!     .with_survivor_method(Survivor::Fitness)
//!     .with_max_generations(5000)
//!     .with_fitness_target(0.0)
//!     .build()
//!     .expect("Invalid configuration");
//! let _population = ga.run();
//! ```
//!
//! Modules of interest:
//! - `traits`: core traits for genes, chromosomes, and configuration.
//! - `chromosomes`: concrete chromosome implementations (Binary, Range).
//! - `genotypes`: concrete gene implementations (Binary, Range).
//! - `operations`: selection, crossover, mutation, survivor operators.
//! - `ga`: genetic algorithm orchestrator.
//! - `population`: population management and best tracking.
//! - `fitness`: helpers to wrap user fitness functions.
//! - `initializers`: utilities to build initial DNA.
//!
//! Performance notes:
//! - Chromosome DNA is set via a single `set_dna(Cow<[Gene]>)` method to avoid extra copies.
//! - Operators use pre-allocation and move semantics where possible.
//! - Parallel evaluation leverages threads to scale fitness computation.
extern crate core;

#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;
#[path = "types/chromosomes/mod.rs"]
pub mod chromosomes;
pub mod configuration;
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
#[path = "observe/reporter/mod.rs"]
pub mod reporter;
pub mod rng;
pub mod stats;
pub mod traits;
pub mod validators;
#[cfg(feature = "visualization")]
#[path = "observe/visualization/mod.rs"]
pub mod visualization;

#[path = "engines/de/mod.rs"]
pub mod de;
#[path = "engines/island/mod.rs"]
pub mod island;
pub mod niching;
#[path = "engines/nsga2/mod.rs"]
pub mod nsga2;

pub use ga::TerminationCause;
pub use observer::AllObserver;
pub use observer::CompositeObserver;
pub use observer::ExtensionEvent;
pub use observer::GaObserver;
pub use observer::IslandGaObserver;
pub use observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use observer::MetricsObserver;
pub use observer::NoopObserver;
pub use observer::Nsga2Observer;
#[cfg(feature = "observer-tracing")]
pub use observer::TracingObserver;
