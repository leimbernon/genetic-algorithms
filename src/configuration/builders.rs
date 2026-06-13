//! Builder-trait implementations for [`GaConfiguration`].
//!
//! All `impl XxxConfig for GaConfiguration` blocks were extracted from the
//! parent `configuration.rs` to keep that file focused on type definitions
//! and accessors. There are no behaviour or API changes — every builder
//! method keeps its public path under
//! `genetic_algorithms::traits::XxxConfig` and stays callable on the
//! `Ga` builder unchanged.

use crate::chromosomes::ChromosomeLength;
use crate::configuration::{
    GaConfiguration, LocalSearchConfiguration, LogLevel, ProblemSolving,
};
use crate::extension::configuration::ExtensionConfiguration;
use crate::niching::configuration::NichingConfiguration;
use crate::operations::{Crossover, Extension, Mutation, Selection, Survivor};
use crate::traits::{
    ConfigurationT, CrossoverConfig, ElitismConfig, ExtensionConfig, LocalSearchConfig,
    MutationConfig, NichingConfig, SelectionConfig, StoppingConfig, SurvivorConfig,
};

impl SelectionConfig for GaConfiguration {
    fn with_number_of_couples(mut self, number_of_couples: usize) -> Self {
        self.selection_configuration.number_of_couples = number_of_couples;
        self
    }
    fn with_selection_method(mut self, selection_method: Selection) -> Self {
        self.selection_configuration.method = selection_method;
        self
    }
    fn with_niche_radius(mut self, niche_radius: f64) -> Self {
        self.selection_configuration.niche_radius = niche_radius;
        self
    }
    fn with_epsilon_lexicase(mut self, epsilon: f64) -> Self {
        self.selection_configuration.epsilon = epsilon;
        self
    }
}

impl CrossoverConfig for GaConfiguration {
    fn with_crossover_number_of_points(mut self, number_of_points: usize) -> Self {
        self.crossover_configuration.number_of_points = Some(number_of_points);
        self
    }
    fn with_crossover_probability_max(mut self, probability_max: f64) -> Self {
        self.crossover_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_crossover_probability_min(mut self, probability_min: f64) -> Self {
        self.crossover_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_crossover_method(mut self, method: Crossover) -> Self {
        self.crossover_configuration.method = method;
        self
    }
    fn with_sbx_eta(mut self, eta: f64) -> Self {
        self.crossover_configuration.sbx_eta = Some(eta);
        self
    }
    fn with_blend_alpha(mut self, alpha: f64) -> Self {
        self.crossover_configuration.blend_alpha = Some(alpha);
        self
    }
    fn with_undx_sigma_xi(mut self, value: f64) -> Self {
        self.crossover_configuration.undx_sigma_xi = Some(value);
        self
    }
    fn with_undx_sigma_eta(mut self, value: f64) -> Self {
        self.crossover_configuration.undx_sigma_eta = Some(value);
        self
    }
    fn with_pcx_sigma_eta(mut self, value: f64) -> Self {
        self.crossover_configuration.pcx_sigma_eta = Some(value);
        self
    }
    fn with_pcx_sigma_zeta(mut self, value: f64) -> Self {
        self.crossover_configuration.pcx_sigma_zeta = Some(value);
        self
    }
}

impl MutationConfig for GaConfiguration {
    fn with_mutation_probability_max(mut self, probability_max: f64) -> Self {
        self.mutation_configuration.probability_max = Some(probability_max);
        self
    }
    fn with_mutation_probability_min(mut self, probability_min: f64) -> Self {
        self.mutation_configuration.probability_min = Some(probability_min);
        self
    }
    fn with_mutation_method(mut self, method: Mutation) -> Self {
        self.mutation_configuration.method = method;
        self
    }
    fn with_dynamic_mutation(mut self, enabled: bool) -> Self {
        self.mutation_configuration.dynamic_mutation = enabled;
        self
    }
    fn with_mutation_target_cardinality(mut self, target: f64) -> Self {
        self.mutation_configuration.target_cardinality = Some(target);
        self
    }
    fn with_mutation_probability_step(mut self, step: f64) -> Self {
        self.mutation_configuration.probability_step = Some(step);
        self
    }
}

impl StoppingConfig for GaConfiguration {
    fn with_max_generations(mut self, max_generations: usize) -> Self {
        self.limit_configuration.max_generations = max_generations;
        self
    }
    fn with_fitness_target(mut self, fitness_target: f64) -> Self {
        self.limit_configuration.fitness_target = Some(fitness_target);
        self
    }
    fn with_stagnation_limit(mut self, n: usize) -> Self {
        self.stagnation_generations = Some(n);
        self
    }
    fn with_convergence_threshold(mut self, threshold: f64) -> Self {
        self.convergence_threshold = Some(threshold);
        self
    }
    fn with_max_duration_secs(mut self, secs: f64) -> Self {
        self.max_duration_secs = Some(secs);
        self
    }
}

impl NichingConfig for GaConfiguration {
    fn with_niching_enabled(mut self, enabled: bool) -> Self {
        self.niching_configuration
            .get_or_insert_with(NichingConfiguration::default)
            .enabled = enabled;
        self
    }
    fn with_niching_sigma_share(mut self, sigma_share: f64) -> Self {
        self.niching_configuration
            .get_or_insert_with(NichingConfiguration::default)
            .sigma_share = sigma_share;
        self
    }
    fn with_niching_alpha(mut self, alpha: f64) -> Self {
        self.niching_configuration
            .get_or_insert_with(NichingConfiguration::default)
            .alpha = alpha;
        self
    }
}

impl ElitismConfig for GaConfiguration {
    fn with_elitism(mut self, elitism_count: usize) -> Self {
        self.elitism_count = elitism_count;
        self
    }
}

impl SurvivorConfig for GaConfiguration {
    fn with_length_penalty(mut self, penalty: f64) -> Self {
        self.length_penalty = Some(penalty);
        self
    }
}

impl ExtensionConfig for GaConfiguration {
    fn with_extension_method(mut self, method: Extension) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .method = method;
        self
    }
    fn with_extension_diversity_threshold(mut self, threshold: f64) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .diversity_threshold = threshold;
        self
    }
    fn with_extension_survival_rate(mut self, rate: f64) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .survival_rate = rate;
        self
    }
    fn with_extension_mutation_rounds(mut self, rounds: usize) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .mutation_rounds = rounds;
        self
    }
    fn with_extension_elite_count(mut self, count: usize) -> Self {
        self.extension_configuration
            .get_or_insert_with(ExtensionConfiguration::default)
            .elite_count = count;
        self
    }
}

