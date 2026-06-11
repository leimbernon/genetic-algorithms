# Phase 43: Adaptive Operator Selection (AOS) - Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can configure portfolios of crossover and mutation operators, with Probability Matching, Adaptive Pursuit, or Multi-Armed Bandit selection dynamically choosing the best operator per offspring couple based on recent fitness-improvement credit. AOS integrates into the standard Ga engine's generation loop, replacing the single-crossover/single-mutation dispatch with portfolio-aware selection.

**In scope:**
- AosState struct with three strategy modes: ProbabilityMatching, AdaptivePursuit, MultiArmedBandit
- Separate crossover_portfolio and mutation_portfolio, each with independent AOS state
- Builder API: `.with_crossover_portfolio(Vec<Crossover>)`, `.with_mutation_portfolio(Vec<Mutation>)`, `.with_aos_strategy(AosStrategy)`
- Reward model: offspring-vs-parent normalized improvement, sliding window (W=50 default, configurable)
- AOS replaces single-operator dispatch inside offspring generation loop (per couple)
- Initial exploration phase: first W/2 generations use uniform operator selection to build statistics
- Thread-safe reward accumulator (Mutex), batch-updated after parallel offspring loop
- AOS and existing Adaptive GA coexist: AOS for WHICH operator, Adaptive for rates (probability)
- Validation: warn if both `.with_crossover()` and `.with_crossover_portfolio()` are set
- Ga engine only (consistent with Phases 40-42 pattern)
- WASM compatibility (reward accumulation is data-arithmetic, no Instant/rayon concerns)

**Out of scope:**
- AOS for non-Ga engines (Nsga2Ga, De, Scatter, Cellular, Alps) — deferred to future phase
- Dynamically adjustable AOS strategy parameters mid-run — configure-time only
- Combined flat-list portfolios or paired (crossover+mutation) strategy arms — portfolios are always separate per-operator-type
- Automatic AOS hyperparameter tuning (no meta-optimization of strategy parameters)
- New GaObserver hooks specific to AOS — no new observer events
</domain>

<decisions>
## Implementation Decisions

### Credit Reward Model
- **D-01:** Credit assigned via offspring-vs-parent fitness comparison. Positive delta (offspring better than parent) = reward for the operator that produced it.
- **D-02:** Reward = normalized improvement: `(parent_fitness - offspring_fitness) / best_fitness`. Scale-invariant across problem domains.
- **D-03:** Sliding window reward history (W=50 default). Prevents stale rewards from dominating the credit statistic.
- **D-04:** Window size configurable via `.with_reward_window(usize)` builder method. Default W=50.

### Portfolio Structure
- **D-05:** Separate crossover_portfolio and mutation_portfolio, each with independent AOS state machine. Two independent AOS instances (one for crossover operators, one for mutation operators).
- **D-06:** API: `Vec<Crossover>` and `Vec<Mutation>` (reusing existing enums, no new portfolio wrapper types). Builder: `.with_crossover_portfolio(vec![Crossover::SBX, Crossover::Uniform, ...])`.
- **D-07:** Minimum 2 operators per portfolio. Warn at build time if 1 operator (effectively falls back to single-operator mode for that type).

### GA Loop Integration
- **D-08:** Per-offspring-couple selection. Before generating each offspring, AOS consults the strategy and selects the best currently-estimated operator. Standard AOS literature approach (DaCosta 2008, Fialho 2010).
- **D-09:** AOS replaces the operator dispatch inside the existing offspring generation loop. For each parent pair, AOS selects which crossover operator to apply, then for each child selects which mutation operator to apply.
- **D-10:** Portfolio replaces single-operator setting when configured. Validation warns if both `.with_crossover()` and `.with_crossover_portfolio()` are set. The standard `.with_crossover()` / `.with_mutation()` fields are ignored when their respective portfolios are configured.
- **D-11:** Credit rewards collected via thread-safe accumulator (Mutex<Vec<(operator_idx, reward)>>) during the parallel offspring loop. After the parallel block completes, AOS state is batch-updated with accumulated rewards.
- **D-12:** Initial exploration phase: first W/2 generations, all operators in the portfolio are used with equal probability. After the exploration phase, the configured AOS strategy takes over.

