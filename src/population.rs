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
    // The chromosomes or members of the population.
    pub chromosomes: Vec<U>,

    // Best chromosome of the population
    pub best_chromosome: U,
    best_chromosome_is_set: bool,

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
        Population { chromosomes: vec![], generation_numbers: vec![], f_avg: 0.0, f_max: 0.0,
                     best_chromosome: U::new(), best_chromosome_is_set: false }
    }

    // Creates a new `Population` with the given chromosomes as members.
    pub fn new(chromosomes: Vec<U>) -> Population<U> {
        Population {
            chromosomes, generation_numbers: vec![], f_avg: 0.0, f_max: 0.0,
                     best_chromosome: U::new(), best_chromosome_is_set: false }
    }

    // Function to calculate f_avg and f_max
    pub fn recalculate_aga(& mut self){
        self.f_max = 0.0;
        self.f_avg = 0.0;
        for chromosome in self.chromosomes.as_slice(){
            self.f_max = if chromosome.get_fitness() > self.f_max { chromosome.get_fitness()} else{self.f_max};
            self.f_avg += chromosome.get_fitness();
        }
        self.f_avg /= self.chromosomes.len() as f64;
    }

    //Function to add chromosomes in the list
    pub fn add_chromosomes(&mut self, chromosomes: &mut Vec<U>){
        self.chromosomes.append(chromosomes);
    }

    // Returns the number of chromosomes in the population.
    pub fn size(&self) -> usize {
        self.chromosomes.len()
    }

    // Population fitness calculation
    pub fn fitness_calculation(&mut self, number_of_threads: i32, problem_solving: ProblemSolving)
    {
        debug!(target="population_events", method="fitness_calculation"; "Started the population fitness calculation");
        let (tx, rx) = sync_channel(number_of_threads as usize);

        //Division of the chromosomes in different threads
        let number_of_threads = if number_of_threads > self.chromosomes.len() as i32 {self.chromosomes.len() as i32} else {number_of_threads};

        //Setting the starting point and the jump
        let mut start_index = 0;
        let mut jump = self.chromosomes.len() as i32 / number_of_threads;

        //Cloning the chromosomes for multithreading
        let chromosomes_t = Vec::from_iter(self.chromosomes[..].iter().cloned());
        let chromosomes_t = Arc::new(Mutex::new(chromosomes_t));

        //Walking through the threads
        for _ in 0..number_of_threads {

            //We calculate the next jump
            if jump > self.chromosomes.len() as i32 - (start_index + jump) {
                jump += self.chromosomes.len() as i32 - (start_index + jump);
            }

            //Cloning the information from the main thread
            let (start_index_t, tx, jump_t, chromosomes_t) = (start_index, tx.clone(), jump, Arc::clone(&chromosomes_t));

            //Starting the thread management
            thread::spawn(move || {

                let mut fitness_map = HashMap::new();

                //Calculates the fitness from the corresponding population
                for i in start_index_t..(start_index_t + jump_t){

                    // If the fitness is 0.0 (maybe it has not been already calculated)
                    if chromosomes_t.lock().unwrap()[i as usize].get_fitness() == 0.0 {
                        chromosomes_t.lock().unwrap()[i as usize].calculate_fitness();
                    }
                    fitness_map.insert(i as usize, chromosomes_t.lock().unwrap()[i as usize].get_fitness());
                }

                //Sending the result
                tx.send(fitness_map).unwrap();
            });

            start_index += jump;
        }

        drop(tx);

        //We receive from the threads and set the fitness in chromosomes
        for received in rx {
            for element in received{
                self.chromosomes[element.0].set_fitness(element.1);
                self.decide_best_chromosome(&self.chromosomes[element.0].clone(), problem_solving);
            }
        }
        debug!(target="ga_events", method="population_fitness_calculation"; "Population fitness calculation finished");
    }

    pub fn decide_best_chromosome(&mut self, new_chromosome: &U, problem_solving: ProblemSolving)
    {
        debug!(target="population_events", method="decide_best_chromosome"; "Started the best chromosome method");

        if !self.best_chromosome_is_set {
            self.best_chromosome = new_chromosome.clone();
            self.best_chromosome_is_set = true;
        }else {
            trace!(target="population_events", method="decide_best_chromosome"; "Best chromosome fitness: {} - New chromosome fitness: {}",
                self.best_chromosome.get_fitness(), new_chromosome.get_fitness());

            let is_self_better = match problem_solving {
                ProblemSolving::Maximization => self.best_chromosome.get_fitness() >= new_chromosome.get_fitness(),
                ProblemSolving::Minimization => self.best_chromosome.get_fitness() < new_chromosome.get_fitness(),
                _ => self.best_chromosome.get_fitness() >= new_chromosome.get_fitness(),
            };

            if !is_self_better {
                self.best_chromosome = new_chromosome.clone();
            };
        }

        debug!(target="chromosome_events", method="get_best_chromosome"; "Best chromosome method finished");
    }
}