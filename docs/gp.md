# Genetic Programming (GpGa)

> Tree-based program evolution using the `GpGa<N>` engine.

## Overview

Genetic Programming (GP) evolves tree-structured programs (expression trees) rather than fixed-length gene vectors. Each individual in the population is a `GpChromosome<N>` — a tree whose nodes are elements of a user-defined primitive set `N`. The engine evolves trees by applying subtree crossover, subtree mutation, point mutation, and hoist mutation.

`GpGa<N>` is parameterized on `N: GpNode`, the user's primitive-set enum. The engine works exclusively with `GpChromosome<N>` internally and reuses `selection::factory` and `survivor::factory` from the standard operator system.

**Added in:** v3.0.0

## When to Use

| Problem Type | Why GpGa |
|---|---|
| Symbolic regression | Discover a mathematical expression that fits data |
| Program synthesis | Evolve decision rules or control programs |
| Feature engineering | Automatically construct features from raw inputs |
| Classifier trees | Learn classification rules from labelled data |

## Quick Start

```rust
use genetic_algorithms::gp::{GpConfiguration, GpGa, GpMutation, MathNode, Node};

// Define the fitness function (symbolic regression example).
// This function will receive the root node of each tree.
let fitness_fn = |tree: &Node<MathNode>| -> f64 {
    // Evaluate the tree on training data and return the error.
    let training_data: Vec<(f64, f64)> = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
    training_data.iter().map(|(x, y)| {
        let predicted = Node::<MathNode>::eval_with_vars(tree, &[*x]);
        (predicted - y).powi(2)
    }).sum::<f64>() / training_data.len() as f64
};

let config = GpConfiguration::new()
    .with_population_size(100)
    .with_max_generations(50)
    .with_init_max_depth(4)
    .with_max_depth(8)
    .with_max_node_count(200)
    .with_mutations(vec![
        (GpMutation::SubtreeMutation { mutation_max_depth: 4 }, 0.1),
        (GpMutation::PointMutation { p_per_node: 0.05 }, 0.1),
    ]);

// GpGa::with_ramped_half_and_half uses the standard initializer.
let mut engine = GpGa::<MathNode>::with_ramped_half_and_half(config, fitness_fn);
let result = engine.run().unwrap();

println!("Best fitness: {:.6}", result.best_fitness);
println!("Best program: {}", result.best);   // Lisp S-expression, e.g. (mul x0 (add 1.0 x0))
println!("Generations: {}", result.generations);
```

## Core Types

### `GpNode` Trait

Implement `GpNode` on your own enum to define the function and terminal set.

```rust
use genetic_algorithms::gp::{GpNode, Node};
use rand::Rng;

#[derive(Clone, Debug, Default)]
enum MyNode {
    Add,
    Mul,
    #[default]
    Const(f64),
}

impl GpNode for MyNode {
    fn arity(&self) -> usize {
        match self {
            MyNode::Add | MyNode::Mul => 2,
            MyNode::Const(_) => 0,
        }
    }

    fn evaluate(&self, args: &[f64]) -> f64 {
        match self {
            MyNode::Add => args[0] + args[1],
            MyNode::Mul => args[0] * args[1],
            MyNode::Const(v) => *v,
        }
    }

    fn sample_random_terminal(rng: &mut impl Rng) -> Self {
        MyNode::Const(rng.gen_range(-5.0..=5.0))
    }

    fn all_functions() -> Vec<Self> {
        vec![MyNode::Add, MyNode::Mul]
    }
}
```

**Required methods:**

| Method | Description |
|--------|-------------|
| `arity(&self) -> usize` | Number of child arguments. 0 = terminal (leaf), > 0 = function node. |
| `evaluate(&self, args: &[f64]) -> f64` | Evaluate this node given pre-evaluated child values. |
| `sample_random_terminal(rng) -> Self` | Generate a fresh terminal (used during init and subtree mutation). |
| `all_functions() -> Vec<Self>` | Enumerate all non-terminal variants (used by point mutation). |

### `Node<N>`

The recursive expression tree type:

```rust
pub enum Node<N: GpNode> {
    Function { value: N, children: Vec<Box<Node<N>>> },
    Terminal(N),
}
```

