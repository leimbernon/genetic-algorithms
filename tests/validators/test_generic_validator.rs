use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::{GaConfiguration, ProblemSolving};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::operations;
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{ChromosomeT, GeneT, LinearChromosome};
use genetic_algorithms::validators::generic_validator;
use std::borrow::Cow;

// -- helpers ------------------------------------------------------------------

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

fn make_population(chromosomes: Vec<BinaryChromosome>) -> Population<BinaryChromosome> {
    Population::new(chromosomes)
}

fn default_config() -> GaConfiguration {
    use genetic_algorithms::traits::{ConfigurationT, SelectionConfig};
    GaConfiguration::default().with_number_of_couples(1)
}

// -- unique_gene_ids ----------------------------------------------------------

#[test]
fn unique_gene_ids_ok_when_all_unique() {
    let pop = make_population(vec![
        make_binary_chromosome(&[1, 2, 3]),
        make_binary_chromosome(&[4, 5, 6]),
    ]);
    assert!(generic_validator::unique_gene_ids(&pop).is_ok());
}

#[test]
fn unique_gene_ids_err_on_duplicate() {
    let pop = make_population(vec![make_binary_chromosome(&[1, 2, 1])]);
    let err = generic_validator::unique_gene_ids(&pop).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("duplicate gene id"), "got: {msg}");
}

#[test]
fn unique_gene_ids_ok_empty_population() {
    let pop = make_population(vec![]);
    assert!(generic_validator::unique_gene_ids(&pop).is_ok());
}

#[test]
fn unique_gene_ids_ok_single_gene() {
    let pop = make_population(vec![make_binary_chromosome(&[42])]);
    assert!(generic_validator::unique_gene_ids(&pop).is_ok());
}

// -- fitness_target_is_some ---------------------------------------------------

#[test]
fn fitness_target_is_some_ok_when_set() {
    use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
    let cfg = default_config().with_fitness_target(10.0);
    assert!(generic_validator::fitness_target_is_some(&cfg, "FixedFitness".to_string()).is_ok());
}

#[test]
fn fitness_target_is_some_err_when_none() {
    let cfg = default_config(); // fitness_target is None by default
    let err =
        generic_validator::fitness_target_is_some(&cfg, "FixedFitness".to_string()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("fitness_target must be set"), "got: {msg}");
}

// -- same_dna_length ----------------------------------------------------------

#[test]
fn same_dna_length_ok_when_equal() {
    let pop = make_population(vec![
        make_binary_chromosome(&[1, 2, 3]),
        make_binary_chromosome(&[4, 5, 6]),
    ]);
    assert!(generic_validator::same_dna_length(&pop).is_ok());
}

#[test]
fn same_dna_length_err_when_different() {
    let pop = make_population(vec![
        make_binary_chromosome(&[1, 2]),
        make_binary_chromosome(&[3, 4, 5]),
    ]);
    let err = generic_validator::same_dna_length(&pop).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("same dna length"), "got: {msg}");
}

#[test]
fn same_dna_length_ok_empty_population() {
    let pop = make_population(vec![]);
    assert!(generic_validator::same_dna_length(&pop).is_ok());
}

#[test]
fn same_dna_length_ok_single_chromosome() {
    let pop = make_population(vec![make_binary_chromosome(&[1, 2])]);
    assert!(generic_validator::same_dna_length(&pop).is_ok());
}

// -- chromosome_length_not_bigger_than_alleles --------------------------------

#[test]
fn chromosome_length_ok_when_equal() {
    let alleles = vec![make_binary_gene(0), make_binary_gene(1)];
    assert!(
        generic_validator::chromosome_length_not_bigger_than_alleles::<BinaryChromosome>(
            &alleles, 2
        )
        .is_ok()
    );
}

#[test]
fn chromosome_length_ok_when_smaller() {
    let alleles = vec![
        make_binary_gene(0),
        make_binary_gene(1),
        make_binary_gene(2),
    ];
    assert!(
        generic_validator::chromosome_length_not_bigger_than_alleles::<BinaryChromosome>(
            &alleles, 2
        )
        .is_ok()
    );
}

#[test]
fn chromosome_length_err_when_bigger() {
    let alleles = vec![make_binary_gene(0)];
    let err = generic_validator::chromosome_length_not_bigger_than_alleles::<BinaryChromosome>(
        &alleles, 5,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("should not be higher"), "got: {msg}");
}

// -- aga_crossover_probabilities ----------------------------------------------

