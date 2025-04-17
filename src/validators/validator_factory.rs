use std::any::TypeId;
use std::fmt::Debug;
use crate::chromosomes::{Binary, Range};
use crate::configuration::GaConfiguration;
use crate::population::Population;
use crate::traits::ChromosomeT;
use crate::validators::generic_validator as GenericValidator;

pub fn validate<U>(configuration: Option<&GaConfiguration>, population: Option<&Population<U>>,
                            alleles: Option<&[U::Gene]>)
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
    U::Gene: 'static + Debug,
{
    if TypeId::of::<U::Gene>() == TypeId::of::<Binary>() {
        panic!("Not yet implemented");
    } else if TypeId::of::<U::Gene>() == TypeId::of::<Range<U::Gene>>() {
        panic!("Not yet implemented");
    } else {
        GenericValidator::validate(configuration, population, alleles);
    }
}