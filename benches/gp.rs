use genetic_algorithms::gp::{GpConfiguration, GpGa, MathNode, Node};

// ---------------------------------------------------------------------------
// Symbolic-regression fitness function (target: f(x) = x^2 + x + 1)
// ---------------------------------------------------------------------------

/// Mean squared error of evolved tree vs. `f(x) = x^2 + x + 1` at 21 points.
fn symreg_fitness(tree: &Node<MathNode>) -> f64 {
    let n = 21.0_f64;
    let mse: f64 = (-10..=10)
        .map(|i| {
            let x = i as f64;
            let pred = tree.eval_with_vars(&[x]);
            let target = x * x + x + 1.0;
            (pred - target).powi(2)
        })
        .sum::<f64>()
        / n;
    mse // minimize: lower = better approximation
}

// ---------------------------------------------------------------------------
// GP group (symbolic regression, population-size axis)
// ---------------------------------------------------------------------------

mod gp {
    use super::*;

    /// Symbolic regression across pop_50 / pop_200 / pop_500.
    #[divan::bench(args = [50usize, 200, 500])]
    fn symreg(bencher: divan::Bencher, pop_size: usize) {
        bencher
            .with_inputs(|| {
                GpConfiguration::new()
                    .with_population_size(pop_size)
                    .with_max_generations(20)
                    .with_max_depth(6)
            })
            .bench_values(|config| {
                let mut engine =
                    GpGa::<MathNode>::with_ramped_half_and_half(config, symreg_fitness);
                let _ = engine.run();
            });
    }
}

fn main() {
    divan::main();
}
