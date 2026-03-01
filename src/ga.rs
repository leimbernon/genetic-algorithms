use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::validators::validator_factory as ValidatorFactory;
use crate::{
    configuration::{LimitConfiguration, LogLevel, ProblemSolving},
    operations::{crossover, mutation, selection, survivor},
    population::Population,
    traits::{ChromosomeT, ConfigurationT},
};
use log::{debug, info, trace};
use rand::Rng;
use rayon::prelude::*;
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::sync::Arc;

/// Indicates why a GA run terminated.
///
/// - `GenerationLimitReached`: the maximum number of generations was reached.
/// - `FitnessTargetReached`: a stopping criterion based on fitness was satisfied.
/// - `NotTerminated`: internal state before the run finalizes or if a callback is invoked mid-run.
#[derive(Debug, PartialEq)]
pub enum TerminationCause {
    GenerationLimitReached,
    FitnessTargetReached,
    NotTerminated,
}

/// Type alias for the initialization function signature.
type InitializationFn<G> = dyn Fn(i32, Option<&[G]>, Option<bool>) -> Vec<G> + Send + Sync;

/// Type alias for the fitness function signature.
type FitnessFn<G> = dyn Fn(&[G]) -> f64 + Send + Sync;

/// Generic Genetic Algorithm orchestrator.
///
/// Type parameter:
/// - `U`: Chromosome type implementing `ChromosomeT`.
///
/// Responsibilities:
/// - Manage configuration, alleles, population and termination state.
/// - Provide builder-like configuration methods (`ConfigurationT`) to compose the run.
/// - Coordinate the GA cycle: initialization, selection, crossover, mutation, survivor, evaluation.
pub struct Ga<U>
where
    U: ChromosomeT,
{
    /// Tunable GA configuration (limits, operators, logging, etc.).
    pub configuration: GaConfiguration,
    /// Alleles template for initialization functions (optional).
    pub alleles: Vec<U::Gene>,
    /// Current population.
    pub population: Population<U>,
    /// Termination cause after `run` or `run_with_callback`.
    pub termination_cause: TerminationCause,

    /// Initialization function to build chromosomes' DNA at startup.
    pub initialization_fn: Option<Arc<InitializationFn<U::Gene>>>,
    /// Fitness function applied to chromosomes.
    pub fitness_fn: Option<Arc<FitnessFn<U::Gene>>>,
}

impl<U> Default for Ga<U>
where
    U: ChromosomeT,
{
    fn default() -> Self {
        Ga {
            configuration: GaConfiguration {
                ..Default::default()
            },
            population: Population::new_empty(),
            alleles: Vec::new(),
            termination_cause: TerminationCause::NotTerminated,
            initialization_fn: None,
            fitness_fn: None,
        }
    }
}

