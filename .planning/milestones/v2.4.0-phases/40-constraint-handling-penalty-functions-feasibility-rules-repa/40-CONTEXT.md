# Phase 40: Constraint Handling — Penalty Functions, Feasibility Rules, RepairOperator - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can solve constrained optimization problems by configuring penalty strategies (static, dynamic, adaptive), Deb's feasibility rules for selection/survivor/elitism comparisons, and a RepairOperator trait for repairing infeasible chromosomes after mutation. Foundation code exists as untracked files (`src/constraints.rs`, `tests/test_constraints.rs`) — the phase wires these into the GA configuration, engine loop, and runtime.

**In scope:**
- `PenaltyStrategy` enum (`None`, `Static`, `Dynamic`, `Adaptive`) — already in `src/constraints.rs`
- `ConstraintHandling::FeasibilityRules` — already in `src/constraints.rs`
- `RepairOperator` trait — already committed in `src/traits/operators.rs`
- GA configuration builder methods: `.with_constraint_fns()`, `.with_penalty_strategy()`, `.with_constraint_handling()`, `.with_repair_operator()`
- Penalty strategy validation, constraint validation integration in `Ga::build()`
- Feasibility rules applied in selection, survivor, elitism, and best-chromosome tracking
- Repair operator dispatch after mutation, applied per-offspring before fitness evaluation
- Integration tests for each constraint feature
- WASM compatibility (penalty and feasibility are pure math with no time/thread requirements)

**Out of scope:**
- Multi-objective engine constraint handling (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA) — deferred to future phase
- New GaObserver hooks specific to constraint handling — no new observer events
- Automatic penalty application by Ga — user applies penalty in their fitness function
- Constraint handling for island model engines
- DTLZ/ZDT constrained benchmarks or literature-specific constraint test problems
</domain>

<decisions>
## Implementation Decisions

### Penalty Application Model
- **D-01:** Manual. Ga provides the `PenaltyStrategy` enum, validation helpers, and standalone functions (`apply_static_penalty`, `apply_dynamic_penalty`, `total_violation`). The user calls these in their fitness closure. Ga does NOT automatically apply penalties or modify fitness values.
- **D-02:** Ga stores `constraint_fns: Vec<Arc<dyn Fn(&[U::Gene]) -> f64 + Send + Sync>>` — a vector of per-constraint violation functions. Not used by Ga's run loop (manual mode), but available for observability and future automatic modes.
- **D-03:** Ga stores `penalty_strategy: PenaltyStrategy` (default `None`), stored for validation and reference. Not applied automatically.

### Feasibility Rules Scope
- **D-04:** Deb's feasibility rules apply to ALL comparison contexts: selection (tournament), survivor operators, elitism, and best-chromosome tracking. Implemented as comparison helpers in `src/constraints.rs` that any comparison site can call.
- **D-05:** Feasibility rules are enabled by setting `ConstraintHandling::FeasibilityRules` on the builder. When not set, comparisons behave as today (fitness-only). When set, every comparison first checks feasible vs infeasible, then uses Deb's three rules.
- **D-06:** Comparison helpers signature: `fn compare(fitness_a: f64, violation_a: f64, fitness_b: f64, violation_b: f64, problem_type: ProblemSolving) -> Ordering`. Enables uniform application across all comparison sites.

### Multi-Objective Scope
- **D-07:** Single-objective Ga only. NSGA-II and other MOEA engines do NOT get constraint handling in this phase. Deferred as a separate future feature.

### Constraint Function API
- **D-08:** Users provide a `Vec<fn(&[Gene]) -> f64>` where each function returns a violation >= 0.0 (0.0 = satisfied). Builder: `.with_constraint_fns(vec![...])`.
- **D-09:** `total_violation(&[f64]) -> f64` helper sums the per-constraint violation vector. The constraint fns are stored per-engine; results of evaluating them per chromosome are available as a `Vec<f64>` before `total_violation` is applied.

### RepairOperator Integration
- **D-10:** `Ga` stores `Option<Arc<dyn RepairOperator>>`. Builder: `.with_repair_operator(Arc::new(MyRepair))`.
- **D-11:** Repair is dispatched in the GA loop after mutation, per-offspring, before fitness evaluation. The repair function receives `&mut U` and modifies in-place.
- **D-12:** `RepairOperator` trait already committed in `src/traits/operators.rs` (lines 212-238) with `fn repair<U: ChromosomeT>(&self, chromosome: &mut U) -> Result<(), GaError>`.

### Claude's Discretion
- Adaptive penalty state management: planner chooses where to store per-generation state (separate struct, inside Ga, or configuration-extracted state holder).
- Adaptive penalty algorithm details: generation-window tracking, coefficient update frequency, and initial/boundary conditions for coefficient adjustment.
- Integration specifics for feasibility rules in each operator (selection dispatch, survivor dispatch, elitism gate, best-update gate) — planner follows the `compare()` helper pattern above.
- Validation: planner designs `Ga::build()` constraint validation flow (validate penalty strategy, validate constraint functions count > 0 when feasibility rules enabled, validate repair operator type matching).
- Exact mutation-site placements for repair operator in the offspring loop.
- Constraint strategy persistence: `#[cfg_attr(feature = "serde", ...)]` for constraint fields.
- No `GaObserver` hooks for constraint events — planner discretion if useful.

