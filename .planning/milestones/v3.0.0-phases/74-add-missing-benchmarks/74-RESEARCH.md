# Phase 74: Add Missing Engine and Feature Benchmarks - Research

**Researched:** 2026-06-18
**Domain:** Rust divan benchmarking — genetic algorithm engines (PSO, CMA-ES, EDA, GP) and features (AOS, surrogate, batch fitness)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Each feature gets its own bench file: `benches/aos.rs`, `benches/surrogate.rs`, `benches/batch_fitness.rs`. Mirrors the `benches/metrics_observer.rs` pattern.
- **D-02:** AOS benchmark measures on-vs-off overhead: two groups — GA with AOS enabled vs. GA without AOS, same problem (Rastrigin 10D), same population size.
- **D-03:** Surrogate benchmark measures throughput: surrogate-assisted GA (cheap model replaces most fitness calls) vs. plain GA on a slow-fitness problem.
- **D-04:** Batch fitness benchmark measures throughput: batch evaluator (all chromosomes in one call) vs. individual `FitnessFnWrapper`. Two groups at the same population sizes.
- **D-05:** GP problem: symbolic regression — evolve a tree that approximates a target function (e.g. `f(x) = x^2 + x + 1`). Standard GP benchmark that directly exercises `GpGa`'s intended use case.
- **D-06:** GP benchmark axis: population size — groups `pop_50`, `pop_200`, `pop_500`. The `genes_N` dimension pattern does not apply to tree chromosomes; population size is the natural scaling axis.
- **D-07:** Both sphere and Rastrigin as benchmark problems. Sphere (convex, trivial) and Rastrigin (multimodal, hard) are already used in `benches/alps.rs` and `benches/rastrigin.rs` — using the same problems enables cross-engine comparison.
- **D-08:** Dimension groups: `dims_10`, `dims_30`, `dims_100`. Matches standard continuous optimization benchmark practice and stays within CI-friendly runtimes.
- **D-09:** Use divan throughout. The `[[bench]]` entries in `Cargo.toml` all use `harness = false` with divan. Do not introduce criterion.

### Claude's Discretion

- Exact population size and generation count per engine (keep small enough that `cargo bench --no-run` plus a quick single-iteration smoke passes fast)
- Whether to add `required-features = ["benchmarks"]` on feature bench entries (follow the `de.rs` precedent if the feature requires non-default crate features)
- Exact `Cargo.toml` `[[bench]]` ordering for new entries

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

## Summary

This phase adds divan benchmark files for every engine and feature that currently lacks coverage: PSO, CMA-ES, EDA, GP (engine benches) and AOS, surrogate-assisted evaluation, and batch fitness (feature benches). Seven new `benches/*.rs` files and seven matching `[[bench]]` entries in `Cargo.toml` are the complete deliverable.

All research is grounded in the codebase: every API call, import path, type name, and builder method was verified by reading the source directly. No external packages are introduced — `divan = "0.1.21"` is already in `[dev-dependencies]`. The `de.rs` bench and `metrics_observer.rs` bench serve as the two canonical templates; all new files follow one of those two shapes.

**Primary recommendation:** Copy `benches/alps.rs` for engine benches (PSO/CMA-ES/EDA) and `benches/metrics_observer.rs` for feature benches (AOS/surrogate/batch fitness). Adapt sphere/Rastrigin helpers inline — do not introduce shared utilities. For GP, copy the `de.rs` flat style but use `GpGa::with_ramped_half_and_half` and dimension the benchmark on population size, not genes.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| PSO benchmark | benches/ | src/engines/pso/ | Exercises PsoEngine with RangeChromosome<f64> fitness closure |
| CMA-ES benchmark | benches/ | src/engines/cma/ | Exercises CmaEngine with scalar fitness fn |
| EDA benchmark | benches/ | src/engines/eda/ | Exercises EdaEngine (Bernoulli) and EdaRealEngine (Gaussian) |
| GP benchmark | benches/ | src/engines/gp/ | Exercises GpGa<MathNode> with symbolic regression fitness |
| AOS benchmark | benches/ | src/engines/ga/ + src/aos/ | Compares Ga with and without crossover_portfolio + AosStrategy |
| Surrogate benchmark | benches/ | src/fitness/surrogate.rs + src/engines/ga/ | Compares Ga with SurrogateModel vs plain Ga |
| Batch fitness benchmark | benches/ | src/fitness/batch.rs + src/engines/ga/ | Compares Ga::with_batch_evaluator vs with_fitness_fn |
| Cargo.toml entries | Cargo.toml [[bench]] | — | One entry per new bench file, harness = false |

