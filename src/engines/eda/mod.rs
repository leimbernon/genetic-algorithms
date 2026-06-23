//! EDA engine — Estimation of Distribution Algorithm (UMDA variant).
//!
//! Implements the Univariate Marginal Distribution Algorithm (UMDA) for discrete
//! (binary Bernoulli) and continuous (Gaussian univariate) optimization.
//!
//! Use [`EdaEngine`] for binary/discrete chromosomes and [`EdaRealEngine`] for
//! real-valued chromosomes whose gene implements [`crate::traits::RealGene`].

pub mod configuration;
pub mod engine;

pub use configuration::EdaConfiguration;
pub use engine::{EdaEngine, EdaModel, EdaRealEngine, EdaResult};
