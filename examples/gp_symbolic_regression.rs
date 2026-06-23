/*!
# Genetic Programming: Symbolic Regression

GP symbolic regression using `GpGa<MathNode>` to rediscover the function
`f(x) = x² + x`. The fitness function is mean squared error over 20 training
points in `[-2, 2]`. Lower MSE is better (minimization).

Expression trees use `MathNode` built-ins: `Add`, `Sub`, `Mul`, `ProtectedDiv`,
`Const(f64)` (ERC), and `Var(0)` (the input variable `x`). Variable values are
injected at evaluation time via `Node::<MathNode>::eval_with_vars`.

Run with:
```sh
cargo run --example gp_symbolic_regression
```
*/

use genetic_algorithms::gp::{GpConfiguration, GpGa, MathNode, Node};
use genetic_algorithms::rng;

/// Target function: f(x) = x² + x
fn target(x: f64) -> f64 {
    x * x + x
}

fn main() {
    let _ = env_logger::try_init();
    rng::set_seed(Some(42));

    // 20 evenly-spaced training points in [-2, 2]
    let training: Vec<f64> = (0..20).map(|i| -2.0 + i as f64 * (4.0 / 19.0)).collect();
    let targets: Vec<f64> = training.iter().map(|&x| target(x)).collect();

    let fitness_fn = move |tree: &Node<MathNode>| -> f64 {
        let mse: f64 = training.iter().zip(targets.iter()).map(|(&x, &y)| {
            let pred = tree.eval_with_vars(&[x]);
            (pred - y).powi(2)
        }).sum::<f64>() / training.len() as f64;
        mse
    };

    let config = GpConfiguration::new()
        .with_population_size(200)
        .with_max_generations(80)
        .with_init_max_depth(4)
        .with_max_depth(8)
        .with_max_node_count(150)
        .with_fitness_target(Some(1e-4));

    let mut engine: GpGa<MathNode> =
        GpGa::with_ramped_half_and_half(config, fitness_fn);

    println!("== GP Symbolic Regression: f(x) = x² + x ==");
    println!("population=200, max_generations=80, target_mse=1e-4");
    println!("------------------------------------------------");

    let result = engine.run().expect("GP engine run should succeed");

    println!("Generations: {}", result.generations);
    println!("Best MSE:    {:.8}", result.best_fitness);
    println!("Best expr:   {}", result.best);
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite");
    assert!(result.best_fitness >= 0.0, "MSE must be non-negative");
}
