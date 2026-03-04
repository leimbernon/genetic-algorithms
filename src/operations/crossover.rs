pub use self::cycle::cycle;
pub use self::multipoint::multipoint;
pub use self::order::order;
pub use self::single_point::single_point;
pub use self::uniform_crossover::uniform;
pub(crate) use super::Crossover;
use crate::configuration::CrossoverConfiguration;
use crate::error::GaError;
use crate::traits::{ChromosomeT, CrossoverOperator};

pub mod blend_alpha;
pub mod cycle;
pub mod multipoint;
pub mod order;
pub mod sbx;
pub mod single_point;
pub mod uniform_crossover;

impl CrossoverOperator for Crossover {
    fn crossover<U: ChromosomeT>(&self, parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
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
            Crossover::Sbx => Err(GaError::CrossoverError(
                "SBX crossover requires Range<T> chromosomes. Use crossover::sbx::sbx() directly \
                 or ensure your chromosome type supports SBX."
                    .to_string(),
            )),
            Crossover::BlendAlpha => Err(GaError::CrossoverError(
                "BLX-α crossover requires Range<T> chromosomes. Use crossover::blend_alpha::blend_alpha() \
                 directly or ensure your chromosome type supports BLX-α."
                    .to_string(),
            )),
        }
    }
}

impl CrossoverOperator for CrossoverConfiguration {
    fn crossover<U: ChromosomeT>(&self, parent_1: &U, parent_2: &U) -> Result<Vec<U>, GaError> {
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
            Crossover::Sbx => Err(GaError::CrossoverError(
                "SBX crossover requires Range<T> chromosomes. Use crossover::sbx::sbx() directly \
                 or ensure your chromosome type supports SBX."
                    .to_string(),
            )),
            Crossover::BlendAlpha => Err(GaError::CrossoverError(
                "BLX-α crossover requires Range<T> chromosomes. Use crossover::blend_alpha::blend_alpha() \
                 directly or ensure your chromosome type supports BLX-α."
                    .to_string(),
            )),
        }
    }
}

pub fn factory<U: ChromosomeT>(
    parent_1: &U,
    parent_2: &U,
    configuration: CrossoverConfiguration,
) -> Result<Vec<U>, GaError> {
    configuration.crossover(parent_1, parent_2)
}

//Function to calculate the probability for adaptive genetic algorithms
pub fn aga_probability<U: ChromosomeT>(
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