---

## Standard Stack

### Core (already present — no new installs needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| divan | 0.1.21 | Benchmark harness | [VERIFIED: Cargo.toml dev-dependencies] Already the chosen harness; `[[bench]] harness = false` entries in Cargo.toml |
| rand | 0.9.2 | Population initialization RNG | [VERIFIED: Cargo.toml] Required for make_pop helpers |
| genetic_algorithms | (this crate) | Engine + feature APIs | [VERIFIED: codebase read] All engine/feature types are in-crate |

### No New Dependencies

[VERIFIED: Cargo.toml] All required crate features are already present: `parallel` (default), no feature flags required for PSO/CMA-ES/EDA/GP. The `benchmarks` feature flag is already defined and used by `de.rs`; if a bench exercises something gated on a non-default feature (none applies here), follow that pattern.

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| divan | crates.io | ~3 yrs (2023-06-30) | 143,971/wk | github.com/nvzqz/divan | OK | Approved — already in dev-dependencies |

**Packages removed due to SLOP verdict:** none
**Packages flagged as suspicious SUS:** none

---

## Architecture Patterns

### System Architecture Diagram

```
bench binary (harness = false)
    │
    ├── fn main() { divan::main(); }
    │
    ├── mod <group_name> {              ← one mod per axis (engine variant / on-off / dim)
    │       #[divan::bench(args = [...])]
    │       fn <name>(bencher: divan::Bencher, arg: T) {
    │           bencher
    │               .with_inputs(|| build_<engine>(arg))   ← optional setup outside timer
    │               .bench_values(|mut engine| engine.run())
    │       }
    │   }
    │
    └── local helpers (sphere, rastrigin, make_pop)        ← inline, no shared crate utils
```

### Recommended Project Structure

No new directories. All files land in the existing `benches/` tree:

```
benches/
├── pso.rs           ← NEW: dims_10 / dims_30 / dims_100 on sphere + Rastrigin
├── cma_es.rs        ← NEW: dims_10 / dims_30 / dims_100 on sphere + Rastrigin
├── eda.rs           ← NEW: dims_10 / dims_30 / dims_100 on sphere (Gaussian) + OneMax-like (Bernoulli)
├── gp.rs            ← NEW: pop_50 / pop_200 / pop_500 symbolic regression
├── aos.rs           ← NEW: aos_on vs aos_off, Rastrigin 10D
├── surrogate.rs     ← NEW: surrogate_on vs surrogate_off, Rastrigin 10D
└── batch_fitness.rs ← NEW: batch_evaluator vs fitness_fn, sphere, pop sizes
```

---

### Pattern 1: Engine Bench with Parameterized Dimensions (PSO / CMA-ES / EDA)

**What:** Run the engine for a small fixed number of generations, varying `dims` via `args = [10usize, 30, 100]`. Mirrors `benches/rastrigin.rs` (verified by reading that file).
**When to use:** Real-valued engines that scale with problem dimension.

```rust
// Source: benches/rastrigin.rs + benches/alps.rs (codebase read)
use std::borrow::Cow;
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};
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
                .map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)], rng.random::<f64>() * 10.0 - 5.0))
                .collect();
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(dna));
            c
        })
        .collect()
}

mod pso {
    use super::*;
    #[divan::bench(args = [10usize, 30, 100])]
    fn sphere_dims(bencher: divan::Bencher, dim: usize) {
        bencher
            .with_inputs(|| {
                let config = PsoConfiguration::default()
                    .with_population_size(30)
                    .with_max_generations(50);
                (config, dim)
            })
            .bench_values(|(config, dim)| {
                let mut engine = PsoEngine::new(config, |n| make_pop(n, dim), sphere);
                engine.run()
            });
    }
}

fn main() { divan::main(); }
```

### Pattern 2: Feature On/Off Bench (AOS / Surrogate / Batch)

