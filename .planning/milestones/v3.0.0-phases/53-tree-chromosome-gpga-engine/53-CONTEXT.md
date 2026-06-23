# Phase 53: Tree Chromosome + GpGa Engine - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 53 delivers the complete Genetic Programming (GP) subsystem:

1. `GpNode` trait — user implements this on their own enum to define functions and terminals (arity, evaluation, terminal sampling)
2. `GpChromosome<N: GpNode>` — library-provided concrete chromosome that stores a `Box<Node<N>>` tree and implements `TreeChromosome: ChromosomeT` (NOT `LinearChromosome`)
3. `GpGa<U: TreeChromosome>` — dedicated GP engine with ramped half-and-half initialization, subtree crossover, and tree mutation; reuses standard `Selection` and `Survivor` enums; fires `GaObserver<U>` hooks
4. `GpConfiguration` — owns all tree-specific operator config (GpCrossover, GpMutation variants); does NOT add GP variants to the main `Crossover`/`Mutation` enums
5. Bloat control — `max_depth` and `max_node_count` enforced post-crossover and post-mutation per CHR-05; violations return `GaError::TreeDepthExceeded` / `GaError::TreeSizeExceeded`
6. `avg_node_count: f64` added to `GenerationStats` (required by CHR-05)
7. Serde checkpoint support via `serde_stacker` (gated on existing `serde` feature flag, wasm32 compat must be verified first)
8. `Display` for `GpChromosome` — Lisp/prefix S-expression format (e.g., `(+ (* x 3) 2)`)

</domain>

<decisions>
## Implementation Decisions

### GpNode Trait Design (CHR-03, CHR-04)

- **D-01:** `GpNode` is a trait the user implements on their own enum — same pattern as `GeneT`. Required methods: `fn arity(&self) -> usize`, `fn evaluate(&self, args: &[f64]) -> f64`, `fn is_terminal(&self) -> bool`, `fn sample_random_terminal(rng: &mut impl Rng) -> Self` (for ERCs and fresh terminal sampling during initialization and mutation). The last method has a `Self` return, so it's a factory that can produce any terminal variant the user wants.
- **D-02:** Ephemeral random constants (ERCs) are user-owned: the user includes something like `Const(f64)` in their enum and implements `sample_random_terminal()` to produce a fresh randomized constant. No separate ERC marker or library-side tracking needed.
- **D-03:** `GpChromosome<N: GpNode>` is the library's concrete chromosome type. It stores the tree internally (as `Box<Node<N>>`) and implements both `TreeChromosome: ChromosomeT` and `ChromosomeT`. Users do NOT implement `ChromosomeT` themselves — they implement `GpNode` and instantiate `GpChromosome<MyNode>`. Same pattern as `BinaryChromosome` / `RangeChromosome<T>`.
- **D-04:** `TreeChromosome: ChromosomeT` is a supertrait that adds tree-specific methods (e.g., depth(), node_count(), tree root access). It explicitly does NOT extend `LinearChromosome` — `GpChromosome` never implements `dna()` / `set_dna()` / `set_fitness_fn()`.

### Operator Placement (CHR-03, CHR-04)

- **D-05:** GP operators live in `GpConfiguration` — NOT in the main `Crossover` and `Mutation` enums. `GpConfiguration` has its own `crossover: GpCrossover` and `mutations: Vec<(GpMutation, f64)>` (variant + probability pairs). This keeps GP concerns fully separate from the linear-chromosome operator enums.
- **D-06:** `GpCrossover` enum has one variant: `SubtreeCrossover`. Selects a random crossover point in each parent tree and swaps the subtrees. Respects `max_depth` and `max_node_count` — returns `GaError::TreeDepthExceeded` / `GaError::TreeSizeExceeded` if the result would exceed limits.
- **D-07:** `GpMutation` enum has three variants, all three implemented in Phase 53:
  - `SubtreeMutation` — replaces a random subtree with a newly generated random tree (up to configurable `mutation_max_depth`)
  - `PointMutation` — replaces a node with another node of the **same arity** from the user's primitive set
  - `HoistMutation` — replaces a subtree with one of its own subtrees (structural shrink; reduces tree size)
- **D-08:** Each `GpMutation` variant in the `mutations` vec carries its own application probability. The engine rolls per-mutation per generation (same semantics as multi-operator mutation in the standard GA).

### GpGa Engine Structure (CHR-04, CHR-05)