impl LocalSearchConfig for GaConfiguration {
    fn with_local_search_configuration(mut self, config: LocalSearchConfiguration) -> Self {
        self.local_search_configuration = Some(config);
        self
    }
}

impl ConfigurationT for GaConfiguration {
    fn new() -> Self {
        Self::default()
    }
    fn with_adaptive_ga(mut self, adaptive_ga: bool) -> Self {
        self.adaptive_ga = adaptive_ga;
        self
    }
    fn with_threads(mut self, number_of_threads: usize) -> Self {
        self.number_of_threads = number_of_threads;
        self
    }
    fn with_logs(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }
    fn with_survivor_method(mut self, method: Survivor) -> Self {
        self.survivor = method;
        self
    }

    //Limit configuration
    fn with_problem_solving(mut self, problem_solving: ProblemSolving) -> Self {
        self.limit_configuration.problem_solving = problem_solving;
        self
    }
    fn with_population_size(mut self, population_size: usize) -> Self {
        self.limit_configuration.population_size = population_size;
        self
    }
    fn with_chromosome_length(mut self, length: ChromosomeLength) -> Self {
        self.limit_configuration.chromosome_length = length;
        self
    }

    //Save progress configuration
    fn with_save_progress(mut self, save_progress: bool) -> Self {
        self.save_progress_configuration.save_progress = save_progress;
        self
    }
    fn with_save_progress_interval(mut self, save_progress_interval: usize) -> Self {
        self.save_progress_configuration.save_progress_interval = save_progress_interval;
        self
    }
    fn with_save_progress_path(mut self, save_progress_path: String) -> Self {
        self.save_progress_configuration.save_progress_path = save_progress_path;
        self
    }

    fn with_rng_seed(mut self, seed: u64) -> Self {
        self.rng_seed = Some(seed);
        self
    }

    fn with_crossover_portfolio(mut self, portfolio: Vec<Crossover>) -> Self {
        self.crossover_portfolio = Some(portfolio);
        self
    }
    fn with_mutation_portfolio(mut self, portfolio: Vec<Mutation>) -> Self {
        self.mutation_portfolio = Some(portfolio);
        self
    }
    fn with_aos_strategy(mut self, strategy: crate::aos::AosStrategy) -> Self {
        self.aos_strategy = strategy;
        self
    }
    fn with_reward_window(mut self, window: usize) -> Self {
        self.aos_reward_window = window;
        self
    }
}
