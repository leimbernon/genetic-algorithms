use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::scatter::{ScatterConfiguration, ScatterEngine};
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
                .map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)], rng.random::<f64>() * 10.0 - 5.0))
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

mod scatter_search {
    use super::*;

    #[divan::bench(sample_count = 10)]
    fn no_local_search(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = ScatterConfiguration::default()
                .with_population_size(30)
                .with_reference_set_size(6)
                .with_max_iterations(50)
                .with_local_search(false);
            let mut engine = ScatterEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn with_local_search(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = ScatterConfiguration::default()
                .with_population_size(30)
                .with_reference_set_size(6)
                .with_max_iterations(50)
                .with_local_search(true)
                .with_local_search_steps(10);
            let mut engine = ScatterEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }
}

fn main() {
    divan::main();
}