- **D-09:** `GpGa<U: TreeChromosome>` is a dedicated engine in `src/engines/gp/`. It does NOT share code with `Ga<U: LinearChromosome>` — the loop structure differs (ramped init, per-operator bloat enforcement, GP-specific crossover/mutation dispatch).
- **D-10:** `GpGa` reuses the standard `Selection` enum and `Survivor` enum unchanged. Selection and survival are fitness-based and do not touch the DNA/tree structure — they work on any `ChromosomeT`. `GpConfiguration` embeds `SelectionConfiguration` and `SurvivorConfiguration` sub-configs.
- **D-11:** `GpGa` fires the standard `GaObserver<U>` hooks: `on_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_finish`. No `GpObserver` sub-trait in Phase 53. GP-specific events (e.g., `on_bloat_detected`) are deferred to a future phase.
- **D-12:** `avg_node_count: f64` is added to `GenerationStats`. Computed as the mean node count across the surviving population each generation. `avg_depth: f64` is NOT added — only the required CHR-05 field.

### Serde / Checkpoint (CHR-06)

- **D-13:** Serde support is gated on the existing `serde` feature flag. `serde_stacker` is pulled in conditionally only when `serde` is enabled. **Before committing to `serde_stacker`**: verify wasm32 compatibility with `cargo check --target wasm32-unknown-unknown --features serde`. If it fails, fall back to an iterative serde approach (explicit stack-based serialization without serde_stacker).
- **D-14:** CI must include a serde test with a tree of depth >= 64 to validate that serialization doesn't stack-overflow (per CHR-06 success criterion 4).

### Expression Display (CHR-07)

- **D-15:** `GpChromosome<N>` implements `std::fmt::Display` using Lisp/prefix S-expression format: `(+ (* x 3) 2)`, `(and (gt x 0.5) (lt y 1.0))`. No infix variant in Phase 53. This format is standard in the GP literature, is unambiguous without precedence rules, and the Display impl is a simple recursive prefix walk with no operator-precedence logic.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §CHR (CHR-03, CHR-04, CHR-05, CHR-06, CHR-07) — authoritative scope for Phase 53

### Prior Phase Decisions (architectural constraints)
- `.planning/phases/47-architecture-audit-chromosomet-split/47-CONTEXT.md` — ChromosomeT/LinearChromosome split: `TreeChromosome: ChromosomeT` must NOT extend `LinearChromosome`; `GpChromosome` must not implement `dna()` / `set_dna()` / `set_fitness_fn()`
- `.planning/phases/50-lexicase-selection/50-CONTEXT.md` — `MultiCaseFitness: ChromosomeT` (not LinearChromosome); `GpChromosome` may implement it for GP program synthesis use cases
- `.planning/STATE.md` — v3.0.0 accumulated decisions: `Box<N>` for tree nodes (arena rejected), `GpGa` as separate engine

### Existing Engine Patterns (structural analogs for GpGa)
- `src/engines/ga.rs` — main GA run loop: observer hook pattern, generation cycle structure, `GenerationStats` computation and dispatch
- `src/engines/de/engine.rs` — alternate engine pattern in `src/engines/` subdirectory
- `src/engines/cellular/engine.rs` — another alternate engine; shows how engines embed standard selection/survivor configs

### Existing Chromosome Patterns (analogs for GpChromosome)
- `src/types/chromosomes/binary.rs` — canonical concrete chromosome implementation; `GpChromosome<N>` follows the same impl structure
- `src/types/chromosomes/range.rs` — `RangeChromosome<T>` generic chromosome pattern
- `src/traits/chromosome.rs` — current `ChromosomeT` (all methods); after Phase 47, ChromosomeT becomes minimal (fitness+age only); `GpChromosome` must implement only the minimal core

### Configuration Patterns
- `src/configuration.rs` — `GaConfiguration` and all sub-configs: `SelectionConfiguration`, `SurvivorConfiguration` — `GpConfiguration` embeds these as sub-configs
- `src/operations.rs` — `Selection` and `Survivor` enums: `GpGa` uses these directly (no GP-specific selection/survivor variants)

### Operator Patterns
- `src/operations/crossover/` — existing crossover implementations; `GpCrossover` lives in `src/engines/gp/crossover.rs` (not here, to avoid polluting linear operator enums)
- `src/operations/mutation/` — existing mutation implementations; `GpMutation` lives in `src/engines/gp/mutation.rs`

