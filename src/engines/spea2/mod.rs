//! SPEA2 — Strength Pareto Evolutionary Algorithm 2.
//!
//! SPEA2 (Zitzler, Laumanns & Thiele 2001) is a multi-objective evolutionary
//! algorithm that maintains a fixed-size external archive of non-dominated
//! solutions. Fitness is computed from raw strength (domination count) plus
//! density (k-nearest-neighbour distance), and the archive is truncated using
//! iterative nearest-neighbour removal when it exceeds capacity.
//!
//! Reference: Zitzler, Laumanns & Thiele 2001 (TIK-Report 103).

pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::ObjectiveFn;
use crate::observer::Spea2Observer;
use crate::spea2::configuration::Spea2Configuration;
use crate::traits::{ChromosomeT, InitializationFn};
use std::sync::Arc;

/// SPEA2 strength-Pareto multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct Spea2Ga<U>
where
    U: ChromosomeT,
{
    /// SPEA2-specific configuration.
    pub spea2_config: Spea2Configuration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for SPEA2-specific events.
    pub observer: Option<Arc<dyn Spea2Observer<U> + Send + Sync>>,
}

#[allow(dead_code)]
impl<U> Spea2Ga<U>
where
    U: ChromosomeT,
{
    /// Creates a new `Spea2Ga` with the given configurations.
    pub fn new(spea2_config: Spea2Configuration, ga_config: GaConfiguration) -> Self {
        Spea2Ga {
            spea2_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives SPEA2-specific hooks (D-05).
    pub fn with_observer(mut self, obs: Arc<dyn Spea2Observer<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    pub(crate) fn notify<F: FnOnce(&dyn Spea2Observer<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }

    /// Sets the alleles template.
    pub fn with_alleles(mut self, alleles: Vec<U::Gene>) -> Self {
        self.alleles = alleles;
        self
    }

    /// Sets the initialization function.
    pub fn with_initialization_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(f));
        self
    }

    /// Sets the objective functions.
    pub fn with_objective_fns(mut self, fns: Vec<Box<ObjectiveFn<U::Gene>>>) -> Self {
        self.objective_fns = fns.into_iter().map(Arc::from).collect();
        self
    }

    /// Validates configuration and returns a ready-to-run instance.
    pub fn build(self) -> Result<Self, GaError> {
        self.validate()?;
        Ok(self)
    }

    /// Validates the SPEA2 configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidSpea2Configuration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.spea2_config.num_objectives == 0 {
            return Err(GaError::InvalidSpea2Configuration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.spea2_config.population_size < 2 {
            return Err(GaError::InvalidSpea2Configuration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidSpea2Configuration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.spea2_config.num_objectives {
            return Err(GaError::InvalidSpea2Configuration(format!(
                "Expected {} objective functions, got {}",
                self.spea2_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.spea2_config.objective_directions.is_empty()
            && self.spea2_config.objective_directions.len() != self.spea2_config.num_objectives
        {
            return Err(GaError::InvalidSpea2Configuration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.spea2_config.objective_directions.len(),
                self.spea2_config.num_objectives
            )));
        }
        // D-01: archive_size must be > 0 and <= population_size
        if self.spea2_config.archive_size == 0 {
            return Err(GaError::InvalidSpea2Configuration(
                "archive_size must be > 0".to_string(),
            ));
        }
        if self.spea2_config.archive_size > self.spea2_config.population_size {
            return Err(GaError::InvalidSpea2Configuration(format!(
                "archive_size ({}) must not exceed population_size ({})",
                self.spea2_config.archive_size,
                self.spea2_config.population_size
            )));
        }
        Ok(())
    }
}
