use crate::{population::Population, traits::{ChromosomeT, GeneT}};
use crate::configuration::GaConfiguration;

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
pub fn check_chromosome_length_not_bigger_than_alleles<U>(alleles: &[U::Gene], genes_per_chromosome:i32)
where
    U:ChromosomeT + Send + Sync + 'static + Clone{
    if genes_per_chromosome as usize > alleles.len() {
        panic!("The number of genes within a chromosome should not be higher than the different alleles.");
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