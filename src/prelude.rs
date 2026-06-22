//! # Prelude
//!
//! Convenient glob import for the most commonly used items from `genetic_algorithms`.
//!
//! This module re-exports engine entry points, core traits, operator enums,
//! configuration types, and error types so users can write:
//!
//! ```rust
//! use genetic_algorithms::prelude::*;
//! ```
//!
//! # What's included
//!
//! | Category | Items |
//! |----------|-------|
//! | Engines | `Ga`, `DeEngine`, `ScatterEngine`, `CellularEngine`, `AlpsEngine`, `IslandGa`, `GpGa`, `EdaEngine`, `EdaRealEngine`, `Nsga2Ga`, `Nsga3Ga`, `MoeaDGa`, `Spea2Ga`, `SmsEmoaGa`, `IbeaGa`, `CmaEngine`, `PsoEngine`, `HillClimbEngine`, `PermutateEngine` |
//! | Engine configs | `CmaConfiguration`, `PsoConfiguration`, `EdaConfiguration`, `AlpsConfiguration`, `HillClimbConfiguration`, `PermutateConfiguration`, `DeConfiguration`, `ScatterConfiguration`, `CellularConfiguration`, `GpConfiguration` |
//! | Core traits | `ChromosomeT`, `GeneT`, `LinearChromosome`, `ConfigurationT`, `CrossoverConfig`, `ElitismConfig`, `ExtensionConfig`, `LocalSearchConfig`, `MutationConfig`, `NichingConfig`, `SelectionConfig`, `StoppingConfig`, `SurvivorConfig` |
//! | Operator enums | `Selection`, `Crossover`, `Mutation`, `Survivor` |
//! | Config types | `ProblemSolving`, `ChromosomeLength` |
//! | Error | `GaError` |
//! | Observer | `GaObserver`, `NoopObserver` |
//!
//! Concrete chromosome/genotype types (`Binary`, `Range<T>`, `ListChromosome<T>`) and
//! initializer functions are intentionally excluded — they are problem-specific and should
//! be imported explicitly.

// --- Engine entry points ---
pub use crate::ga::Ga;
pub use crate::de::DeEngine;
pub use crate::scatter::ScatterEngine;
pub use crate::cellular::CellularEngine;
pub use crate::alps::AlpsEngine;
pub use crate::island::IslandGa;
pub use crate::gp::GpGa;
pub use crate::eda::{EdaEngine, EdaRealEngine};
pub use crate::nsga2::Nsga2Ga;
pub use crate::nsga3::Nsga3Ga;
pub use crate::moead::MoeaDGa;
pub use crate::spea2::Spea2Ga;
pub use crate::sms_emoa::SmsEmoaGa;
pub use crate::ibea::IbeaGa;
pub use crate::cma::CmaEngine;
pub use crate::pso::PsoEngine;
pub use crate::hill_climb::HillClimbEngine;
pub use crate::permutate::PermutateEngine;

// --- Engine-specific configuration structs ---
pub use crate::cma::CmaConfiguration;
pub use crate::pso::PsoConfiguration;
pub use crate::eda::EdaConfiguration;
pub use crate::alps::AlpsConfiguration;
pub use crate::hill_climb::HillClimbConfiguration;
pub use crate::permutate::PermutateConfiguration;
pub use crate::de::DeConfiguration;
pub use crate::scatter::ScatterConfiguration;
pub use crate::cellular::CellularConfiguration;
pub use crate::gp::GpConfiguration;

// --- Core traits ---
pub use crate::traits::{
    ChromosomeT, ConfigurationT, GeneT, LinearChromosome, CrossoverConfig, ElitismConfig,
    ExtensionConfig, LocalSearchConfig, MutationConfig, NichingConfig, SelectionConfig,
    StoppingConfig, SurvivorConfig,
};

// --- Operator enums ---
pub use crate::operations::{Crossover, Mutation, Selection, Survivor};

// --- Configuration types ---
pub use crate::configuration::ProblemSolving;
pub use crate::chromosomes::ChromosomeLength;

// --- Error ---
pub use crate::error::GaError;

// --- Observer (minimal) ---
pub use crate::observer::{GaObserver, NoopObserver};

// --- Feature-gated observers ---
#[cfg(feature = "logging")]
pub use crate::observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use crate::observer::MetricsObserver;
#[cfg(feature = "observer-tracing")]
pub use crate::observer::TracingObserver;