Key methods:
- `node.depth() -> usize` — maximum depth from root to any leaf.
- `node.node_count() -> usize` — total number of nodes.
- `Node::<N>::eval_with_vars(node, &[x0, x1, ...]) -> f64` — evaluates the tree with variable injection (for `MathNode::Var(i)` and `BoolNode` variable support).

### `GpChromosome<N>`

The concrete tree chromosome type. Implements `ChromosomeT` but has no flat DNA representation — `dna()`, `dna_mut()`, and `set_dna()` always panic. Always use `GpGa` instead of `Ga` for tree chromosomes.

```rust
use genetic_algorithms::gp::{GpChromosome, MathNode, Node};

// Construct a chromosome from an existing tree.
let root = Node::Terminal(MathNode::Const(1.0));
let chrom = GpChromosome::with_root(Box::new(root));

// Display as Lisp S-expression (requires N: Display).
println!("{}", chrom);  // → "1.0000"
```

### `TreeChromosome` Supertrait

```rust
pub trait TreeChromosome: ChromosomeT {
    type GpNodeType: GpNode;
    fn tree(&self) -> &Node<Self::GpNodeType>;
    fn tree_mut(&mut self) -> &mut Node<Self::GpNodeType>;
    fn depth(&self) -> usize;
    fn node_count(&self) -> usize;
}
```

Provides tree-specific accessors used internally by the engine and operators.

## Built-in Primitive Sets

### `MathNode` — Symbolic Regression

For evolving mathematical expressions over real-valued inputs.

| Variant | Arity | Operation |
|---------|-------|-----------|
| `Add` | 2 | `args[0] + args[1]` |
| `Sub` | 2 | `args[0] - args[1]` |
| `Mul` | 2 | `args[0] * args[1]` |
| `ProtectedDiv` | 2 | `args[0] / args[1]` (returns 1.0 when `|args[1]| < 1e-10`) |
| `Const(f64)` | 0 | Ephemeral random constant (ERC) — value set during init |
| `Var(usize)` | 0 | Variable placeholder — injected by `eval_with_vars` |

**Variable injection:**

```rust
use genetic_algorithms::gp::{MathNode, Node};

// Evaluate (add x0 (mul x1 2.0)) with x0=3.0, x1=4.0.
// eval_with_vars replaces Var(i) with vars[i] automatically.
let vars = [3.0_f64, 4.0_f64];
let result = Node::<MathNode>::eval_with_vars(&tree, &vars);
```

### `BoolNode` — Classification Trees

For evolving Boolean decision rules.

| Variant | Arity | Operation |
|---------|-------|-----------|
| `And` | 2 | Boolean AND |
| `Or` | 2 | Boolean OR |
| `Not` | 1 | Boolean NOT |
| `Gt` | 2 | `args[0] > args[1]` (returns 1.0 or 0.0) |
| `Lt` | 2 | `args[0] < args[1]` (returns 1.0 or 0.0) |

## Configuration

`GpConfiguration` uses a fluent builder. All parameters are validated at `.build()`.

```rust
use genetic_algorithms::gp::{GpConfiguration, GpCrossover, GpMutation};
use genetic_algorithms::operations::{Selection, Survivor};
use genetic_algorithms::configuration::SelectionConfiguration;

let config = GpConfiguration::new()
    .with_population_size(200)
    .with_max_generations(100)
    .with_init_max_depth(4)       // ramped half-and-half up to this depth
    .with_max_depth(8)             // hard depth limit enforced after crossover/mutation
    .with_max_node_count(200)      // hard node-count limit
    .with_crossover(GpCrossover::SubtreeCrossover)
    .with_mutations(vec![
        (GpMutation::SubtreeMutation { mutation_max_depth: 4 }, 0.10),
        (GpMutation::PointMutation { p_per_node: 0.05 }, 0.10),
        (GpMutation::HoistMutation, 0.05),
    ])
    .with_is_maximization(false)   // minimization (default)
    .with_max_stagnation(Some(20)) // stop after 20 generations without improvement
    .with_fitness_target(Some(0.001)); // stop when best_fitness <= 0.001
```

**Parameter table:**

