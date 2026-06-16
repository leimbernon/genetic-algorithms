use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::de::{DeConfiguration, DeEngine, DeMutationStrategy};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::LinearChromosome;
use rand::Rng;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

fn make_pop(n: usize, dim: usize) -> Vec<RangeChromosome<f64>> {
    let mut rng = rand::rng();
    (0..n)
        .map(|_| {
            let dna: Vec<RangeGene<f64>> = (0..dim)
                .map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)], rng.random::<f64>() * 10.0 - 5.0))
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

fn run_de(strategy: DeMutationStrategy) {
    let config = DeConfiguration::default()
        .with_population_size(30)
        .with_max_generations(100)
        .with_mutation_strategy(strategy)
        .with_problem_solving(ProblemSolving::Minimization);
    let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
    let _ = engine.run();
}

mod de_mutation_strategies {
    use super::*;

    #[divan::bench(sample_count = 10)]
    fn rand1(bencher: divan::Bencher) {
        bencher.bench(|| run_de(DeMutationStrategy::Rand1));
    }

    #[divan::bench(sample_count = 10)]
    fn best1(bencher: divan::Bencher) {
        bencher.bench(|| run_de(DeMutationStrategy::Best1));
    }

    #[divan::bench(sample_count = 10)]
    fn current_to_best1(bencher: divan::Bencher) {
        bencher.bench(|| run_de(DeMutationStrategy::CurrentToBest1));
    }

    #[divan::bench(sample_count = 10)]
    fn rand2(bencher: divan::Bencher) {
        bencher.bench(|| run_de(DeMutationStrategy::Rand2));
    }

    #[divan::bench(sample_count = 10)]
    fn best2(bencher: divan::Bencher) {
        bencher.bench(|| run_de(DeMutationStrategy::Best2));
    }
}

fn main() {
    divan::main();
}
