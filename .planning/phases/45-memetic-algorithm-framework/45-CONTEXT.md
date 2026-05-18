# Phase 45: Memetic Algorithm Framework - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can attach a `LocalSearchOperator` to the GA loop with configurable application strategies (AllOffspring, BestN, Probabilistic, EveryNGenerations) and Lamarckian/Baldwinian modes, with parallel execution via rayon.

**In scope:**
- New `LocalSearchOperator` trait with enum variants + factory dispatch (consistent with all existing operator patterns)
- Application strategies: AllOffspring, BestN, Probabilistic, EveryNGenerations
- Lamarckian mode (DNA + fitness updated) and Baldwinian mode (fitness only, DNA preserved)
- Parallel local search execution via rayon `par_iter()` over selected individuals
- Integration in Ga engine only (consistent with Phases 40-43 framework extension pattern)
- Local search executes after crossover+mutation+fitness and after repair+constraint penalty, before survivor selection
- Local search NOT applied to extension regrown individuals
- Local search operator receives fitness_fn as a trait method parameter (Arc::clone shared across rayon tasks)
- WASM compatibility (falls back to sequential iter())

**Out of scope:**
- Local search for non-Ga engines (Nsga2Ga, De, Scatter, Cellular, Alps) — deferred
- New GaObserver hooks specific to memetic events — standard observer hooks fire around the generation
- Per-gene or per-individual local search tracking in observability — too granular
- Built-in complex local search strategies (simulated annealing, tabu search, gradient descent) — Phase 45 ships the framework + HillClimbing; advanced strategies deferred
</domain>

<decisions>
## Implementation Decisions

### Trait Design
- **D-01:** LocalSearchOperator uses the full trait + enum + factory pattern, consistent with `CrossoverOperator`, `MutationOperator`, `SelectionOperator`, `SurvivorOperator`, and `ExtensionOperator`
- **D-02:** The operator trait method receives `&mut U + &dyn Fn(&[U::Gene]) -> f64` as parameters, enabling the local search to re-evaluate fitness during refinement
- **D-03:** Fitness function is shared across rayon tasks via `Arc::clone()` at the call site — the operator does NOT store a fitness function reference

### GA Loop Placement
- **D-04:** Local search executes AFTER crossover+mutation+fitness, AFTER repair operator and constraint penalty, BEFORE survivor selection. The application strategy selects which offspring to refine, then local search improves them, then the improved individuals compete in survivor selection.
- **D-05:** Local search is NOT applied to individuals created by extension regrowth (extension is a diversity-rescue mechanism; applying local search to random regrowth is wasteful).

### Lamarckian vs Baldwinian
- **D-06:** Both modes are supported via a config flag on `LocalSearchConfiguration` (e.g., `.with_local_search_mode(LocalSearchMode::Lamarckian)`). Default is Lamarckian (more common in literature).
- **D-07:** Lamarckian = updates both DNA and fitness after local search. Baldwinian = updates only fitness, original DNA preserved.

### Parallel Execution
- **D-08:** Parallelism via rayon `par_iter()` over selected individuals. Each parallel task receives `Arc::clone(&fitness_fn)`.
- **D-09:** WASM: `#[cfg(target_arch = "wasm32")]` fallback to sequential `iter()` (local search is pure computation, no thread requirement).

### Engine Scope
- **D-10:** Ga engine only (consistent with Phases 40-43 framework extension pattern — constraints, HOF, warm starting, AOS all shipped Ga-only first).

### Interaction Ordering
- **D-11:** The GA generation cycle pipeline becomes: Selection → Crossover+Mutation+Fitness → Repair → Constraints → **Local Search** → Population Merge → HOF → Elitism → Survivor → Elite Reinsertion → Niching → Best Update → Stats → Extension → Checkpoint → Observer → Stop Check

