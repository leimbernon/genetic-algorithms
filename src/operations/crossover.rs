//! Crossover (recombination) operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! crossover implementations (uniform, single-point, multi-point, cycle,
//! order, PMX, SBX, BLX-alpha, arithmetic, rejuvenate). The correct
//! implementation is selected at runtime based on the [`Crossover`] variant
//! in the configuration.

pub use self::clone::clone_crossover;
pub use self::cycle::cycle;
pub use self::edge_recombination::erx;
pub use self::multipoint::multipoint;
pub use self::order::order;
pub use self::pmx::pmx;
pub use self::rejuvenate::rejuvenate;
pub use self::single_point::single_point;
pub use self::uniform_crossover::uniform;
pub use self::variable_length::variable_length_crossover;
pub(crate) use super::Crossover;
use crate::chromosomes::Range as RangeChromosome;
use crate::configuration::CrossoverConfiguration;
use crate::error::GaError;
use crate::traits::{CrossoverOperator, LinearChromosome, RealValued};
use std::any::Any;

pub mod arithmetic;
pub mod blend_alpha;
pub mod clone;
pub mod cycle;
pub mod edge_recombination;
pub mod multi_group_ox;
pub mod multi_group_pmx;
pub mod multipoint;
pub mod order;
pub mod pcx;
pub mod pmx;
pub mod rejuvenate;
pub mod sbx;
pub mod single_point;
pub mod spx;
pub mod undx;
pub mod uniform_crossover;
pub mod variable_length;

use multi_group_ox::multi_group_ox;
use multi_group_pmx::multi_group_pmx;

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
    fn crossover<U: LinearChromosome>(
        &self,
        parent_1: &U,
        parent_2: &U,
    ) -> Result<Vec<U>, GaError> {
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
            Crossover::VariableLength(strategy) => {
                variable_length_crossover(parent_1, parent_2, *strategy)
            }
            Crossover::MultiGroupPmx => multi_group_pmx(parent_1, parent_2),
            Crossover::MultiGroupOx => multi_group_ox(parent_1, parent_2),
            Crossover::Undx { .. } | Crossover::Spx { .. } | Crossover::Pcx { .. } => {
                Err(GaError::CrossoverError(
                    "Multi-parent crossover variant invoked through 2-parent factory; \
                     use factory_multi_parent"
                        .to_string(),
                ))
            }
        }
    }
}

impl CrossoverOperator for CrossoverConfiguration {
    fn crossover<U: LinearChromosome>(
        &self,
        parent_1: &U,
        parent_2: &U,
    ) -> Result<Vec<U>, GaError> {
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
            Crossover::VariableLength(strategy) => {
                variable_length_crossover(parent_1, parent_2, strategy)
            }
            Crossover::MultiGroupPmx => multi_group_pmx(parent_1, parent_2),
            Crossover::MultiGroupOx => multi_group_ox(parent_1, parent_2),
            Crossover::Undx { .. } | Crossover::Spx { .. } | Crossover::Pcx { .. } => {
                Err(GaError::CrossoverError(
                    "Multi-parent crossover variant invoked through 2-parent factory; \
                     use factory_multi_parent"
                        .to_string(),
                ))
            }
        }
    }
}

