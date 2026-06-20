use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::traits::LinearChromosome;
use rand::Rng;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

fn rastrigin(genes: &[RangeGene<f64>]) -> f64 {
    let a = 10.0_f64;
    let n = genes.len() as f64;
    a * n
        + genes
            .iter()
            .map(|g| {
                let x = g.value();
                x * x - a * (2.0 * std::f64::consts::PI * x).cos()
            })
            .sum::<f64>()
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

mod cma_sphere {
    use super::*;

    #[divan::bench(args = [10usize, 30, 100])]
    fn run(bencher: divan::Bencher, dim: usize) {
        bencher
            .with_inputs(|| CmaConfiguration::default_for_dim(dim).with_max_generations(50))
            .bench_values(|config| {
                let mut engine = CmaEngine::new(config, move |n| make_pop(n, dim), sphere);
                let _ = engine.run();
            });
    }
}

mod cma_rastrigin {
    use super::*;

    #[divan::bench(args = [10usize, 30, 100])]
    fn run(bencher: divan::Bencher, dim: usize) {
        bencher
            .with_inputs(|| CmaConfiguration::default_for_dim(dim).with_max_generations(50))
            .bench_values(|config| {
                let mut engine = CmaEngine::new(config, move |n| make_pop(n, dim), rastrigin);
                let _ = engine.run();
            });
    }
}

fn main() {
    divan::main();
}
