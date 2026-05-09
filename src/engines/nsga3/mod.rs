//! NSGA-III many-objective genetic algorithm.
//!
//! NSGA-III extends NSGA-II for problems with three or more objectives by
//! replacing crowding-distance survivor selection with reference-point niche
//! association on the unit hyperplane. Reference points are either
//! auto-generated via the Das-Dennis simplex lattice
//! ([`Nsga3Configuration::with_reference_points_auto`](configuration::Nsga3Configuration::with_reference_points_auto))
//! or user-supplied
//! ([`Nsga3Configuration::with_reference_points`](configuration::Nsga3Configuration::with_reference_points)).
//!
//! Reference: Deb & Jain 2014 (IEEE-TEC 18(4):577-601).

pub mod configuration;
pub mod das_dennis;

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::multi_objective::pareto::ParetoFront;
use crate::multi_objective::ObjectiveFn;
use crate::nsga3::configuration::Nsga3Configuration;
use crate::observer::Nsga3Observer;
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
use std::sync::Arc;

/// NSGA-III many-objective genetic algorithm orchestrator.
///
/// # Type Parameters
///
/// * `U` - Chromosome type implementing `ChromosomeT`.
pub struct Nsga3Ga<U>
where
    U: ChromosomeT,
{
    /// NSGA-III specific configuration.
    pub nsga3_config: Nsga3Configuration,
    /// Base GA configuration (operators, limits).
    pub ga_config: GaConfiguration,
    /// Alleles template for initialization.
    pub alleles: Vec<U::Gene>,
    /// Initialization function.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Objective functions (one per objective).
    pub objective_fns: Vec<Arc<ObjectiveFn<U::Gene>>>,
    /// Optional structured lifecycle observer for NSGA-III-specific events.
    pub observer: Option<Arc<dyn Nsga3Observer<U> + Send + Sync>>,
}

impl<U> Nsga3Ga<U>
where
    U: ChromosomeT,
{
    /// Creates a new `Nsga3Ga` with the given configurations.
    pub fn new(nsga3_config: Nsga3Configuration, ga_config: GaConfiguration) -> Self {
        Nsga3Ga {
            nsga3_config,
            ga_config,
            alleles: Vec::new(),
            initialization_fn: None,
            objective_fns: Vec::new(),
            observer: None,
        }
    }

    /// Attaches a structured lifecycle observer that receives NSGA-III-specific hooks.
    pub fn with_observer(mut self, obs: Arc<dyn Nsga3Observer<U> + Send + Sync>) -> Self {
        self.observer = Some(obs);
        self
    }

    /// Dispatches an observer hook if an observer is attached. No-op when `self.observer` is `None`.
    #[inline]
    #[allow(dead_code)] // Used by Plan 03 run() loop; kept here so the helper lives with the struct.
    fn notify<F: FnOnce(&dyn Nsga3Observer<U>)>(&self, f: F) {
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

    /// Validates the NSGA-III configuration.
    ///
    /// # Errors
    ///
    /// Returns `GaError::InvalidNsga3Configuration` if parameters are invalid.
    pub fn validate(&self) -> Result<(), GaError> {
        if self.nsga3_config.num_objectives == 0 {
            return Err(GaError::InvalidNsga3Configuration(
                "num_objectives must be > 0".to_string(),
            ));
        }
        if self.nsga3_config.population_size < 2 {
            return Err(GaError::InvalidNsga3Configuration(
                "population_size must be >= 2".to_string(),
            ));
        }
        if self.initialization_fn.is_none() {
            return Err(GaError::InvalidNsga3Configuration(
                "initialization_fn is required".to_string(),
            ));
        }
        if self.objective_fns.len() != self.nsga3_config.num_objectives {
            return Err(GaError::InvalidNsga3Configuration(format!(
                "Expected {} objective functions, got {}",
                self.nsga3_config.num_objectives,
                self.objective_fns.len()
            )));
        }
        if !self.nsga3_config.objective_directions.is_empty()
            && self.nsga3_config.objective_directions.len() != self.nsga3_config.num_objectives
        {
            return Err(GaError::InvalidNsga3Configuration(format!(
                "objective_directions length ({}) must match num_objectives ({})",
                self.nsga3_config.objective_directions.len(),
                self.nsga3_config.num_objectives
            )));
        }
        // Reference points must be configured (auto or custom).
        let ref_points = self.nsga3_config.effective_reference_points();
        match ref_points {
            None => {
                return Err(GaError::InvalidNsga3Configuration(
                    "reference points must be configured via with_reference_points_auto(p) or with_reference_points(points)".to_string(),
                ));
            }
            Some(points) => {
                for (i, pt) in points.iter().enumerate() {
                    if pt.len() != self.nsga3_config.num_objectives {
                        return Err(GaError::InvalidNsga3Configuration(format!(
                            "reference point {} has dimension {}, expected {}",
                            i,
                            pt.len(),
                            self.nsga3_config.num_objectives
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

impl<U> Nsga3Ga<U>
where
    U: ChromosomeT + mutation::ValueMutable,
{
    /// Runs the NSGA-III algorithm and returns the first Pareto front.
    ///
    /// # Note
    ///
    /// This is a placeholder implementation that returns an error. The full
    /// generation loop and reference-point environmental selection are
    /// implemented in Plan 35-03.
    pub fn run(&mut self) -> Result<ParetoFront<U>, GaError> {
        self.validate()?;
        Err(GaError::InvalidNsga3Configuration(
            "Nsga3Ga::run() not yet implemented — Plan 35-03 will replace this stub".to_string(),
        ))
    }
}
