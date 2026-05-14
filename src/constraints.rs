//! Constraints — penalty-based constraint handling for single-objective GA.
//!
//! This module defines the strategy types and helper functions used by
//! [`Ga`](crate::ga::Ga) to handle constrained optimization problems.
//! Three penalty strategies (static, dynamic, adaptive) modify fitness based
//! on constraint violation severity. Deb's feasibility rules compare solutions
//! by feasibility first, then fitness. The [`RepairOperator`] trait fixes
//! infeasible chromosomes.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`ConstraintHandling`] | Master configuration struct for constraint enforcement |
//! | [`PenaltyStrategy`] | Enum: Static, Dynamic, or Adaptive penalty scaling |
//! | [`RepairOperator`] | Trait for repairing infeasible chromosomes |
//!
//! # When to use
//! Enable constraint handling when your optimization problem has constraints
//! that cannot be handled by bounding genes alone (e.g., inequality constraints,
//! non-linear constraints). Pass a [`ConstraintHandling`] instance to the
//! engine builder via `with_constraint_handling()`.

use crate::error::GaError;

/// Strategy for applying penalty to infeasible solutions.
///
/// The penalty is computed per-generation based on constraint violations
/// and then added to the raw fitness value. All variants work with
/// minimization, maximization, and fixed-fitness problems.
#[derive(Clone, Debug, PartialEq)]
pub enum PenaltyStrategy {
    /// No penalty applied (default).
    None,
    /// Fixed penalty coefficient: `fitness_penalized = fitness + R * sum(violations)`
    Static {
        /// Fixed penalty coefficient R.
        coefficient: f64,
    },
    /// Joines & Houck (1994) dynamic penalty:
    /// `fitness_penalized = fitness + (C * gen)^alpha * sum(violations^beta)`
    Dynamic {
        /// Scale factor C.
        c: f64,
        /// Exponent for generation number.
        alpha: f64,
        /// Exponent for violation magnitude.
        beta: f64,
    },
    /// Bean & Hadj-Alouane (1997) adaptive penalty.
    /// The penalty coefficient is adjusted every `window_size` generations:
    /// increased if the best individual has been feasible, decreased if infeasible.
    Adaptive {
        /// Initial penalty coefficient.
        initial_coefficient: f64,
        /// Number of generations to observe before adjusting.
        window_size: usize,
    },
}

impl Default for PenaltyStrategy {
    fn default() -> Self {
        PenaltyStrategy::None
    }
}

/// Constraint handling method for comparisons in selection, survivor, and elite operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConstraintHandling {
    /// Deb's feasibility rules (Deb, 2000):
    /// 1. Between two feasible individuals: the one with better fitness wins.
    /// 2. Feasible always beats infeasible.
    /// 3. Between two infeasible individuals: the one with lower total violation wins.
    FeasibilityRules,
}

/// Computes the total constraint violation from the per-constraint violations.
///
/// Each constraint violation should be >= 0 (0 means the constraint is satisfied).
/// Returns the sum of all violations.
pub fn total_violation(violations: &[f64]) -> f64 {
    violations.iter().sum()
}

/// Applies a static penalty to the raw fitness value.
///
/// `fitness_penalized = fitness + R * total_violation`
pub fn apply_static_penalty(fitness: f64, total_violation: f64, coefficient: f64) -> f64 {
    fitness + coefficient * total_violation
}

/// Applies the Joines & Houck (1994) dynamic penalty.
///
/// `fitness_penalized = fitness + (C * generation)^alpha * total_violation^beta`
pub fn apply_dynamic_penalty(
    fitness: f64,
    total_violation: f64,
    generation: usize,
    c: f64,
    alpha: f64,
    beta: f64,
) -> f64 {
    let gen_factor = (c * generation as f64).powf(alpha);
    let violation_factor = total_violation.powf(beta);
    fitness + gen_factor * violation_factor
}

/// Validates a penalty strategy configuration.
///
/// Returns `Ok(())` if valid, or `Err(GaError::InvalidConstraintConfiguration)` with
/// a descriptive message.
pub fn validate_penalty_strategy(strategy: &PenaltyStrategy) -> Result<(), GaError> {
    match strategy {
        PenaltyStrategy::None => Ok(()),
        PenaltyStrategy::Static { coefficient } => {
            if *coefficient < 0.0 {
                return Err(GaError::InvalidConstraintConfiguration(
                    format!("Static penalty coefficient must be non-negative, got {}", coefficient),
                ));
            }
            Ok(())
        }
        PenaltyStrategy::Dynamic { c, alpha, beta } => {
            if *c < 0.0 {
                return Err(GaError::InvalidConstraintConfiguration(
                    format!("Dynamic penalty C must be non-negative, got {}", c),
                ));
            }
            if *alpha < 0.0 {
                return Err(GaError::InvalidConstraintConfiguration(
                    format!("Dynamic penalty alpha must be non-negative, got {}", alpha),
                ));
            }
            if *beta < 0.0 {
                return Err(GaError::InvalidConstraintConfiguration(
                    format!("Dynamic penalty beta must be non-negative, got {}", beta),
                ));
            }
            Ok(())
        }
        PenaltyStrategy::Adaptive {
            initial_coefficient,
            window_size,
        } => {
            if *initial_coefficient < 0.0 {
                return Err(GaError::InvalidConstraintConfiguration(
                    format!(
                        "Adaptive penalty initial_coefficient must be non-negative, got {}",
                        initial_coefficient
                    ),
                ));
            }
            if *window_size == 0 {
                return Err(GaError::InvalidConstraintConfiguration(
                    "Adaptive penalty window_size must be > 0".to_string(),
                ));
            }
            Ok(())
        }
    }
}
