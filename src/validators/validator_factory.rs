//! Validator factory — dispatches validation to the correct validator.
//!
//! Provides a factory function that selects the appropriate validator based
//! on the engine type and configuration. Currently delegates to the generic
//! validator for all standard GA configurations.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`validate`] | Factory function that dispatches to the correct validator |
//!
//! [`validate`]: crate::validators::validator_factory::validate

use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::population::Population;
use crate::traits::ChromosomeT;
use crate::validators::generic_validator as GenericValidator;
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
    // All chromosome types (including Binary and Range) use the generic validator.
    // The previous type-gating that returned "Not yet implemented" for built-in types
    // has been removed — there is no reason to reject the library's own types.
    GenericValidator::validate(configuration, population, alleles)
}
