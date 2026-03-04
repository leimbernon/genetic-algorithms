use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;
use log::{debug, trace};
use rayon::prelude::*;

/// Population of chromosomes with aggregate statistics and best tracking.
///
/// Responsibilities:
/// - Store the evolving set of chromosomes.
/// - Maintain best chromosome according to the configured problem objective.
/// - Compute average (f_avg) and maximum (f_max) fitness (used by adaptive GA).
/// - Parallel fitness calculation to leverage multiple threads.
pub struct Population<U>
where
    U: ChromosomeT,
{
    /// The chromosomes (members) of the population.
    pub chromosomes: Vec<U>,

    /// Best chromosome in the population (by fitness, according to `ProblemSolving`).
    pub best_chromosome: U,
    pub(crate) best_chromosome_is_set: bool,

    /// Generation numbers associated with this population (optional tracking).
    pub generation_numbers: Vec<usize>,

    /// Average fitness across the population.
    pub f_avg: f64,

    /// Largest fitness value in the population.
    pub f_max: f64,
}

impl<U> Population<U>
where
    U: ChromosomeT,
{
    /// Creates a new empty `Population`.
    pub fn new_empty() -> Population<U> {
        Population {
            chromosomes: vec![],
            generation_numbers: vec![],
            f_avg: 0.0,
            f_max: 0.0,
            best_chromosome: U::new(),
            best_chromosome_is_set: false,
        }
    }

    /// Creates a `Population` with the given chromosomes as members.
    pub fn new(chromosomes: Vec<U>) -> Population<U> {
        Population {
            chromosomes,
            generation_numbers: vec![],
            f_avg: 0.0,
            f_max: 0.0,
            best_chromosome: U::new(),
            best_chromosome_is_set: false,
        }
    }

    /// Recalculate `f_avg` and `f_max` (used by adaptive GA probabilities).
    pub fn recalculate_aga(&mut self) {
        self.f_max = f64::NEG_INFINITY;
        self.f_avg = 0.0;
        for chromosome in self.chromosomes.as_slice() {
            self.f_max = if chromosome.fitness() > self.f_max {
                chromosome.fitness()
            } else {
                self.f_max
            };
            self.f_avg += chromosome.fitness();
        }
        self.f_avg /= self.chromosomes.len() as f64;
    }

    /// Appends a list of chromosomes into the current population.
    pub fn add_chromosomes(&mut self, chromosomes: &mut Vec<U>) {
        self.chromosomes.append(chromosomes);
    }

    /// Returns the number of chromosomes in the population.
    pub fn size(&self) -> usize {
        self.chromosomes.len()
    }

    /// Computes fitness for unevaluated chromosomes in parallel and updates the best chromosome.
    ///
    /// Uses rayon's parallel iterators for efficient work distribution.
    /// A chromosome is considered unevaluated if its fitness is `NaN`.
    pub fn fitness_calculation(
        &mut self,
        _number_of_threads: usize,
        problem_solving: ProblemSolving,
    ) {
        debug!(target="population_events", method="fitness_calculation"; "Started the population fitness calculation");

        // Calculate fitness in parallel for chromosomes that have not yet been evaluated.
        // NaN fitness indicates a chromosome whose fitness has never been computed.
        self.chromosomes.par_iter_mut().for_each(|chromosome| {
            if chromosome.fitness().is_nan() {
                chromosome.calculate_fitness();
            }
        });

        // Update best chromosome: scan for the best fitness among all
        // chromosomes and clone only the winner (instead of cloning every
        // chromosome in the loop).
        if let Some(best_idx) = find_best_index(&self.chromosomes, problem_solving) {
            if !self.best_chromosome_is_set {
                self.best_chromosome = self.chromosomes[best_idx].clone();
                self.best_chromosome_is_set = true;
            } else {
                let candidate_fitness = self.chromosomes[best_idx].fitness();
                let current_fitness = self.best_chromosome.fitness();
                let candidate_is_better = match problem_solving {
                    ProblemSolving::Maximization | ProblemSolving::FixedFitness => {
                        candidate_fitness > current_fitness
                    }
                    ProblemSolving::Minimization => candidate_fitness < current_fitness,
                };
                if candidate_is_better {
                    self.best_chromosome = self.chromosomes[best_idx].clone();
                }
            }
        }

        debug!(target="ga_events", method="population_fitness_calculation"; "Population fitness calculation finished");
    }

    /// Update the best chromosome given a candidate, according to the problem objective.
    pub fn decide_best_chromosome(&mut self, new_chromosome: &U, problem_solving: ProblemSolving) {
        debug!(target="population_events", method="decide_best_chromosome"; "Started the best chromosome method");

        if !self.best_chromosome_is_set {
            self.best_chromosome = new_chromosome.clone();
            self.best_chromosome_is_set = true;
        } else {
            trace!(target="population_events", method="decide_best_chromosome"; "Best chromosome fitness: {} - New chromosome fitness: {}",
                self.best_chromosome.fitness(), new_chromosome.fitness());

            let is_self_better = match problem_solving {
                ProblemSolving::Maximization => {
                    self.best_chromosome.fitness() >= new_chromosome.fitness()
                }
                ProblemSolving::Minimization => {
                    self.best_chromosome.fitness() < new_chromosome.fitness()
                }
                _ => self.best_chromosome.fitness() >= new_chromosome.fitness(),
            };

            if !is_self_better {
                self.best_chromosome = new_chromosome.clone();
            };
        }

        debug!(target="chromosome_events", method="get_best_chromosome"; "Best chromosome method finished");
    }
}

/// Finds the index of the best chromosome in a slice according to `problem_solving`.
///
/// Returns `None` for an empty slice.
fn find_best_index<U: ChromosomeT>(
    chromosomes: &[U],
    problem_solving: ProblemSolving,
) -> Option<usize> {
    if chromosomes.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_fit = chromosomes[0].fitness();
    for (i, c) in chromosomes.iter().enumerate().skip(1) {
        let fit = c.fitness();
        let is_better = match problem_solving {
            ProblemSolving::Maximization | ProblemSolving::FixedFitness => fit > best_fit,
            ProblemSolving::Minimization => fit < best_fit,
        };
        if is_better {
            best_idx = i;
            best_fit = fit;
        }
    }
    Some(best_idx)
}
