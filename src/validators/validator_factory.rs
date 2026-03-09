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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::chromosomes::Range as RangeChromosome;
    use crate::genotypes::Binary as BinaryGene;
    use crate::traits::{ChromosomeT, GeneT};
    use std::borrow::Cow;

    fn make_binary_gene(id: i32) -> BinaryGene {
        let mut g = <BinaryGene as Default>::default();
        g.set_id(id);
        g
    }

    fn make_binary_chromosome(ids: &[i32]) -> BinaryChromosome {
        let mut c = BinaryChromosome::new();
        let dna: Vec<BinaryGene> = ids.iter().map(|&id| make_binary_gene(id)).collect();
        c.set_dna(Cow::Owned(dna));
        c.set_fitness(1.0);
        c
    }

    fn default_config() -> GaConfiguration {
        let mut cfg = GaConfiguration::default();
        cfg.selection_configuration.number_of_couples = 1;
        cfg
    }

    #[test]
    fn validate_binary_chromosome_ok() {
        let cfg = default_config();
        let pop = Population::new(vec![
            make_binary_chromosome(&[1, 2, 3]),
            make_binary_chromosome(&[4, 5, 6]),
        ]);
        assert!(validate(Some(&cfg), Some(&pop), None).is_ok());
    }

    #[test]
    fn validate_binary_chromosome_err_different_dna_lengths() {
        let cfg = default_config();
        let pop = Population::new(vec![
            make_binary_chromosome(&[1, 2]),
            make_binary_chromosome(&[3, 4, 5]),
        ]);
        assert!(validate(Some(&cfg), Some(&pop), None).is_err());
    }

    #[test]
    fn validate_range_chromosome_ok() {
        let cfg = default_config();
        let mut c1 = RangeChromosome::<f64>::new();
        c1.set_fitness(1.0);
        let mut c2 = RangeChromosome::<f64>::new();
        c2.set_fitness(2.0);
        let pop = Population::new(vec![c1, c2]);
        assert!(validate(Some(&cfg), Some(&pop), None).is_ok());
    }

    #[test]
    fn validate_none_everything() {
        assert!(validate::<BinaryChromosome>(None, None, None).is_ok());
    }
}