**What:** Two `#[divan::bench]` functions within one `mod` — one with the feature enabled, one without. Mirrors `benches/metrics_observer.rs` (verified by reading that file).
**When to use:** Feature-overhead comparison benches.

```rust
// Source: benches/metrics_observer.rs + src/engines/ga/mod.rs (codebase read)
use std::sync::Arc;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection, Survivor};
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
use genetic_algorithms::ChromosomeLength;

fn rastrigin(genes: &[RangeGenotype<f64>]) -> f64 { /* same as benches/rastrigin.rs */ }

mod surrogate_benchmark {
    use super::*;

    #[divan::bench]
    fn with_surrogate(bencher: divan::Bencher) {
        let model = Arc::new(/* LinearSurrogate */);
        bencher.bench(|| {
            let mut ga = Ga::new()
                .with_chromosome_length(ChromosomeLength::Fixed(10))
                .with_population_size(100)
                /* ... */
                .with_surrogate(Arc::clone(&model), 0.4)
                .build().unwrap();
            let _ = ga.run();
        });
    }

    #[divan::bench]
    fn without_surrogate(bencher: divan::Bencher) {
        bencher.bench(|| {
            let mut ga = Ga::new()
                .with_chromosome_length(ChromosomeLength::Fixed(10))
                .with_population_size(100)
                /* ... */
                .build().unwrap();
            let _ = ga.run();
        });
    }
}

fn main() { divan::main(); }
```

### Pattern 3: GP Bench with Population Size Axis

**What:** Use `GpGa::with_ramped_half_and_half`, `MathNode` primitive set, run with `args = [50usize, 200, 500]` as population sizes.
**When to use:** GP engine bench; dimension axis doesn't apply to tree chromosomes.

```rust
// Source: src/engines/gp/engine.rs + src/engines/gp/primitives.rs (codebase read)
use genetic_algorithms::gp::{GpConfiguration, GpGa, MathNode, Node};

fn symreg_fitness(tree: &Node<MathNode>) -> f64 {
    // Evaluate at 20 sample points; MSE against target x^2 + x + 1
    let points: Vec<f64> = (-10..=10).map(|i| i as f64).collect();
    let mse: f64 = points.iter().map(|&x| {
        let pred = tree.eval_with_vars(&[x]);
        let target = x * x + x + 1.0;
        (pred - target).powi(2)
    }).sum::<f64>() / points.len() as f64;
    mse
}

mod gp {
    use super::*;

    #[divan::bench(args = [50usize, 200, 500])]
    fn symreg(bencher: divan::Bencher, pop_size: usize) {
        bencher
            .with_inputs(|| {
                GpConfiguration::new()
                    .with_population_size(pop_size)
                    .with_max_generations(20)
                    .build().unwrap()
            })
            .bench_values(|config| {
                let mut engine = GpGa::with_ramped_half_and_half(config, symreg_fitness);
                let _ = engine.run();
            });
    }
}

fn main() { divan::main(); }
```

---

### Anti-Patterns to Avoid

