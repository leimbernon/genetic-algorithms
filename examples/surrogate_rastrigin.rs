/*!
# Surrogate-Assisted Rastrigin Minimization

Demonstrates [`SurrogateModel`] prescreening reducing true fitness call count on the
10-dimensional Rastrigin function — a multimodal benchmark with many local optima.

The `LinearSurrogate` provides a cheap linear approximation of the fitness landscape.
Before each generation's offspring are passed to the expensive true Rastrigin evaluator,
the engine sorts them by surrogate-predicted score and retains only the top 40%
(`prescreening_fraction = 0.4`). This reduces true fitness calls while preserving
selection pressure.

Expected output: Per-generation table of `generation | best_fitness | true_fitness_calls`,
followed by confirmation that at least one generation achieved surrogate reduction
(i.e. `true_fitness_calls < offspring_count`).

Run with:
```sh
cargo run --example surrogate_rastrigin --release
```
*/

use std::f64::consts::PI;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
use genetic_algorithms::{ChromosomeLength, SurrogateModel};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DIMENSIONS: usize = 10;
const SEARCH_LO: f64 = -5.12;
const SEARCH_HI: f64 = 5.12;
const POPULATION_SIZE: usize = 100;
const GENERATIONS: usize = 30;
/// Fraction of offspring that survive surrogate prescreening before true evaluation.
const PRESCREENING_FRACTION: f64 = 0.4;

// ---------------------------------------------------------------------------
// Surrogate model: cheap linear approximation (D-01 implementor example)
// ---------------------------------------------------------------------------

/// Cheap linear surrogate — returns the negated weighted sum of gene values.
///
/// For minimization, we want to pass chromosomes with *lower* expected fitness
/// to the true evaluator first. `predict` must return *higher* values for better
/// (lower Rastrigin) candidates. We approximate with the negative of a weighted
/// l1-norm so that chromosomes near the origin (global minimum) score highest.
struct LinearSurrogate {
    coeffs: Vec<f64>,
}

impl SurrogateModel<RangeChromosome<f64>> for LinearSurrogate {
    fn predict(&self, chromosome: &RangeChromosome<f64>) -> f64 {
        // Negated weighted l1 norm: chromosomes closer to 0 score higher.
        use genetic_algorithms::traits::LinearChromosome;
        -chromosome
            .dna()
            .iter()
            .zip(self.coeffs.iter())
            .map(|(g, c): (&RangeGenotype<f64>, &f64)| g.value() * c)
            .sum::<f64>()
    }
}

// ---------------------------------------------------------------------------
// True fitness: Rastrigin function with invocation counter
// ---------------------------------------------------------------------------

fn make_rastrigin_fn(
    counter: Arc<AtomicUsize>,
) -> impl Fn(&[RangeGenotype<f64>]) -> f64 + Send + Sync + 'static {
    move |dna: &[RangeGenotype<f64>]| {
        counter.fetch_add(1, Ordering::Relaxed);
        let a = 10.0_f64;
        let n = dna.len() as f64;
        a * n
            + dna
                .iter()
                .map(|g| {
                    let x = g.value();
                    x * x - a * (2.0 * PI * x).cos()
                })
                .sum::<f64>()
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let _ = env_logger::try_init();
    // Atomic counter to track true fitness invocations.
    let eval_counter = Arc::new(AtomicUsize::new(0));
    let fitness_fn = make_rastrigin_fn(Arc::clone(&eval_counter));

    let alleles = vec![RangeGenotype::new(0, vec![(SEARCH_LO, SEARCH_HI)], 0.0_f64)];
    let alleles_clone = alleles.clone();

    let surrogate = Arc::new(LinearSurrogate {
        coeffs: vec![1.0; DIMENSIONS],
    });

    let mut ga = Ga::new()
        .with_chromosome_length(ChromosomeLength::Fixed(DIMENSIONS))
        .with_population_size(POPULATION_SIZE)
        .with_initialization_fn(move |genes_per_chromosome, _| {
            range_random_initialization(genes_per_chromosome, Some(&alleles_clone))
        })
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Gaussian { sigma: None })
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_max_generations(GENERATIONS)
        .with_surrogate(surrogate, PRESCREENING_FRACTION)
        .build()
        .expect("Failed to build GA configuration");

    println!("== Surrogate-Assisted Rastrigin Minimization ==");
    println!(
        "Dimensions: {DIMENSIONS}, Population: {POPULATION_SIZE}, Generations: {GENERATIONS}"
    );
    println!("Surrogate prescreening fraction: {PRESCREENING_FRACTION}");
    println!("-------------------------------------------------------");
    println!(
        "{:<12} {:<16} {:<22}",
        "Generation", "Best Fitness", "True Fitness Calls"
    );
    println!("{}", "-".repeat(52));

    ga.run().expect("GA run failed");

    let stats = ga.stats();

    let mut found_reduction = false;

    for s in stats.iter() {
        let tfc_display = match s.true_fitness_calls {
            Some(n) => format!("{n}"),
            None => "N/A".to_string(),
        };
        println!(
            "{:<12} {:<16.6} {:<22}",
            s.generation, s.best_fitness, tfc_display
        );

        // Check if surrogate reduced evaluations below offspring count.
        // offspring_count per generation is approximately population_size (pairs produce 2 children).
        // Any generation where true_fitness_calls < POPULATION_SIZE shows reduction.
        if let Some(tfc) = s.true_fitness_calls {
            if tfc < POPULATION_SIZE as u64 {
                found_reduction = true;
            }
        }
    }

    let total_evals = eval_counter.load(Ordering::Relaxed);
    let max_without_surrogate = GENERATIONS * POPULATION_SIZE;

    println!("{}", "-".repeat(52));
    println!("Total true fitness evaluations: {total_evals}");
    println!("Max evaluations without surrogate (est): {max_without_surrogate}");
    println!(
        "Evaluation savings: {:.1}%",
        100.0 * (1.0 - total_evals as f64 / max_without_surrogate as f64)
    );

    assert!(
        found_reduction,
        "Expected at least one generation where true_fitness_calls < {POPULATION_SIZE} \
        (surrogate prescreening at fraction={PRESCREENING_FRACTION}), \
        but no such generation was found in stats: {stats:?}"
    );

    println!("-------------------------------------------------------");
    println!("Surrogate-reduction assertion PASSED: at least one generation had");
    println!("  true_fitness_calls < {POPULATION_SIZE} (offspring_count).");
}
