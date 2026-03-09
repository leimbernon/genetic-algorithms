use crate::configuration::{GaConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::genotypes::Range;
use crate::operations;
use crate::population::Population;
use crate::traits::{ChromosomeT, GeneT};
use std::any::TypeId;
use std::collections::HashSet;

pub fn validate<U>(
    configuration: Option<&GaConfiguration>,
    population: Option<&Population<U>>,
    alleles: Option<&[U::Gene]>,
) -> Result<(), GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
    U::Gene: 'static,
{
    //1 We call the condition for checking the length of every chromosome
    if let Some(population) = population {
        same_dna_length(population)?;
    }

    //2 Checks the configuration
    if let Some(configuration) = configuration {
        //2.1- We call the condition for fixed fitness
        if configuration.limit_configuration.problem_solving == ProblemSolving::FixedFitness {
            fitness_target_is_some(
                configuration,
                configuration
                    .limit_configuration
                    .problem_solving
                    .to_string(),
            )?;
        }

        //2.2 Checks the population
        if let Some(population) = population {
            //2.2.1- Checks the conditions for cycle crossover operation
            if configuration.crossover_configuration.method == operations::Crossover::Cycle {
                unique_gene_ids(population)?;
            }
        }

        //2.3 Condition checkers for the adaptive genetic algorithms
        if configuration.adaptive_ga {
            //2.3.1- Checks for the crossover parameters
            aga_crossover_probabilities(configuration)?;
        }

        //2.4 Condition checkers for the repetition of the alleles
        if configuration.limit_configuration.alleles_can_be_repeated {
            if let Some(alleles) = alleles {
                // If the alleles are not range genotypes, we check that the chromosome length is not bigger than the alleles
                if TypeId::of::<U::Gene>() != TypeId::of::<Range<U::Gene>>() {
                    chromosome_length_not_bigger_than_alleles::<U>(
                        alleles,
                        configuration.limit_configuration.genes_per_chromosome,
                    )?;
                }
            }
        }

        //2.6 Condition checker for the couples
        number_of_couples_is_set(configuration)?;
    }

    Ok(())
}

/// Checks that every chromosome has unique id's within their dna.
///
/// Uses a `HashSet` for O(N) per chromosome instead of O(N²) nested loop.
pub fn unique_gene_ids<U>(population: &Population<U>) -> Result<(), GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    for (chromosome_number, chromosome) in population.chromosomes.iter().enumerate() {
        let mut seen = HashSet::with_capacity(chromosome.dna().len());
        for (gene_number, gene) in chromosome.dna().iter().enumerate() {
            if !seen.insert(gene.id()) {
                return Err(GaError::ValidationError(format!(
                    "Gene id must be unique within the DNA. The chromosome #{} has a duplicate gene id {} at gene #{}",
                    chromosome_number, gene.id(), gene_number
                )));
            }
        }
    }
    Ok(())
}

/// This function checks that fitness target is not none
pub fn fitness_target_is_some(
    configuration: &GaConfiguration,
    problem_type: String,
) -> Result<(), GaError> {
    if configuration.limit_configuration.fitness_target.is_none() {
        return Err(GaError::ConfigurationError(format!(
            "For {} problems, fitness_target must be set.",
            problem_type
        )));
    }
    Ok(())
}

/// Checks that all the chromosomes have the same dna length.
///
/// Compares each chromosome to the first one in O(N) instead of O(N²).
pub fn same_dna_length<U>(population: &Population<U>) -> Result<(), GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    let Some(first) = population.chromosomes.first() else {
        return Ok(());
    };
    let expected_len = first.dna().len();
    for (i, chromosome) in population.chromosomes.iter().enumerate().skip(1) {
        let len = chromosome.dna().len();
        if len != expected_len {
            return Err(GaError::ValidationError(format!(
                "All the chromosomes must have the same dna length. Chromosome #0 has a dna with length {} and chromosome #{} has a dna with length {}.",
                expected_len, i, len
            )));
        }
    }
    Ok(())
}

/// Function to check that the chromosome length is not bigger than the alleles
pub fn chromosome_length_not_bigger_than_alleles<U>(
    alleles: &[U::Gene],
    genes_per_chromosome: usize,
) -> Result<(), GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
{
    if genes_per_chromosome > alleles.len() {
        return Err(GaError::ConfigurationError(
            "The number of genes within a chromosome should not be higher than the different alleles.".to_string()));
    }
    Ok(())
}

/// Checks that for adaptive crossover all the requirements are set
pub fn aga_crossover_probabilities(configuration: &GaConfiguration) -> Result<(), GaError> {
    if configuration
        .crossover_configuration
        .probability_max
        .is_none()
        || configuration
            .crossover_configuration
            .probability_min
            .is_none()
    {
        return Err(GaError::ConfigurationError(
            "For Adaptive Genetic Algorithms, the probability_max and probability_min in the crossover_configuration are mandatory.".to_string()));
    } else if configuration.crossover_configuration.probability_max
        <= configuration.crossover_configuration.probability_min
    {
        return Err(GaError::ConfigurationError(
            "For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the crossover_configuration.".to_string()));
    }
    Ok(())
}

