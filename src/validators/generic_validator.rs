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