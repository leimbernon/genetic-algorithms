# Phase 10: Single-population Examples - Research

**Researched:** 2026-03-22
**Domain:** Rust GA library example authoring — continuous optimization, binary feature selection, multimodal niching
**Confidence:** HIGH (all findings from direct source inspection)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Follow `examples/onemax_binary.rs` as the reference template for all three examples
- Each example must have a `/*!` doc block at the top with: problem description, features demonstrated, and `cargo run --example <name>` command
- Constants section at the top of `main()` for all configurable parameters (pop size, generations, etc.)
- Progress via `run_with_callback` reporting every N generations (same pattern as onemax)
- Final result shown via `match result { Ok(...) => ..., Err(...) => ... }`
- Feature Selection: small embedded realistic dataset — Iris-style with hardcoded data (no external file or dependency)
- Feature Selection: 20 features total, with a known subset of relevant ones baked into the fitness function
- Feature Selection: fitness = count relevant features selected minus penalty for irrelevant ones selected
- Feature Selection: show best binary feature mask in output
- Feature Selection: adaptive GA enabled via `with_adaptive_ga(true)`

### Claude's Discretion
- Rastrigin: number of dimensions, specific operators (Gaussian vs Creep), convergence criterion
- Niching: which multimodal function to use, how to print multiple distinct peaks in output
- Exact dataset values for Feature Selection (Iris-style structure, values Claude's choice)
- Report interval N for each example

### Deferred Ideas (OUT OF SCOPE)
- None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| EX-01 | User can run a Rastrigin continuous optimization example using `Range<f64>` chromosomes and gaussian/creep mutation operators | `range_random_initialization` + `Mutation::Gaussian`/`Mutation::Creep` confirmed in source; `ProblemSolving::Minimization` applies |
| EX-05 | User can run a Feature Selection example using Binary chromosomes with adaptive GA to select optimal ML feature subsets | `binary_random_initialization` confirmed; `with_adaptive_ga(true)` confirmed in `Ga` impl; Binary chromosome confirmed |
| EX-06 | User can run a Niching / Fitness Sharing example that maintains multiple solutions in a multimodal optimization landscape | `NichingConfiguration` confirmed; builder methods `with_niching_enabled`, `with_niching_sigma_share`, `with_niching_alpha` confirmed |
</phase_requirements>

---

## Summary

Phase 10 produces three new files in `examples/`: `rastrigin.rs`, `feature_selection.rs`, and `niching.rs`. All three are pure example files — no changes to `src/`. The existing examples (`onemax_binary.rs`, `nqueens_range.rs`) serve as complete style and API references. All required operators, initializers, and configuration builder methods already exist in the library.

The primary risk is API surface detail: getting the exact method names and type signatures correct for each example without compilation errors. This research resolves all of them from direct source inspection.

**Primary recommendation:** Implement all three examples as mechanical translations of the `onemax_binary.rs` template, substituting the correct chromosome type, initializer, fitness function, and configuration methods verified below.

---

## Standard Stack

No new dependencies. All imports come from the `genetic_algorithms` crate itself.

### Core Imports (all three examples share this base)
```rust
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig};
use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
```

### Per-Example Additional Imports

**rastrigin.rs:**
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
```

**feature_selection.rs:**
```rust
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
```

**niching.rs:**
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::traits::NichingConfig;
```

---

## Architecture Patterns

### File Layout
```
examples/
├── onemax_binary.rs      (existing — style reference)
├── nqueens_range.rs      (existing — Range<T> reference)
├── rastrigin.rs          (NEW — EX-01)
├── feature_selection.rs  (NEW — EX-05)
└── niching.rs            (NEW — EX-06)
```

### Canonical Example Structure (from onemax_binary.rs)
```
/*! doc block — problem, features, cargo run command */

use ...;

fn main() {
    // --- Constants ---
    const POP_SIZE: usize = ...;
    const MAX_GENERATIONS: usize = ...;
    // ...

    // --- Fitness function ---
    let fitness_fn = |dna: &[GeneType]| -> f64 { ... };

    // --- Build GA ---
    let mut ga = Ga::new()
        .with_genes_per_chromosome(N)
        .with_population_size(POP_SIZE)
        .with_initialization_fn(...)
        .with_fitness_fn(fitness_fn)
        .with_selection_method(Selection::...)
        .with_crossover_method(Crossover::...)
        .with_mutation_method(Mutation::...)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::...)
        .with_max_generations(MAX_GENERATIONS)
        .build()
        .expect("...");

    println!("== Example Name ==");
    // print config summary

    // --- Run with callback ---
    let report_interval = N;
    let result = ga.run_with_callback(
        Some(|gen: &usize, pop: &Population<ChromosomeType>, _stats: &GenerationStats, _cause: &TerminationCause| -> std::ops::ControlFlow<()> {
            println!("...");
            std::ops::ControlFlow::Continue(())
        }),
        report_interval,
    );

    // --- Match result ---
    match result {
        Ok(population) => { println!("Best: ...", population.best_chromosome.fitness); }
        Err(e) => { println!("GA failed: {:?}", e); }
    }
}
```

---

## Per-Example Implementation Notes

### rastrigin.rs (EX-01)

**Problem:** Minimize the Rastrigin function — a standard continuous optimization benchmark with many local minima. Global minimum is 0.0 at origin.

**Rastrigin formula (n dimensions):**
```
f(x) = A*n + sum_i [ x_i^2 - A * cos(2*pi*x_i) ]
where A = 10, x_i in [-5.12, 5.12]
```

**Recommended discretion choices:**
- Dimensions: 5 (small enough to converge reliably, interesting enough to demo)
- Mutation: `Mutation::Gaussian` (natural fit for continuous; Creep is also valid)
- Report interval: 100 generations

**Initializer pattern** (from nqueens_range.rs):
```rust
let alleles = vec![RangeGenotype::new(0.0_f64, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();
let mut ga = Ga::new()
    .with_initialization_fn(move |genes_per_chromosome, _, _| {
        range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
    })
```

**Fitness function:**
```rust
let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
    let a = 10.0;
    let n = dna.len() as f64;
    a * n + dna.iter().map(|g| {
        g.value.powi(2) - a * (2.0 * std::f64::consts::PI * g.value).cos()
    }).sum::<f64>()
};
```

**ProblemSolving:** `Minimization` — lower is better, global min = 0.0
**Operators:** `Selection::Tournament`, `Crossover::Uniform`, `Mutation::Gaussian`
**Success check in match:** print whether fitness < 1.0 (near-global-minimum)

---

### feature_selection.rs (EX-05)

**Problem:** Select which of 20 binary features are relevant for a classification task. Embedded Iris-style dataset — 4 truly relevant features padded to 20 with noise columns.

**Adaptive GA:** `with_adaptive_ga(true)` — method verified in `src/ga.rs` line 349.

**Dataset design (Claude's discretion — hardcoded):**
- 20 features: features 0-3 are "relevant" (Iris sepal/petal dims), features 4-19 are noise
- `RELEVANT_FEATURES: [usize; 4] = [0, 1, 2, 3]`
- Fitness: count of relevant selected - 0.5 * count of irrelevant selected

**Fitness function:**
```rust
const RELEVANT: &[usize] = &[0, 1, 2, 3];
let fitness_fn = |dna: &[Binary]| -> f64 {
    let relevant_selected = RELEVANT.iter().filter(|&&i| dna[i].value).count() as f64;
    let irrelevant_selected = dna.iter().enumerate()
        .filter(|(i, g)| !RELEVANT.contains(i) && g.value)
        .count() as f64;
    relevant_selected - 0.5 * irrelevant_selected
};
```

**Initializer:** `binary_random_initialization` used directly (no closure wrapper needed — matches `with_initialization_fn` signature).

**ProblemSolving:** `Maximization`
**Output:** print best chromosome's feature mask (which bits are true/false) + fitness

**Printing the feature mask in match Ok:**
```rust
let mask: Vec<usize> = population.best_chromosome.dna()
    .iter().enumerate()
    .filter(|(_, g)| g.value)
    .map(|(i, _)| i)
    .collect();
println!("Selected features: {:?}", mask);
```

---

### niching.rs (EX-06)

**Problem:** Multimodal optimization — find multiple peaks of a function simultaneously using fitness sharing to prevent convergence to a single peak.

**Recommended function:** A simple 1D or 2D multimodal function. For clear demo: sine-based multimodal over [0, 10] with multiple clear peaks.

```rust
// f(x) = sin(x) * x / 2 + 5   (has ~3 local maxima in [0, 10])
// Or simpler: count how many "peak positions" the population covers
```

Better choice for clear multi-peak demo — 1D with 3 explicit peaks:
```rust
let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
    let x = dna[0].value;
    // Three Gaussian peaks at x=2, x=5, x=8
    let peak = |c: f64, h: f64| h * (-((x - c).powi(2)) / (2.0 * 0.5_f64.powi(2))).exp();
    peak(2.0, 1.0) + peak(5.0, 0.9) + peak(8.0, 0.8)
};
```

**Niching builder methods** (verified from `src/traits/configuration.rs`):
```rust
.with_niching_enabled(true)
.with_niching_sigma_share(0.5)   // sharing radius — must match peak separation
.with_niching_alpha(1.0)         // sharing function shape
```

**Note:** `NichingConfig` trait must be imported explicitly for these methods to be in scope.

**Output strategy:** After `Ok(population)`, scan all chromosomes and report distinct peaks by clustering near known peak positions (x ≈ 2, 5, 8). Or simply print top-5 chromosomes sorted by position to show spread.

**ProblemSolving:** `Maximization`
**Recommended discretion choices:**
- 1 dimension, allele range [0.0, 10.0]
- `sigma_share = 1.5` (half the gap between peaks)
- Report interval: 50 generations

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Adaptive parameter adjustment | Custom crossover/mutation probability logic | `with_adaptive_ga(true)` | Already implemented in GA loop |
| Fitness sharing | Custom penalty function | `with_niching_enabled(true)` + sigma/alpha | NichingConfiguration + sharing.rs already implements |
| Continuous gene initialization | Custom random Vec generation | `range_random_initialization` | Handles allele bounds correctly |
| Binary gene initialization | Custom bool Vec | `binary_random_initialization` | Correct genotype struct wrapping |

---

## Common Pitfalls

### Pitfall 1: Wrong adaptive GA method name
**What goes wrong:** Calling `with_adaptive_enabled(true)` (non-existent) instead of `with_adaptive_ga(true)` — compile error.
**How to avoid:** Method is `with_adaptive_ga` — verified at `src/ga.rs:349`.

### Pitfall 2: Forgetting `NichingConfig` trait import
**What goes wrong:** `with_niching_enabled` not found on `Ga` — compile error because the trait is not in scope.
**How to avoid:** Add `use genetic_algorithms::traits::NichingConfig;` to niching.rs.

### Pitfall 3: `binary_random_initialization` passed directly vs wrapped
**What goes wrong:** `binary_random_initialization` can be passed directly as a function pointer (unlike `range_random_initialization` which requires a closure to capture alleles). Wrapping binary in a closure is unnecessary but harmless; using it directly matches onemax_binary.rs.
**How to avoid:** Pass `binary_random_initialization` directly without a closure wrapper.

### Pitfall 4: Range allele setup for Rastrigin
**What goes wrong:** Passing `None` for alleles in `range_random_initialization` gives genes with default bounds, not `[-5.12, 5.12]`. The function requires explicit alleles.
**How to avoid:** Always construct the allele vec and pass `Some(&alleles_clone)` in the closure, mirroring nqueens_range.rs pattern.

### Pitfall 5: `ProblemSolving` mismatch
**What goes wrong:** Using `Maximization` for Rastrigin (minimize) or `Minimization` for feature selection (maximize) — the GA will evolve in the wrong direction silently.
**How to avoid:** Rastrigin = `Minimization`, Feature Selection = `Maximization`, Niching = `Maximization`.

### Pitfall 6: `sigma_share` too small or too large for niching
**What goes wrong:** If `sigma_share` is smaller than the intra-peak variation, sharing has no effect. If too large, it penalizes solutions at different peaks equally and collapses diversity.
**How to avoid:** Set `sigma_share` to approximately half the distance between adjacent peaks.

---

## Code Examples

### Verified: Niching builder chain
```rust
// Source: src/traits/configuration.rs:67-74, src/niching/configuration.rs
use genetic_algorithms::traits::NichingConfig;

let mut ga = Ga::new()
    // ... other config ...
    .with_niching_enabled(true)
    .with_niching_sigma_share(1.5)
    .with_niching_alpha(1.0)
    .build()
    .expect("...");
```

### Verified: Adaptive GA builder
```rust
// Source: src/ga.rs:349, src/traits/configuration.rs:114
let mut ga = Ga::new()
    // ... other config ...
    .with_adaptive_ga(true)
    .build()
    .expect("...");
```

### Verified: Range initializer with bounds
```rust
// Source: examples/nqueens_range.rs:48-55
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;

let alleles = vec![RangeGenotype::new(0.0_f64, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();
// In builder:
.with_initialization_fn(move |genes_per_chromosome, _, _| {
    range_random_initialization(genes_per_chromosome, Some(&alleles_clone), Some(false))
})
```

### Verified: TerminationCause variants (for callback)
```rust
// Source: src/ga.rs:77-87
// GenerationLimitReached, FitnessTargetReached, StagnationReached,
// ConvergenceReached, TimeLimitReached, CallbackRequested, NotTerminated
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (Cargo.toml controls) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| EX-01 | `cargo run --example rastrigin` compiles and runs without error | smoke | `cargo build --example rastrigin` | ❌ Wave 0 |
| EX-05 | `cargo run --example feature_selection` compiles and runs without error | smoke | `cargo build --example feature_selection` | ❌ Wave 0 |
| EX-06 | `cargo run --example niching` compiles and runs without error | smoke | `cargo build --example niching` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo build --example <name>` (compile check)
- **Per wave merge:** `cargo build --examples && cargo test && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `examples/rastrigin.rs` — covers EX-01
- [ ] `examples/feature_selection.rs` — covers EX-05
- [ ] `examples/niching.rs` — covers EX-06

---

## Sources

### Primary (HIGH confidence)
- `examples/onemax_binary.rs` — canonical style template (doc block, constants, callback, match)
- `examples/nqueens_range.rs` — Range chromosome initialization pattern
- `src/ga.rs:349` — `with_adaptive_ga(true)` method confirmed
- `src/traits/configuration.rs:67-74, 114` — `NichingConfig` trait, `with_adaptive_ga` signature
- `src/niching/configuration.rs` — `NichingConfiguration` fields and builder methods
- `src/initializers/binary_initializer.rs:30` — `binary_random_initialization` signature
- `src/initializers/range_initializer.rs:37` — `range_random_initialization` signature
- `src/operations/mutation.rs:61-165` — `Mutation::Gaussian`, `Mutation::Creep`, `Mutation::BitFlip` confirmed

### Secondary (MEDIUM confidence)
- Rastrigin function formula — well-established benchmark, standard mathematical definition

---

## Metadata

**Confidence breakdown:**
- API method names: HIGH — verified directly in source files
- Example structure: HIGH — copied from existing working examples
- Rastrigin fitness function: HIGH — standard mathematical formula
- Niching sigma_share tuning guidance: MEDIUM — derived from fitness-sharing theory
- Feature selection fitness design: HIGH — straightforward count/penalty design matching CONTEXT.md spec

**Research date:** 2026-03-22
**Valid until:** 2026-04-22 (stable library, no external deps to go stale)
