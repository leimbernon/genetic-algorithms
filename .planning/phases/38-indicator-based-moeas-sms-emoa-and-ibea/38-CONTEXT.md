# Phase 38: Indicator-based MOEAs — SMS-EMOA and IBEA - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning (blocked on Phase 39 completion)

<domain>
## Phase Boundary

Add two new indicator-based MOEA engines in `src/engines/`: SMS-EMOA (S-metric selection / hypervolume contribution-based steady-state removal, Beume et al. 2007) and IBEA (indicator-based evolutionary algorithm, Zitzler & Kunzli 2004, using additive epsilon indicator). Both engines use the quality-indicator library built in Phase 39 for their selection criteria.

**In scope:**
- `src/engines/sms_emoa/` — `SmsEmoaGa<U>` engine, `SmsEmoaConfiguration`, hypervolume contribution-based steady-state removal (remove individual with smallest contribution to hypervolume of the current population)
- `src/engines/ibea/` — `IbeaGa<U>` engine, `IbeaConfiguration`, additive epsilon-indicator fitness assignment (pairwise I_eps+ comparisons determine fitness) and environmental selection
- `SmsEmoaObserver<U>` and `IbeaObserver<U>` sub-traits in `src/observe/observer/mod.rs` — engine-specific generation-level hooks
- `LogObserver` gains `impl SmsEmoaObserver<U>` and `impl IbeaObserver<U>` blocks in `src/observe/observer/log.rs`
- `src/lib.rs` — `pub mod sms_emoa` and `pub mod ibea` re-exports via `#[path]`
- `GaError::InvalidSmsEmoaConfiguration` and `GaError::InvalidIbeaConfiguration` variants
- `tests/engines/sms_emoa/` and `tests/engines/ibea/` — integration tests mirroring SPEA2/MOEA/D test structure
- `examples/sms_emoa_zdt1.rs` and `examples/ibea_zdt1.rs` — runnable ZDT1 examples (2-objective, 30 variables)

**Out of scope:**
- The quality-indicator library itself (Hypervolume, GD, IGD, Spread) — that's Phase 39
- Alternative indicator functions for IBEA beyond additive epsilon (e.g., I_HD hypervolume indicator)
- Constraint handling for SMS-EMOA or IBEA
- Updating `AllObserver<U>` to include the new observer traits (same rationale as Phase 35 D-10, Phase 36 D-13, Phase 37 D-07)
- WASM-specific examples

</domain>

<decisions>
## Implementation Decisions

### Phase Ordering

- **D-01:** Phase 39 (quality indicator library) MUST be built before Phase 38. The SMS-EMOA and IBEA engines depend on hypervolume and epsilon-indicator computations exported by the Phase 39 shared library. This was the user's explicit preference over bootstrapping inline indicators or merging the phases.

### Engine Directory Layout

- **D-02:** Separate directories per engine: `src/engines/sms_emoa/` and `src/engines/ibea/`. Each directory contains its own `mod.rs` (engine struct + run loop), `configuration.rs` (builder + validate), and any engine-specific helpers. Follows the established one-engine-per-directory pattern from NSGA2, NSGA3, MOEA/D, and SPEA2. No shared `indicator_based/` parent module — the shared code is Phase 39's quality-indicator library.

### Observer Traits

- **D-03:** `SmsEmoaObserver<U>` sub-trait in `src/observe/observer/mod.rs` — SMS-EMOA-specific hooks (e.g., `on_hypervolume_contribution_assigned`, `on_steady_state_removal`). Exact hooks TBD during research and planning.
- **D-04:** `IbeaObserver<U>` sub-trait in `src/observe/observer/mod.rs` — IBEA-specific hooks (e.g., `on_indicator_fitness_assigned`, `on_environmental_selection`). Exact hooks TBD during research and planning.
- Both traits have default no-op implementations and `Send + Sync` supertraits.
- `LogObserver` implements both traits with debug-level log messages on `"sms_emoa_events"` and `"ibea_events"` targets respectively.
- `AllObserver<U>` is NOT updated.

### Established Patterns (carried forward from Phases 35-37)

