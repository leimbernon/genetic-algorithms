//! Parent-selection operators.
//!
//! This module provides the [`factory`] dispatch function and individual
//! selection implementations (tournament, roulette wheel, stochastic
//! universal sampling, random, rank, Boltzmann, truncation). The correct
//! implementation is selected at runtime based on the [`Selection`] variant
//! in the configuration.

use crate::configuration::SelectionConfiguration;
use crate::error::GaError;
use crate::traits::{ChromosomeT, SelectionOperator, VectorFitness};

pub use self::boltzmann::boltzmann_selection;
pub use self::clearing::clearing_selection;
pub use self::fitness_proportionate::roulette_wheel_selection;
pub use self::fitness_proportionate::stochastic_universal_sampling;
pub use self::lexicase::{epsilon_lexicase_selection, lexicase_selection};
pub use self::random::random;
pub use self::rank::rank_selection;
pub use self::tournament::tournament;
pub use self::truncation::truncation_selection;

use super::Selection;

pub mod boltzmann;
pub mod clearing;
pub mod fitness_proportionate;
pub mod lexicase;
pub mod random;
pub mod rank;
pub mod tournament;
pub mod truncation;

impl SelectionOperator for Selection {
    fn select<U>(
        &self,
        chromosomes: &[U],
        number_of_couples: usize,
        number_of_threads: usize,
        num_parents: usize,
    ) -> Result<Vec<Vec<usize>>, GaError>
    where
        U: ChromosomeT + Sync + Send + 'static + Clone,
    {
        match self {
            Selection::Random => Ok(random(chromosomes, num_parents)),
            Selection::RouletteWheel => Ok(roulette_wheel_selection(
                chromosomes,
                number_of_couples,
                num_parents,
            )),
            Selection::StochasticUniversalSampling => Ok(stochastic_universal_sampling(
                chromosomes,
                number_of_couples,
                num_parents,
            )),
            Selection::Tournament => Ok(tournament(
                chromosomes,
                number_of_couples,
                number_of_threads,
                num_parents,
            )),
            Selection::Rank => Ok(rank_selection(chromosomes, number_of_couples, num_parents)),
            Selection::Boltzmann => Ok(boltzmann_selection(
                chromosomes,
                number_of_couples,
                1.0,
                num_parents,
            )),
            Selection::Truncation => Ok(truncation_selection(
                chromosomes,
                number_of_couples,
                num_parents,
            )),
            // WARNING: The `SelectionOperator` trait does not carry operator-specific
            // configuration, so `niche_radius` defaults to 0.1 on this path.
            // Island-model and NSGA-II callers that use `Selection::Clearing` with
            // a custom `niche_radius` must go through `selection::factory` instead.
            // The single-population GA always uses the factory path and is unaffected.
            Selection::Clearing => {
                crate::log_warn!(target: "selection_events",
                    "Selection::Clearing called through SelectionOperator trait: \
                     niche_radius defaults to 0.1 (configured value ignored). \
                     Use selection::factory for the full configuration.");
                Ok(clearing_selection(
                    chromosomes,
                    0.1,
                    number_of_couples,
                    num_parents,
                ))
            }
            Selection::Lexicase | Selection::EpsilonLexicase => Err(GaError::SelectionError(
                "Selection::Lexicase/EpsilonLexicase cannot be called through the \
                     SelectionOperator trait: use selection::factory_lexicase. \
                     Island-model and NSGA-II paths do not support VectorFitness."
                    .to_string(),
            )),
        }
    }
}