impl<U> ConfigurationT for Ga<U>
where
    U: ChromosomeT,
{
    fn new() -> Self {
        Self::default()
    }
    fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self {
        self.configuration.with_adaptive_ga(adaptive_ga);
        self
    }
    fn with_threads(&mut self, number_of_threads: i32) -> &mut Self {
        self.configuration.with_threads(number_of_threads);
        self
    }
    fn with_logs(&mut self, log_level: LogLevel) -> &mut Self {
        self.configuration.with_logs(log_level);
        self
    }
    fn with_survivor_method(&mut self, method: crate::operations::Survivor) -> &mut Self {
        self.configuration.with_survivor_method(method);
        self
    }

    //Limit configuration
    fn with_problem_solving(&mut self, problem_solving: ProblemSolving) -> &mut Self {
        self.configuration.with_problem_solving(problem_solving);
        self
    }
    fn with_max_generations(&mut self, max_generations: i32) -> &mut Self {
        self.configuration.with_max_generations(max_generations);
        self
    }
    fn with_fitness_target(&mut self, fitness_target: f64) -> &mut Self {
        self.configuration.with_fitness_target(fitness_target);
        self
    }
    fn with_population_size(&mut self, population_size: i32) -> &mut Self {
        self.configuration.with_population_size(population_size);

        // Setting the number of couples
        self.configuration.selection_configuration.number_of_couples =
            if self.configuration.selection_configuration.number_of_couples == 0 {
                ((self.configuration.limit_configuration.population_size / 2) as f64).round() as i32
            } else {
                self.configuration.selection_configuration.number_of_couples
            };

        self
    }
    fn with_genes_per_chromosome(&mut self, genes_per_chromosome: i32) -> &mut Self {
        self.configuration
            .with_genes_per_chromosome(genes_per_chromosome);
        self
    }
    fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self {
        self.configuration.with_needs_unique_ids(needs_unique_ids);
        self
    }
    fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self {
        self.configuration
            .with_alleles_can_be_repeated(alleles_can_be_repeated);
        self
    }

    //Selection configuration
    fn with_number_of_couples(&mut self, number_of_couples: i32) -> &mut Self {
        self.configuration.with_number_of_couples(number_of_couples);
        self
    }
    fn with_selection_method(
        &mut self,
        selection_method: crate::operations::Selection,
    ) -> &mut Self {
        self.configuration.with_selection_method(selection_method);
        self
    }

    //Crossover configuration
    fn with_crossover_number_of_points(&mut self, number_of_points: i32) -> &mut Self {
        self.configuration
            .with_crossover_number_of_points(number_of_points);
        self
    }
    fn with_crossover_probability_max(&mut self, probability_max: f64) -> &mut Self {
        self.configuration
            .with_crossover_probability_max(probability_max);
        self
    }
    fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self {
        self.configuration
            .with_crossover_probability_min(probability_min);
        self
    }
    fn with_crossover_method(&mut self, method: crossover::Crossover) -> &mut Self {
        self.configuration.with_crossover_method(method);
        self
    }

    //Mutation configuration
    fn with_mutation_probability_max(&mut self, probability_max: f64) -> &mut Self {
        self.configuration
            .with_mutation_probability_max(probability_max);
        self
    }
    fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self {
        self.configuration
            .with_mutation_probability_min(probability_min);
        self
    }
    fn with_mutation_method(&mut self, method: crate::operations::Mutation) -> &mut Self {
        self.configuration.with_mutation_method(method);
        self
    }

    //Save progress configuration
    fn with_save_progress(&mut self, save_progress: bool) -> &mut Self {
        self.configuration.with_save_progress(save_progress);
        self
    }
    fn with_save_progress_interval(&mut self, save_progress_interval: i32) -> &mut Self {
        self.configuration
            .with_save_progress_interval(save_progress_interval);
        self
    }
    fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self {
        self.configuration
            .with_save_progress_path(save_progress_path);
        self
    }
}

