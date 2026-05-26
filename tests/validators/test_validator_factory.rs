use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::GaConfiguration;
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};
use genetic_algorithms::validators::validator_factory;
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
    use genetic_algorithms::traits::SelectionConfig;
    GaConfiguration::default().with_number_of_couples(1)
}

#[test]
fn validate_binary_chromosome_ok() {
    let cfg = default_config();
    let pop = Population::new(vec![
        make_binary_chromosome(&[1, 2, 3]),
        make_binary_chromosome(&[4, 5, 6]),
    ]);
    assert!(validator_factory::validate(Some(&cfg), Some(&pop), None).is_ok());
}

#[test]
fn validate_binary_chromosome_err_different_dna_lengths() {
    let cfg = default_config();
    let pop = Population::new(vec![
        make_binary_chromosome(&[1, 2]),
        make_binary_chromosome(&[3, 4, 5]),
    ]);
    assert!(validator_factory::validate(Some(&cfg), Some(&pop), None).is_err());
}

#[test]
fn validate_range_chromosome_ok() {
    let cfg = default_config();
    let mut c1 = RangeChromosome::<f64>::new();
    c1.set_fitness(1.0);
    let mut c2 = RangeChromosome::<f64>::new();
    c2.set_fitness(2.0);
    let pop = Population::new(vec![c1, c2]);
    assert!(validator_factory::validate(Some(&cfg), Some(&pop), None).is_ok());
}

#[test]
fn validate_none_everything() {
    assert!(validator_factory::validate::<BinaryChromosome>(None, None, None).is_ok());
}
