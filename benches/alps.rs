use std::borrow::Cow;

use genetic_algorithms::alps::{AlpsAgeScheme, AlpsConfiguration, AlpsEngine};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::de::{DeConfiguration, DeEngine, DeMutationStrategy};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::traits::LinearChromosome;
use rand::Rng;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

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

mod alps_vs_de {
    use super::*;

    #[divan::bench(sample_count = 10)]
    fn alps_fibonacci(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = AlpsConfiguration::default()
                .with_n_layers(5)
                .with_layer_size(20)
                .with_age_scheme(AlpsAgeScheme::Fibonacci)
                .with_age_gap(5)
                .with_injection_interval(10)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(
                    genetic_algorithms::operations::GaussianParams { sigma: Some(0.3) },
                ));
            let mut engine =
                AlpsEngine::new(config, |n| make_pop(n, 5), sphere).expect("valid bench config");
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn de_rand1(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = DeConfiguration::default()
                .with_population_size(100)
                .with_max_generations(100)
                .with_mutation_strategy(DeMutationStrategy::Rand1)
                .with_mutation_factor(0.8)
                .with_crossover_rate(0.9);
            let mut engine = DeEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }
}

mod alps_age_schemes {
    use super::*;

    #[divan::bench(sample_count = 10)]
    fn linear(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = AlpsConfiguration::default()
                .with_n_layers(4)
                .with_layer_size(15)
                .with_age_scheme(AlpsAgeScheme::Linear)
                .with_age_gap(5)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(
                    genetic_algorithms::operations::GaussianParams { sigma: Some(0.3) },
                ));
            let mut engine =
                AlpsEngine::new(config, |n| make_pop(n, 5), sphere).expect("valid bench config");
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn fibonacci(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = AlpsConfiguration::default()
                .with_n_layers(4)
                .with_layer_size(15)
                .with_age_scheme(AlpsAgeScheme::Fibonacci)
                .with_age_gap(5)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(
                    genetic_algorithms::operations::GaussianParams { sigma: Some(0.3) },
                ));
            let mut engine =
                AlpsEngine::new(config, |n| make_pop(n, 5), sphere).expect("valid bench config");
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn polynomial(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = AlpsConfiguration::default()
                .with_n_layers(4)
                .with_layer_size(15)
                .with_age_scheme(AlpsAgeScheme::Polynomial)
                .with_age_gap(5)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(
                    genetic_algorithms::operations::GaussianParams { sigma: Some(0.3) },
                ));
            let mut engine =
                AlpsEngine::new(config, |n| make_pop(n, 5), sphere).expect("valid bench config");
            engine.run()
        });
    }
}

fn main() {
    divan::main();
}