/// Attempt UNDX crossover by downcasting generic parents to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(...))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_undx<U: LinearChromosome>(
    parents: &[&U],
    configuration: CrossoverConfiguration,
) -> Option<Result<Vec<U>, GaError>> {
    macro_rules! try_type {
        ($t:ty) => {{
            let mut typed: Vec<&RangeChromosome<$t>> = Vec::with_capacity(parents.len());
            let mut all_ok = true;
            for p in parents.iter() {
                if let Some(r) = (*p as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    typed.push(r);
                } else {
                    all_ok = false;
                    break;
                }
            }
            if all_ok && !typed.is_empty() {
                let result = undx::undx(
                    &typed,
                    typed.len(),
                    configuration.undx_sigma_xi,
                    configuration.undx_sigma_eta,
                );
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
        }};
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Attempt SPX crossover by downcasting generic parents to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(...))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_spx<U: LinearChromosome>(
    parents: &[&U],
    _configuration: CrossoverConfiguration,
) -> Option<Result<Vec<U>, GaError>> {
    macro_rules! try_type {
        ($t:ty) => {{
            let mut typed: Vec<&RangeChromosome<$t>> = Vec::with_capacity(parents.len());
            let mut all_ok = true;
            for p in parents.iter() {
                if let Some(r) = (*p as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    typed.push(r);
                } else {
                    all_ok = false;
                    break;
                }
            }
            if all_ok && !typed.is_empty() {
                let result = spx::spx(&typed, typed.len());
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
        }};
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Attempt PCX crossover by downcasting generic parents to `Range<T>`.
///
/// Tries `f64`, `f32`, `i32`, `i64` in order. Returns `Some(Ok(...))` or
/// `Some(Err(...))` if the type matched, `None` if no supported type matched.
fn try_pcx<U: LinearChromosome>(
    parents: &[&U],
    configuration: CrossoverConfiguration,
) -> Option<Result<Vec<U>, GaError>> {
    macro_rules! try_type {
        ($t:ty) => {{
            let mut typed: Vec<&RangeChromosome<$t>> = Vec::with_capacity(parents.len());
            let mut all_ok = true;
            for p in parents.iter() {
                if let Some(r) = (*p as &dyn Any).downcast_ref::<RangeChromosome<$t>>() {
                    typed.push(r);
                } else {
                    all_ok = false;
                    break;
                }
            }
            if all_ok && !typed.is_empty() {
                let result = pcx::pcx(
                    &typed,
                    typed.len(),
                    configuration.pcx_sigma_eta,
                    configuration.pcx_sigma_zeta,
                );
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
        }};
    }
    try_type!(f64);
    try_type!(f32);
    try_type!(i32);
    try_type!(i64);
    None
}

/// Dispatches multi-parent crossover (`Crossover::Undx`, `Crossover::Spx`, `Crossover::Pcx`)
/// for chromosomes satisfying the `RealValued` bound.
///
/// This function is the entry point for multi-parent crossover operations. It requires
/// at least 3 parents and a configuration whose method is one of the three multi-parent
/// variants. For standard 2-parent crossover, use [`factory`] instead.
///
/// # Generic bounds
///
/// `U: LinearChromosome + RealValued` — the `RealValued` marker trait ensures at compile
/// time that only real-valued chromosomes (e.g., `RangeChromosome<T>`) are accepted.
///
/// # Parent count requirement
///
/// `parents.len() >= 3` is enforced at runtime; fewer parents return
/// `GaError::CrossoverError("Multi-parent crossover requires at least 3 parents")`.
///
/// # See also
///
/// `ga.rs` (Plan 04) — integration point that calls `factory_multi_parent` when
/// `configuration.crossover.method` is `Undx`, `Spx`, or `Pcx`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Range;
/// use genetic_algorithms::configuration::CrossoverConfiguration;
/// use genetic_algorithms::operations::{crossover::factory_multi_parent, Crossover};
///
/// let p1 = Range::<f64>::new();
/// let p2 = Range::<f64>::new();
/// let p3 = Range::<f64>::new();
/// let config = CrossoverConfiguration { method: Crossover::Spx { num_parents: 3 }, ..Default::default() };
/// let _offspring = factory_multi_parent(&[&p1, &p2, &p3], config);
/// ```
pub fn factory_multi_parent<U: LinearChromosome + RealValued>(
    parents: &[&U],
    configuration: CrossoverConfiguration,
) -> Result<Vec<U>, GaError> {
    if parents.len() < 3 {
        return Err(GaError::CrossoverError(
            "Multi-parent crossover requires at least 3 parents".to_string(),
        ));
    }
    match configuration.method {
        Crossover::Undx { .. } => try_undx(parents, configuration).ok_or_else(|| {
            GaError::CrossoverError(
                "UNDX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".to_string(),
            )
        })?,
        Crossover::Spx { .. } => try_spx(parents, configuration).ok_or_else(|| {
            GaError::CrossoverError(
                "SPX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".to_string(),
            )
        })?,
        Crossover::Pcx { .. } => try_pcx(parents, configuration).ok_or_else(|| {
            GaError::CrossoverError(
                "PCX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".to_string(),
            )
        })?,
        _ => Err(GaError::CrossoverError(
            "factory_multi_parent called with non-multi-parent crossover method".to_string(),
        )),
    }
}

/// Dispatches multi-parent crossover without requiring the `RealValued` bound.
///
/// This is the `ga.rs` integration entry point. It uses the same downcast dispatchers
/// as [`factory_multi_parent`] but accepts any `U: LinearChromosome + 'static`. If the
/// chromosome does not downcast to a supported real-valued type, a `CrossoverError` is
/// returned at runtime.
///
/// `parents` must contain at least 3 references and `configuration.method` must be
/// `Crossover::Undx`, `Crossover::Spx`, or `Crossover::Pcx`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Range;
/// use genetic_algorithms::configuration::CrossoverConfiguration;
/// use genetic_algorithms::operations::{crossover::factory_multi_parent_dispatch, Crossover};
///
/// let p1 = Range::<f64>::new();
/// let p2 = Range::<f64>::new();
/// let p3 = Range::<f64>::new();
/// let config = CrossoverConfiguration { method: Crossover::Undx { num_parents: 3 }, ..Default::default() };
/// let _result = factory_multi_parent_dispatch(&[&p1, &p2, &p3], config);
/// ```
pub fn factory_multi_parent_dispatch<U: LinearChromosome + 'static>(
    parents: &[&U],
    configuration: CrossoverConfiguration,
) -> Result<Vec<U>, GaError> {
    if parents.len() < 3 {
        return Err(GaError::CrossoverError(
            "Multi-parent crossover requires at least 3 parents".to_string(),
        ));
    }
    match configuration.method {
        Crossover::Undx { .. } => try_undx(parents, configuration).ok_or_else(|| {
            GaError::CrossoverError(
                "UNDX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".to_string(),
            )
        })?,
        Crossover::Spx { .. } => try_spx(parents, configuration).ok_or_else(|| {
            GaError::CrossoverError(
                "SPX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".to_string(),
            )
        })?,
        Crossover::Pcx { .. } => try_pcx(parents, configuration).ok_or_else(|| {
            GaError::CrossoverError(
                "PCX requires Range<T> chromosomes where T is f64, f32, i32, or i64.".to_string(),
            )
        })?,
        _ => Err(GaError::CrossoverError(
            "factory_multi_parent_dispatch called with non-multi-parent crossover method"
                .to_string(),
        )),
    }
}

/// Dispatches crossover according to the configured method and parameters.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::CrossoverConfiguration;
/// use genetic_algorithms::operations::crossover::factory;
///
/// let parent_1 = Binary::new();
/// let parent_2 = Binary::new();
/// let config = CrossoverConfiguration::default();
/// let _offspring = factory(&parent_1, &parent_2, config).unwrap();
/// ```
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
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::ChromosomeT;
/// use genetic_algorithms::operations::crossover::aga_probability;
///
/// let mut p1 = Binary::new();
/// p1.set_fitness(0.9);
/// let mut p2 = Binary::new();
/// p2.set_fitness(0.6);
/// let prob = aga_probability(&p1, &p2, 1.0, 0.7, 0.9, 0.1);
/// assert!(prob >= 0.0 && prob <= 0.9);
/// ```
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