- **Shared bench utilities module:** Do not create `benches/common.rs` or similar. Each bench file defines its own `sphere`, `rastrigin`, and `make_pop` helpers inline. This matches every existing bench file in the codebase. [VERIFIED: codebase read — no shared module exists]
- **Using `bench` (not `bench_values`) when setup is expensive:** The `bencher.with_inputs(|| setup()).bench_values(|x| use(x))` form excludes setup from the timing window. Use it for engine benches where population construction is non-trivial. [VERIFIED: benches/rastrigin.rs pattern]
- **Using Criterion:** Do not add `criterion` to `[dev-dependencies]`. The `Cargo.toml` comment (from CONTEXT.md) notes that ROADMAP.md saying "Criterion" is a documentation error. [VERIFIED: CONTEXT.md D-09]
- **Calling `with_fitness_fn` and `with_batch_evaluator` together:** These are mutually exclusive — `build()` returns `GaError::ConfigurationError` if both are set. The batch bench must use ONLY `with_batch_evaluator`. [VERIFIED: src/engines/ga/mod.rs line 854-857]
- **Using `Ga::new()` directly for AOS:** AOS requires `with_crossover_portfolio` + `with_aos_strategy`. Without both set, the AOS loop in `generation.rs` is a no-op, so the "off" bench must omit both, and the "on" bench must include both. [VERIFIED: src/engines/ga/mod.rs lines 755-770]
- **Calling `engine.run()` without `let _ = ...`:** `PsoEngine::run()`, `CmaEngine::run()`, `EdaEngine::run()`, `GpGa::run()` all return result structs. Discard with `let _ =` to avoid unused-value warnings that fail CI. [VERIFIED: example files + engine source]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rastrigin function | Custom inline fn | Copy from `benches/rastrigin.rs` | Already calibrated and validated |
| Sphere function | Custom inline fn | Copy from `benches/alps.rs` or `benches/de.rs` | Consistent form across benches |
| Population construction | Novel RNG setup | Copy `make_pop` from `benches/alps.rs` | Exact `RangeGene::new(j, bounds, val)` form required |
| Surrogate model | Elaborate approximation | Simple negated l1-norm (see `examples/surrogate_rastrigin.rs`) | Benchmark measures framework overhead, not surrogate quality |
| Batch evaluator | Async/GPU mock | Synchronous per-chromosome loop in `evaluate_batch` | Benchmark measures batch dispatch overhead vs. per-call |
| GP fitness function | Complex problem | 20-point MSE against x^2+x+1 via `eval_with_vars` | Minimal work, exercises full tree evaluation path |

**Key insight:** Benchmarks are framework hot-path tests. The fitness functions should be trivially cheap so benchmark timings measure the engine overhead, not accidental problem complexity.

---

## Critical API Facts

### PSO Engine API [VERIFIED: src/engines/pso/configuration.rs + examples/pso_rastrigin.rs]

```rust
// Import path (verified from src/lib.rs):
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine, PsoInertia, PsoTopology};

// Construction (verified from examples/pso_rastrigin.rs):
let config = PsoConfiguration::default()
    .with_population_size(30)
    .with_max_generations(50);
// OR direct struct literal (all fields pub):
let config = PsoConfiguration {
    population_size: 30,
    max_generations: 50,
    problem_solving: ProblemSolving::Minimization,
    ..PsoConfiguration::default()
};

// Engine construction (verified from engine.rs): PsoEngine::new(config, init_fn, fitness_fn)
// init_fn signature: Fn(usize) -> Vec<RangeChromosome<f64>>
// fitness_fn signature: Fn(&[RangeGene<f64>]) -> f64
let mut engine = PsoEngine::new(config, |n| make_pop(n, dim), sphere);
let _result = engine.run();  // returns PsoResult
```

### CMA-ES Engine API [VERIFIED: src/engines/cma/configuration.rs + src/engines/cma/engine.rs]

```rust
use genetic_algorithms::cma::{CmaConfiguration, CmaEngine};

// Auto-sized for dimension (verified: CmaConfiguration::default_for_dim):
let config = CmaConfiguration::default_for_dim(dim)
    .with_max_generations(50);

// CmaEngine::new(config, init_fn, fitness_fn) — same shape as PSO
// fitness_fn: Fn(&[RangeGene<f64>]) -> f64
let mut engine = CmaEngine::new(config, |n| make_pop(n, dim), sphere);
let _result = engine.run();  // returns CmaResult
```

### EDA Engine API [VERIFIED: src/engines/eda/configuration.rs + src/engines/eda/engine.rs + examples/eda_trap.rs]

```rust
use genetic_algorithms::eda::{EdaConfiguration, EdaEngine, EdaRealEngine};
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary as BinaryGene;

// Bernoulli variant (binary chromosomes):
let mut engine = EdaEngine::bernoulli(config, init_fn, fitness_fn);

// Gaussian variant (real-valued chromosomes — EdaRealEngine):
// EdaRealEngine::new(config, init_fn, fitness_fn)
// fitness_fn: Fn(&[RangeGene<f64>]) -> f64
```

EDA dimensions benchmark: use Gaussian (`EdaRealEngine`) for `dims_N` groups (Sphere problem); Bernoulli lives in a separate group with fixed binary chromosome length (OneMax). Both in one `benches/eda.rs` file.

### GP Engine API [VERIFIED: src/engines/gp/engine.rs + src/engines/gp/primitives.rs]

