use std::any::TypeId;
use crate::configuration::{GaConfiguration, ProblemSolving};
use crate::genotypes::{Binary, Range};
use crate::operations;
use crate::population::Population;
use crate::traits::{ChromosomeT, GeneT};
pub fn validate<U>(configuration: Option<&GaConfiguration>, population: Option<&Population<U>>,
                   alleles: Option<&[U::Gene]>)
where
    U: ChromosomeT + Send + Sync + 'static + Clone,
    U::Gene: 'static,
{
    //1 We call the condition for checking the length of every chromosome
    if let Some(population) = population{
        same_dna_length(population);
    }

    //2 Checks the configuration
    if let Some(configuration) = configuration {
        //2.1- We call the condition for fixed fitness
        if configuration.limit_configuration.problem_solving == ProblemSolving::FixedFitness{
            fitness_target_is_some(configuration, configuration.limit_configuration.problem_solving.to_string());
        }

        //2.2 Checks the population
        if let Some(population) = population {

            //2.2.1- Checks the conditions for cycle crossover operation
            if configuration.crossover_configuration.method == operations::Crossover::Cycle{
                unique_gene_ids(population);
            }
        }

        //2.3 Condition checkers for the adaptive genetic algorithms
        if configuration.adaptive_ga{
            //2.3.1- Checks for the crossover parameters
            aga_crossover_probabilities(configuration);
        }

        //2.4 Condition checkers for the repetition of the alleles
        if configuration.limit_configuration.alleles_can_be_repeated{
            if let Some(alleles) = alleles {
                // If the alleles are not range genotypes, we check that the chromosome length is not bigger than the alleles
                if TypeId::of::<U::Gene>() != TypeId::of::<Range<U::Gene>>() {
                    chromosome_length_not_bigger_than_alleles::<U>(alleles, configuration.limit_configuration.genes_per_chromosome);
                }
            }
        }

        //2.5 Condition checkers for the default population
        /*if population.is_none() || population.unwrap().chromosomes.is_empty(){
            check_genes_per_chromosome_is_set(configuration);
            check_population_size_is_set(configuration);
            check_alleles_are_set::<U>(alleles);
        }*/

        //2.6 Condition checker for the couples
        number_of_couples_is_set(configuration);
    }
}


/**
* Function to check that every chromosome has unique id's within their dna
*/
pub fn unique_gene_ids<U>(population: &Population<U>)
where
    U:ChromosomeT + Send + Sync + 'static + Clone{

    //We analyze chromosome by chromosome
    for (chromosome_number, chromosome) in population.chromosomes.iter().enumerate(){
        //We check if the gene id is none or if it already exists in the dna
        for(gene_number, gene) in chromosome.get_dna().iter().enumerate(){
            for i in gene_number+1..chromosome.get_dna().len(){
                //If the gene id is equal to any other, we stop the run
                if gene.get_id().eq(&chromosome.get_dna().get(i).unwrap().get_id()){
                    panic!("Gene id must be unique within the DNA. The chromosome #{}, has same gene id at gene #{} and gene #{}",
                           chromosome_number, gene_number, i);
                }
            }
        }
    }
}

/**
* This function checks that fitness target is not none
*/
pub fn fitness_target_is_some(configuration: &GaConfiguration, problem_type: String){

    //Checks that the fitness target is some
    if configuration.limit_configuration.fitness_target.is_none(){
        panic!("For {} problems, fitness_target must be set.", problem_type);
    }
}

/**
* Checks that all the chromosomes have the same dna length
*/
pub fn same_dna_length<U>(population: &Population<U>)
where
    U:ChromosomeT + Send + Sync + 'static + Clone{
    //We analyze chromosome by chromosome
    for (chromosome_number, chromosome) in population.chromosomes.iter().enumerate(){
        for i in chromosome_number +1..population.chromosomes.len(){
            if chromosome.get_dna().len() != population.chromosomes.get(i).unwrap().get_dna().len(){
                panic!("All the chromosomes must have the same dna length. Chromosome #{} has a dna with length {} and chromosome #{} has a dna with length {}.",
                       chromosome_number, chromosome.get_dna().len(), i, population.chromosomes.get(i).unwrap().get_dna().len());
            }
        }
    }
}

/**
* Function to check that the chromosome length is not bigger than the alleles
*/
pub fn chromosome_length_not_bigger_than_alleles<U>(alleles: &[U::Gene], genes_per_chromosome:i32)
where
    U:ChromosomeT + Send + Sync + 'static + Clone{
    if genes_per_chromosome as usize > alleles.len() {
        panic!("The number of genes within a chromosome should not be higher than the different alleles.");
    }
}

/**
* Function to check that the number of genes per chromosome is set
*/
pub fn genes_per_chromosome_is_set(configuration: &GaConfiguration){
    if configuration.limit_configuration.genes_per_chromosome <= 0 {
        panic!("The number of genes per chromosome must be set.");
    }
}

/**
* Function to check that the alleles are set
*/
pub fn alleles_are_set<U>(alleles: Option<&[U::Gene]>)
where U:ChromosomeT + Send + Sync + 'static + Clone{
    if alleles.is_none() {
        panic!("The alleles must be set.");
    }else if let Some(alleles) = alleles{
        if alleles.is_empty() {
            panic!("The alleles must be set.");
        }
    }
}

/**
* Checks that for adaptive crossover all the requirements are set
*/
pub fn aga_crossover_probabilities(configuration: &GaConfiguration){
    if configuration.crossover_configuration.probability_max.is_none() || configuration.crossover_configuration.probability_min.is_none() {
        core::panic!("For Adaptive Genetic Algorithms, the probability_max and probability_min in the crossover_configuration are mandatory.");
    }else if configuration.crossover_configuration.probability_max <=  configuration.crossover_configuration.probability_min {
        core::panic!("For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the crossover_configuration.");
    }
}

/**
 * Checks that for adaptive mutation all the requirements are set
 */
pub fn aga_mutation_probabilities(configuration: GaConfiguration){
    if configuration.mutation_configuration.probability_max.is_none() || configuration.mutation_configuration.probability_min.is_none(){
        core::panic!("For Adaptive Genetic Algorithms, the probability_max and probability_min in the mutation_configuration are mandatory.");
    }else if configuration.mutation_configuration.probability_max <= configuration.mutation_configuration.probability_min {
        core::panic!("For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the mutation_configuration.");
    }
}

/**
 * Function to check that the number of couples is set
 */
pub fn number_of_couples_is_set(configuration: &GaConfiguration){
    if configuration.selection_configuration.number_of_couples <= 0 {
        core::panic!("The number of couples must be set.");
    }
}

/**
* Function to check that the population size is set
*/
pub fn population_size_is_set(configuration: &GaConfiguration){
    if configuration.limit_configuration.population_size <= 0 {
        panic!("The population size must be set.");
    }
}