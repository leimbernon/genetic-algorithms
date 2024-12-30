use core::panic;

use crate::{population::Population, traits::{ChromosomeT, GeneT}, configuration::GaConfiguration};

/**
 * Function to check that every individual has unique id's within their dna
 */
pub fn unique_gene_ids<U>(population: &Population<U>)
where 
U:ChromosomeT + Send + Sync + 'static + Clone{

    //We analyze individual by individual
    for (individual_number, individual) in population.individuals.iter().enumerate(){
        //We check if the gene id is none or if it already exists in the dna
        for(gene_number, gene) in individual.get_dna().iter().enumerate(){
            for i in gene_number+1..individual.get_dna().len(){
                //If the gene id is equal to any other, we stop the run
                if gene.get_id().eq(&individual.get_dna().get(i).unwrap().get_id()){
                    panic!("Gene id must be unique within the DNA. The individual #{}, has same gene id at gene #{} and gene #{}", 
                            individual_number, gene_number, i);
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
 * Checks that all the individual have the same dna length
 */
pub fn same_dna_length<U>(population: &Population<U>)
where 
U:ChromosomeT + Send + Sync + 'static + Clone{
    //We analyze individual by individual
    for (individual_number, individual) in population.individuals.iter().enumerate(){
        for i in individual_number+1..population.individuals.len(){
            if individual.get_dna().len() != population.individuals.get(i).unwrap().get_dna().len(){
                panic!("All the individuals must have the same dna length. Individual #{} has a dna with length {} and individual #{} has a dna with length {}.",
                        individual_number, individual.get_dna().len(), i, population.individuals.get(i).unwrap().get_dna().len());
            }
        }
    }
}

/**
 * Checks that for adaptive crossover all the requirements are set
 */
pub fn aga_crossover_probabilities(configuration: &GaConfiguration){
    if configuration.crossover_configuration.probability_max.is_none() || configuration.crossover_configuration.probability_min.is_none() {
        panic!("For Adaptive Genetic Algorithms, the probability_max and probability_min in the crossover_configuration are mandatory.");
    }else if configuration.crossover_configuration.probability_max <=  configuration.crossover_configuration.probability_min {
        panic!("For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the crossover_configuration.");
    }
}

/**
 * Checks that for adaptive mutation all the requirements are set
 */
pub fn aga_mutation_probabilities(configuration: GaConfiguration){
    if configuration.mutation_configuration.probability_max.is_none() || configuration.mutation_configuration.probability_min.is_none(){
        panic!("For Adaptive Genetic Algorithms, the probability_max and probability_min in the mutation_configuration are mandatory.");
    }else if configuration.mutation_configuration.probability_max <= configuration.mutation_configuration.probability_min {
        panic!("For Adaptive Genetic Algorithms, the probability_max must be greater than probability_min in the mutation_configuration.");
    }
}

/**
 * Function to check that the genotype length is not bigger than the alleles
 */
pub fn check_genotype_length_not_bigger_than_alleles<U>(alleles: &[U::Gene], genes_per_individual:i32)
where 
U:ChromosomeT + Send + Sync + 'static + Clone{
    if genes_per_individual as usize > alleles.len() {
        panic!("The number of genes within an individual should not be more than the different alleles.");
    }
}

/**
 * Function to check that the number of genes per chromosome is set
 */
pub fn check_genes_per_chromosome_is_set(configuration: &GaConfiguration){
    if configuration.limit_configuration.genes_per_chromosome <= 0 {
        panic!("The number of genes per chromosome must be set.");
    }
}

/**
 * Function to check that the population size is set
 */
pub fn check_population_size_is_set(configuration: &GaConfiguration){
    if configuration.limit_configuration.population_size <= 0 {
        panic!("The population size must be set.");
    }
}

/**
 * Function to check that the alleles are set
 */
pub fn check_alleles_are_set<U>(alleles: Option<&[U::Gene]>)
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
 * Function to check that the number of couples is set
 */
pub fn check_number_of_couples_is_set(configuration: &GaConfiguration){
    if configuration.selection_configuration.number_of_couples <= 0 {
        panic!("The number of couples must be set.");
    }
}