```rust
use genetic_algorithms::gp::{GpConfiguration, GpGa, MathNode, Node};

// Primary constructor for most use cases (verified: GpGa::with_ramped_half_and_half):
let config = GpConfiguration::new()
    .with_population_size(pop_size)
    .with_max_generations(20)
    .build()        // returns Result<GpConfiguration, GaError>
    .unwrap();

let mut engine = GpGa::<MathNode>::with_ramped_half_and_half(config, |tree: &Node<MathNode>| {
    // fitness fn — use tree.eval_with_vars(&[x]) for Var nodes
    todo!()
});
let _result = engine.run();  // returns Result<GpResult<MathNode>, GaError>
```

**Important:** `GpGa::run()` returns `Result<GpResult<N>, GaError>`. Discard with `let _ = engine.run();`. [VERIFIED: src/engines/gp/engine.rs]

**`eval_with_vars` is on `Node<MathNode>` only:** The method is defined in `src/engines/gp/primitives.rs` as an inherent impl on `Node<MathNode>` (not on the generic `Node<N>`). [VERIFIED: src/engines/gp/primitives.rs lines 274-284]

### AOS API [VERIFIED: src/engines/ga/mod.rs + src/aos.rs]

```rust
use genetic_algorithms::aos::AosStrategy;
use genetic_algorithms::operations::Crossover;
// Import path from traits:
use genetic_algorithms::traits::{AosConfig};

// AOS wiring on Ga (verified from src/engines/ga/mod.rs lines 755-770):
Ga::new()
    /* ...standard config... */
    .with_crossover_portfolio(vec![Crossover::SinglePoint, Crossover::TwoPoint, Crossover::Uniform])
    .with_aos_strategy(AosStrategy::pm_default())
    .with_reward_window(5)
    /* ...build()... */
```

For the "off" bench: just use `.with_crossover_method(Crossover::Uniform)` with no portfolio or AOS strategy.

### Surrogate API [VERIFIED: src/engines/ga/mod.rs lines 1069-1076 + src/fitness/surrogate.rs]

```rust
use genetic_algorithms::SurrogateModel;  // re-exported from src/lib.rs line 406

// Implement on a local struct:
struct LinearSurrogate;
impl SurrogateModel<RangeChromosome<f64>> for LinearSurrogate {
    fn predict(&self, chromosome: &RangeChromosome<f64>) -> f64 { /* cheap fn */ }
}

// Attach to Ga (verified from src/engines/ga/mod.rs):
.with_surrogate(Arc::new(LinearSurrogate), 0.4)
// prescreening_fraction in (0.0, 1.0] — validated at build time
```

### Batch Fitness API [VERIFIED: src/engines/ga/mod.rs lines 1032-1037 + src/fitness/batch.rs]

```rust
use genetic_algorithms::BatchFitnessEvaluator;  // re-exported from src/lib.rs line 405

struct MyBatchEval;
impl BatchFitnessEvaluator<RangeChromosome<f64>> for MyBatchEval {
    fn evaluate_batch(&self, chromosomes: &[RangeChromosome<f64>]) -> Vec<f64> {
        // MUST return Vec of same length as chromosomes slice
        chromosomes.iter().map(|_c| /* fitness */ 0.0).collect()
    }
}

// Attach to Ga — MUTUALLY EXCLUSIVE with with_fitness_fn:
.with_batch_evaluator(Arc::new(MyBatchEval) as Arc<dyn BatchFitnessEvaluator<RangeChromosome<f64>> + Send + Sync>)
```

---

## Cargo.toml Entries Required [VERIFIED: Cargo.toml read]

Seven new `[[bench]]` entries, following the existing ordering pattern:

```toml
[[bench]]
name = "pso"
harness = false

[[bench]]
name = "cma_es"
harness = false

[[bench]]
name = "eda"
harness = false

[[bench]]
name = "gp"
harness = false

[[bench]]
name = "aos"
harness = false

[[bench]]
name = "surrogate"
harness = false

[[bench]]
name = "batch_fitness"
harness = false
```

None of these require `required-features` — PSO, CMA-ES, EDA, and GP are available under the default feature set. AOS, surrogate, and batch are all part of the default GA engine (no non-default feature flag). [VERIFIED: src/lib.rs + Cargo.toml features section]

