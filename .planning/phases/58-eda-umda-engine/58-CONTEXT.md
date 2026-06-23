# Phase 58: EDA / UMDA Engine - Context

**Gathered:** 2026-06-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 58 implements `EdaEngine<U>` — an Estimation of Distribution Algorithm (UMDA variant) — as a new engine module under `src/engines/eda/`. Instead of crossover and mutation, EDA learns a probabilistic model from the best individuals each generation and samples the new population from that model.

This phase:
1. **Implements `EdaEngine<U>`** following the established engine pattern (`engine.rs`, `configuration.rs`, `mod.rs` under `src/engines/eda/`) with an `EdaResult<U>` return type.
2. **Supports any `LinearChromosome`** with dual-strategy probabilistic model: Bernoulli for binary genes (when `U::Gene` does NOT implement `RealGene`), Gaussian univariate for real-valued genes (when `U::Gene: RealGene`).
3. **Selects parents via configurable `selection_ratio: f64`** (default 0.5) — the top `selection_ratio` fraction by fitness feeds the model estimation.
4. **Returns `learned_model: EdaModel`** in `EdaResult` — an enum capturing either the Bernoulli probability vector or Gaussian mean/std vectors learned at the final generation.
5. **Wires `GaObserver` hooks** from day 1 (mandatory per CLAUDE.md observability initiative).
6. **Demonstrates on `eda_trap`** example — a deceptive trap function where GA crossover/mutation fails but EDA succeeds by modeling the joint distribution correctly.

</domain>

<decisions>
## Implementation Decisions

### Chromosome Scope

- **D-01:** `EdaEngine<U>` accepts any `U: LinearChromosome`. The probabilistic model strategy is selected at compile time via trait bounds on `U::Gene`:
  - If `U::Gene: RealGene` → Gaussian univariate model: estimate `(mean_i, std_i)` per gene position from selected parents; sample `N(mean_i, std_i)` for offspring.
  - Otherwise → Bernoulli model (UMDA classic): estimate `p_i = count(gene_i == 1) / num_parents` per position; sample Bernoulli(`p_i`) for offspring. Uses `gene.id()` == 1 as the "1" indicator for binary genes.
  - Compile-time dispatch via two `impl` blocks or a helper trait — planner decides the mechanism.

### Parent Selection

- **D-02:** `EdaConfiguration` exposes `selection_ratio: f64` (default `0.5`). At each generation, the top `floor(pop_size * selection_ratio)` individuals by fitness feed the model estimation. Minimum 1 parent enforced (clamp). `ProblemSolving` field controls sort direction (Maximize vs. Minimize).

### EdaResult

- **D-03:** `EdaResult<U>` contains:
  - `population: Vec<U>` — final population
  - `best: U` — best individual found
  - `best_fitness: f64` — best fitness achieved
  - `generations: usize` — actual generations run
  - `learned_model: EdaModel` — the probabilistic model estimated at the final generation

- **D-04:** `EdaModel` is a public enum:
  ```rust
  pub enum EdaModel {
      Bernoulli(Vec<f64>),                          // p_i per gene position
      Gaussian { means: Vec<f64>, stds: Vec<f64> }, // mean_i, std_i per gene
  }
  ```

### Observer Hooks

