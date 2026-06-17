/*!
# EDA: Deceptive Trap Function

Demonstrates the Univariate Marginal Distribution Algorithm (UMDA) on a deceptive
trap function — a classic benchmark that challenges both standard GA and EDA.

## Trap Function

The trap function divides a binary chromosome of length `L` into non-overlapping
blocks of size `k`. Within each block:
- If all `k` bits are **1** → reward `k` (the global optimum)
- Otherwise → reward `k - 1 - count_of_ones` (deceptive: more zeros = better)

This landscape has a deceptive local attractor (all-zeros: reward `L - L/k`) and
a global optimum (all-ones: reward `L`). The fitness landscape misdirects local
search toward all-zeros because partial ones decrease fitness within a block.

Classical GA crossover/mutation fails because:
1. Flipping a bit from 0→1 inside a partial block *decreases* fitness.
2. Crossover disrupts the block structure, preventing coordinated convergence.

UMDA (this engine) models per-position marginals `p_i`. With sufficient population
diversity (large `POP_SIZE`) and truncation selection (`selection_ratio`), UMDA can
learn to increase `p_i` toward 1.0. However, note that UMDA is a *univariate* model
— it does not capture dependencies between positions within a block. For fully
reliable trap solving, multivariate EDAs (BMDA, MIMIC, BOA) are needed. This example
demonstrates the UMDA approach and its probabilistic convergence behavior.

## Setup

- Chromosome: `Binary` of length 30 (6 blocks × 5 genes)
- Block size `k = 5`
- Population: 300 (larger populations needed for deceptive landscapes)
- Max generations: 500
- Selection ratio: 0.3 (top 30% feed the model)
- Target: `30.0` (all-ones = maximum trap fitness)

Run with:
```sh
cargo run --release --example eda_trap
```
*/

use std::borrow::Cow;
use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::rng;
use genetic_algorithms::traits::LinearChromosome;
use genetic_algorithms::LogObserver;
use rand::Rng;

const CHROMOSOME_LEN: usize = 30;
const BLOCK_SIZE: usize = 5;
const POP_SIZE: usize = 300;
const MAX_GENERATIONS: usize = 500;
const SELECTION_RATIO: f64 = 0.3;

/// Trap function.
///
/// Divides the chromosome into non-overlapping blocks of `BLOCK_SIZE`.
/// For each block: if all bits are 1 → reward `BLOCK_SIZE`, else → `BLOCK_SIZE - 1 - count`.
///
/// Global maximum: all bits 1 → fitness = CHROMOSOME_LEN
fn trap_fitness(dna: &[BinaryGene]) -> f64 {
    dna.chunks(BLOCK_SIZE)
        .map(|block| {
            let ones = block.iter().filter(|g| g.id == 1).count();
            if ones == BLOCK_SIZE {
                BLOCK_SIZE as f64
            } else {
                (BLOCK_SIZE - 1 - ones) as f64
            }
        })
        .sum()
}

/// Build the initial population: `n` random binary chromosomes of `CHROMOSOME_LEN` genes.
///
/// Gene `id` is 1 (bit = 1) or 0 (bit = 0) sampled uniformly at random.
fn init_population(n: usize) -> Vec<BinaryChromosome> {
    let mut rng = rng::make_rng();
    (0..n)
        .map(|_| {
            let dna: Vec<BinaryGene> = (0..CHROMOSOME_LEN)
                .map(|_| {
                    let v = rng.random::<bool>();
                    BinaryGene { id: if v { 1 } else { 0 }, value: v }
                })
                .collect();
            let mut c = BinaryChromosome::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

fn main() {
    let _ = env_logger::try_init();
    rng::set_seed(Some(42));

    let config = EdaConfiguration {
        population_size: POP_SIZE,
        max_generations: MAX_GENERATIONS,
        problem_solving: ProblemSolving::Maximization,
        fitness_target: Some(CHROMOSOME_LEN as f64),
        selection_ratio: SELECTION_RATIO,
    };

    let mut engine = EdaEngine::bernoulli(config, init_population, trap_fitness)
        .with_observer(Arc::new(LogObserver));

    println!("== EDA (UMDA): Deceptive Trap Function ==");
    println!(
        "chromosome_len={}, block_size={}, blocks={}",
        CHROMOSOME_LEN,
        BLOCK_SIZE,
        CHROMOSOME_LEN / BLOCK_SIZE
    );
    println!(
        "pop={}, max_gen={}, selection_ratio={}",
        POP_SIZE, MAX_GENERATIONS, SELECTION_RATIO
    );
    println!("target=30.0 (all-ones = global maximum)");
    println!("---------------------------------------------");

    let result = engine.run();

    // Display result
    let best_dna: String = result
        .best
        .dna()
        .iter()
        .map(|g| if g.id == 1 { '1' } else { '0' })
        .collect();

    println!("Generations: {}", result.generations);
    println!("Best fitness: {:.1}", result.best_fitness);
    println!("Best DNA:     {}", best_dna);

    // Display learned model
    match &result.learned_model {
        genetic_algorithms::eda::EdaModel::Bernoulli(probs) => {
            let probs_str: Vec<String> = probs.iter().map(|p| format!("{:.2}", p)).collect();
            println!("Learned probs: [{}]", probs_str.join(", "));

            let converged = probs.iter().filter(|&&p| !(0.1..=0.9).contains(&p)).count();
            println!("Converged positions (p > 0.9 or p < 0.1): {}/{}", converged, probs.len());
        }
        _ => unreachable!("Binary chromosomes always use Bernoulli model"),
    }

    println!("---------------------------------------------");
    if result.best_fitness >= CHROMOSOME_LEN as f64 {
        println!("SUCCESS: Found global optimum (all-ones)");
    } else {
        println!(
            "PARTIAL: Best fitness {:.1}/{} (increase generations or population for full convergence)",
            result.best_fitness,
            CHROMOSOME_LEN
        );
    }

    // Verify result is finite
    assert!(
        result.best_fitness.is_finite(),
        "best_fitness must be finite"
    );
}
