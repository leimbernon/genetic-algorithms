use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::sync_channel;
use std::thread;
use log::{debug, trace};
use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;

pub struct Population<U>
where
    U: ChromosomeT
{
    // The individuals or members of the population.
    pub individuals: Vec<U>,

    // Best individual of the population
    pub best_individual: U,
    best_individual_is_set: bool,

    //The numbers of the generation of this population
    pub generation_numbers: Vec<i32>,

    //Average fitness of the population
    pub f_avg: f64,

    //Population largest fitness value
    pub f_max: f64,
}

impl<U> Population<U>
where
    U: ChromosomeT
{
    // Creates a new empty `Population`
    pub fn new_empty() -> Population<U> {
        Population { individuals: vec![], generation_numbers: vec![], f_avg: 0.0, f_max: 0.0,
                     best_individual: U::new(), best_individual_is_set: false }
    }

    // Creates a new `Population` with the given individuals as members.
    pub fn new(individuals: Vec<U>) -> Population<U> {
        Population { individuals, generation_numbers: vec![], f_avg: 0.0, f_max: 0.0,
                     best_individual: U::new(), best_individual_is_set: false }
    }

    // Function to calculate f_avg and f_max
    pub fn recalculate_aga(& mut self){
        self.f_max = 0.0;
        self.f_avg = 0.0;
        for individual in self.individuals.as_slice(){
            self.f_max = if individual.get_fitness() > self.f_max {individual.get_fitness()} else{self.f_max};
            self.f_avg += individual.get_fitness();
        }
        self.f_avg /= self.individuals.len() as f64;
    }

    //Function to add individuals in the list
    pub fn add_individuals(&mut self, individuals: &mut Vec<U>){
        self.individuals.append(individuals);
    }

    // Returns the number of individuals in the population.
    pub fn size(&self) -> usize {
        self.individuals.len()
    }

    // Population fitness calculation
    pub fn fitness_calculation(&mut self, number_of_threads: i32, problem_solving: ProblemSolving)
    {
        debug!(target="population_events", method="fitness_calculation"; "Started the population fitness calculation");
        let (tx, rx) = sync_channel(number_of_threads as usize);

        //Division of the individuals in different threads
        let number_of_threads = if number_of_threads > self.individuals.len() as i32 {self.individuals.len() as i32} else {number_of_threads};

        //Setting the starting point and the jump
        let mut start_index = 0;
        let mut jump = self.individuals.len() as i32 / number_of_threads;

        //Cloning the individuals for multithreading
        let individuals_t = Vec::from_iter(self.individuals[..].iter().cloned());
        let individuals_t = Arc::new(Mutex::new(individuals_t));

        //Walking through the threads
        for _ in 0..number_of_threads {

            //We calculate the next jump
            if jump > self.individuals.len() as i32 - (start_index + jump) {
                jump += self.individuals.len() as i32 - (start_index + jump);
            }

            //Cloning the information from the main thread
            let (start_index_t, tx, jump_t, individuals_t) = (start_index, tx.clone(),  jump, Arc::clone(&individuals_t));

            //Starting the thread management
            thread::spawn(move || {

                let mut fitness_map = HashMap::new();

                //Calculates the fitness from the corresponding population
                for i in start_index_t..(start_index_t + jump_t){

                    // If the fitness is 0.0 (maybe it has not been already calculated)
                    if individuals_t.lock().unwrap()[i as usize].get_fitness() == 0.0 {
                        individuals_t.lock().unwrap()[i as usize].calculate_fitness();
                    }
                    fitness_map.insert(i as usize, individuals_t.lock().unwrap()[i as usize].get_fitness());
                }

                //Sending the result
                tx.send(fitness_map).unwrap();
            });

            start_index += jump;
        }

        drop(tx);

        //We receive from the threads and set the fitness in individuals
        for received in rx {
            for element in received{
                self.individuals[element.0].set_fitness(element.1);
                self.decide_best_individual(&self.individuals[element.0].clone(), problem_solving);
            }
        }
        debug!(target="ga_events", method="population_fitness_calculation"; "Population fitness calculation finished");
    }

    pub fn decide_best_individual(&mut self, new_individual: &U, problem_solving: ProblemSolving)
    {
        debug!(target="population_events", method="decide_best_individual"; "Started the best individual method");

        if !self.best_individual_is_set{
            self.best_individual = new_individual.clone();
            self.best_individual_is_set = true;
        }else {
            trace!(target="population_events", method="decide_best_individual"; "Best individual fitness: {} - New indivicual fitness: {}",
                self.best_individual.get_fitness(), new_individual.get_fitness());

            let is_self_better = match problem_solving {
                ProblemSolving::Maximization => self.best_individual.get_fitness() >= new_individual.get_fitness(),
                ProblemSolving::Minimization => self.best_individual.get_fitness() < new_individual.get_fitness(),
                _ => self.best_individual.get_fitness() >= new_individual.get_fitness(),
            };

            if !is_self_better {
                self.best_individual = new_individual.clone();
            };
        }

        debug!(target="chromosome_events", method="get_best_chromosome"; "Best chromosome method finished");
    }
}