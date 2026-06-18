use std::borrow::Cow;

use genetic_algorithms::cellular::{
    CellularConfiguration, CellularEngine, Neighborhood, UpdateMode,
};
use genetic_algorithms::chromosomes::Range as RangeChromosome;
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

mod cellular_ga {
    use super::*;

    #[divan::bench(sample_count = 10)]
    fn von_neumann(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = CellularConfiguration::default()
                .with_grid(8, 8)
                .with_neighborhood(Neighborhood::VonNeumann)
                .with_update_mode(UpdateMode::Asynchronous)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(genetic_algorithms::operations::GaussianParams {
                    sigma: Some(0.3),
                }));
            let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn moore(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = CellularConfiguration::default()
                .with_grid(8, 8)
                .with_neighborhood(Neighborhood::Moore)
                .with_update_mode(UpdateMode::Asynchronous)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(genetic_algorithms::operations::GaussianParams {
                    sigma: Some(0.3),
                }));
            let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn compact_r2(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = CellularConfiguration::default()
                .with_grid(8, 8)
                .with_neighborhood(Neighborhood::CompactR2)
                .with_update_mode(UpdateMode::Asynchronous)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(genetic_algorithms::operations::GaussianParams {
                    sigma: Some(0.3),
                }));
            let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn linear(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = CellularConfiguration::default()
                .with_grid(8, 8)
                .with_neighborhood(Neighborhood::Linear)
                .with_update_mode(UpdateMode::Asynchronous)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(genetic_algorithms::operations::GaussianParams {
                    sigma: Some(0.3),
                }));
            let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }
}

mod cellular_sync_vs_async {
    use super::*;

    #[divan::bench(sample_count = 10)]
    fn synchronous(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = CellularConfiguration::default()
                .with_grid(8, 8)
                .with_neighborhood(Neighborhood::Moore)
                .with_update_mode(UpdateMode::Synchronous)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(genetic_algorithms::operations::GaussianParams {
                    sigma: Some(0.3),
                }));
            let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }

    #[divan::bench(sample_count = 10)]
    fn asynchronous(bencher: divan::Bencher) {
        bencher.bench(|| {
            let config = CellularConfiguration::default()
                .with_grid(8, 8)
                .with_neighborhood(Neighborhood::Moore)
                .with_update_mode(UpdateMode::Asynchronous)
                .with_max_generations(100)
                .with_mutation(genetic_algorithms::operations::Mutation::Gaussian(genetic_algorithms::operations::GaussianParams {
                    sigma: Some(0.3),
                }));
            let mut engine = CellularEngine::new(config, |n| make_pop(n, 5), sphere);
            engine.run()
        });
    }
}

fn main() {
    divan::main();
}