- Configuration builder pattern with `validate()` returning `GaError::InvalidSmsEmoaConfiguration` / `GaError::InvalidIbeaConfiguration`
- `run()` returns `Result<ParetoFront<U>, GaError>`
- Engine stores `Option<Arc<dyn XxxObserver<U> + Send + Sync>>` — zero overhead when `None`
- WASM cfg-gating on `Instant::now()` and `par_iter()` calls
- `#[path]` re-export pattern in `src/lib.rs`
- Integration tests in `tests/engines/<name>/`
- Observer hooks are generation-level only — no sub-iteration hooks

### Example Benchmarks

- **D-05:** Both engines get a ZDT1 example: `examples/sms_emoa_zdt1.rs` and `examples/ibea_zdt1.rs`. ZDT1 is the canonical 2-objective benchmark (30 variables) used by SPEA2 and NSGA2 — users can directly compare all three engines. The examples mirror `examples/spea2_zdt1.rs` structure with engine-specific adaptations.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Engine Pattern Reference (most recent)
- `.planning/phases/37-spea2-strength-pareto-evolutionary-algorithm/37-CONTEXT.md` — Latest MOEA engine: observer pattern, configuration, run(), WASM gating, tests, examples
- `src/engines/spea2/mod.rs` — Latest reference implementation: fitness assignment, archive management, observer notification, ParetoFront extraction

### Prior Engine Contexts
- `.planning/phases/36-moea-d-decomposition-based-multi-objective-optimization/36-CONTEXT.md` — MOEA/D engine: scalarization, weight vectors, neighbourhood
- `.planning/phases/35-nsga-iii-for-many-objective-optimization/35-CONTEXT.md` — NSGA-III engine: reference points, Das-Dennis lattice, multi_objective module extraction

### Requirements
- `.planning/REQUIREMENTS.md` — MOO-04 (indicator-based MOEAs), MOO-05 (quality indicators)
- `.planning/ROADMAP.md` — Phase 38 goal: "Users can run SMS-EMOA and IBEA; both share the quality-indicator library from Phase 39"

### Shared Utilities (Phase 35 extraction)
- `src/engines/multi_objective/` — Shared: non_dominated_sort, ParetoIndividual, ParetoFront, ObjectiveFn, ObjectiveDirection
- `src/observe/observer/mod.rs` — Observer trait storage (Nsga2Observer, Nsga3Observer, MoeaDObserver, Spea2Observer)
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/engines/multi_objective/non_dominated_sort.rs` — Non-dominated sorting used for post-hoc Pareto front extraction
- `src/engines/multi_objective/pareto.rs` — ParetoIndividual, ParetoFront, dominates_with_directions()
- `src/observe/observer/log.rs` — LogObserver with existing impl blocks for 4 observer traits (pattern to follow)
- `src/nsga2/configuration.rs` — ObjectiveDirection enum (reused by all engines)

### Established Patterns
- Engine struct fields: `spea2_config`, `ga_config`, `alleles`, `initialization_fn`, `objective_fns`, `observer`
- Configuration struct: 5 fields (num_objectives, population_size, max_generations, objective_directions, engine-specific)
- Builder methods: `with_num_objectives()`, `with_population_size()`, etc.
- `validate()` enforces: population > 0, num_objectives > 0, directions length matches, engine-specific constraints
- ZDT1 example: 100 pop, 250 gens, 0.9 crossover, 0.1 mutation, 2-objective, 30 vars

### Integration Points
- `src/lib.rs` — Add `pub mod sms_emoa` and `pub mod ibea` with `#[path]` attributes
- `src/observe/observer/mod.rs` — Add two new sub-traits
- `src/observe/observer/log.rs` — Add two new impl blocks
- `src/error.rs` — Add two new error variants
- `tests/test_engines.rs` — Add test modules
</code_context>

<specifics>
## Specific Ideas

No specific requirements — engines follow established patterns. The algorithms are well-defined in the literature:
- SMS-EMOA: Beume et al. 2007 — steady-state (mu+1), remove individual with smallest hypervolume contribution
- IBEA: Zitzler & Kunzli 2004 — pairwise I_eps+ indicator for fitness, binary indicator-based environmental selection
</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---
*Phase: 38-indicator-based-moeas-sms-emoa-and-ibea*
*Context gathered: 2026-05-10*