- **D-05:** `EdaEngine` includes `Option<Arc<dyn GaObserver<U> + Send + Sync>>` and a `with_observer()` builder method from day 1. Standard 5 hooks: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end`. Mandatory per CLAUDE.md.

### Example

- **D-06:** Example is `eda_trap` — a deceptive trap function on a binary chromosome. The trap function rewards all-zeros or all-ones (the two global optima) but misdirects local search toward an all-zeros local optimum. Demonstrates EDA's advantage over classical GA crossover/mutation on deceptive landscapes.

### Claude's Discretion

- Probability clamping for Bernoulli model (e.g., clip to `[0.01, 0.99]` to avoid degenerate distributions)
- Std deviation floor for Gaussian model to prevent degenerate sampling
- Whether `EdaConfiguration` also exposes `max_generations`, `fitness_target`, `population_size`, `problem_solving` as direct fields (mirrors `CmaConfiguration` — likely yes)
- Whether the `Gaussian` variant is gated on `U::Gene: RealGene` at the type level or dispatched via an internal helper
- `GenerationStats` field population (follow PSO/CMA pattern — use existing fields or add model-diversity proxy)
- Default `population_size` if 0 is passed (suggest `100` — EDA typically needs larger populations than GA)
- Whether `EdaModel` derives `Debug`, `Clone`, `serde::Serialize` (if `serde` feature enabled)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Engine Pattern (follow exactly)
- `src/engines/pso/engine.rs` — Most recent engine implementation; `EdaEngine` follows the same struct layout, `new()`, and `run()` signature
- `src/engines/pso/configuration.rs` — Most recent configuration pattern; `EdaConfiguration` mirrors this structure
- `src/engines/pso/mod.rs` — Module wiring pattern (most recent)
- `src/engines/cma/engine.rs` — Secondary reference; internal state pattern (`CmaState`)

### Trait System (chromosome bound)
- `src/traits/linear_chromosome.rs` — `LinearChromosome` supertrait; `EdaEngine` chromosome bound
- `src/traits/real_gene.rs` — `RealGene: GeneT` trait; used for Gaussian model dispatch (`U::Gene: RealGene`)
- `src/traits/chromosome.rs` — `ChromosomeT` base trait

### Gene Types for Example
- `src/types/genotypes/binary.rs` — `Binary` gene type; `id() == 1` used as Bernoulli indicator for the `eda_trap` example
- `src/types/chromosomes/binary.rs` — `Binary` chromosome; primary type for `eda_trap`

### Observer Integration
- `src/engines/pso/engine.rs` — Most recent observer wiring; use as reference for all 5 hook call sites
- `src/observer/mod.rs` — `GaObserver<U>` trait definition

### lib.rs Re-export Pattern
- `src/lib.rs` — add `pub use engines::eda::{EdaEngine, EdaConfiguration, EdaResult, EdaModel}` re-exports

### Configuration Pattern
- `src/engines/cma/configuration.rs` — Reference for field layout: `population_size`, `max_generations`, `problem_solving`, `fitness_target`, optional tuning fields

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/pso/engine.rs` (run loop, observer wiring) — Direct template: initialize, loop generations, fire hooks, return `*Result`
- `crate::rng::make_rng()` — Standard RNG initialization (used by all engines)
- `src/configuration.rs` (`ProblemSolving`) — Reuse for `EdaConfiguration.problem_solving` (sort direction)
- `src/stats.rs` (`GenerationStats`) — Reuse existing stats struct per generation
- `src/traits/real_gene.rs` (`RealGene`) — `gene.real_value()` to read float value for Gaussian estimation; `gene.with_real_value(v)` to write sampled value

### Established Patterns
- Engine struct holds `config`, `init_fn: Arc<Fn>`, `fitness_fn: Arc<FitnessFn>`, `observer: Option<Arc<dyn GaObserver<U>>>`
- `run()` returns `*Result<U>` with `population`, `best`, `best_fitness`, `generations`
- `#[cfg(not(target_arch = "wasm32"))]` gates around `Instant::now()` / `elapsed()`
- EDA main loop is inherently sequential (no rayon needed for model estimation) — WASM-safe by default; fitness evaluation of offspring CAN use rayon (gate with cfg)
- All tests in `tests/eda.rs`, never inline in `src/`

### Integration Points
- `src/engines/mod.rs` → add `pub mod eda`
- `src/lib.rs` → add EDA re-exports
- `examples/eda_trap.rs` → new example (Binary chromosome, deceptive trap function)
- `tests/eda.rs` → new test file

### UMDA Core Loop (for planner reference)
Standard UMDA per generation:
```
1. Sort population by fitness (top selection_ratio fraction = selected_parents)
2. Estimate model from selected_parents:
   - Bernoulli: p_i = mean(gene_i.id() as f64) for each position i
   - Gaussian:  mean_i = mean(gene_i.real_value()), std_i = std(gene_i.real_value())
3. Sample pop_size new individuals from model:
   - Bernoulli: gene_i = Bernoulli(p_i) → gene with id = rng.random() < p_i ? 1 : 0
   - Gaussian:  gene_i = clamp(N(mean_i, std_i), lo_i, hi_i) via gene.with_real_value(v)
4. Evaluate fitness of new population (parallel where possible)
5. Update best if improved
6. Fire observer hooks
```

</code_context>

<specifics>
## Specific Ideas

- **`eda_trap` example:** Trap function on a Binary chromosome of length 30-40. The trap function divides genes into blocks of k (e.g., k=5); within each block, if all bits are 1 → reward 5, else reward (k-1-count). GA crossover/mutation fails because it disrupts block structure; EDA succeeds by learning marginal probabilities per position that converge to the global optimum.
- **`EdaModel::Bernoulli`** stores one `f64` per gene position — after convergence, values should approach 0.0 or 1.0.
- **`EdaModel::Gaussian`** stores parallel `means` and `stds` vecs — both length = chromosome length.
- Probability clamping for Bernoulli: suggested `[0.01, 0.99]` to prevent early degeneration.

</specifics>

<deferred>
## Deferred Ideas

- **Multivariate EDA (BMDA, MIMIC, BOA)** — Learning dependencies between gene positions. Full dependency structure is a separate algorithm family; out of scope for UMDA phase.
- **Population-Based Incremental Learning (PBIL)** — Maintains probability vector across generations without resampling full population. Related but distinct; could be a future `EdaVariant` enum variant.
- **Adaptive selection_ratio** — Decay the selection ratio over generations as the model converges. Future enhancement.
- **Discrete PSO** — Came up as related concept during discussion; belongs in PSO phase extension if ever needed.

</deferred>

---

*Phase: 58-eda-umda-engine*
*Context gathered: 2026-06-04*
