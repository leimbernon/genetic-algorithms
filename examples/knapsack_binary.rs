use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::genotypes::Binary as BinaryGenotype;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::LogObserver;
use std::sync::Arc;

// see https://en.wikipedia.org/wiki/Knapsack_problem
// With 10 items the optimal value is 1270.

const EXCESS_WEIGHT_PENALTY: f64 = 1000.0;
const MAX_WEIGHT: f64 = 67.0;

#[derive(Clone)]
struct Item {
    weight: f64,
    value: f64,
}

const ITEMS: [Item; 10] = [
    Item {
        weight: 23.0,
        value: 505.0,
    },
    Item {
        weight: 26.0,
        value: 352.0,
    },
    Item {
        weight: 20.0,
        value: 458.0,
    },
    Item {
        weight: 18.0,
        value: 220.0,
    },
    Item {
        weight: 32.0,
        value: 354.0,
    },
    Item {
        weight: 27.0,
        value: 414.0,
    },
    Item {
        weight: 29.0,
        value: 498.0,
    },
    Item {
        weight: 26.0,
        value: 545.0,
    },
    Item {
        weight: 30.0,
        value: 473.0,
    },
    Item {
        weight: 27.0,
        value: 543.0,
    },
];

fn fitness_fn(dna: &[BinaryGenotype]) -> f64 {
    let mut total_weight = 0.0;
    let mut total_value = 0.0;

    for (item, gene) in ITEMS.iter().zip(dna.iter()) {
        if gene.value {
            total_weight += item.weight;
            total_value += item.value;
        }
    }

    if total_weight > MAX_WEIGHT {
        total_value -= EXCESS_WEIGHT_PENALTY * (total_weight - MAX_WEIGHT);
    }

    total_value
}

fn report(
    generation: &usize,
    population: &Population<BinaryChromosome>,
    _stats: &GenerationStats,
    termination_cause: &TerminationCause,
) -> std::ops::ControlFlow<()> {
    println!(
        "Generation: {} - Best Score: {} - Termination Cause: {:?}",
        generation, population.best_chromosome.fitness, termination_cause
    );
    std::ops::ControlFlow::Continue(())
}

fn main() {
    // For maximization problem
    let mut ga = Ga::new()
        .with_genes_per_chromosome(10)
        .with_population_size(100)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(5000)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Invalid GA configuration");
    let population = ga.run_with_callback(Some(report), 100).unwrap();

    println!(
        "Best chromosome for maximization: {}",
        population.best_chromosome.phenotype()
    );

    // For fixed fitness problem
    let mut ga_fixed = Ga::new()
        .with_genes_per_chromosome(10)
        .with_population_size(10)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Swap)
        .with_problem_solving(ProblemSolving::FixedFitness)
        .with_fitness_target(1270.0)
        .with_max_generations(5000)
        // Observer: LogObserver logs every lifecycle hook via the `log` crate
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Invalid GA configuration");
    let population = ga_fixed.run_with_callback(Some(report), 100).unwrap();

    println!(
        "Best chromosome for fixed fitness: {}",
        population.best_chromosome.phenotype()
    );
}