### Existing Foundation (pre-existing code)
- `src/constraints.rs` — untracked: `PenaltyStrategy` enum (None, Static, Dynamic, Adaptive), `ConstraintHandling::FeasibilityRules`, helpers (`total_violation`, `apply_static_penalty`, `apply_dynamic_penalty`, `validate_penalty_strategy`)
- `src/traits/operators.rs` — committed: `RepairOperator` trait (Send + Sync, `fn repair(&self, &mut U) -> Result<(), GaError>`)
- `src/error.rs` — committed: `GaError::InvalidConstraintConfiguration(String)` variant
- `src/lib.rs` — committed: `pub mod constraints;` + re-exports of `ConstraintHandling;` and `PenaltyStrategy;`
- `tests/test_constraints.rs` — untracked: unit tests for penalty functions and GA integration tests using builder methods

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Constraint Foundation
- `src/constraints.rs` — Existing untracked code: PenaltyStrategy, ConstraintHandling, apply_static_penalty, apply_dynamic_penalty, total_violation, validate_penalty_strategy
- `src/traits/operators.rs` (lines 212-238) — RepairOperator trait definition
- `src/error.rs` (line 49) — GaError::InvalidConstraintConfiguration variant
- `tests/test_constraints.rs` — Integration tests using builder methods and repair operator

### GA Engine and Configuration Patterns
- `src/ga.rs` — Ga engine: run loop, fitness evaluation, offspring generation — constraint integration targets
- `src/configuration.rs` — GaConfiguration: constraint fields (constraint_fns, penalty_strategy, constraint_handling, repair_operator) — builder methods, validate
- `src/traits/configuration.rs` — ConfigurationT trait: add constraint builder methods
- `src/validators/mod.rs` — Ga build validation: extend with constraint validation

### Existing Operator Patterns
- `src/operations/selection/tournament.rs` — Tournament selection: integrate feasibility rules
- `src/operations/survivor/mod.rs` — Survivor operator factory: pass constraint context
- `src/ga.rs` (elite block) — Elitism logic: apply feasibility rules for elite selection

### WASM Compatibility
- `CLAUDE.md` §WASM Compatibility — Must apply cfg-gating rules (penalty/feasibility are pure math, no Instant/rayon needed)

### Requirements and Roadmap
- `.planning/ROADMAP.md` §Phase 40 — Goal: "Users can solve constrained optimization problems by configuring penalty functions (static, dynamic, adaptive), Deb's feasibility rules for selection/survivor comparison, and a RepairOperator trait for fixing infeasible chromosomes after mutation"
- Issues #212 (penalty functions), #213 (Deb's feasibility rules), #214 (RepairOperator)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`src/constraints.rs`** — Already has PenaltyStrategy enum, ConstraintHandling enum, helper functions, validation. Untracked but ready for commit.
- **`src/traits/operators.rs::RepairOperator`** — Trait committed and ready. Single method: `fn repair(&self, chromosome: &mut U) -> Result<(), GaError>`.
- **`GaError::InvalidConstraintConfiguration(String)`** — Error variant committed.
- **`total_violation(&[f64]) -> f64`** — Helper sums per-constraint violations. Already in constraints.rs.

### Established Patterns
- Configuration builder: fluent builder pattern in `src/traits/configuration.rs` with methods returning `Self`. Constraint methods follow existing patterns (e.g., `with_observer()`, `with_selection_method()`).
- Operator dispatch: `RepairOperator` follows the same trait pattern as other operators but is a single-instance field (like fitness_fn), not an enum + factory.
- Ga stores `Option<Arc<dyn Trait>>` for optional features — zero overhead when `None` (same pattern as GaObserver, Reporter).
- `#[cfg_attr(feature = "serde", derive(...))]` for configuration structs.

### Integration Points
- `src/configuration.rs` — Add fields: `constraint_fns`, `penalty_strategy`, `constraint_handling`, `repair_operator`
- `src/traits/configuration.rs` — Add builder methods + trait bounds
- `src/validators/mod.rs` — Add constraint validation in build()
- `src/ga.rs` — Add fields on Ga struct, implement repair dispatch after mutation, feasibility comparison in all comparison sites
- `src/operations/selection/` — Feasibility-aware comparison in each selection strategy
- `src/operations/survivor/` — Feasibility-aware survivor selection
- `src/ga.rs` elite block and best-chromosome update — Feasibility-aware comparison

</code_context>

<specifics>
## Specific Ideas

- Feasibility comparison helper signature: `fn compare(fitness_a: f64, violation_a: f64, fitness_b: f64, violation_b: f64, problem_type: ProblemSolving) -> Ordering`
- Repair operator applied in offspring loop after mutation, before fitness evaluation: matching iterator pairing each mutated offspring with repair call
- Validation: if constraint_fns is non-empty but penalty_strategy is `None`, warn but don't error (user may apply penalty manually in fitness function)
- Validation: if ConstraintHandling::FeasibilityRules is set but constraint_fns is empty, return `InvalidConstraintConfiguration("Feasibility rules require at least one constraint function")`

</specifics>

<deferred>
## Deferred Ideas

- Multi-objective constraint handling (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA) — separate future phase. NSGA-II constraint-dominance is well-defined (Deb 2000) and could follow the same feasibility rules pattern.
- Automatic penalty application by Ga — if users want Ga to auto-apply penalty in the future, this can be added as a mode without breaking manual mode.
- GaObserver hooks for constraint events (violation totals, penalty applied, repair performed) — no immediate demand but easy additive change later.
- Constraint handling in island model engines — cross-layer constraint handling has no clear use case yet.

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 40-constraint-handling-penalty-functions-feasibility-rules-repa*
*Context gathered: 2026-05-11*
