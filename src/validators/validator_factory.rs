use crate::chromosomes::{Binary, Range};
use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::population::Population;
use crate::traits::ChromosomeT;
use crate::validators::generic_validator as GenericValidator;
use std::any::TypeId;
use std::fmt::Debug;

pub fn validate<U>(
    configuration: Option<&GaConfiguration>,
    population: Option<&Population<U>>,
    alleles: Option<&[U::Gene]>,
) -> Result<(), GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
    U::Gene: 'static + Debug,
{
    if TypeId::of::<U::Gene>() == TypeId::of::<Binary>()
        || TypeId::of::<U::Gene>() == TypeId::of::<Range<U::Gene>>()
    {
        Err(GaError::ValidationError("Not yet implemented".to_string()))
    } else {
        GenericValidator::validate(configuration, population, alleles)
    }
}