---

## Common Pitfalls

### Pitfall 1: `with_fitness_fn` + `with_batch_evaluator` Together

**What goes wrong:** `build()` returns `GaError::ConfigurationError` at runtime ("Cannot use both fitness_fn and with_batch_evaluator() — they are mutually exclusive").
**Why it happens:** The batch bench "off" variant is tempted to share a builder base that already calls `with_fitness_fn`.
**How to avoid:** The batch bench "off" variant uses `.with_fitness_fn(...)` with NO `.with_batch_evaluator()`. The "on" variant uses `.with_batch_evaluator(...)` with NO `.with_fitness_fn()`. Keep them as entirely separate `Ga::new()` chains.
**Warning signs:** `unwrap()` panic at bench start, not during the benchmark loop.

### Pitfall 2: `GpConfiguration::build()` Returns Result — Must Unwrap in `with_inputs`

**What goes wrong:** Calling `.build().unwrap()` inside the timed `bench_values` closure includes configuration validation overhead in the measurement.
**Why it happens:** Following `benches/de.rs` pattern which constructs config inside `bencher.bench(|| ...)`.
**How to avoid:** For GP, use `bencher.with_inputs(|| config.build().unwrap()).bench_values(|config| ...)` so configuration construction and validation are outside the timed window. Or keep it simple — `.with_inputs` is optional if config validation is negligible, but be consistent.

### Pitfall 3: EDA Gaussian Needs `EdaRealEngine`, Not `EdaEngine`

**What goes wrong:** `EdaEngine` is generic over `U: LinearChromosome` but the Gaussian variant requires `U::Gene: RealGene`. Calling `EdaEngine::new(...)` with `RangeChromosome<f64>` compiles only via the Gaussian code path in `EdaRealEngine`.
**Why it happens:** `EdaEngine::bernoulli` is the well-documented path; `EdaRealEngine` is a separate type.
**How to avoid:** For real-valued benchmarks use `EdaRealEngine::new(config, init_fn, fitness_fn)`. For binary/discrete benchmarks use `EdaEngine::bernoulli(config, init_fn, fitness_fn)`. [VERIFIED: src/engines/eda/mod.rs — both types exported]
**Warning signs:** Compile error on `EdaEngine::new` with `RangeChromosome<f64>`.

### Pitfall 4: `PsoEngine::run()` Signature Returns Struct, Not Result

**What goes wrong:** Treating all engine `.run()` returns as `Result<_, _>` when PSO/CMA-ES/DE return plain structs.
**Why it happens:** `GpGa::run()` returns `Result<GpResult<N>, GaError>` but `PsoEngine::run()` and `CmaEngine::run()` return their result structs directly (confirmed by reading engine source).
**How to avoid:** Use `let _result = engine.run();` (no `?` or `unwrap()`). For GP: `let _ = engine.run();` (discards the Result entirely).

### Pitfall 5: AOS `with_aos_strategy` without `with_crossover_portfolio` is a No-Op

**What goes wrong:** AOS is silently disabled if only `with_aos_strategy` is called without `with_crossover_portfolio`. The engine falls back to the single crossover method.
**Why it happens:** The engine checks the portfolio is non-empty before activating AOS logic.
**How to avoid:** Always pair `with_crossover_portfolio(vec![...])` with `with_aos_strategy(...)` for the "on" bench. For the "off" bench, use only `with_crossover_method(Crossover::Uniform)` and omit both portfolio and strategy.

### Pitfall 6: Deep Trees in GP Bench with Large Population + Generations

**What goes wrong:** `cargo bench` (vs `--no-run`) runs very slowly for pop_500 + 50 generations due to subtree crossover producing deep trees on a multimodal fitness landscape.
**Why it happens:** GP trees grow unbounded without tight limits; deep trees are expensive to clone during crossover.
**How to avoid:** Keep `max_generations = 20` and `max_depth = 6` (tighter than the default 8). [VERIFIED: GpConfiguration defaults — max_generations=50, max_depth=8; reduce both for bench]

---

## Code Examples

### Complete PSO Bench Sketch [VERIFIED: assembled from codebase sources]