/// Function to check that the number of couples is set
pub fn number_of_couples_is_set(configuration: &GaConfiguration) -> Result<(), GaError> {
    if configuration.selection_configuration.number_of_couples == 0 {
        return Err(GaError::ConfigurationError(
            "The number of couples must be set.".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chromosomes::Binary as BinaryChromosome;
    use crate::genotypes::Binary as BinaryGene;
    use crate::traits::{ChromosomeT, GeneT};
    use std::borrow::Cow;

    // ── helpers ──────────────────────────────────────────────────────────

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
        let mut cfg = GaConfiguration::default();
        cfg.selection_configuration.number_of_couples = 1;
        cfg
    }

    // ── unique_gene_ids ─────────────────────────────────────────────────

    #[test]
    fn unique_gene_ids_ok_when_all_unique() {
        let pop = make_population(vec![
            make_binary_chromosome(&[1, 2, 3]),
            make_binary_chromosome(&[4, 5, 6]),
        ]);
        assert!(unique_gene_ids(&pop).is_ok());
    }

    #[test]
    fn unique_gene_ids_err_on_duplicate() {
        let pop = make_population(vec![make_binary_chromosome(&[1, 2, 1])]);
        let err = unique_gene_ids(&pop).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("duplicate gene id"), "got: {msg}");
    }

    #[test]
    fn unique_gene_ids_ok_empty_population() {
        let pop = make_population(vec![]);
        assert!(unique_gene_ids(&pop).is_ok());
    }

    #[test]
    fn unique_gene_ids_ok_single_gene() {
        let pop = make_population(vec![make_binary_chromosome(&[42])]);
        assert!(unique_gene_ids(&pop).is_ok());
    }

    // ── fitness_target_is_some ──────────────────────────────────────────

    #[test]
    fn fitness_target_is_some_ok_when_set() {
        let mut cfg = default_config();
        cfg.limit_configuration.fitness_target = Some(10.0);
        assert!(fitness_target_is_some(&cfg, "FixedFitness".to_string()).is_ok());
    }

    #[test]
    fn fitness_target_is_some_err_when_none() {
        let cfg = default_config(); // fitness_target is None by default
        let err = fitness_target_is_some(&cfg, "FixedFitness".to_string()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("fitness_target must be set"), "got: {msg}");
    }

    // ── same_dna_length ─────────────────────────────────────────────────

    #[test]
    fn same_dna_length_ok_when_equal() {
        let pop = make_population(vec![
            make_binary_chromosome(&[1, 2, 3]),
            make_binary_chromosome(&[4, 5, 6]),
        ]);
        assert!(same_dna_length(&pop).is_ok());
    }

    #[test]
    fn same_dna_length_err_when_different() {
        let pop = make_population(vec![
            make_binary_chromosome(&[1, 2]),
            make_binary_chromosome(&[3, 4, 5]),
        ]);
        let err = same_dna_length(&pop).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("same dna length"), "got: {msg}");
    }

    #[test]
    fn same_dna_length_ok_empty_population() {
        let pop = make_population(vec![]);
        assert!(same_dna_length(&pop).is_ok());
    }

    #[test]
    fn same_dna_length_ok_single_chromosome() {
        let pop = make_population(vec![make_binary_chromosome(&[1, 2])]);
        assert!(same_dna_length(&pop).is_ok());
    }

    // ── chromosome_length_not_bigger_than_alleles ───────────────────────

    #[test]
    fn chromosome_length_ok_when_equal() {
        let alleles = vec![make_binary_gene(0), make_binary_gene(1)];
        assert!(chromosome_length_not_bigger_than_alleles::<BinaryChromosome>(&alleles, 2).is_ok());
    }

    #[test]
    fn chromosome_length_ok_when_smaller() {
        let alleles = vec![
            make_binary_gene(0),
            make_binary_gene(1),
            make_binary_gene(2),
        ];
        assert!(chromosome_length_not_bigger_than_alleles::<BinaryChromosome>(&alleles, 2).is_ok());
    }

    #[test]
    fn chromosome_length_err_when_bigger() {
        let alleles = vec![make_binary_gene(0)];
        let err =
            chromosome_length_not_bigger_than_alleles::<BinaryChromosome>(&alleles, 5).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("should not be higher"), "got: {msg}");
    }

    // ── aga_crossover_probabilities ─────────────────────────────────────

    #[test]
    fn aga_crossover_probabilities_ok() {
        let mut cfg = default_config();
        cfg.crossover_configuration.probability_max = Some(0.9);
        cfg.crossover_configuration.probability_min = Some(0.1);
        assert!(aga_crossover_probabilities(&cfg).is_ok());
    }

    #[test]
    fn aga_crossover_probabilities_err_missing_max() {
        let mut cfg = default_config();
        cfg.crossover_configuration.probability_max = None;
        cfg.crossover_configuration.probability_min = Some(0.1);
        assert!(aga_crossover_probabilities(&cfg).is_err());
    }

    #[test]
    fn aga_crossover_probabilities_err_missing_min() {
        let mut cfg = default_config();
        cfg.crossover_configuration.probability_max = Some(0.9);
        cfg.crossover_configuration.probability_min = None;
        assert!(aga_crossover_probabilities(&cfg).is_err());
    }

    #[test]
    fn aga_crossover_probabilities_err_max_le_min() {
        let mut cfg = default_config();
        cfg.crossover_configuration.probability_max = Some(0.5);
        cfg.crossover_configuration.probability_min = Some(0.5);
        assert!(aga_crossover_probabilities(&cfg).is_err());
    }

    #[test]
    fn aga_crossover_probabilities_err_max_less_than_min() {
        let mut cfg = default_config();
        cfg.crossover_configuration.probability_max = Some(0.1);
        cfg.crossover_configuration.probability_min = Some(0.9);
        assert!(aga_crossover_probabilities(&cfg).is_err());
    }

    // ── number_of_couples_is_set ────────────────────────────────────────

    #[test]
    fn number_of_couples_is_set_ok() {
        let cfg = default_config(); // number_of_couples = 1
        assert!(number_of_couples_is_set(&cfg).is_ok());
    }

    #[test]
    fn number_of_couples_is_set_err_when_zero() {
        let mut cfg = default_config();
        cfg.selection_configuration.number_of_couples = 0;
        let err = number_of_couples_is_set(&cfg).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("number of couples"), "got: {msg}");
    }

    // ── validate (compound orchestrator) ────────────────────────────────

    #[test]
    fn validate_ok_minimal() {
        // No configuration, no population, no alleles — should pass
        assert!(validate::<BinaryChromosome>(None, None, None).is_ok());
    }

    #[test]
    fn validate_ok_with_population_and_config() {
        let cfg = default_config();
        let pop = make_population(vec![
            make_binary_chromosome(&[1, 2, 3]),
            make_binary_chromosome(&[4, 5, 6]),
        ]);
        assert!(validate(Some(&cfg), Some(&pop), None).is_ok());
    }

    #[test]
    fn validate_err_different_dna_lengths() {
        let pop = make_population(vec![
            make_binary_chromosome(&[1, 2]),
            make_binary_chromosome(&[3, 4, 5]),
        ]);
        assert!(validate::<BinaryChromosome>(None, Some(&pop), None).is_err());
    }

    #[test]
    fn validate_err_fixed_fitness_without_target() {
        let mut cfg = default_config();
        cfg.limit_configuration.problem_solving = ProblemSolving::FixedFitness;
        cfg.limit_configuration.fitness_target = None;
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, None).is_err());
    }

    #[test]
    fn validate_ok_fixed_fitness_with_target() {
        let mut cfg = default_config();
        cfg.limit_configuration.problem_solving = ProblemSolving::FixedFitness;
        cfg.limit_configuration.fitness_target = Some(42.0);
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, None).is_ok());
    }

    #[test]
    fn validate_err_cycle_crossover_duplicate_ids() {
        let mut cfg = default_config();
        cfg.crossover_configuration.method = operations::Crossover::Cycle;
        let pop = make_population(vec![make_binary_chromosome(&[1, 2, 1])]);
        assert!(validate(Some(&cfg), Some(&pop), None).is_err());
    }

    #[test]
    fn validate_err_adaptive_without_probabilities() {
        let mut cfg = default_config();
        cfg.adaptive_ga = true;
        // probability_max / probability_min are both None
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, None).is_err());
    }

    #[test]
    fn validate_ok_adaptive_with_probabilities() {
        let mut cfg = default_config();
        cfg.adaptive_ga = true;
        cfg.crossover_configuration.probability_max = Some(0.9);
        cfg.crossover_configuration.probability_min = Some(0.1);
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, None).is_ok());
    }

    #[test]
    fn validate_err_alleles_can_be_repeated_chromosome_too_large() {
        let mut cfg = default_config();
        cfg.limit_configuration.alleles_can_be_repeated = true;
        cfg.limit_configuration.genes_per_chromosome = 5;
        let alleles = vec![make_binary_gene(0), make_binary_gene(1)];
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, Some(&alleles)).is_err());
    }

    #[test]
    fn validate_ok_alleles_can_be_repeated_chromosome_fits() {
        let mut cfg = default_config();
        cfg.limit_configuration.alleles_can_be_repeated = true;
        cfg.limit_configuration.genes_per_chromosome = 2;
        let alleles = vec![
            make_binary_gene(0),
            make_binary_gene(1),
            make_binary_gene(2),
        ];
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, Some(&alleles)).is_ok());
    }

    #[test]
    fn validate_err_zero_couples() {
        let cfg = GaConfiguration::default();
        // number_of_couples defaults to 0
        assert!(validate::<BinaryChromosome>(Some(&cfg), None, None).is_err());
    }
}
