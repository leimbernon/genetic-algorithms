use crate::configuration::{LogLevel, ProblemSolving};
use crate::operations::{Crossover, Mutation, Selection, Survivor};

pub trait ConfigurationT{
    fn new() -> Self;
    fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self;
    fn with_threads(&mut self, number_of_threads: i32)-> &mut Self;
    fn with_logs(&mut self, log_level: LogLevel) -> &mut Self;
    fn with_survivor_method(&mut self, method: Survivor) -> &mut Self;

    //Limit configuration
    fn with_problem_solving(&mut self, problem_solving: ProblemSolving)->&mut Self;
    fn with_max_generations(&mut self, max_generations: i32)-> &mut Self;
    fn with_fitness_target(&mut self, fitness_target: f64)-> &mut Self;
    fn with_best_individual_by_generation(&mut self, best_individual_by_generation: bool) -> &mut Self;
    fn with_population_size(&mut self, population_size: i32) -> &mut Self;
    fn with_genes_per_chromosome(&mut self, genes_per_chromosome: i32) -> &mut Self;
    fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self;
    fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self;

    //Selection configuration
    fn with_number_of_couples(&mut self, number_of_couples: i32)->&mut Self;
    fn with_selection_method(&mut self, selection_method: Selection)->&mut Self;

    //Crossover configuration
    fn with_crossover_number_of_points(&mut self, number_of_points: i32)->&mut Self;
    fn with_crossover_probability_max(&mut self, probability_max: f64)->&mut Self;
    fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self;
    fn with_crossover_method(&mut self, method: Crossover) -> &mut Self;

    //Mutation configuration
    fn with_mutation_probability_max(&mut self, probability_max: f64)->&mut Self;
    fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self;
    fn with_mutation_method(&mut self, method: Mutation) -> &mut Self;

    //Save progress configuration
    fn with_save_progress(&mut self, save_progress: bool) -> &mut Self;
    fn with_save_progress_interval(&mut self, save_progress_interval: i32) -> &mut Self;
    fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self;

}