### Interaction with Existing Adaptive GA
- **D-13:** AOS and Adaptive GA coexist. AOS handles WHICH operator to use; Adaptive GA handles the PROBABILITY (rate) of applying operators. The rate check still gates whether crossover/mutation occurs; AOS selects which operator is used when it does.
- **D-14:** AOS strategy parameters (MAB's C, PM's alpha/learning rate, AP's beta) are set at configure time only — no mid-run dynamic adjustment.
- **D-15:** Ga engine only for this phase. Consistent with Phases 40-42 framework extension pattern (constraints, HOF, warm starting all shipped Ga-only).

### Claude's Discretion
- AosState struct design: separate AosCrossoverState and AosMutationState, or a single AosState<Op> generic. Plausible design: `AosStrategyState` enum with per-strategy data (ProbabilityMatchingState, AdaptivePursuitState, MultiArmedBanditState). Ga stores `aos: Option<AosController<U>>`.
- Strategy parameter defaults (PM: alpha=0.8, learning_rate=0.3; AP: beta=0.5, C=1.5; MAB: C=1.0, epsilon=0.1 — literature-standard). Planner should confirm defaults and document them.
- Reward normalization reference: store `best_fitness` per-generation for normalization denominator. Clamp at epsilon to avoid div-by-zero.
- Exploration phase: uniform random selection with equal probability. No strategy-state updates during exploration; first W/2 generations purely collect data.
- AOS controller placement: Ga struct stores `aos_crossover: Option<AosStrategy>`, `aos_mutation: Option<AosStrategy>` — zero overhead when None.
- Serialization: `#[cfg_attr(feature = "serde", derive(...))]` on AosStrategy and associated state structs. Sliding window Vec<f64> and probability Vec<f64> both serializable.
- GaObserver hooks: not required for this phase. Standard Observer hooks fire for each generation and can observe AOS-related stats through existing observer callbacks if the planner decides to expose them.
- `Crossover` and `Mutation` enums already implement Clone — the AOS portfolio stores cloned references (cloning enums is cheap).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### GA Engine — Primary Integration Target
- `src/engines/ga.rs` — Ga struct, builder methods, `ConfigurationT` implementation, generation loop (offspring generation, adaptive GA block, statistics)
- `src/engines/ga.rs` §1128 — Existing Adaptive GA initialization block (reference for AOS coexistence — D-13)
- `src/engines/ga.rs` §1353 — Existing Adaptive GA per-generation update (AOS updates near here)
- `src/configuration.rs` — GaConfiguration: existing `adaptive_ga` field pattern (D-13 coexistence)

### Operator Enums (Portfolio Types)
- `src/operations/mod.rs` — Crossover, Mutation enums (reused directly for portfolio vectors — D-06)
- `src/operations/crossover/mod.rs` — Crossover factory: `crossover::factory()` dispatch
- `src/operations/mutation/mod.rs` — Mutation factory: `mutation::factory_with_params()` dispatch

### Existing Patterns to Follow
- `src/engines/ga.rs` — Observer wiring: `Option<Arc<dyn GaObserver<U>>>` with builder method. AOS controller follows same Option pattern (Ga-only consistency — D-15)
- `src/engines/ga.rs` §699 — `with_hall_of_fame()` builder method (Phase 41) — most recent analogous optional-feature pattern
- `src/engines/ga.rs` — Builder methods return `Self` for chaining
- `.planning/phases/41-hall-of-fame-solution-archive/41-CONTEXT.md` — D-13: Ga-only pattern for framework extensions

### Existing Adaptive GA Reference
- `src/engines/ga.rs` — `adaptive_ga: bool` field, adaptive probability computation in generation loop
- `src/configuration.rs` — Configuration fields: `adaptive_crossover_rate`, `adaptive_mutation_rate`, `update_interval`

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Must apply cfg-gating rules (AOS reward accumulation is data-arithmetic, no Instant/rayon. Mutex is fine on wasm via std::sync::Mutex.)

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 43 — Goal: "Users can configure portfolios of crossover and mutation operators, with Probability Matching, Adaptive Pursuit, or Multi-Armed Bandit selection dynamically choosing the best operator based on recent fitness-improvement credit"
- Issue #218 — Adaptive Operator Selection (AOS)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Crossover/Mutation enums** (`src/operations/mod.rs`) — Already Clone, can be stored directly in portfolio Vecs
- **Existing Adaptive GA state** (`src/engines/ga.rs` §1128) — Per-generation update pattern to follow for AOS statistics update
- **GaObserver pattern** — `Option<Arc<dyn ...>>` with builder method — AOS controller follows same optional-field pattern
- **`#[cfg_attr(feature = "serde", derive(...))]`** — Established pattern for optional serialization of state structs

### Established Patterns
- Ga stores optional features as `Option<...>` with builder methods — zero overhead when None
- Builder methods return `Self` for chaining
- ``

### Integration Points
- `src/engines/ga.rs` — Add `aos_crossover: Option<AosStrategy>` and `aos_mutation: Option<AosStrategy>` fields on Ga struct
- `src/engines/ga.rs` — Add `.with_crossover_portfolio()`, `.with_mutation_portfolio()`, `.with_aos_strategy()` builder methods
- `src/engines/ga.rs` — Modify offspring generation loop to dispatch through AOS selection instead of single-operator call when AOS is configured
- `src/engines/ga.rs` — Add AOS statistics update block after parallel offspring+fitness evaluation
- `src/engines/ga.rs` — Add AOS reward accumulation in or after the offspring-generation parallel block
- `src/configuration.rs` — Add AOS configuration fields (portfolios, strategy config, reward window) to GaConfiguration
- `src/traits/configuration.rs` — Add AOS builder methods to ConfigurationT trait
- `src/validators/mod.rs` — Add AOS validation (minimum portfolio size, crossover/mutex-exclusivity warnings)

</code_context>

<specifics>
## Specific Ideas

- Strategy parameter defaults (literature standard): PM alpha=0.8, learning_rate=0.3; AP beta=0.5, C=1.5; MAB C=1.0, epsilon=0.1
- Reward accumulator: `Mutex<Vec<(usize, f64)>>` (operator_index, reward). Batch update: for each reward, update the sliding window for the operator's slot, recompute strategy statistics.
- Exploration phase uniform selection: `rng.gen_range(0..portfolio.len())`.
- AOS strategy should be generic over `Crossover`/`Mutation` enum or have separate state per portfolio type.
- WASM: `Mutex` from std works on was32-unknown-unknown. No `Instant` or `rayon` in AOS state logic.

</specifics>

<deferred>
## Deferred Ideas

- AOS support in non-Ga engines (Nsga2Ga, De, Scatter, Cellular, Alps) — future phase
- Dynamically adjustable AOS parameters mid-run — no immediate demand, easy additive change
- Combined flat-list portfolios (mixing crossover and mutation in one portfolio) — would need a unified AosOperator enum
- GaObserver hooks for AOS events (operator selected per couple, reward recorded, strategy switch) — no immediate demand

</deferred>

---

*Phase: 43-Adaptive Operator Selection (AOS)*
*Context gathered: 2026-05-13*
