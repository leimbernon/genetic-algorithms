//! MOEA/D — Decomposition-based multi-objective genetic algorithm.
//!
//! MOEA/D (Zhang & Li 2007) decomposes a multi-objective problem into N scalar
//! sub-problems using weight vectors; each sub-problem maintains a
//! neighbourhood of similar weight vectors, and offspring compete only within
//! that neighbourhood via Tchebycheff or PBI scalarization.
//!
//! Weight vectors are either auto-generated via the Das-Dennis simplex lattice
//! ([`MoeaDConfiguration::with_weight_vectors_auto`](configuration::MoeaDConfiguration::with_weight_vectors_auto))
//! or user-supplied
//! ([`MoeaDConfiguration::with_weight_vectors`](configuration::MoeaDConfiguration::with_weight_vectors)).
//!
//! Reference: Zhang & Li 2007 (IEEE-TEC 11(6):712-731).

pub mod configuration;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::moead::configuration::MoeaDConfiguration;
use crate::multi_objective::ObjectiveFn;
use crate::observer::MoeaDObserver;
use crate::traits::{ChromosomeT, InitializationFn};
use std::sync::Arc;

/// MOEA/D decomposition-based multi-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct MoeaDGa<U>
where
    U: ChromosomeT,
{
    /// MOEA/D specific configuration.
    pub moead_config: MoeaDConfiguration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for MOEA/D-specific events.
    pub observer: Option<Arc<dyn MoeaDObserver<U> + Send + Sync>>,
}

impl<U> MoeaDGa<U>
where
    U: ChromosomeT,
{
    /// Creates a new `MoeaDGa` with the given configurations.
    pub fn new(moead_config: MoeaDConfiguration, ga_config: GaConfiguration) -> Self {
        MoeaDGa {
            moead_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives MOEA/D-specific hooks.
    pub fn with_observer(mut self, obs: Arc<dyn MoeaDObserver<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    pub(crate) fn notify<F: FnOnce(&dyn MoeaDObserver<U>)>(&self, f: F) {
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

    /// Validates the MOEA/D configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidMoeaDConfiguration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.moead_config.num_objectives == 0 {
            return Err(GaError::InvalidMoeaDConfiguration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.moead_config.population_size < 2 {
            return Err(GaError::InvalidMoeaDConfiguration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidMoeaDConfiguration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.moead_config.num_objectives {
            return Err(GaError::InvalidMoeaDConfiguration(format!(
                "Expected {} objective functions, got {}",
                self.moead_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.moead_config.objective_directions.is_empty()
            && self.moead_config.objective_directions.len() != self.moead_config.num_objectives
        {
            return Err(GaError::InvalidMoeaDConfiguration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.moead_config.objective_directions.len(),
                self.moead_config.num_objectives
            )));
        }
        // Das-Dennis subdivision count must be >= 1 to avoid a degenerate all-zero point.
        if let Some(p) = self.moead_config.weight_vectors_auto_p() {
            if p == 0 {
                return Err(GaError::InvalidMoeaDConfiguration(
                    "Das-Dennis subdivision count p must be >= 1".to_string(),
                ));
            }
        }
        // Weight vectors must be configured (auto or custom) per D-06.
        let wvs = self.moead_config.effective_weight_vectors();
        match wvs {
            None => {
                return Err(GaError::InvalidMoeaDConfiguration(
                    "weight vectors must be configured via with_weight_vectors_auto(p) or with_weight_vectors(vecs)".to_string(),
                ));
            }
            Some(vecs) => {
                if vecs.is_empty() {
                    return Err(GaError::InvalidMoeaDConfiguration(
                        "weight vector list must not be empty".to_string(),
                    ));
                }
                for (i, wv) in vecs.iter().enumerate() {
                    if wv.len() != self.moead_config.num_objectives {
                        return Err(GaError::InvalidMoeaDConfiguration(format!(
                            "weight vector {} has dimension {}, expected {}",
                            i,
                            wv.len(),
                            self.moead_config.num_objectives
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validates configuration and returns the materialised weight vectors.
    ///
    /// Combines `validate()` and `effective_weight_vectors()` into a single
    /// call so `run()` does not invoke the Das-Dennis generator twice.
    pub(crate) fn validate_and_get_weight_vectors(&self) -> Result<Vec<Vec<f64>>, GaError> {
        if self.moead_config.num_objectives == 0 {
            return Err(GaError::InvalidMoeaDConfiguration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.moead_config.population_size < 2 {
            return Err(GaError::InvalidMoeaDConfiguration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidMoeaDConfiguration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.moead_config.num_objectives {
            return Err(GaError::InvalidMoeaDConfiguration(format!(
                "Expected {} objective functions, got {}",
                self.moead_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.moead_config.objective_directions.is_empty()
            && self.moead_config.objective_directions.len() != self.moead_config.num_objectives
        {
            return Err(GaError::InvalidMoeaDConfiguration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.moead_config.objective_directions.len(),
                self.moead_config.num_objectives
            )));
        }
        if let Some(p) = self.moead_config.weight_vectors_auto_p() {
            if p == 0 {
                return Err(GaError::InvalidMoeaDConfiguration(
                    "Das-Dennis subdivision count p must be >= 1".to_string(),
                ));
            }
        }
        let vecs = self
            .moead_config
            .effective_weight_vectors()
            .ok_or_else(|| {
                GaError::InvalidMoeaDConfiguration(
                    "weight vectors must be configured via with_weight_vectors_auto(p) or with_weight_vectors(vecs)".to_string(),
                )
            })?;
        if vecs.is_empty() {
            return Err(GaError::InvalidMoeaDConfiguration(
                "weight vector list must not be empty".to_string(),
            ));
        }
        for (i, wv) in vecs.iter().enumerate() {
            if wv.len() != self.moead_config.num_objectives {
                return Err(GaError::InvalidMoeaDConfiguration(format!(
                    "weight vector {} has dimension {}, expected {}",
                    i,
                    wv.len(),
                    self.moead_config.num_objectives
                )));
            }
        }
        Ok(vecs)
    }
}