impl<U> Ga<U>
where
    U: ChromosomeT + Send + Sync + 'static + Clone + Debug + mutation::ValueMutable,
    U::Gene: 'static + Debug,
{
    /**
     * Function to set the alleles
     */
    pub fn with_alleles(&mut self, alleles: Vec<U::Gene>) -> &mut Self {
        self.alleles = alleles;
        self
    }

    /**
     * Function to set the population
     */
    pub fn with_population(&mut self, population: Population<U>) -> &mut Self {
        self.population = population;

        //Checks if the number of couples is 0, sets the number of couples to the half of the population
        if self.configuration.selection_configuration.number_of_couples == 0 {
            self.configuration.selection_configuration.number_of_couples =
                ((self.population.size() / 2) as f64).round() as i32;
        }
        self
    }

    /**
     * Function to set the fitness function
     */
    pub fn with_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[U::Gene]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Some(Arc::new(fitness_fn));
        self
    }

    /**
     * Sets the initialization function
     */
    pub fn with_initialization_fn<F>(&mut self, initialization_fn: F) -> &mut Self
    where
        U: ChromosomeT + Send + Sync + 'static + Clone,
        F: Fn(i32, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync + 'static,
    {
        self.initialization_fn = Some(Arc::new(initialization_fn));
        self
    }

    /// Randomly initializes the population using the provided initialization function.
    ///
    /// Behavior:
    /// - Validates configuration and alleles before starting.
    /// - Spawns threads to create and evaluate chromosomes in parallel.
    /// - Sets the internal `population` with the collected chromosomes.
    pub fn initialization(&mut self) -> Result<&mut Self, GaError>
    where
        U: ChromosomeT + Send + Sync + 'static + Clone,
    {
        // Before starting initialization, we should verify that initializer is set
        if self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        //Before starting the run, we will check the conditions
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        info!("Initialization started");

        let population_size = self.configuration.limit_configuration.population_size;
        let genes_per_chromosome = self.configuration.limit_configuration.genes_per_chromosome;
        let needs_unique_ids = self.configuration.limit_configuration.needs_unique_ids;
        let initialization_fn = self.initialization_fn.clone().unwrap();
        let fitness_fn = self.fitness_fn.clone().unwrap();
        let alleles = self.alleles.clone();

        // Use rayon to initialize chromosomes in parallel
        let chromosomes: Vec<U> = (0..population_size)
            .into_par_iter()
            .map(|_| {
                let mut chromosome = U::new();

                // Gets the dna randomly
                let dna_chromosome = (initialization_fn)(
                    genes_per_chromosome,
                    Some(&alleles),
                    Some(needs_unique_ids),
                );
                chromosome.set_dna(Cow::Owned(dna_chromosome));

                // Wrap the fitness function in a closure
                let fitness_fn_clone = fitness_fn.clone();
                let fitness_closure = move |genes: &[U::Gene]| (fitness_fn_clone)(genes);

                // Sets the dna of the chromosome, the age, sets the fitness fn and calculates fitness
                chromosome.set_age(0);
                chromosome.set_fitness_fn(fitness_closure);
                chromosome.calculate_fitness();

                chromosome
            })
            .collect();

        self.with_population(Population::new(chromosomes));
        Ok(self)
    }

    /// Runs the GA without callbacks and returns a reference to the final population.
    ///
    /// Equivalent to `run_with_callback(None, 0)`.
    pub fn run(&mut self) -> Result<&Population<U>, GaError> {
        self.run_with_callback(None::<fn(&i32, &Population<U>, &TerminationCause)>, 0)
    }

    /// Runs the GA and optionally invokes a callback every `generations_to_callback` generations.
    ///
    /// Execution cycle per generation:
    /// 1) Selection of parents, 2) Crossover to produce offspring, 3) Mutation of offspring,
    /// 4) Survivor selection to prune population, 5) Best chromosome update, 6) Stop check.
    ///
    /// Logging is controlled by configuration log level; adaptive GA updates use f_avg and f_max.
    pub fn run_with_callback<F>(
        &mut self,
        callback: Option<F>,
        generations_to_callback: i32,
    ) -> Result<&Population<U>, GaError>
    where
        U: ChromosomeT + Send + Sync + 'static + Clone,
        F: Fn(&i32, &Population<U>, &TerminationCause),
    {
        //Before starting the run, we will check the conditions
        ValidatorFactory::validate::<U>(Some(&self.configuration), None, Some(&self.alleles))?;

        //If we want to initialize the population randomly
        if self.population.size() == 0 && self.initialization_fn.is_some() {
            self.initialization()?;
        } else if self.population.size() == 0 && self.initialization_fn.is_none() {
            return Err(GaError::InitializationError(
                "No initialization function set".to_string(),
            ));
        }

        //We set the environment variable from the configuration value
        let key = "RUST_LOG";
        let log_level = match self.configuration.log_level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        };
        env::set_var(key, log_level.as_str());
        let _ = env_logger::try_init();

        //Initialize the adaptive ga
        if self.configuration.adaptive_ga {
            self.population.recalculate_aga();
        }

        //Best chromosome within the generations and population returned
        let initial_population_size = self.population.size();
        let mut age = 0;

        //Calculation of the fitness and the best chromosome
        self.population.fitness_calculation(
            self.configuration.number_of_threads,
            self.configuration.limit_configuration.problem_solving,
        );

        // Starting counting the generations for the callback
        let mut generation_callback_count = 0;

        //We start the cycles
        for i in 0..self.configuration.limit_configuration.max_generations {
            info!(target="ga_events", method="run"; "Generation number: {}", i+1);
            age += 1;

            //1- Parent selection for reproduction
            let parents = selection::factory(
                &self.population.chromosomes,
                self.configuration.selection_configuration,
                self.configuration.number_of_threads,
            )?;
            debug!(target="ga_events", method="run"; "Parents selected for reproduction");

            //2- Getting the offspring
            let mut offspring = parent_crossover(
                &parents,
                &self.population.chromosomes,
                &self.configuration,
                age,
                self.population.f_max,
                self.population.f_avg,
            )?;
            debug!(target="ga_events", method="run"; "Offspring created");

            //3- Insert the children in the population
            self.population.add_chromosomes(&mut offspring);

            //4- Survivor selection
            survivor::factory(
                self.configuration.survivor,
                &mut self.population.chromosomes,
                initial_population_size,
                self.configuration.limit_configuration,
            )?;
            if self.configuration.adaptive_ga {
                self.population.recalculate_aga();
            }
            debug!(target="ga_events", method="run"; "Survivors selected");

            //5- Sets the best chromosome
            for chromosome in &self.population.chromosomes.clone() {
                self.population.decide_best_chromosome(
                    chromosome,
                    self.configuration.limit_configuration.problem_solving,
                );
            }
            debug!(target="ga_events", method="run"; "Best chromosome calculated - generation {}", i+1);

            // If we want to perform a callback
            if let Some(func) = &callback {
                if (generation_callback_count + 1) == generations_to_callback {
                    func(&i, &self.population, &self.termination_cause);
                    generation_callback_count = 0;
                } else {
                    generation_callback_count += 1;
                }
            }

            //6- Identifies if the limit has been reached or not
            if limit_reached(
                self.configuration.limit_configuration,
                &self.population.chromosomes,
            ) {
                // If we want to perform a callback
                if let Some(func) = &callback {
                    self.termination_cause = TerminationCause::FitnessTargetReached;
                    func(&i, &self.population, &self.termination_cause);
                }
                break;
            }
        }

        // If we want to perform a callback and the fitness target is not reached
        if let Some(func) = &callback {
            if self.termination_cause == TerminationCause::NotTerminated {
                self.termination_cause = TerminationCause::GenerationLimitReached;
                func(
                    &self.configuration.limit_configuration.max_generations,
                    &self.population,
                    &self.termination_cause,
                );
            }
        }

        Ok(&self.population)
    }
}