| Parameter | Default | Description |
|-----------|---------|-------------|
| `population_size` | 100 | Number of individuals in the population |
| `max_generations` | 50 | Maximum number of generations |
| `init_max_depth` | 4 | Maximum depth used during ramped half-and-half initialization |
| `max_depth` | 8 | Hard depth limit enforced after crossover and mutation (max 1000) |
| `max_node_count` | 200 | Hard node-count limit enforced after crossover and mutation (max 100000) |
| `crossover` | `SubtreeCrossover` | GP crossover operator |
| `mutations` | `[(SubtreeMutation{4}, 0.1)]` | List of `(GpMutation, probability)` pairs |
| `selection` | `SelectionConfiguration::default()` (Tournament) | Parent selection configuration |
| `survivor` | `Survivor::Fitness` | Survivor selection strategy |
| `is_maximization` | `false` | `true` → higher fitness is better |
| `max_stagnation` | `None` | Stop after N generations without improvement |
| `fitness_target` | `None` | Stop when best fitness reaches this threshold |

**Validation constraints:**

- `max_depth > 0` and `<= 1000`
- `max_node_count >= max_depth` and `<= 100000`
- `init_max_depth > 0` and `<= max_depth`
- `population_size > 0`
- `max_generations > 0`
- `mutations` list not empty; each probability in `[0.0, 1.0]`

## GP Operators

### Crossover

#### `GpCrossover::SubtreeCrossover`

Standard GP subtree crossover: randomly selects one subtree in each parent and swaps them. Produces two offspring. After the swap, both offspring are checked against `max_depth` and `max_node_count`. If a limit is violated the engine retries (up to 3 attempts); if all attempts fail, the original parents are cloned as offspring.

### Mutation

All mutation operators are applied independently per offspring on each generation. For each operator in the `mutations` list, the engine rolls `rng.random::<f64>() < probability` to decide whether to apply it.

#### `GpMutation::SubtreeMutation { mutation_max_depth }`

Replaces a randomly chosen subtree with a freshly grown random tree (grow method, up to `mutation_max_depth`). The resulting chromosome is checked against the engine's bloat limits. Returns an error if the replacement still exceeds limits — the engine skips mutation and leaves the original.

#### `GpMutation::PointMutation { p_per_node }`

Walks every node in the tree and with probability `p_per_node` replaces its primitive value:
- **Terminal nodes:** replaced with a fresh terminal from `N::sample_random_terminal`.
- **Function nodes:** picks a random alternative from `N::all_functions()` with the same arity. If no alternative exists, the node is unchanged (no error, no panic).

Tree shape (depth, node count) is always preserved.

#### `GpMutation::HoistMutation`

Replaces a randomly chosen function subtree (S1) with one of its own descendants (S2). The chromosome always shrinks. Cannot increase bloat. If no function nodes exist (all terminals), the tree is unchanged.

## Initialization

### `ramped_half_and_half`

The default initialization strategy. Generates diverse trees by iterating over depths from `2` to `init_max_depth` and producing:
- One tree per depth using the **full** method (every branch reaches exactly `d` levels — produces bushy trees).
- One tree per depth using the **grow** method (50% chance of terminal at each level — produces variable-size trees).

Remainder slots (when `pop_size` is not divisible by the number of depths) are filled with `grow_tree(init_max_depth)`.

```rust
// Using the built-in initializer (recommended).
let mut engine = GpGa::<MathNode>::with_ramped_half_and_half(config, fitness_fn);

// Custom initialization function.
let mut engine = GpGa::<MathNode>::new(config, fitness_fn, |pop_size, init_max_depth| {
    // Return Vec<GpChromosome<MathNode>> of length pop_size.
    genetic_algorithms::gp::ramped_half_and_half(pop_size, init_max_depth, &mut rand::rng())
});
```

## GpGa Engine

### Construction

```rust
// Standard: ramped half-and-half initialization.
let engine = GpGa::<MathNode>::with_ramped_half_and_half(config, fitness_fn);

// Custom initialization function.
let engine = GpGa::<MathNode>::new(config, fitness_fn, my_init_fn);
```

### Attaching an Observer

