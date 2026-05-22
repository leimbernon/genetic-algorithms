//! Crossover (recombination) operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! crossover implementations (uniform, single-point, multi-point, cycle,
//! order, PMX, SBX, BLX-alpha, arithmetic, rejuvenate). The correct
//! implementation is selected at runtime based on the [`Crossover`] variant
//! in the configuration.

pub use self::clone::clone_crossover;
pub use self::cycle::cycle;
pub use self::multipoint::multipoint;
pub use self::order::order;
pub use self::edge_recombination::erx;
pub use self::pmx::pmx;
pub use self::rejuvenate::rejuvenate;
pub use self::single_point::single_point;
pub use self::uniform_crossover::uniform;
pub(crate) use super::Crossover;
use crate::chromosomes::Range as RangeChromosome;
use crate::configuration::CrossoverConfiguration;
use crate::error::GaError;
use crate::traits::{LinearChromosome, CrossoverOperator};
use std::any::Any;

pub mod arithmetic;
pub mod blend_alpha;
pub mod clone;
pub mod cycle;
pub mod multipoint;
pub mod order;
pub mod edge_recombination;
pub mod pmx;
pub mod rejuvenate;
pub mod sbx;
pub mod single_point;
pub mod uniform_crossover;

/// Attempt SBX crossover by downcasting generic parents to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(...))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_sbx<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    eta: f64,
) -> Option<Result<Vec<U>, GaError>> {
    // Helper macro: attempt downcast to Range<$t>, call sbx, cast children back to U.
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(p1) = (parent_1 as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                if let Some(p2) = (parent_2 as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    let result = sbx::sbx(p1, p2, eta);
                    return Some(result.map(|children| {
                        children
                            .into_iter()
                            .map(|c| {
                                // SAFETY: we confirmed U == RangeChromosome<$t> via downcast
                                let boxed: Box<dyn Any> = Box::new(c);
                                *boxed
                                    .downcast::<U>()
                                    .expect("type confirmed by downcast_ref")
                            })
                            .collect()
                    }));
                }
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Attempt BLX-α crossover by downcasting generic parents to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(...))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_blend_alpha<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    alpha: f64,
) -> Option<Result<Vec<U>, GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(p1) = (parent_1 as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                if let Some(p2) = (parent_2 as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    let result = blend_alpha::blend_alpha(p1, p2, alpha);
                    return Some(result.map(|children| {
                        children
                            .into_iter()
                            .map(|c| {
                                let boxed: Box<dyn Any> = Box::new(c);
                                *boxed
                                    .downcast::<U>()
                                    .expect("type confirmed by downcast_ref")
                            })
                            .collect()
                    }));
                }
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Default SBX distribution index when none is configured.
const DEFAULT_SBX_ETA: f64 = 2.0;
/// Default BLX-α alpha parameter when none is configured.
const DEFAULT_BLEND_ALPHA: f64 = 0.5;
/// Default arithmetic crossover alpha when none is configured.
const DEFAULT_ARITHMETIC_ALPHA: f64 = 0.5;

/// Attempt Arithmetic crossover by downcasting generic parents to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(...))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_arithmetic<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    alpha: f64,
) -> Option<Result<Vec<U>, GaError>> {
    macro_rules! try_type {
        ($t:ty) => {
            if let Some(p1) = (parent_1 as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                if let Some(p2) = (parent_2 as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    let result = arithmetic::arithmetic(p1, p2, alpha);
                    return Some(result.map(|children| {
                        children
                            .into_iter()
                            .map(|c| {
                                let boxed: Box<dyn Any> = Box::new(c);
                                *boxed
                                    .downcast::<U>()
                                    .expect("type confirmed by downcast_ref")
                            })
                            .collect()
                    }));
                }
            }
        };
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

impl CrossoverOperator for Crossover {
    fn crossover<U: LinearChromosome>(&self, parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
        match self {
            Crossover::Cycle => cycle(parent_1, parent_2),
            Crossover::MultiPoint => Err(GaError::CrossoverError(
                "MultiPoint crossover requires number_of_points. \
                 Use CrossoverConfiguration as the operator or call multipoint() directly."
                    .to_string(),
            )),
            Crossover::Uniform => uniform(parent_1, parent_2),
            Crossover::SinglePoint => single_point(parent_1, parent_2),
            Crossover::Order => order(parent_1, parent_2),
            Crossover::Pmx => pmx(parent_1, parent_2),
            Crossover::Sbx => try_sbx(parent_1, parent_2, DEFAULT_SBX_ETA).unwrap_or_else(|| {
                Err(GaError::CrossoverError(
                    "SBX crossover requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                        .to_string(),
                ))
            }),
            Crossover::BlendAlpha => {
                try_blend_alpha(parent_1, parent_2, DEFAULT_BLEND_ALPHA).unwrap_or_else(|| {
                    Err(GaError::CrossoverError(
                        "BLX-α crossover requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                })
            }
            Crossover::Arithmetic => {
                try_arithmetic(parent_1, parent_2, DEFAULT_ARITHMETIC_ALPHA).unwrap_or_else(|| {
                    Err(GaError::CrossoverError(
                        "Arithmetic crossover requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                })
            }
            Crossover::Clone => clone_crossover(parent_1, parent_2),
            Crossover::Rejuvenate => rejuvenate(parent_1, parent_2),
            Crossover::EdgeRecombination => erx(parent_1, parent_2),
            Crossover::MultiGroupPmx | Crossover::MultiGroupOx => {
                Err(GaError::CrossoverError(
                    "MultiGroupPmx and MultiGroupOx dispatch is not yet implemented. \
                     Use MultiUniqueChromosome with these operators (48-04)."
                        .to_string(),
                ))
            }
        }
    }
}

impl CrossoverOperator for CrossoverConfiguration {
    fn crossover<U: LinearChromosome>(&self, parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
        match self.method {
            Crossover::Cycle => cycle(parent_1, parent_2),
            Crossover::MultiPoint => {
                let points = self.number_of_points.ok_or_else(|| {
                    GaError::ConfigurationError(
                        "MultiPoint crossover requires number_of_points to be set".to_string(),
                    )
                })?;
                multipoint(parent_1, parent_2, points)
            }
            Crossover::Uniform => uniform(parent_1, parent_2),
            Crossover::SinglePoint => single_point(parent_1, parent_2),
            Crossover::Order => order(parent_1, parent_2),
            Crossover::Pmx => pmx(parent_1, parent_2),
            Crossover::Sbx => {
                let eta = self.sbx_eta.unwrap_or(DEFAULT_SBX_ETA);
                try_sbx(parent_1, parent_2, eta).unwrap_or_else(|| {
                    Err(GaError::CrossoverError(
                        "SBX crossover requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                })
            }
            Crossover::BlendAlpha => {
                let alpha = self.blend_alpha.unwrap_or(DEFAULT_BLEND_ALPHA);
                try_blend_alpha(parent_1, parent_2, alpha).unwrap_or_else(|| {
                    Err(GaError::CrossoverError(
                        "BLX-α crossover requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                })
            }
            Crossover::Arithmetic => {
                let alpha = self.arithmetic_alpha.unwrap_or(DEFAULT_ARITHMETIC_ALPHA);
                try_arithmetic(parent_1, parent_2, alpha).unwrap_or_else(|| {
                    Err(GaError::CrossoverError(
                        "Arithmetic crossover requires Range<T> chromosomes where T is f64, f32, i32, or i64."
                            .to_string(),
                    ))
                })
            }
            Crossover::Clone => clone_crossover(parent_1, parent_2),
            Crossover::Rejuvenate => rejuvenate(parent_1, parent_2),
            Crossover::EdgeRecombination => erx(parent_1, parent_2),
            Crossover::MultiGroupPmx | Crossover::MultiGroupOx => {
                Err(GaError::CrossoverError(
                    "MultiGroupPmx and MultiGroupOx dispatch is not yet implemented. \
                     Use MultiUniqueChromosome with these operators (48-04)."
                        .to_string(),
                ))
            }
        }
    }
}

/// Dispatches crossover according to the configured method and parameters.
pub fn factory<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    configuration: CrossoverConfiguration,
) -> Result<Vec<U>, GaError> {
    configuration.crossover(parent_1, parent_2)
}

/// Calculates the crossover probability for adaptive genetic algorithms (AGA).
///
/// Returns a probability between `probability_min` and `probability_max`
/// based on how the fitter parent compares to the population average.
pub fn aga_probability<U: LinearChromosome>(
    parent_1: &U,
    parent_2: &U,
    f_max: f64,
    f_avg: f64,
    probability_max: f64,
    probability_min: f64,
) -> f64 {
    let larger_f = if parent_1.fitness() > parent_2.fitness() {
        parent_1.fitness()
    } else {
        parent_2.fitness()
    };

    if larger_f >= f_avg {
        if (f_max - f_avg).abs() < f64::EPSILON {
            probability_max
        } else {
            probability_max * ((f_max - larger_f) / (f_max - f_avg))
        }
    } else {
        probability_min
    }
}
