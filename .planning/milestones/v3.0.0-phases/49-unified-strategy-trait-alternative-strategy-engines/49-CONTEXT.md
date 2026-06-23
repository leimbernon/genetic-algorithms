# Phase 49: Unified Strategy Trait + Alternative Strategy Engines - Context

**Gathered:** 2026-05-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 49 delivers a `Strategy<U>` trait that unifies `Ga<U>`, `HillClimbEngine<U>`, and `PermutateEngine<U>` under a common `run()` / `best()` interface, enabling runtime algorithm swapping via `Box<dyn Strategy<U>>`. It adds two new engine types: `HillClimbEngine<U>` (stochastic and steepest-ascent modes behind a `HillClimbMode` enum field) and `PermutateEngine<U>` (exhaustive evaluation of a user-supplied candidate set with a configurable safety gate). Both engines wire `GaObserver` hooks. No changes to `Ga<U>` internals.

</domain>

<decisions>
## Implementation Decisions

### Strategy<U> Trait Shape (STR-01)

- **D-01:** `Strategy<U>` trait exposes exactly two methods: `fn run(&mut self) -> Result<(), GaError>` and `fn best(&self) -> Option<&U>`. No builder methods on the trait — stays minimal and dyn-safe.
- **D-02:** `with_observer()` lives only on individual engine structs (concrete types), not on the trait. Observer wiring happens before boxing: `Box::new(engine.with_observer(obs)) as Box<dyn Strategy<U>>`. Consistent with how `Ga::with_observer()` works.
- **D-03:** The trait must be dyn-safe. `run() -> Result<(), GaError>` and `best() -> Option<&U>` are both dyn-safe. No associated types, no generics on methods.

### Observer Hook Mapping (STR-02, STR-03, STR-04)

- **D-04:** Each hill-climb iteration is treated as a "generation" for observer purposes. Hooks that fire per iteration: `on_run_start`, `on_generation_start(iteration)`, `on_new_best(iteration, best)` (only when a better neighbor is accepted), `on_generation_end(stats)`, `on_run_end`. The GA-specific hooks (`on_selection_complete`, `on_crossover_complete`, `on_mutation_complete`, `on_survivor_selection_complete`, `on_fitness_evaluation_complete`, `on_extension_triggered`, `on_stagnation`) do NOT fire in hill-climb or permutation engines — they have no meaning outside a GA loop.
- **D-05:** `PermutateEngine` fires the same hook set: `on_run_start`, `on_generation_start(candidate_index)`, `on_new_best` (when a new best candidate is found), `on_generation_end`, `on_run_end`. Each candidate evaluation is one "generation" for observer purposes.
- **D-06:** When `PermutateEngine` exceeds the safety gate (default 100,000 candidates), it emits a `log::warn!(target = "ga_events", ...)` and returns `Ok(())` with the best-found candidate accessible via `best()`. No observer hook for gate overflow — uses existing log pattern from `ga.rs`.

### HillClimbEngine Structure (STR-02, STR-03)

- **D-07:** Single `HillClimbEngine<U>` struct with a `mode: HillClimbMode` enum field. `HillClimbMode` has two variants: `Stochastic` and `SteepestAscent`. Both modes share the same struct, `neighbor_fn`, observer wiring, and stopping logic — the mode only changes how a neighbor is selected from the returned set.
  - `Stochastic`: accepts the first neighbor with higher fitness (early exit); stops when no improvement found within `no_improvement_limit` iterations.
  - `SteepestAscent`: evaluates all neighbors, accepts only the single best; stops when best neighbor is not better than current.
- **D-08:** `neighbor_fn` is stored as `Arc<dyn Fn(&U) -> Vec<U> + Send + Sync>`. Consistent with how `GaObserver` is stored (`Option<Arc<dyn GaObserver<U> + Send + Sync>>`). Clonable for potential future parallel variants.

### PermutateEngine Candidate Generation (STR-04)

- **D-09:** `PermutateEngine<U>` accepts candidates as `Vec<U>` at build time (no lazy iterator — user materializes the candidate list). The engine iterates lazily over the provided `Vec<U>` (one at a time via `.iter()`), evaluating each, tracking the running best, and dropping no reference beyond current + best. No full re-materialization inside the engine loop.
- **D-10:** Safety gate is a configurable `usize` field on `PermutateConfiguration` (default 100,000). If the provided `Vec<U>` length exceeds the gate, `run()` evaluates up to the gate, then `log::warn!` and returns `Ok(())` with best-found.

### Module Layout