`GpGa` uses the standard `GaObserver<GpChromosome<N>>` system. All lifecycle hooks fire (on_generation_end with `GenerationStats`, on_new_best, on_run_start/end, etc.).

```rust
use std::sync::Arc;
use genetic_algorithms::LogObserver;

let mut engine = GpGa::<MathNode>::with_ramped_half_and_half(config, fitness_fn)
    .with_observer(Arc::new(LogObserver));
```

`GenerationStats` includes `avg_node_count: Option<f64>` which is populated by the GP engine with the mean node count across all individuals — useful for tracking bloat.

### Running

```rust
let result: GpResult<MathNode> = engine.run().unwrap();

println!("Best fitness:   {}", result.best_fitness);
println!("Best program:   {}", result.best);        // Lisp S-expression
println!("Tree depth:     {}", result.best.depth());
println!("Node count:     {}", result.best.node_count());
println!("Generations:    {}", result.generations);
```

### `GpResult<N>`

| Field | Type | Description |
|-------|------|-------------|
| `population` | `Vec<GpChromosome<N>>` | Final evaluated population |
| `best` | `GpChromosome<N>` | Best individual found |
| `best_fitness` | `f64` | Fitness of the best individual |
| `generations` | `usize` | Number of completed generations |

## Bloat Control

Bloat (uncontrolled tree growth) is a well-known challenge in GP. `GpGa` provides two mechanisms:

1. **Hard limits** — `max_depth` and `max_node_count` in `GpConfiguration`. Enforced after every crossover and mutation. Operations that exceed limits are retried (crossover: up to 3 attempts) or discarded (mutation: original unchanged).

2. **Parsimony pressure** — Not yet integrated into `GpGa` directly. For linear chromosomes, use `with_length_penalty` on `Ga`. A GP-specific parsimony mechanism is planned.

## Serialization (serde feature)

With the `serde` feature enabled, `GpChromosome<N>` and `Node<N>` are fully serializable (assuming `N: Serialize + Deserialize`). The fitness function is skipped during serialization and must be re-installed after loading.

```toml
genetic_algorithms = { version = "3.0.0", features = ["serde"] }
```

For trees with depth > ~500 (very unusual), use `serde_stacker` wrappers at the serialization call site to grow the stack dynamically:

```rust
// Serialize a very deep tree safely.
let json = serde_json::to_string_pretty(&chromosome).unwrap();  // Fine for depth <= 64

// For extreme depths only:
// let mut buf = Vec::new();
// let mut serializer = serde_json::Serializer::new(&mut buf);
// chromosome.serialize(serde_stacker::Serializer::new(&mut serializer)).unwrap();
```

> **Security note:** Checkpoint files should be treated as trusted input. Deserialization does not bound allocation — a maliciously crafted JSON blob with an unbounded `children` array could exhaust process memory.

## WASM Compatibility

`GpGa` compiles for `wasm32-unknown-unknown`. Fitness evaluation uses sequential `iter_mut()` instead of `par_iter_mut()` on WASM targets. Tree operators (crossover, mutation) are single-threaded on all targets. No `Instant`-based timing is used unconditionally.

## Performance Tips

- Keep `max_depth` and `max_node_count` reasonable (depth ≤ 17, nodes ≤ 10000 for fast evaluation). Large trees evaluate slowly and consume significant memory.
- `GpMutation::HoistMutation` is free from a bloat perspective — use it to shrink bloated populations.
- `Node<N>::eval_with_vars` avoids variable mutation in the tree — use it whenever your fitness function injects input variables.
- For symbolic regression with large datasets, consider batching: evaluate only a random subset each generation for speed.

## Related

- [Chromosomes & Genotypes](chromosomes.md) — `ChromosomeLength`, `GpChromosome`, `TreeChromosome`
- [Engines Overview](engines.md) — Decision matrix for choosing an engine
- [Error Handling](error.md) — `GaError::TreeDepthExceeded`, `TreeSizeExceeded`
- [Observer System](observer.md) — `GaObserver` lifecycle hooks and `GenerationStats`
- [Source: `src/engines/gp/`](../src/engines/gp/)
- [Tests: `tests/gp.rs`](../tests/gp.rs)
