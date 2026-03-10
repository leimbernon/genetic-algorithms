/*!
# Genetic Algorithm Engine

This module provides the main `Ga` struct and related functionality for running genetic algorithms.

# Examples

```rust
use genetic_algorithms::ga::Ga;
let ga = Ga::new();
```
*/

use crate::chromosomes::{Chromosome, ChromosomeFactory};
use crate::configuration::{Configuration, ProblemSolving};
use crate::fitness::{FitnessFn, FitnessValue};
use crate::initializers::{InitializerFn, InitializerResult};
use crate::operations::{Crossover, Mutation, Selection, Survivor};
use crate::population::{Population, PopulationFactory};
use crate::stats::{GenerationStats, StatsCollector};
use crate::error::GaError;
use log::{debug, info, trace};

/// Termination cause for the genetic algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminationCause {
    /// Maximum number of generations reached.
    MaxGenerations,
    /// Target fitness reached.
    FitnessTargetReach,
    /// No improvement for N generations.
    NoImprovement,
    /// Custom user-defined termination.
    Custom,
    /// Other termination cause.
    Other(String),
}

/// Result of running the genetic algorithm.
#[derive(Debug)]
pub struct GaResult<C: Chromosome> {
    /// Final population.
    pub population: Population<C>,
    /// Statistics for each generation.
    pub stats: Vec<GenerationStats>,
    /// Cause of termination.
    pub termination_cause: TerminationCause,
}

/// Genetic Algorithm Engine.
pub struct Ga<C: Chromosome> {
    config: Configuration<C>,
}

impl<C: Chromosome> Ga<C> {
    /// Create a new GA builder.
    pub fn new() -> Self {
        Self {
            config: Configuration::default(),
        }
    }

    /// Set the number of genes per chromosome.
    pub fn with_genes_per_chromosome(mut self, n: usize) -> Self {
        self.config.genes_per_chromosome = n;
        self
    }

    /// Set the population size.
    pub fn with_population_size(mut self, n: usize) -> Self {
        self.config.population_size = n;
        self
    }

    /// Set the initialization function.
    pub fn with_initialization_fn(mut self, f: InitializerFn<C>) -> Self {
        self.config.initialization_fn = Some(f);
        self
    }

    /// Set the fitness function.
    pub fn with_fitness_fn(mut self, f: FitnessFn<C>) -> Self {
        self.config.fitness_fn = Some(f);
        self
    }

    /// Set the selection method.
    pub fn with_selection_method(mut self, sel: Selection) -> Self {
        self.config.selection_method = sel;
        self
    }

    /// Set the crossover method.
    pub fn with_crossover_method(mut self, cross: Crossover) -> Self {
        self.config.crossover_method = cross;
        self
    }

    /// Set the mutation method.
    pub fn with_mutation_method(mut self, muta: Mutation) -> Self {
        self.config.mutation_method = muta;
        self
    }

    /// Set the survivor selection method.
    pub fn with_survivor_method(mut self, surv: Survivor) -> Self {
        self.config.survivor_method = surv;
        self
    }

    /// Set the problem solving mode.
    pub fn with_problem_solving(mut self, mode: ProblemSolving) -> Self {
        self.config.problem_solving = mode;
        self
    }

    /// Set the fitness target.
    pub fn with_fitness_target(mut self, target: FitnessValue) -> Self {
        self.config.fitness_target = Some(target);
        self
    }

    /// Set the maximum number of generations.
    pub fn with_max_generations(mut self, max: usize) -> Self {
        self.config.max_generations = max;
        self
    }

    /// Build the GA configuration.
    pub fn build(self) -> Result<Self, GaError> {
        // Validate configuration
        self.config.validate()?;
        Ok(self)
    }

    /// Run the genetic algorithm.
    pub fn run(&mut self) -> GaResult<C> {
        self.run_with_callback(None)
    }

    /// Run the genetic algorithm with a progress callback.
    ///
    /// # Arguments
    /// * `callback` - Optional callback function called at each generation.
    ///
    /// # Returns
    /// * `GaResult<C>` - Result of the GA run.
    pub fn run_with_callback<F>(&mut self, callback: Option<F>) -> GaResult<C>
    where
        F: Fn(&usize, &Population<C>, &GenerationStats) + 'static,
    {
        // Initialization
        let mut population = PopulationFactory::new(&self.config).initialize();
        let mut stats_collector = StatsCollector::new();

        let mut best_fitness = None;
        let mut no_improvement_count = 0;
        let mut termination_cause = TerminationCause::MaxGenerations;

        for gen in 1..=self.config.max_generations {
            // Evaluate fitness
            population.evaluate_fitness(&self.config.fitness_fn);

            // Collect stats
            let stats = stats_collector.collect(&population, gen);
            if let Some(ref cb) = callback {
                cb(&gen, &population, &stats);
            }

            // Check termination
            match self.config.problem_solving {
                ProblemSolving::FixedFitness => {
                    if let Some(target) = self.config.fitness_target {
                        if stats.best_fitness >= target {
                            termination_cause = TerminationCause::FitnessTargetReach;
                            break;
                        }
                    }
                }
                _ => {}
            }

            // Survivor selection, crossover, mutation, etc.
            population = population.next_generation(&self.config);

            // Track improvement
            if let Some(bf) = best_fitness {
                if stats.best_fitness > bf {
                    best_fitness = Some(stats.best_fitness);
                    no_improvement_count = 0;
                } else {
                    no_improvement_count += 1;
                }
            } else {
                best_fitness = Some(stats.best_fitness);
            }
        }

        GaResult {
            population,
            stats: stats_collector.into_vec(),
            termination_cause,
        }
    }
}
