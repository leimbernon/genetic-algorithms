use std::borrow::Cow;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine, EdaRealEngine};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::traits::LinearChromosome;
use rand::Rng;

// ---------------------------------------------------------------------------
// Sphere fitness function (inline — copied from benches/de.rs pattern)
// ---------------------------------------------------------------------------

/// Sphere function: f(x) = sum(x_i^2)
///
/// Global minimum is 0.0 at x_i = 0 for all i.
fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

// ---------------------------------------------------------------------------
// OneMax fitness function
// ---------------------------------------------------------------------------

/// OneMax: count of true-valued genes.
///
/// Global maximum is DNA length (all bits = true).
fn onemax(dna: &[BinaryGene]) -> f64 {
    dna.iter().filter(|g| g.value).count() as f64
}

// ---------------------------------------------------------------------------
// Population helpers
// ---------------------------------------------------------------------------

/// Build `n` random real-valued chromosomes of `dim` dimensions.
fn make_pop(n: usize, dim: usize) -> Vec<RangeChromosome<f64>> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..dim)
                .map(|j| {
                    RangeGene::new(
                        j as i32,
                        vec![(-5.0_f64, 5.0)],
                        rng.random::<f64>() * 10.0 - 5.0,
                    )
                })
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

/// Build `n` random binary chromosomes of `len` bits.
fn make_binary_pop(n: usize, len: usize) -> Vec<BinaryChromosome> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| {
            let dna: Vec<BinaryGene> = (0..len)
                .map(|_| {
                    let v = rng.random::<bool>();
                    BinaryGene {
                        id: if v { 1 } else { 0 },
                        value: v,
                    }
                })
                .collect();
            let mut c = BinaryChromosome::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Gaussian group (EdaRealEngine — sphere, dims axis)
// ---------------------------------------------------------------------------

mod eda_gaussian {
    use super::*;

    /// Sphere fitness across dims_10 / dims_30 / dims_100.
    #[divan::bench(args = [10usize, 30, 100])]
    fn sphere_dims(bencher: divan::Bencher, dim: usize) {
        bencher
            .with_inputs(|| {
                let config = EdaConfiguration::default()
                    .with_population_size(100)
                    .with_max_generations(50)
                    .with_problem_solving(ProblemSolving::Minimization);
                (config, dim)
            })
            .bench_values(|(config, dim)| {
                let mut engine =
                    EdaRealEngine::new(config, move |n| make_pop(n, dim), sphere);
                let _ = engine.run();
            });
    }
}

// ---------------------------------------------------------------------------
// Bernoulli group (EdaEngine — binary OneMax, fixed length 64)
// ---------------------------------------------------------------------------

mod eda_bernoulli {
    use super::*;

    /// OneMax-64 (fixed binary length, maximization).
    #[divan::bench]
    fn onemax_64(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| {
                EdaConfiguration::default()
                    .with_population_size(100)
                    .with_max_generations(50)
                    .with_problem_solving(ProblemSolving::Maximization)
            })
            .bench_values(|config| {
                let mut engine =
                    EdaEngine::bernoulli(config, |n| make_binary_pop(n, 64), onemax);
                let _ = engine.run();
            });
    }
}

fn main() {
    divan::main();
}