### Stats
- `src/stats.rs` — `GenerationStats` struct: add `avg_node_count: f64` here

### Observer
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait: `GpGa` fires the standard hooks (no new methods)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/de/engine.rs` — best structural analog for `GpGa`: a dedicated engine in its own subdirectory that holds standard `SelectionConfiguration`, runs a generation loop, and fires `GaObserver` hooks. Copy this structure, not `ga.rs` (which is 123K and carries much non-GP machinery).
- `src/operations/selection.rs` `factory()` — reused by `GpGa` unchanged; tournament, roulette etc. work on scalar `fitness()` which `GpChromosome` implements.
- `src/rng::make_rng()` — needed for random subtree crossover point selection, random mutation point selection, and ramped half-and-half initialization.
- `src/stats.rs` `GenerationStats` — extend with `avg_node_count: f64`; all existing fields stay.

### Established Patterns
- `GeneT` trait pattern in `src/traits/gene.rs` — `GpNode` follows the same "trait-on-user-type" design philosophy.
- `#[cfg(not(target_arch = "wasm32"))]` gates on `par_iter()` — `GpGa` initialization and crossover/mutation MUST use the same gates.
- `Option<Arc<dyn GaObserver<U>>>` — zero-cost observer pattern; `GpGa` stores `observer: Option<Arc<dyn GaObserver<U>>>` identically to `ga.rs`.
- `pub(crate)` fields + public accessors on config — ARCH-04 pattern; `GpConfiguration` follows same convention.
- Tests in `tests/` only — per project feedback, no inline `#[cfg(test)]` modules.

### Integration Points
- `src/lib.rs` — new public exports: `GpNode`, `GpChromosome`, `GpGa`, `GpConfiguration`, `TreeChromosome`
- `src/engines/` — add `gp/` subdirectory alongside `de/`, `cellular/`, etc.
- `src/stats.rs` — add `avg_node_count: f64` to `GenerationStats`
- `src/error.rs` — add `TreeDepthExceeded` and `TreeSizeExceeded` to `GaError` enum

</code_context>

<specifics>
## Specific Ideas

- `Box<N>` (not arena) for the tree node recursive enum — locked in STATE.md. Subtree clone is O(subtree); arena index-remapping across arenas was judged too complex.
- Ramped half-and-half init: half the initial population uses "grow" (random trees stopping early when a terminal is chosen), half use "full" (all leaves at exactly `init_max_depth`). `init_max_depth` is separate from the runtime `max_depth` in `GpConfiguration` (common practice: init trees shallower than the allowed max).
- `serde_stacker` depth threshold: depth >= 64 must serialize without stack overflow (CHR-06 success criterion). CI serde test with a hand-crafted depth-64 tree validates this.
- `sample_random_terminal(rng)` on `GpNode` is a static-factory style method (`Self` return, not `&self`) — enables the engine to produce fresh terminals without holding a chromosome reference.
- `GpNode::arity()` takes `&self` (dynamic per-variant arity). PointMutation must query arity to find a compatible replacement node — the engine iterates all non-terminal variants of the user's primitive set. This requires the user to provide a way to enumerate all function variants. Consider `fn all_functions() -> Vec<Self>` on `GpNode` (or `fn all_non_terminals() -> Vec<Self>` for point mutation).

</specifics>

<deferred>
## Deferred Ideas

- `GpObserver` sub-trait with `on_bloat_detected(generation, avg_depth, avg_nodes)` and `on_tree_depth_exceeded(count)` — GP-specific observability. Deferred to a future phase (mentioned in REQUIREMENTS.md §Future Requirements).
- `avg_depth: f64` in `GenerationStats` — not required by any current requirement; low cost to add later.
- Strongly-typed GP (type-checked function/terminal compatibility) — out of scope per REQUIREMENTS.md §Out of Scope.
- Grammar-guided evolution / grammatical evolution — separate paradigm, out of scope.
- Infix expression display — prefix S-expression is the canonical form; infix can be added as a `.to_infix()` helper in a future phase if users request it.
- `fn all_non_terminals() -> Vec<Self>` on `GpNode` for PointMutation — researcher should evaluate whether this is the right API or if there's a cleaner approach (e.g., the engine tries all arities and the user can mark unsupported combinations by returning `None`).

</deferred>

---

*Phase: 53-tree-chromosome-gpga-engine*
*Context gathered: 2026-05-25*
