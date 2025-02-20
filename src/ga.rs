use std::{sync::{mpsc::sync_channel, Mutex, Arc}, thread, collections::HashMap};
use rand::Rng;
use log::{trace, debug, info};
use std::env;
use crate::{configuration::{LimitConfiguration, LogLevel, ProblemSolving}, helpers::condition_checker_factory, operations::{crossover, mutation, selection, survivor}, population::Population, traits::{ChromosomeT, ConfigurationT}};
use crate::configuration::GaConfiguration;

#[derive(Debug, PartialEq)]
pub enum TerminationCause {
    GenerationLimitReached,
    FitnessTargetReached,
    NotTerminated
}

pub struct Ga<U>
where
    U:ChromosomeT
{
    pub configuration: GaConfiguration,
    pub alleles: Vec<U::Gene>,
    pub population: Population<U>,
    pub termination_cause: TerminationCause,

    pub initialization_fn: Option<Arc<dyn Fn(i32, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync>>,
    pub fitness_fn: Option<Arc<dyn Fn(&[U::Gene]) -> f64 + Send + Sync>>,
}


impl<U> Default for Ga<U>
where
    U:ChromosomeT
{
    fn default() -> Self {
        Ga { 
            configuration: GaConfiguration{..Default::default()},
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
    U:ChromosomeT
{
    fn new()->Self{
        Self::default()
    }
    fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self {
        self.configuration.with_adaptive_ga(adaptive_ga);
        self
    }
    fn with_threads(&mut self, number_of_threads: i32)-> &mut Self {
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
    fn with_problem_solving(&mut self, problem_solving: ProblemSolving)->&mut Self {
        self.configuration.with_problem_solving(problem_solving);
        self
    }
    fn with_max_generations(&mut self, max_generations: i32)-> &mut Self {
        self.configuration.with_max_generations(max_generations);
        self
    }
    fn with_fitness_target(&mut self, fitness_target: f64)-> &mut Self {
        self.configuration.with_fitness_target(fitness_target);
        self
    }
    fn with_population_size(&mut self, population_size: i32) -> &mut Self {
        self.configuration.with_population_size(population_size);
        self
    }
    fn with_genes_per_chromosome(&mut self, genes_per_chromosome: i32) -> &mut Self {
        self.configuration.with_genes_per_chromosome(genes_per_chromosome);
        self
    }
    fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self {
        self.configuration.with_needs_unique_ids(needs_unique_ids);
        self
    }
    fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self {
        self.configuration.with_alleles_can_be_repeated(alleles_can_be_repeated);
        self
    }

    //Selection configuration
    fn with_number_of_couples(&mut self, number_of_couples: i32)->&mut Self {
        self.configuration.with_number_of_couples(number_of_couples);
        self
    }
    fn with_selection_method(&mut self, selection_method: crate::operations::Selection)->&mut Self {
        self.configuration.with_selection_method(selection_method);
        self
    }

    //Crossover configuration
    fn with_crossover_number_of_points(&mut self, number_of_points: i32)->&mut Self {
        self.configuration.with_crossover_number_of_points(number_of_points);
        self
    }
    fn with_crossover_probability_max(&mut self, probability_max: f64)->&mut Self {
        self.configuration.with_crossover_probability_max(probability_max);
        self
    }
    fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self {
        self.configuration.with_crossover_probability_min(probability_min);
        self
    }
    fn with_crossover_method(&mut self, method: crossover::Crossover) -> &mut Self {
        self.configuration.with_crossover_method(method);
        self
    }

    //Mutation configuration
    fn with_mutation_probability_max(&mut self, probability_max: f64)->&mut Self {
        self.configuration.with_mutation_probability_max(probability_max);
        self
    }
    fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self {
        self.configuration.with_mutation_probability_min(probability_min);
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
        self.configuration.with_save_progress_interval(save_progress_interval);
        self
    }
    fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self {
        self.configuration.with_save_progress_path(save_progress_path);
        self
    }
}


impl<U>Ga<U>
where
    U:ChromosomeT + Send + Sync + 'static + Clone,
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
            self.configuration.selection_configuration.number_of_couples = ((self.population.size() / 2) as f64).round() as i32;
        }
        self
    }

    /**
     * Function to set the fitness function
     */
    pub fn with_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[U::Gene]) -> f64 + Send + Sync + 'static
    {
        self.fitness_fn = Some(Arc::new(fitness_fn));
        self
    }

    /**
    * Sets the initialization function
    */
    pub fn with_initialization_fn<F>(&mut self, initialization_fn: F) -> &mut Self
    where
        U:ChromosomeT + Send + Sync + 'static + Clone,
        F: Fn(i32, Option<&[U::Gene]>, Option<bool>) -> Vec<U::Gene> + Send + Sync + 'static
    {
        self.initialization_fn = Some(Arc::new(initialization_fn));
        self
    }

    /**
     * Function to randomly initialize the population
     */
    pub fn initialization(&mut self) -> &mut Self
    where U:ChromosomeT + Send + Sync + 'static + Clone
    {

        // Before starting initialization, we should verify that initializer is set
        if self.initialization_fn.is_none(){
            panic!("No initialization function set");
        }

        //Before starting the run, we will check the conditions
        condition_checker_factory::<U>(Some(&self.configuration), None, Some(&self.alleles));

        info!("Initialization started");
        let (tx, rx) = sync_channel(self.configuration.number_of_threads as usize);

        //Setting the number of chromosomes per thread
        let chromosomes_per_thread = self.configuration.limit_configuration.population_size / self.configuration.number_of_threads;

        //Cloning the chromosomes and fitness function for multithreading
        let alleles_t = Arc::new(Mutex::new(self.alleles.clone()));

        //Walking through the threads
        for _ in 0..self.configuration.number_of_threads {

            //Cloning the information from the main thread
            let (tx,
            alleles_t,
            genes_per_chromosome_t,
            chromosomes_per_thread_t,
            needs_unique_ids_t,
            initialization_fn_t,
            fitness_fn_t) = (tx.clone(), Arc::clone(&alleles_t),
                             self.configuration.limit_configuration.genes_per_chromosome,
                             chromosomes_per_thread,
                             self.configuration.limit_configuration.needs_unique_ids,
                             self.initialization_fn.clone().unwrap(),
                             self.fitness_fn.clone().unwrap());

            //Starting the thread management
            thread::spawn(move || {

                let mut chromosomes = Vec::new();

                for _ in 0..chromosomes_per_thread_t {

                    let mut chromosome = U::new();

                    //Gets the dna randomly
                    let dna_chromosome = (initialization_fn_t)(genes_per_chromosome_t, Some(&alleles_t.lock().unwrap()), Some(needs_unique_ids_t));
                    chromosome.set_dna(dna_chromosome.as_slice());


                    // Wrap the fitness function in a closure
                    let fitness_fn = {
                        let fitness_fn_t = fitness_fn_t.clone();
                        move |genes: &[U::Gene]| (fitness_fn_t)(genes)
                    };

                    //Sets the dna of the chromosome, the age, sets the fitness fn and calculates fitness
                    chromosome.set_age(0);
                    chromosome.set_fitness_fn(fitness_fn);
                    chromosome.calculate_fitness();

                    //Adds the chromosome in the vector
                    chromosomes.push(chromosome);

                }

                //we send the chromosomes randomly initialized
                tx.send(chromosomes).unwrap();
            });
        }

        drop(tx);

        // We receive from the threads and add them into chromosomes
        let mut chromosomes = Vec::new();
        for mut received in rx {
            chromosomes.append(&mut received);
        }

        self.with_population(Population::new(chromosomes));
        self

    }

    pub fn run(&mut self)->&Population<U>{
        self.run_with_callback(None::<fn(&i32, &Population<U>, &TerminationCause)>, 0)
    }

    /**
     * Method for running the Genetic Algorithms with callback
     */
    pub fn run_with_callback<F>(&mut self, callback: Option<F>, generations_to_callback: i32)->&Population<U>
    where 
        U:ChromosomeT + Send + Sync + 'static + Clone,
        F: Fn(&i32, &Population<U>, &TerminationCause)
    {
        //Before starting the run, we will check the conditions
        condition_checker_factory::<U>(Some(&self.configuration), Some(&self.population), Some(&self.alleles));

        //If we want to initialize the population randomly
        if self.population.size() == 0 && self.initialization_fn.is_some() {
            self.initialization();
        } else if self.population.size() == 0 && self.initialization_fn.is_none() {
            panic!("No initialization function set");
        }

        //We set the environment variable from the configuration value
        let key = "RUST_LOG";
        let log_level = match self.configuration.log_level{
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
        if self.configuration.adaptive_ga{
            self.population.recalculate_aga();
        }

        //Best chromosome within the generations and population returned
        let initial_population_size = self.population.size();
        let mut age = 0;

        //Calculation of the fitness and the best chromosome
        self.population.fitness_calculation(self.configuration.number_of_threads, self.configuration.limit_configuration.problem_solving);

        // Starting counting the generations for the callback
        let mut generation_callback_count = 0;

        //We start the cycles
        for i in 0..self.configuration.limit_configuration.max_generations {

            info!(target="ga_events", method="run"; "Generation number: {}", i+1);
            age += 1;

            //1- Parent selection for reproduction
            let mut parents = selection::factory(&self.population.chromosomes, self.configuration.selection_configuration, self.configuration.number_of_threads);
            debug!(target="ga_events", method="run"; "Parents selected for reproduction");

            //2- Getting the offspring
            let mut offspring = parent_crossover(&mut parents, &self.population.chromosomes, &self.configuration, age, self.population.f_max, self.population.f_avg);
            debug!(target="ga_events", method="run"; "Offspring created");

            //3- Sets the best chromosome
            for child in &offspring{
                self.population.decide_best_chromosome(child, self.configuration.limit_configuration.problem_solving);
            }
            debug!(target="ga_events", method="run"; "Best chromosome calculated - generation {}", i+1);

            //4- Insert the children in the population
            self.population.add_chromosomes(&mut offspring);

            //5- Survivor selection
            survivor::factory(self.configuration.survivor, &mut self.population.chromosomes, initial_population_size, self.configuration.limit_configuration);
            if self.configuration.adaptive_ga{
                self.population.recalculate_aga();
            }
            debug!(target="ga_events", method="run"; "Survivors selected");

            // If we want to perform a callback
            if let Some(func) = &callback {
                if (generation_callback_count+1) == generations_to_callback {
                    func(&i, &self.population, &self.termination_cause);
                    generation_callback_count = 0;
                } else {
                    generation_callback_count+=1;
                }
            }

            //6- Identifies if the limit has been reached or not
            if limit_reached(self.configuration.limit_configuration, &self.population.chromosomes){

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
                func(&self.configuration.limit_configuration.max_generations, &self.population, &self.termination_cause);
            }
        }

        &self.population
    }
}

/**
 * Function to identify if the limit has been reached or not in the current generation
 */
fn limit_reached<U>(limit: LimitConfiguration, chromosomes: &Vec<U>) ->bool
where
U:ChromosomeT
{

    debug!(target="ga_events", method="limit_reached"; "Started limit reached method");
    let mut result = false;

    if limit.problem_solving == ProblemSolving::Minimization{
        //If the problem-solving is minimization, fitness must be 0
        for chromosome in chromosomes {
            if chromosome.get_fitness() == 0.0 {
                trace!(target="ga_events", method="limit_reached"; "limit reached for minimization");
                result = true;
                break;
            }
        }
    }else if limit.problem_solving == ProblemSolving::FixedFitness{

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

/**
 * Function for parent crossover
 */
fn parent_crossover<U>(parents: &mut HashMap<usize, usize>, chromosomes: &Vec<U>, configuration: &GaConfiguration, age: i32, f_max: f64, f_avg: f64) -> Vec<U>
where 
U:ChromosomeT + Send + Sync + 'static + Clone
{
    //Setting the control variables
    debug!(target="ga_events", method="parent_crossover"; "Started the parent crossover");
    let number_of_threads = if configuration.number_of_threads < parents.len() as i32 {parents.len() as i32}
        else if configuration.number_of_threads > 0 {configuration.number_of_threads} else {1};
    let jump = parents.len() / number_of_threads as usize;

    let mut handles = Vec::new();
    let offspring = Arc::new(Mutex::new(Vec::new()));

    /*
        Gets the static crossover probability config and the static mutation probability config
        This way we avoid of passing by these conditions at each thread if it's not necessary
    */
    let crossover_probability_config = 
            if configuration.crossover_configuration.probability_max.is_none(){
                Some(1.0)
            }else if !configuration.adaptive_ga{
                Some(configuration.crossover_configuration.probability_max.unwrap())
            }else{
                    None
            };

    let mutation_probability_config =
            if configuration.mutation_configuration.probability_max.is_none(){
                Some(1.0)
            }else if !configuration.adaptive_ga{
                Some(configuration.mutation_configuration.probability_max.unwrap())
            }else{
                None
            };

    //Run all the threads
    for t in 0..number_of_threads{

        //We copy the parents that we want to crossover inside the thread
        let (chromosomes, configuration, offspring, crossover_probability_config, mutation_probability_config) = (chromosomes.clone(), configuration.clone(), Arc::clone(&offspring), crossover_probability_config, mutation_probability_config);
        let mut parents_t = HashMap::new();
        let parents_c = parents.clone();

        for (index, i) in parents_c.keys().enumerate(){

            //If we reach the number of crossovers / thread
            if t < number_of_threads - 1 && index >= jump {
                break;
            }

            let key = *parents.get_key_value(i).unwrap().0;
            parents_t.insert(key, *parents.get_key_value(i).unwrap().1);
            parents.remove(&key);
        }

        //Starts the thread
        let handle = thread::spawn(move || {

            //Getting random numbers in this thread
            let mut rng = rand::thread_rng();

            for(key, value) in parents_t.iter(){
                //Getting the parent 1 and 2 for crossover                
                let parent_1 = chromosomes.get(*key).unwrap().clone();
                let parent_2 = chromosomes.get(*value).unwrap().clone();

                //Making the crossover of the parents when the random number is below or equal to the given probability
                let crossover_probability = rng.gen_range(0.0..1.0);
                let crossover_probability_config = 
                    if crossover_probability_config.is_some(){
                        crossover_probability_config.unwrap()
                    }else{
                        crossover::aga_probability(&parent_1, &parent_2, f_max, f_avg, configuration.crossover_configuration.probability_max.unwrap(), configuration.crossover_configuration.probability_min.unwrap())
                    };
                

                //Making the mutation of each child when the random number is below or equal the given probability
                let mut mutation_probability = rng.gen_range(0.0..1.0);
                let mutation_probability_config = 
                    if mutation_probability_config.is_some(){
                        mutation_probability_config.unwrap()
                    }else{
                        mutation::aga_probability(&parent_1, &parent_2, f_avg, configuration.mutation_configuration.probability_max.unwrap(), configuration.mutation_configuration.probability_min.unwrap())
                    };

                debug!(target="ga_events", method="parent_crossover"; "Started the parent crossover");

                let mut child_1: U;
                let mut child_2: U;
                let mut offspring_t: Vec<U> = vec![];

                if crossover_probability <= crossover_probability_config {
                    offspring_t = crossover::factory(&parent_1, &parent_2, configuration.crossover_configuration).unwrap();
                    child_1 = offspring_t.pop().unwrap();
                    child_2 = offspring_t.pop().unwrap();
                }else{
                    child_1 = parent_1;
                    child_2 = parent_2;
                }
                
                if configuration.mutation_configuration.probability_max.is_none(){1.0}else{configuration.mutation_configuration.probability_max.unwrap()};
                debug!(target="ga_events", method="parent_crossover"; "mutation_probability_config {} - mutation probability {}", mutation_probability_config, mutation_probability);

                if mutation_probability < mutation_probability_config {
                    mutation::factory(configuration.mutation_configuration.method, &mut child_1);
                }

                mutation_probability = rng.gen_range(0.0..1.0);
                if mutation_probability <= mutation_probability_config {
                    mutation::factory(configuration.mutation_configuration.method, &mut child_2);
                }

                //Calculate the fitness of both children and set their age
                child_1.calculate_fitness();
                child_2.calculate_fitness();

                child_1.set_age(age);
                child_2.set_age(age);

                //Adds the children in the offspring
                offspring_t.push(child_1);
                offspring_t.push(child_2);
                
                //Then sets the offspring in the result vector
                offspring.lock().unwrap().append(&mut offspring_t);
            }
            
        });
        handles.push(handle);
    }

    //Joining all the threads
    for handle in handles{
        handle.join().unwrap();
    }

    debug!(target="ga_events", method="parent_crossover"; "Parent crossover finished");
    return offspring.lock().unwrap().to_vec();
}