<!-- generated-by: gsd-doc-writer -->
# Getting Started

A step-by-step guide to installing and running your first genetic algorithm with `genetic_algorithms`.

## Prerequisites

- **Rust** `>= 1.81.0` — required by the `rust-version` field in `Cargo.toml`.
- **Cargo** — included with the standard Rust toolchain via [rustup](https://rustup.rs/).

Install or update Rust:

```bash
rustup update stable
rustc --version   # should print 1.81.0 or later
```

No other system dependencies are required. The library is pure Rust with no C FFI; the optional `visualization` feature links against system font libraries via `plotters`, but you do not need it to get started.

## Installation

Add the crate to your project's `Cargo.toml`:

```toml
[dependencies]
genetic_algorithms = "3.0.0"
```

Then fetch dependencies:

```bash
cargo build
```

### Optional feature flags

Enable only what you need — the default build has zero optional dependencies:

```toml
# PNG/SVG fitness and diversity charts
genetic_algorithms = { version = "3.0.0", features = ["visualization"] }

# Checkpoint serialization (serde / serde_json)
genetic_algorithms = { version = "3.0.0", features = ["serde"] }

# Observer integration with the `tracing` crate
genetic_algorithms = { version = "3.0.0", features = ["observer-tracing"] }

# Observer integration with the `metrics` crate
genetic_algorithms = { version = "3.0.0", features = ["observer-metrics"] }
```

## First Run

The canonical "Hello World" for genetic algorithms is the **OneMax** problem: evolve a binary chromosome until all bits are `1`.

Create a new binary project and add the dependency:

```bash
cargo new my_ga_project
cd my_ga_project
# add genetic_algorithms = "3.0.0" to Cargo.toml [dependencies]
```

Replace `src/main.rs` with:

> **Logging:** The library emits `log!()` events but does not install a logger itself. Call
> `env_logger::init()` (or any other `log` subscriber) as the very first statement of `main()`
> if you want to see log output. Add `env_logger = "0.11"` to your `[dev-dependencies]`.

```rust
use std::sync::Arc;

use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::fitness::count_true;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
use genetic_algorithms::LogObserver;

fn main() {
    env_logger::init(); // install logger before running the GA; set RUST_LOG=info to see events

    const N_BITS: usize = 100;
    const POP_SIZE: usize = 50;
    const MAX_GENERATIONS: usize = 1000;
    const FITNESS_TARGET: f64 = N_BITS as f64;

    let fitness_fn = |dna: &[Binary]| count_true(dna);

    let mut ga = Ga::new()
        .with_genes_per_chromosome(N_BITS)
        .with_population_size(POP_SIZE)
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::RouletteWheel)
        .with_crossover_method(Crossover::SinglePoint)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::FixedFitness)
        .with_fitness_target(FITNESS_TARGET)
        .with_max_generations(MAX_GENERATIONS)
        .with_observer(Arc::new(LogObserver))
        .build()
        .expect("Invalid GA configuration");

    match ga.run() {
        Ok(population) => {
            println!("Best fitness: {}", population.best_chromosome.fitness);
        }
        Err(e) => eprintln!("GA failed: {:?}", e),
    }
}
```

Run it:

```bash
cargo run
```

Expected output (generation count will vary):

```
Best fitness: 100
```

## Disabling logging for ultra-lean builds

The `logging` feature is enabled by default. It activates the `log` crate dependency and makes
`LogObserver` available. To shed the `log` dependency entirely — useful for embedded targets,
WASM builds where every kilobyte counts, or any environment where you control observability
entirely through a different mechanism — disable it via:

```toml
genetic_algorithms = { version = "3.0.0", default-features = false }
```

With `logging` off, all internal log emissions expand to `()` at compile time (zero overhead,
no code generation). `LogObserver` is not exported. You can re-enable it selectively alongside
other disabled-by-default features:

```toml
genetic_algorithms = { version = "3.0.0", default-features = false, features = ["serde"] }
```

If you want the `serde` feature but also logging, add both explicitly:

```toml
genetic_algorithms = { version = "3.0.0", default-features = false, features = ["serde", "logging"] }
```

## Running the Bundled Examples

The repository ships with runnable examples in `examples/`. Each example has a doc comment explaining the problem and operators used. Run any example with:

```bash
cargo run --example onemax_binary       # OneMax: maximize true bits
cargo run --example nqueens_range       # N-Queens: minimize conflicts
cargo run --example knapsack_binary     # 0/1 Knapsack
cargo run --example rastrigin           # Rastrigin continuous minimization
cargo run --example nsga2_zdt1          # Multi-objective NSGA-II (ZDT1)
cargo run --example island_model        # Island model with migration
cargo run --example niching             # Fitness sharing / niching
cargo run --example onemax_extension    # Mass extinction diversity control
cargo run --example job_scheduling      # Job scheduling permutation problem
cargo run --example feature_selection   # Feature selection with binary GA
```

## Running the Test Suite

```bash
cargo test                        # All tests
cargo test --features serde       # Including checkpoint serialization tests
cargo clippy                      # Lint checks
cargo doc --no-deps               # Local API documentation
```

## Common Setup Issues

**Wrong Rust version**
The minimum supported Rust version (MSRV) is `1.81.0`. If your build fails with syntax errors or missing trait impls, update your toolchain:

```bash
rustup update stable
```

**Linker errors with the `visualization` feature**
The `plotters` crate used by the `visualization` feature requires a C compiler and system font headers. On Debian/Ubuntu: `sudo apt install build-essential libfontconfig-dev`. On macOS: Xcode Command Line Tools (`xcode-select --install`) is sufficient.

**`observer-metrics` bench requires explicit feature activation**
The `metrics_observer` benchmark requires the `observer-metrics` feature. Always activate it explicitly:

```bash
cargo bench --features observer-metrics --bench metrics_observer
```

**Building for WebAssembly (`wasm32-unknown-unknown`)**

The crate supports WASM out of the box. The repository ships a `.cargo/config.toml` that sets the required `getrandom` backend flag automatically, so a standard `wasm-pack build` works without any extra configuration.

If you use `genetic_algorithms` as a dependency inside your own WASM crate (not inside this repo), add the following to your project's `.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = ["--cfg", "getrandom_backend=\"wasm_js\""]
```

This is needed because `rand` pulls in `getrandom 0.3`, which requires an explicit JS backend declaration for `wasm32-unknown-unknown`. Without this flag the build fails with a `compile_error!` inside `getrandom`.

Rayon parallelism and `Instant`-based timing are automatically disabled on WASM (via `#[cfg]` gates); use `max_generations` or `fitness_target` as stopping criteria instead of `max_duration_secs`.

## Next Steps

- **Configuration reference** — see `docs/configuration.md` for all builder methods, defaults, and stopping criteria.
- **Architecture overview** — see `docs/ARCHITECTURE.md` for the engine model, operator dispatch, and module map.
- **All engines** — `docs/engines.md` covers `IslandGa`, `Nsga2`, `De`, `Scatter`, `CellularGa`, and `Alps`.
- **Operators** — `docs/operators/` covers selection, crossover, mutation, survivor, and extension operators.
- **API reference** — `docs/api-reference.md` or [docs.rs/genetic_algorithms](https://docs.rs/genetic_algorithms/latest/genetic_algorithms).