/// Checks termination limits according to `LimitConfiguration`.
///
/// - For Minimization: stops when any chromosome has fitness exactly `0.0`.
/// - For FixedFitness: stops when any chromosome has fitness exactly `fitness_target`.
fn limit_reached<U>(limit: LimitConfiguration, chromosomes: &[U]) -> bool
where
    U: ChromosomeT,
{
    debug!(target="ga_events", method="limit_reached"; "Started limit reached method");
    let mut result = false;

    if limit.problem_solving == ProblemSolving::Minimization {
        //If the problem-solving is minimization, fitness must be 0
        for chromosome in chromosomes {
            if chromosome.get_fitness() == 0.0 {
                trace!(target="ga_events", method="limit_reached"; "limit reached for minimization");
                result = true;
                break;
            }
        }
    } else if limit.problem_solving == ProblemSolving::FixedFitness {
        //If the problem-solving is a fixed fitness
        for chromosome in chromosomes {
            if chromosome.get_fitness() == limit.fitness_target.unwrap() {
                trace!(target="ga_events", method="limit_reached"; "limit reached for fixed fitness");
                result = true;
                break;
            }
        }
    }

    debug!(target="ga_events", method="limit_reached"; "Limit reached method finished");
    result
}

/// Performs parent crossover using the configured crossover and mutation strategies.
///
/// Behavior:
/// - Splits work among threads considering available parent pairs.
/// - Computes adaptive probabilities when enabled; otherwise uses static ones.
/// - Produces children, mutates them, computes their fitness, and returns the offspring.
fn parent_crossover<U>(
    parents: &[(usize, usize)],
    chromosomes: &[U],
    configuration: &GaConfiguration,
    age: i32,
    f_max: f64,
    f_avg: f64,
) -> Result<Vec<U>, GaError>
where
    U: ChromosomeT + Send + Sync + 'static + Clone + mutation::ValueMutable,
{
    debug!(target="ga_events", method="parent_crossover"; "Started the parent crossover");

    /*
        Gets the static crossover probability config and the static mutation probability config
        This way we avoid of passing by these conditions at each thread if it's not necessary
    */
    let crossover_probability_config =
        if let Some(p) = configuration.crossover_configuration.probability_max {
            if !configuration.adaptive_ga {
                Some(p)
            } else {
                None
            }
        } else {
            Some(1.0)
        };

    let mutation_probability_config =
        if let Some(p) = configuration.mutation_configuration.probability_max {
            if !configuration.adaptive_ga {
                Some(p)
            } else {
                None
            }
        } else {
            Some(1.0)
        };

    // Use rayon to process parent pairs in parallel
    let results: Vec<Result<Vec<U>, GaError>> = parents
        .par_iter()
        .map(|(key, value)| {
            let mut rng = rand::rng();

            // Getting the parent 1 and 2 for crossover
            let parent_1 = chromosomes.get(*key).unwrap().clone();
            let parent_2 = chromosomes.get(*value).unwrap().clone();

            // Making the crossover of the parents when the random number is below or equal to the given probability
            let crossover_probability = rng.random_range(0.0..1.0);
            let effective_crossover_prob =
                if let Some(p) = crossover_probability_config {
                    p
                } else {
                    crossover::aga_probability(
                        &parent_1,
                        &parent_2,
                        f_max,
                        f_avg,
                        configuration.crossover_configuration.probability_max.unwrap(),
                        configuration.crossover_configuration.probability_min.unwrap(),
                    )
                };

            // Making the mutation of each child when the random number is below or equal the given probability
            let mut mutation_probability = rng.random_range(0.0..1.0);
            let effective_mutation_prob =
                if let Some(p) = mutation_probability_config {
                    p
                } else {
                    mutation::aga_probability(
                        &parent_1,
                        &parent_2,
                        f_avg,
                        configuration.mutation_configuration.probability_max.unwrap(),
                        configuration.mutation_configuration.probability_min.unwrap(),
                    )
                };

            debug!(target="ga_events", method="parent_crossover"; "Processing parent pair");

            let mut child_1: U;
            let mut child_2: U;

            if crossover_probability <= effective_crossover_prob {
                let mut children = crossover::factory(&parent_1, &parent_2, configuration.crossover_configuration)?;
                child_2 = children.pop().unwrap();
                child_1 = children.pop().unwrap();
            } else {
                child_1 = parent_1;
                child_2 = parent_2;
            }

            debug!(target="ga_events", method="parent_crossover"; "mutation_probability_config {} - mutation probability {}", effective_mutation_prob, mutation_probability);

            if mutation_probability < effective_mutation_prob {
                mutation::factory(configuration.mutation_configuration.method, &mut child_1)?;
            }

            mutation_probability = rng.random_range(0.0..1.0);
            if mutation_probability <= effective_mutation_prob {
                mutation::factory(configuration.mutation_configuration.method, &mut child_2)?;
            }

            // Calculate the fitness of both children and set their age
            child_1.calculate_fitness();
            child_2.calculate_fitness();

            child_1.set_age(age);
            child_2.set_age(age);

            Ok(vec![child_1, child_2])
        })
        .collect();

    // Check for any errors and flatten the results
    let mut offspring = Vec::new();
    for result in results {
        offspring.extend(result?);
    }

    debug!(target="ga_events", method="parent_crossover"; "Parent crossover finished");
    Ok(offspring)
}
