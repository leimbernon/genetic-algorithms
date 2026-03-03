use std::fmt;

use crate::{
    operations::{Crossover, Mutation, Selection, Survivor},
    traits::ConfigurationT,
};

#[derive(Copy, Clone, PartialEq)]
pub enum ProblemSolving {
    Minimization,
    Maximization,
    FixedFitness,
}
impl fmt::Display for ProblemSolving {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProblemSolving::Minimization => write!(f, "Minimization"),
            ProblemSolving::Maximization => write!(f, "Maximization"),
            ProblemSolving::FixedFitness => write!(f, "FixedFitness"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Copy, Clone)]
pub struct SelectionConfiguration {
    pub number_of_couples: i32,
    pub method: Selection,
}
impl Default for SelectionConfiguration {
    fn default() -> Self {
        SelectionConfiguration {
            number_of_couples: 0,
            method: Selection::Tournament,
        }
    }
}

#[derive(Copy, Clone)]
pub struct CrossoverConfiguration {
    pub number_of_points: Option<i32>,
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Crossover,
    /// Distribution index for SBX crossover. Higher values produce children
    /// closer to parents. Typical range: 2–20. Default is 2.0.
    pub sbx_eta: Option<f64>,
    /// Alpha parameter for BLX-α crossover. Controls exploration range.
    /// Typical value: 0.5. Default is 0.5.
    pub blend_alpha: Option<f64>,
}
impl Default for CrossoverConfiguration {
    fn default() -> Self {
        CrossoverConfiguration {
            number_of_points: None,
            probability_max: None,
            probability_min: None,
            method: Crossover::Uniform,
            sbx_eta: None,
            blend_alpha: None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct MutationConfiguration {
    pub probability_max: Option<f64>,
    pub probability_min: Option<f64>,
    pub method: Mutation,
    /// Step size for Creep mutation. Only used when method is `Mutation::Creep`.
    /// Default is 1.0.
    pub step: Option<f64>,
    /// Standard deviation for Gaussian mutation. Only used when method is `Mutation::Gaussian`.
    /// Default is 1.0.
    pub sigma: Option<f64>,
}
impl Default for MutationConfiguration {
    fn default() -> Self {
        MutationConfiguration {
            probability_max: None,
            probability_min: None,
            method: Mutation::Swap,
            step: None,
            sigma: None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LimitConfiguration {
    pub problem_solving: ProblemSolving,
    pub max_generations: i32,
    pub fitness_target: Option<f64>,
    pub population_size: i32,
    pub genes_per_chromosome: i32,
    pub needs_unique_ids: bool,
    pub alleles_can_be_repeated: bool,
}
impl Default for LimitConfiguration {
    fn default() -> Self {
        LimitConfiguration {
            problem_solving: ProblemSolving::Minimization,
            max_generations: 100,
            fitness_target: None,
            population_size: 0,
            genes_per_chromosome: 0,
            needs_unique_ids: false,
            alleles_can_be_repeated: false,
        }
    }
}

#[derive(Clone, Default)]
pub struct SaveProgressConfiguration {
    pub save_progress: bool,
    pub save_progress_interval: i32,
    pub save_progress_path: String,
}

/// Compound stopping criteria for the GA.
///
/// Multiple criteria can be enabled simultaneously. The GA stops when **any** of them is met.
#[derive(Clone, Debug, Default)]
pub struct StoppingCriteria {
    /// Stop after N generations without fitness improvement.
    /// `None` means this criterion is disabled.
    pub stagnation_generations: Option<i32>,
    /// Stop when the fitness standard deviation drops below this threshold.
    /// `None` means this criterion is disabled.
    pub convergence_threshold: Option<f64>,
    /// Stop after the specified elapsed time (in seconds).
    /// `None` means this criterion is disabled.
    pub max_duration_secs: Option<f64>,
}

#[derive(Clone)]
pub struct GaConfiguration {
    pub adaptive_ga: bool,
    pub number_of_threads: i32,
    pub limit_configuration: LimitConfiguration,
    pub selection_configuration: SelectionConfiguration,
    pub crossover_configuration: CrossoverConfiguration,
    pub mutation_configuration: MutationConfiguration,
    pub survivor: Survivor,
    pub log_level: LogLevel,
    pub save_progress_configuration: SaveProgressConfiguration,
    /// Number of best individuals to preserve unchanged between generations (elitism).
    /// Default is 0 (no elitism).
    pub elitism_count: usize,
    /// Compound stopping criteria. These are checked in addition to
    /// max_generations and fitness_target.
    pub stopping_criteria: StoppingCriteria,
}
impl Default for GaConfiguration {
    fn default() -> Self {
        GaConfiguration {
            adaptive_ga: false,
            number_of_threads: 1,
            survivor: Survivor::Fitness,
            log_level: LogLevel::Off,
            limit_configuration: LimitConfiguration {
                ..Default::default()
            },
            selection_configuration: SelectionConfiguration {
                ..Default::default()
            },
            crossover_configuration: CrossoverConfiguration {
                ..Default::default()
            },
            mutation_configuration: MutationConfiguration {
                ..Default::default()
            },
            save_progress_configuration: SaveProgressConfiguration {
                ..Default::default()
            },
            elitism_count: 0,
            stopping_criteria: StoppingCriteria::default(),
        }
    }
}

impl ConfigurationT for GaConfiguration {
    fn new() -> Self {
        Self::default()
    }
    fn with_adaptive_ga(&mut self, adaptive_ga: bool) -> &mut Self {
        self.adaptive_ga = adaptive_ga;
        self
    }
    fn with_threads(&mut self, number_of_threads: i32) -> &mut Self {
        self.number_of_threads = number_of_threads;
        self
    }
    fn with_logs(&mut self, log_level: LogLevel) -> &mut Self {
        self.log_level = log_level;
        self
    }
    fn with_survivor_method(&mut self, method: Survivor) -> &mut Self {
        self.survivor = method;
        self
    }

    //Limit configuration
    fn with_problem_solving(&mut self, problem_solving: ProblemSolving) -> &mut Self {
        self.limit_configuration.problem_solving = problem_solving;
        self
    }
    fn with_max_generations(&mut self, max_generations: i32) -> &mut Self {
        self.limit_configuration.max_generations = max_generations;
        self
    }
    fn with_fitness_target(&mut self, fitness_target: f64) -> &mut Self {
        self.limit_configuration.fitness_target = Some(fitness_target);
        self
    }

    fn with_population_size(&mut self, population_size: i32) -> &mut Self {
        self.limit_configuration.population_size = population_size;
        self
    }
    fn with_genes_per_chromosome(&mut self, genes_per_chromosome: i32) -> &mut Self {
        self.limit_configuration.genes_per_chromosome = genes_per_chromosome;
        self
    }
    fn with_needs_unique_ids(&mut self, needs_unique_ids: bool) -> &mut Self {
        self.limit_configuration.needs_unique_ids = needs_unique_ids;
        self
    }
    fn with_alleles_can_be_repeated(&mut self, alleles_can_be_repeated: bool) -> &mut Self {
        self.limit_configuration.alleles_can_be_repeated = alleles_can_be_repeated;
        self
    }

    //Selection configuration
    fn with_number_of_couples(&mut self, number_of_couples: i32) -> &mut Self {
        self.selection_configuration.number_of_couples = number_of_couples;
        self
    }
    fn with_selection_method(&mut self, selection_method: Selection) -> &mut Self {
        self.selection_configuration.method = selection_method;
        self
    }

    //Crossover configuration
    fn with_crossover_number_of_points(&mut self, number_of_points: i32) -> &mut Self {
        self.crossover_configuration.number_of_points = Some(number_of_points);
        self
    }
    fn with_crossover_probability_max(&mut self, probability_max: f64) -> &mut Self {
        self.crossover_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_crossover_probability_min(&mut self, probability_min: f64) -> &mut Self {
        self.crossover_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_crossover_method(&mut self, method: Crossover) -> &mut Self {
        self.crossover_configuration.method = method;
        self
    }
    fn with_sbx_eta(&mut self, eta: f64) -> &mut Self {
        self.crossover_configuration.sbx_eta = Some(eta);
        self
    }
    fn with_blend_alpha(&mut self, alpha: f64) -> &mut Self {
        self.crossover_configuration.blend_alpha = Some(alpha);
        self
    }

    //Mutation configuration
    fn with_mutation_probability_max(&mut self, probability_max: f64) -> &mut Self {
        self.mutation_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_mutation_probability_min(&mut self, probability_min: f64) -> &mut Self {
        self.mutation_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_mutation_method(&mut self, method: Mutation) -> &mut Self {
        self.mutation_configuration.method = method;
        self
    }
    fn with_mutation_step(&mut self, step: f64) -> &mut Self {
        self.mutation_configuration.step = Some(step);
        self
    }
    fn with_mutation_sigma(&mut self, sigma: f64) -> &mut Self {
        self.mutation_configuration.sigma = Some(sigma);
        self
    }

    //Save progress configuration
    fn with_save_progress(&mut self, save_progress: bool) -> &mut Self {
        self.save_progress_configuration.save_progress = save_progress;
        self
    }
    fn with_save_progress_interval(&mut self, save_progress_interval: i32) -> &mut Self {
        self.save_progress_configuration.save_progress_interval = save_progress_interval;
        self
    }
    fn with_save_progress_path(&mut self, save_progress_path: String) -> &mut Self {
        self.save_progress_configuration.save_progress_path = save_progress_path;
        self
    }

    fn with_elitism(&mut self, elitism_count: usize) -> &mut Self {
        self.elitism_count = elitism_count;
        self
    }

    fn with_stopping_criteria(&mut self, criteria: StoppingCriteria) -> &mut Self {
        self.stopping_criteria = criteria;
        self
    }
}
