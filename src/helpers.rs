use crate::{configuration::GaConfiguration, operations::{self, survivor::fitness::ProblemSolving}, population::Population, traits::ChromosomeT};

pub mod ga_condition_checker;
pub mod chromosome_condition_checker;
pub mod population_condition_checker;
/*
 * Function to call the different condition checkers 
 */
pub fn condition_checker_factory<U>(configuration: Option<&GaConfiguration>, population: Option<&Population<U>>, 
                                    alleles: Option<&[U::Gene]>)
where
U: ChromosomeT + Send + Sync + 'static + Clone
{
    //1- We call the condition for checking the length of every individual
    if let Some(population) = population{
        chromosome_condition_checker::same_dna_length(population);
    }

    //2- Checks the configuration
    if let Some(configuration) = configuration {

        //2.1- We call the condition for fixed fitness
        if configuration.limit_configuration.problem_solving == ProblemSolving::FixedFitness{
            chromosome_condition_checker::fitness_target_is_some(configuration, configuration.limit_configuration.problem_solving.to_string());
        }

        //2.2- Checks the population
        if let Some(population) = population {

            //2.2.1- Checks the conditions for cycle crossover operation
            if configuration.crossover_configuration.method == operations::Crossover::Cycle{
                chromosome_condition_checker::unique_gene_ids(population);
            }
        }

        //2.3- Condition checkers for the adaptive genetic algorithms
        if configuration.adaptive_ga{
            //2.3.1- Checks for the crossover parameters
            ga_condition_checker::aga_crossover_probabilities(configuration);
        } 

        //2.4- Condition checkers for the repetition of the alleles
        if configuration.limit_configuration.alleles_can_be_repeated{
            if let Some(alleles) = alleles {
                chromosome_condition_checker::check_chromosome_length_not_bigger_than_alleles::<U>(alleles, configuration.limit_configuration.genes_per_chromosome);
            }
        }

        //2.5- Condition checkers for the default population
        if population.is_none() || population.unwrap().individuals.is_empty(){
            if configuration.limit_configuration.genes_per_chromosome <= 0 {
                panic!("The number of genes per chromosome must be set.");
            };
            population_condition_checker::check_population_size_is_set(configuration);

            //If the Gene is not a BinaryGenotype, we check that the alleles are set
            //condition_checker::check_alleles_are_set::<U>(alleles);
            
        } 

        //2.6- Condition checker for the couples
        ga_condition_checker::check_number_of_couples_is_set(configuration);
    } 
}
