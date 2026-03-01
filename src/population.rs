use rayon::prelude::*;
use log::{debug, trace};
use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;

/// Population of chromosomes with aggregate statistics and best tracking.
///
/// Responsibilities:
/// - Store the evolving set of chromosomes.
/// - Maintain best chromosome according to the configured problem objective.
/// - Compute average (f_avg) and maximum (f_max) fitness (used by adaptive GA).
/// - Parallel fitness calculation to leverage multiple threads.
pub struct Population<U>
where
    U: ChromosomeT
{
    /// The chromosomes (members) of the population.
    pub chromosomes: Vec<U>,

    /// Best chromosome in the population (by fitness, according to `ProblemSolving`).
    pub best_chromosome: U,
    best_chromosome_is_set: bool,

    /// Generation numbers associated with this population (optional tracking).
    pub generation_numbers: Vec<i32>,

    /// Average fitness across the population.
    pub f_avg: f64,

    /// Largest fitness value in the population.
    pub f_max: f64,
}

impl<U> Population<U>
where
    U: ChromosomeT
{
    /// Creates a new empty `Population`.
    pub fn new_empty() -> Population<U> {
        Population { chromosomes: vec![], generation_numbers: vec![], f_avg: 0.0, f_max: 0.0,
                     best_chromosome: U::new(), best_chromosome_is_set: false }
    }

    /// Creates a `Population` with the given chromosomes as members.
    pub fn new(chromosomes: Vec<U>) -> Population<U> {
        Population {
            chromosomes, generation_numbers: vec![], f_avg: 0.0, f_max: 0.0,
                     best_chromosome: U::new(), best_chromosome_is_set: false }
    }

    /// Recalculate `f_avg` and `f_max` (used by adaptive GA probabilities).
    pub fn recalculate_aga(& mut self){
        self.f_max = 0.0;
        self.f_avg = 0.0;
        for chromosome in self.chromosomes.as_slice(){
            self.f_max = if chromosome.get_fitness() > self.f_max { chromosome.get_fitness()} else{self.f_max};
            self.f_avg += chromosome.get_fitness();
        }
        self.f_avg /= self.chromosomes.len() as f64;
    }

    /// Appends a list of chromosomes into the current population.
    pub fn add_chromosomes(&mut self, chromosomes: &mut Vec<U>){
        self.chromosomes.append(chromosomes);
    }

    /// Returns the number of chromosomes in the population.
    pub fn size(&self) -> usize {
        self.chromosomes.len()
    }

    /// Computes fitness for all chromosomes in parallel and updates the best chromosome.
    ///
    /// Uses rayon's parallel iterators for efficient work distribution.
    /// If a chromosome already has non-zero fitness, it is reused.
    pub fn fitness_calculation(&mut self, _number_of_threads: i32, problem_solving: ProblemSolving)
    {
        debug!(target="population_events", method="fitness_calculation"; "Started the population fitness calculation");

        // Calculate fitness in parallel for chromosomes with fitness == 0.0
        self.chromosomes.par_iter_mut().for_each(|chromosome| {
            if chromosome.get_fitness() == 0.0 {
                chromosome.calculate_fitness();
            }
        });

        // Update best chromosome sequentially (needs &mut self)
        for i in 0..self.chromosomes.len() {
            let chromosome = self.chromosomes[i].clone();
            self.decide_best_chromosome(&chromosome, problem_solving);
        }

        debug!(target="ga_events", method="population_fitness_calculation"; "Population fitness calculation finished");
    }

    /// Update the best chromosome given a candidate, according to the problem objective.
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