```rust
// benches/pso.rs
use std::borrow::Cow;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::pso::{PsoConfiguration, PsoEngine};
use genetic_algorithms::traits::LinearChromosome;
use rand::Rng;

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

fn make_pop(n: usize, dim: usize) -> Vec<RangeChromosome<f64>> {
    let mut rng = rand::rng();
    (0..n).map(|_| {
        let dna: Vec<RangeGene<f64>> = (0..dim)
            .map(|j| RangeGene::new(j as i32, vec![(-5.0_f64, 5.0)], rng.random::<f64>() * 10.0 - 5.0))
            .collect();
        let mut c = <RangeChromosome<f64> as Default>::default();
        c.set_dna(Cow::Owned(dna));
        c
    }).collect()
}

mod pso_sphere {
    use super::*;
    #[divan::bench(args = [10usize, 30, 100])]
    fn sphere_dims(bencher: divan::Bencher, dim: usize) {
        bencher
            .with_inputs(|| PsoConfiguration::default()
                .with_population_size(30)
                .with_max_generations(50)
                .with_problem_solving(ProblemSolving::Minimization))
            .bench_values(|config| {
                let mut engine = PsoEngine::new(config, |n| make_pop(n, dim), sphere);
                let _ = engine.run();
            });
    }
}

fn main() { divan::main(); }
```

### GP Symbolic Regression Fitness Function [VERIFIED: src/engines/gp/primitives.rs eval_with_vars]

```rust
// Evaluate evolved tree at 21 points; MSE against x^2 + x + 1
fn symreg_fitness(tree: &Node<MathNode>) -> f64 {
    let n = 21usize;
    let mse: f64 = (-10..=10).map(|i| {
        let x = i as f64;
        let pred = tree.eval_with_vars(&[x]);
        let target = x * x + x + 1.0;
        (pred - target).powi(2)
    }).sum::<f64>() / n as f64;
    mse  // minimize: lower = better approximation
}
```

---

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| `criterion` (mentioned in ROADMAP.md) | `divan 0.1.21` | CONTEXT.md D-09: ROADMAP mention is a docs error; divan is the actual harness in Cargo.toml |
| Per-engine no coverage | Seven new bench files | This phase adds coverage for PSO, CMA-ES, EDA, GP, AOS, surrogate, batch |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `PsoEngine::run()` and `CmaEngine::run()` return plain result structs (not `Result<_, _>`) | Critical API Facts — PSO/CMA | Bench code would need `unwrap()` or `?`; compile error if wrong, no silent failure |
| A2 | `EdaRealEngine` is the correct type for Gaussian/real-valued EDA benchmarks | Critical API Facts — EDA | Compile error if wrong |
| A3 | `GpConfiguration::build()` must be called before `GpGa::with_ramped_half_and_half` (i.e. `build()` exists and returns Result) | Critical API Facts — GP | Compile error if wrong |

**Note on A1:** `PsoEngine::run()` returns `PsoResult` (verified by reading examples/pso_rastrigin.rs lines 104 and `result.generations`). `CmaEngine::run()` returns `CmaResult` (verified by reading src/engines/cma/mod.rs re-export). No `?` or `unwrap` needed on run().

---

## Open Questions

1. **AOS trait import path for `with_aos_strategy` / `with_crossover_portfolio`**
   - What we know: The methods are implemented via `AosConfig` trait from `src/traits/configuration.rs:354` and appear in the `Ga<U>` impl at `src/engines/ga/mod.rs:755-770`. The `AosConfig` trait is implemented on `Ga<U>` automatically.
   - What's unclear: Whether `use genetic_algorithms::traits::AosConfig;` must be in scope for the builder methods to resolve, or if they are inherent methods.
   - Recommendation: Add `use genetic_algorithms::traits::AosConfig;` in `benches/aos.rs` to be safe. If the methods are already in scope via the inherent impl on `Ga`, the unused import will produce a warning — remove it then. Grep `src/traits/configuration.rs` for `pub trait AosConfig` to confirm.

2. **`GpGa<MathNode>` type annotation requirement**
   - What we know: `GpGa::with_ramped_half_and_half` is generic; the `N` type parameter must be inferred or annotated.
   - What's unclear: Whether the fitness closure type-annotates `&Node<MathNode>` sufficiently for inference to work without an explicit `GpGa::<MathNode>::` call.
   - Recommendation: Use explicit turbofish `GpGa::<MathNode>::with_ramped_half_and_half(config, symreg_fitness)` to be explicit and avoid inference failures.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | All benches | Yes | (project already builds) | — |