- **D-11:** New engines land in `src/engines/hill_climb/` and `src/engines/permutate/` following the existing `src/engines/de/` pattern: `mod.rs`, `engine.rs`, `configuration.rs`. The `Strategy<U>` trait itself lives in `src/traits/strategy.rs` and is re-exported from `src/traits.rs` alongside `ChromosomeT` and `LinearChromosome`.
- **D-12:** `Ga<U>` implements `Strategy<U>` as a blanket or explicit impl — no changes to `Ga` internals.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §STR — STR-01 through STR-04 (authoritative scope for this phase)

### Prior Phase Context
- `.planning/phases/47-architecture-audit-chromosomet-split/47-CONTEXT.md` — D-01/D-02 (`LinearChromosome` supertrait bound that all new engines must use)
- `.planning/STATE.md` — v3.0.0 decisions: new engines in `src/engines/`, observer pattern (`Option<Arc<dyn GaObserver<U>>>`)

### Existing Engine to Mirror (structural pattern)
- `src/engines/de/mod.rs` — module layout pattern (`configuration.rs`, `engine.rs`, `mod.rs` with re-exports)
- `src/engines/de/engine.rs` — engine struct + `run()` + `find_best()` helper pattern; note it has NO observer wiring — `ga.rs` is the observer pattern source
- `src/engines/de/configuration.rs` — configuration struct + builder methods pattern

### Observer Wiring Pattern (copy from ga.rs, not de/)
- `src/engines/ga.rs` lines ~276-278, ~852-870 — `observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>`, `with_observer()` builder, `notify()` helper. Hill-climb and permutation engines MUST follow this exact pattern.
- `src/observe/observer/mod.rs` — `GaObserver<U>` trait definition: all 12 hooks. Hooks D-04 covers which subset fires in these engines.

### GaError (error propagation)
- `src/error.rs` — `GaError` enum variants. `run()` returns `Result<(), GaError>` — add variants if needed (e.g., `EmptyPopulation`, `ConfigurationError` already exist; check before adding).

### Trait Architecture
- `src/traits/chromosome.rs` — `ChromosomeT` (new engines bound on `U: LinearChromosome`)
- `src/traits.rs` — re-export point for new `Strategy<U>` trait

### Existing Ga<U> impl (must implement Strategy<U>)
- `src/engines/ga.rs` lines ~247-360 — `Ga<U>` struct definition and `run()` / `best()` methods (verify exact signatures before writing the trait)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/de/engine.rs` `find_best()` helper — same pattern needed in `HillClimbEngine` and `PermutateEngine`
- `src/engines/ga.rs` `notify()` helper — copy this 3-line pattern verbatim for observer dispatch in new engines
- `src/rng::make_rng()` — available if stochastic neighbor selection needs randomness (stochastic hill climb early-exit may need shuffled neighbor order)

### Established Patterns
- Observer stored as `Option<Arc<dyn GaObserver<U> + Send + Sync>>` — zero overhead when `None`; `notify()` is a single-closure call pattern. Do not deviate.
- WASM mandatory: `Instant::now()` gated with `#[cfg(not(target_arch = "wasm32"))]`; `par_iter()` never called unconditionally. Hill-climb neighbor evaluation must use `.iter()` not `.par_iter()`.
- Engine module re-exports via `mod.rs` `pub use` — follow `src/engines/de/mod.rs` exactly.
- `log::warn!(target = "ga_events", ...)` is the established warning target — use it for the permutation gate overflow message.

### Integration Points
- `src/lib.rs` — add `pub mod engines` re-exports for `HillClimbEngine`, `PermutateEngine`, `Strategy` trait, `HillClimbMode`, `HillClimbConfiguration`, `PermutateConfiguration`
- `src/traits.rs` — add `pub use strategy::Strategy` re-export
- `src/engines/ga.rs` — add `impl Strategy<U> for Ga<U>` (explicit impl block, minimal surface)

</code_context>

<specifics>
## Specific Ideas

- The two `HillClimbMode` variants differ only in neighbor selection: `Stochastic` early-exits on first improvement; `SteepestAscent` evaluates all and picks global best. The shared run loop structure means the mode branches only at the "pick next candidate" step.
- `PermutateEngine` is intentionally simple — it's a for-loop with a best-tracker and a counter. Its value is the `Strategy<U>` unification, not algorithmic sophistication.
- Observer hook naming for iterations: `on_generation_start(iteration)` and `on_generation_end(stats)` with `GenerationStats` populated with iteration count and best fitness. Downstream observer users get per-iteration stats without new hooks.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 49-unified-strategy-trait-alternative-strategy-engines*
*Context gathered: 2026-05-22*