#[test]
fn aga_crossover_probabilities_ok() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config()
        .with_crossover_probability_max(0.9)
        .with_crossover_probability_min(0.1);
    assert!(generic_validator::aga_crossover_probabilities(&cfg).is_ok());
}

#[test]
fn aga_crossover_probabilities_err_missing_max() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config().with_crossover_probability_min(0.1);
    assert!(generic_validator::aga_crossover_probabilities(&cfg).is_err());
}

#[test]
fn aga_crossover_probabilities_err_missing_min() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config().with_crossover_probability_max(0.9);
    assert!(generic_validator::aga_crossover_probabilities(&cfg).is_err());
}

#[test]
fn aga_crossover_probabilities_err_max_le_min() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config()
        .with_crossover_probability_max(0.5)
        .with_crossover_probability_min(0.5);
    assert!(generic_validator::aga_crossover_probabilities(&cfg).is_err());
}

#[test]
fn aga_crossover_probabilities_err_max_less_than_min() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config()
        .with_crossover_probability_max(0.1)
        .with_crossover_probability_min(0.9);
    assert!(generic_validator::aga_crossover_probabilities(&cfg).is_err());
}

// -- number_of_couples_is_set -------------------------------------------------

#[test]
fn number_of_couples_is_set_ok() {
    let cfg = default_config(); // number_of_couples = 1
    assert!(generic_validator::number_of_couples_is_set(&cfg).is_ok());
}

#[test]
fn number_of_couples_is_set_err_when_zero() {
    use genetic_algorithms::traits::{ConfigurationT, SelectionConfig};
    let cfg = GaConfiguration::default().with_number_of_couples(0);
    let err = generic_validator::number_of_couples_is_set(&cfg).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("number of couples"), "got: {msg}");
}

// -- validate (compound orchestrator) -----------------------------------------

#[test]
fn validate_ok_minimal() {
    // No configuration, no population, no alleles -- should pass
    assert!(generic_validator::validate::<BinaryChromosome>(None, None, None).is_ok());
}

#[test]
fn validate_ok_with_population_and_config() {
    let cfg = default_config();
    let pop = make_population(vec![
        make_binary_chromosome(&[1, 2, 3]),
        make_binary_chromosome(&[4, 5, 6]),
    ]);
    assert!(generic_validator::validate(Some(&cfg), Some(&pop), None).is_ok());
}

#[test]
fn validate_err_different_dna_lengths() {
    let pop = make_population(vec![
        make_binary_chromosome(&[1, 2]),
        make_binary_chromosome(&[3, 4, 5]),
    ]);
    assert!(generic_validator::validate::<BinaryChromosome>(None, Some(&pop), None).is_err());
}

#[test]
fn validate_err_fixed_fitness_without_target() {
    use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
    let cfg = default_config().with_problem_solving(ProblemSolving::FixedFitness);
    assert!(generic_validator::validate::<BinaryChromosome>(Some(&cfg), None, None).is_err());
}

#[test]
fn validate_ok_fixed_fitness_with_target() {
    use genetic_algorithms::traits::{ConfigurationT, StoppingConfig};
    let cfg = default_config()
        .with_problem_solving(ProblemSolving::FixedFitness)
        .with_fitness_target(42.0);
    assert!(generic_validator::validate::<BinaryChromosome>(Some(&cfg), None, None).is_ok());
}

#[test]
fn validate_err_cycle_crossover_duplicate_ids() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config().with_crossover_method(operations::Crossover::Cycle);
    let pop = make_population(vec![make_binary_chromosome(&[1, 2, 1])]);
    assert!(generic_validator::validate(Some(&cfg), Some(&pop), None).is_err());
}

#[test]
fn validate_err_adaptive_without_probabilities() {
    use genetic_algorithms::traits::ConfigurationT;
    let cfg = default_config().with_adaptive_ga(true);
    // probability_max / probability_min are both None
    assert!(generic_validator::validate::<BinaryChromosome>(Some(&cfg), None, None).is_err());
}

#[test]
fn validate_ok_adaptive_with_probabilities() {
    use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
    let cfg = default_config()
        .with_adaptive_ga(true)
        .with_crossover_probability_max(0.9)
        .with_crossover_probability_min(0.1);
    assert!(generic_validator::validate::<BinaryChromosome>(Some(&cfg), None, None).is_ok());
}

#[test]
fn validate_err_zero_couples() {
    let cfg = GaConfiguration::default();
    // number_of_couples defaults to 0
    assert!(generic_validator::validate::<BinaryChromosome>(Some(&cfg), None, None).is_err());
}