### Claude's Discretion
- LocalSearch enum variants (HillClimbing as the first variant; others reserved for future)
- Application strategy implementation details (how BestN selects top-N, Probabilistic probability defaults, EveryNGenerations interval defaults)
- `LocalSearchOperator` trait method signature details (whether to pass strategy params per-call or store in struct)
- `LocalSearchConfiguration` struct fields and builder methods (`with_local_search_strategy()`, `with_local_search_mode()`, etc.)
- HillClimbing specific config: step_size, max_iterations (draw from ScatterEngine's existing local_search_step_size / local_search_steps defaults)
- Factory function location: `src/operations/local_search.rs`
- Serde derives on `LocalSearchConfiguration` and operator state structs (follow established `#[cfg_attr(feature = "serde", derive(...))]` pattern)
- Whether to support user-supplied custom local search strategies via closures (possible extension, not required for phase 45)
- Ga struct field: `local_search: Option<Box<dyn LocalSearchOperator<U>>>` (Option+Box for zero overhead when None, consistent with other optional operators)
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### GA Engine — Primary Integration Target
- `src/engines/ga.rs` — Ga struct, builder methods, generation loop. Local search inserts after crossover+mutation+fitness block (around line 1392, before population merge at line 1395).
- `src/engines/ga.rs` §1089-1699 — Full `run_with_callback()` generation loop (D-04, D-11)
- `src/engines/ga.rs` §1344-1392 — Repair operator + constraint penalty block (local search inserts AFTER this, D-11)

### Existing Operator Pattern to Follow
- `src/traits/operators.rs` — All 5 existing operator trait definitions. LocalSearchOperator follows the same pattern (D-01).
- `src/operations/crossover.rs` — Enum + factory dispatch pattern (reference for LocalSearch enum + factory)
- `src/operations/mutation.rs` — `factory_with_params()` pattern if LocalSearch needs config params at factory time

### Scatter Engine local_search (reference implementation)
- `src/engines/scatter/engine.rs` §238-258 — `local_search_improve()` hill-climbing implementation. Reference for HillClimbing strategy.
- `src/engines/scatter/configuration.rs` — Existing `local_search: bool`, `local_search_steps`, `local_search_step_size` config fields. Reference defaults: steps=20, step_size=0.1.

### Existing Patterns
- `src/engines/ga.rs` — `Option<...>` zero-overhead pattern for optional features (D-10 Ga-only)
- `src/engines/ga.rs` — Builder methods return `Self` for chaining
- `src/configuration.rs` — GaConfiguration fields for optional features
- `src/traits/configuration.rs` — Builder trait methods

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Must apply cfg-gating rules. Local search is pure computation — no Instant/rayon constraints beyond the standard par_iter gate.

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 45 — Goal: "Users can attach a LocalSearchOperator to the GA loop with configurable application strategies (AllOffspring, BestN, Probabilistic, EveryNGenerations) and Lamarckian/Baldwinian modes, with parallel execution via rayon"
- Issue #215 — Memetic Algorithm Framework
- `.planning/REQUIREMENTS.md` §MEM-01 — Single requirement: memetic algorithm with local search operator trait

### Prior Phase Patterns (Ga-only Framework Extensions)
- `.planning/phases/41-hall-of-fame-solution-archive/41-CONTEXT.md` — D-13: Ga-only pattern
- `.planning/phases/42-warm-starting-population-seeding/42-CONTEXT.md` — D-09: Ga-only pattern
- `.planning/phases/43-adaptive-operator-selection-aos/43-CONTEXT.md` — D-15: Ga-only pattern
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **ScatterEngine `local_search_improve()`** (`src/engines/scatter/engine.rs:238-258`) — Hill-climbing implementation with random perturbation, accept/revert logic. Reference for the HillClimbing variant built-in strategy.
- **Repair operator pattern** (`src/engines/ga.rs:1344-1350`) — Recent `Option<Box<dyn Fn(...)>>` pattern for operator-like feature. LocalSearch follows similar struct field + integration pattern.
- **AOS integration** (`src/engines/ga.rs:1199-1213`) — Most recent complex feature integration in Ga engine. Reference for struct field, builder method, init block, and loop integration pattern.

### Established Patterns
- Operator trait + enum variant + factory dispatch
- `Option<Box<dyn OperatorTrait<U>>>` zero-overhead pattern for optional features
- Builder methods returning `Self` for chaining
- `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` for optional serialization
- Config struct fields with builder pattern in `src/configuration.rs` / `src/traits/configuration.rs`

### Integration Points
- `src/engines/ga.rs` — Add `local_search: Option<Box<dyn LocalSearchOperator<U>>>` field on Ga struct
- `src/engines/ga.rs` — Add `.with_local_search()` builder method
- `src/engines/ga.rs` — Insert local search block between constraint penalty block (~line 1392) and population merge (~line 1395)
- `src/configuration.rs` — Add `LocalSearchConfiguration` (strategy, mode, hill-climbing params)
- `src/traits/configuration.rs` — Add local search builder methods to `ConfigurationT` trait
- `src/traits/operators.rs` — Add `LocalSearchOperator` trait
- `src/operations/local_search.rs` (new) — LocalSearch enum + factory + HillClimbing implementation
- `src/operations/mod.rs` — Add `pub mod local_search` and `pub use` re-exports
- `src/lib.rs` — Re-export new public types

### Creative Options
- The HillClimbing implementation can be extracted from ScatterEngine and made generic via the new trait
- Application strategies can be applied using a simple enum with dispatch, or as separate strategy structs
- BestN could reuse the elite extraction logic (sort fitness values, select top N)
</code_context>

<specifics>
No specific references beyond standard memetic algorithm patterns. Standard approaches are preferred.

Key behaviors derived from discussion:
- Local search refines individuals AFTER repair/constraints fix them — operates on valid individuals
- Extension regrown individuals do NOT get local search — wasteful on random initializations
- HillClimbing is the first built-in strategy (draw from ScatterEngine defaults: step_size=0.1, steps=20)
- Application strategies: AllOffspring (every offspring refined), BestN (top N by fitness), Probabilistic (each with configurable probability p), EveryNGenerations (apply every Nth generation)
- Lamarckian default: `individual.set_dna(modified_dna)` + `individual.set_fitness(new_fitness)`
- Baldwinian alternative: `individual.set_fitness(new_fitness)` only, original DNA preserved
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 45-Memetic Algorithm Framework*
*Context gathered: 2026-05-14*