/// Dispatches parent selection according to the configured method.
///
/// # Arguments
///
/// * `chromosomes` - Population to select from.
/// * `configuration` - Selection configuration (method, number_of_couples, etc.).
/// * `number_of_threads` - Parallelism hint passed to the operator.
/// * `num_parents` - Number of parents per group. Pass `2` for standard crossover;
///   pass the operator's `num_parents` for multi-parent operators (UNDX/SPX/PCX).
///
/// # Returns
///
/// `Ok(Vec<Vec<usize>>)` with parent groups (each inner Vec has `num_parents` indices),
/// or `Err(GaError::SelectionError)` if the population is too small to form groups.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::SelectionConfiguration;
/// use genetic_algorithms::operations::selection::factory;
/// use genetic_algorithms::traits::ChromosomeT;
///
/// let mut c1 = Binary::new(); c1.set_fitness(1.0);
/// let mut c2 = Binary::new(); c2.set_fitness(2.0);
/// let config = SelectionConfiguration { number_of_couples: 2, ..Default::default() };
/// let groups = factory(&[c1, c2], config, 1, 2).unwrap();
/// assert_eq!(groups.len(), 2);
/// ```
pub fn factory<U>(
    chromosomes: &[U],
    configuration: SelectionConfiguration,
    number_of_threads: usize,
    num_parents: usize,
) -> Result<Vec<Vec<usize>>, GaError>
where
    U: ChromosomeT + Sync + Send + 'static + Clone,
{
    if chromosomes.len() < 2 {
        return Err(GaError::SelectionError(format!(
            "Population size {} is too small for selection (minimum 2)",
            chromosomes.len()
        )));
    }

    // Guard: reject NaN fitness values which corrupt selection logic
    for (i, chromosome) in chromosomes.iter().enumerate() {
        if chromosome.fitness().is_nan() {
            return Err(GaError::SelectionError(format!(
                "Chromosome at index {} has NaN fitness. All chromosomes must have valid fitness before selection.",
                i
            )));
        }
    }

    let groups = match configuration.method {
        Selection::Boltzmann => boltzmann_selection(
            chromosomes,
            configuration.number_of_couples,
            configuration.boltzmann_temperature,
            num_parents,
        ),
        Selection::Clearing => clearing_selection(
            chromosomes,
            configuration.niche_radius,
            configuration.number_of_couples,
            num_parents,
        ),
        Selection::Lexicase | Selection::EpsilonLexicase => {
            return Err(GaError::ConfigurationError(
                "Use selection::factory_lexicase for Lexicase/EpsilonLexicase; \
                 standard factory() does not support VectorFitness bound."
                    .into(),
            ));
        }
        _ => configuration.method.select(
            chromosomes,
            configuration.number_of_couples,
            number_of_threads,
            num_parents,
        )?,
    };

    Ok(groups)
}

/// Dispatches parent selection for [`Selection::Lexicase`] and [`Selection::EpsilonLexicase`].
///
/// Unlike [`factory`], this function requires chromosomes to implement [`VectorFitness`].
/// It also syncs each chromosome's scalar fitness to the mean of its fitness values after selection
/// (D-04: lexicase mean-fitness sync).
///
/// Lexicase always produces groups of 2 (standard 2-parent crossover); `num_parents` is
/// accepted for API consistency but clamped to 2 for this operator.
///
/// # Errors
///
/// - `GaError::SelectionError` if the population has fewer than 2 individuals.
/// - `GaError::SelectionError` if `fitness_values()` is empty on the first chromosome.
/// - `GaError::SelectionError` if any chromosome has a NaN fitness value.
/// - `GaError::ConfigurationError` if called with a non-lexicase selection method.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::SelectionConfiguration;
/// use genetic_algorithms::operations::{selection::factory_lexicase, Selection};
/// use genetic_algorithms::traits::{ChromosomeT, VectorFitness};
///
/// let mut c1 = Binary::new();
/// c1.set_fitness_values(vec![1.0, 0.5]);
/// let mut c2 = Binary::new();
/// c2.set_fitness_values(vec![0.8, 0.9]);
/// let config = SelectionConfiguration {
///     method: Selection::Lexicase,
///     number_of_couples: 2,
///     ..Default::default()
/// };
/// let groups = factory_lexicase(&mut [c1, c2], config, 1).unwrap();
/// assert_eq!(groups.len(), 2);
/// ```
pub fn factory_lexicase<U>(
    chromosomes: &mut [U],
    configuration: SelectionConfiguration,
    _number_of_threads: usize,
) -> Result<Vec<Vec<usize>>, GaError>
where
    U: ChromosomeT + VectorFitness + Sync + Send + 'static + Clone,
{
    if chromosomes.len() < 2 {
        return Err(GaError::SelectionError(
            "Population must have at least 2 chromosomes".into(),
        ));
    }
    if chromosomes[0].fitness_values().is_empty() {
        return Err(GaError::SelectionError(
            "fitness_values() is empty — call set_fitness_values in calculate_fitness".into(),
        ));
    }
    // NaN guard
    for (i, c) in chromosomes.iter().enumerate() {
        if c.fitness_values().iter().any(|&s| s.is_nan()) {
            return Err(GaError::SelectionError(format!(
                "NaN in fitness_values at chromosome {}",
                i
            )));
        }
    }

    // Lexicase always produces 2-parent groups
    let groups = match configuration.method {
        Selection::Lexicase => lexicase_selection(chromosomes, configuration.number_of_couples, 2),
        Selection::EpsilonLexicase => epsilon_lexicase_selection(
            chromosomes,
            configuration.number_of_couples,
            if configuration.epsilon == 0.0 {
                None
            } else {
                Some(configuration.epsilon)
            },
            2,
        ),
        _ => {
            return Err(GaError::ConfigurationError(
                "factory_lexicase called with non-lexicase selection method".into(),
            ));
        }
    };

    // D-04: sync scalar fitness to mean of fitness values
    for c in chromosomes.iter_mut() {
        let scores = c.fitness_values().to_vec();
        if !scores.is_empty() {
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            c.set_fitness(mean);
        }
    }

    Ok(groups)
}