| divan 0.1.21 | All bench binaries | Yes — in [dev-dependencies] | 0.1.21 | — |
| rand 0.9.2 | make_pop helpers | Yes — in [dependencies] | 0.9.2 | — |

All dependencies are already present. `cargo bench --no-run` is the validation gate.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | divan 0.1.21 (benchmark harness, not a test framework) |
| Config file | none — each bench file has `fn main() { divan::main(); }` |
| Quick run command | `cargo bench --no-run` (compile all benches, no measurement) |
| Full suite command | `cargo bench` |

### Phase Requirements → Test Map

This phase has no named requirement IDs (it is a performance/quality improvement phase). The single success gate is:

| Behavior | Test Type | Command | File Exists? |
|----------|-----------|---------|-------------|
| All new bench files compile | compile check | `cargo bench --no-run` | Wave 0: create files |
| All bench groups run | smoke bench | `cargo bench` | After file creation |

### Sampling Rate

- **Per task commit:** `cargo bench --no-run` — confirms compilation
- **Per wave merge:** `cargo bench --no-run` (full measurement optional in CI)
- **Phase gate:** `cargo bench --no-run` green before `/gsd-verify-work`

### Wave 0 Gaps

The following files must be created before any subsequent wave:

- [ ] `benches/pso.rs`
- [ ] `benches/cma_es.rs`
- [ ] `benches/eda.rs`
- [ ] `benches/gp.rs`
- [ ] `benches/aos.rs`
- [ ] `benches/surrogate.rs`
- [ ] `benches/batch_fitness.rs`
- [ ] `Cargo.toml` — seven new `[[bench]]` entries

---

## Security Domain

This phase adds no network calls, no user input handling, no authentication, no cryptography, and no external service integration. Benchmark functions are pure computation over in-memory data. ASVS categories V2-V6 do not apply. Security enforcement: not applicable to this phase.

---

## Sources

### Primary (HIGH confidence — verified by direct codebase read)

- `benches/alps.rs` — engine bench pattern with sphere + make_pop helpers
- `benches/de.rs` — flat bench style, required-features pattern
- `benches/rastrigin.rs` — parameterized `args = [...]` pattern, `with_inputs` / `bench_values`
- `benches/metrics_observer.rs` — feature on/off comparison pattern
- `src/engines/pso/configuration.rs` + `examples/pso_rastrigin.rs` — PsoEngine API
- `src/engines/cma/configuration.rs` + `src/engines/cma/mod.rs` — CmaEngine API
- `src/engines/eda/configuration.rs` + `src/engines/eda/mod.rs` + `examples/eda_trap.rs` — EdaEngine/EdaRealEngine API
- `src/engines/gp/engine.rs` + `src/engines/gp/primitives.rs` + `src/engines/gp/mod.rs` — GpGa API + eval_with_vars
- `src/engines/ga/mod.rs` — Ga builder: with_batch_evaluator, with_surrogate, with_aos_strategy, with_crossover_portfolio
- `src/fitness/batch.rs` + `src/fitness/surrogate.rs` — BatchFitnessEvaluator + SurrogateModel traits
- `src/aos.rs` — AosStrategy enum + pm_default()
- `Cargo.toml` — existing [[bench]] entries, divan version, feature flags
- `src/lib.rs` — public re-export paths for all types

### Secondary (MEDIUM confidence)

- Package legitimacy seam: `divan` → verdict OK (143,971 weekly downloads, crates.io, github.com/nvzqz/divan)

---

## Metadata

**Confidence breakdown:**
- Engine APIs (PSO, CMA-ES, EDA, GP): HIGH — read from source + example files directly
- Feature APIs (AOS, surrogate, batch): HIGH — read from source + Ga builder methods
- Cargo.toml entries: HIGH — read existing entries, confirmed feature flags
- Divan API patterns: HIGH — read all existing bench files

**Research date:** 2026-06-18
**Valid until:** 2026-08-18 (stable internal API — valid until next breaking refactor)
