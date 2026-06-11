# Phase 55: RFC Multi-Valued Fitness - Context

**Gathered:** 2026-05-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 55 resolves the design RFC for multi-valued fitness in the library. It:

1. **Renames `MultiCaseFitness`** → `VectorFitness` with renamed methods `case_fitness()` → `fitness_values()` and `set_case_fitness()` → `set_fitness_values()`. Adds a default impl for `fitness_values()` that wraps scalar `fitness()` for backward-compatible single-objective use.

2. **Unifies the vector-fitness API** across two prior separate concerns:
   - **Lexicase selection** (Phase 50): previously used `MultiCaseFitness::case_fitness()` — now reads `VectorFitness::fitness_values()`
   - **MO engines** (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA): previously used external `objective_fns: Vec<Arc<Fn>>` closures stored on the engine — now require the chromosome to implement `VectorFitness` and store objective values inside `calculate_fitness()` via `set_fitness_values()`

3. **Removes the external closure machinery** from all MO engine configurations. `objective_fns` fields are removed entirely (v3.0.0 breaking change). Users migrate by moving objective evaluation into their `calculate_fitness()` implementation.

4. Documents the migration in `MIGRATION.md` (Phase 65).

</domain>

<decisions>
## Implementation Decisions

### Trait Topology

- **D-01:** `MultiCaseFitness` is renamed to `VectorFitness`. It remains a supertrait of `ChromosomeT` (opt-in — users implement it explicitly). It is NOT auto-implemented for all `ChromosomeT` types via a blanket impl.

- **D-02:** `VectorFitness` has **no default implementation** for `fitness_values()`. Rust's type system prevents a `&[f64]` return from the value-returning `ChromosomeT::fitness() -> f64` (temporary does not live long enough). Every implementor must provide explicit `fitness_values()` and `set_fitness_values()` methods backed by a stored `fitness_values: Vec<f64>` field. This is consistent with how `MultiCaseChromosome` in tests already works. *(Amended from original "default impl" decision — resolved 2026-05-30 via user confirmation of Option A.)*

- **D-03:** Method rename: `case_fitness() -> &[f64]` → `fitness_values() -> &[f64]`; `set_case_fitness(Vec<f64>)` → `set_fitness_values(Vec<f64>)`. Both the trait definition and all call sites in the library update.

### Semantic Separation

- **D-04:** One trait, two use cases. `VectorFitness` is used for both lexicase (test case scores) and multi-objective (Pareto objective values). The semantic difference (lexicase shuffles indices; MO engines apply fixed objective directions per index) is handled by engine behavior and documented clearly — not separated into two different traits.

- **D-05:** The renamed `VectorFitness` trait is re-exported from `src/lib.rs` at `genetic_algorithms::VectorFitness` (replacing the old `MultiCaseFitness` re-export). No aliases.

### MO Engine Integration

- **D-06:** All MO engines (NSGA-II, NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA) add a `VectorFitness` bound to their chromosome type parameter `U`. The engine reads `chromosome.fitness_values()` to get objective values — it no longer evaluates external closures.

- **D-07:** `objective_fns: Vec<Arc<dyn Fn(&[Gene]) -> f64>>` is **removed** from all MO engine configurations. This is a v3.0.0 breaking change. Users who previously passed objective closures must now implement `VectorFitness` on their chromosome and populate `fitness_values` inside `calculate_fitness()`.

- **D-08:** `ParetoIndividual<U>` retains its `objectives: Vec<f64>` sidecar field internally — but it is now populated by calling `chromosome.fitness_values().to_vec()` during population initialization and re-evaluation, not by calling external closures. The `ParetoIndividual` wrapper is an internal detail unchanged from the user's perspective.

### Migration Strategy

- **D-09:** **Hard rename** in v3.0.0. No type alias bridge, no deprecation period. `MultiCaseFitness` disappears; `VectorFitness` is the replacement. All call sites in the library and in examples update atomically. MIGRATION.md (Phase 65) documents the before/after pattern.

- **D-10:** Existing `factory_lexicase<U: ChromosomeT + MultiCaseFitness>()` bound updates to `factory_lexicase<U: ChromosomeT + VectorFitness>()`. Behavior is unchanged.

- **D-11:** Phase 50 `select_parents_lexicase()` on `Ga<U>` updates its `U: MultiCaseFitness` bound to `U: VectorFitness`. No behavioral change.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Trait Definitions
- `src/traits/multi_case_fitness.rs` — Current `MultiCaseFitness` definition being renamed to `VectorFitness`
- `src/traits/chromosome.rs` — `ChromosomeT` base trait; `VectorFitness` is a supertrait of this
- `src/traits/linear_chromosome.rs` — `LinearChromosome` supertrait; MO engines bound on this + `VectorFitness`
- `src/lib.rs` — Re-export location; `MultiCaseFitness` export must become `VectorFitness`

### MO Engine Integration Points
- `src/engines/nsga2/mod.rs` — NSGA-II engine; `objective_fns` removal, `VectorFitness` bound add, `ParetoIndividual` population from `fitness_values()`
- `src/engines/nsga2/pareto.rs` — `ParetoIndividual` struct; `objectives` population changes
- `src/engines/nsga2/configuration.rs` — `objective_fns` field removal

### Lexicase Callers (update bounds only)
- `src/operations/selection.rs` — `factory_lexicase` bound update
- `src/operations/selection/lexicase.rs` — Lexicase operators; method name update

### Requirements
- `.planning/REQUIREMENTS.md` — `TRAITS-01` (MultiCaseFitness / VectorFitness), `SEL-02`/`SEL-03` (lexicase selection)

### Breaking Change Docs
- `.planning/phases/65-v3-0-0-migration-guide/` — MIGRATION.md target for before/after code snippets

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/traits/multi_case_fitness.rs` — 15-line file; rename is mechanical. `case_fitness()` and `set_case_fitness()` rename to `fitness_values()` and `set_fitness_values()`
- `src/engines/nsga2/pareto.rs` — `ParetoIndividual::new(chromosome, objectives)` already accepts a `Vec<f64>`; just change the source from external closure eval to `chromosome.fitness_values().to_vec()`

### Established Patterns
- All MO engines already have `nsga*_config.num_objectives` validated against closure count — this validation changes to validating against `chromosome.fitness_values().len()` at runtime or a new `num_objectives` config field
- WASM: no `Instant`/`par_iter` concerns in this phase — trait renaming is pure synchronous Rust
- `#[cfg(not(target_arch = "wasm32"))]` guards in engines are unaffected

### Integration Points
- All 6 MO engines must add `U: VectorFitness` bound to their `impl` blocks and the `run()` / `run_generation()` methods
- `Nsga2Configuration.objective_fns` removal propagates to the builder trait in `src/traits/configuration.rs`
- `src/operations/selection.rs::factory_lexicase` bound update is the only change in the selection module

</code_context>

<specifics>
## Specific Ideas

- The `std::slice::from_ref(&self.fitness())` default impl for `fitness_values()` is tricky — `fitness()` returns `f64` by value, so a lifetime-safe default requires storing the value. The planner should check whether a default impl is achievable (returning `&[f64]` from a `&self` method where the `f64` is not already behind a reference). May require storing a `Vec<f64>` field or using a different default strategy.

</specifics>

<deferred>
## Deferred Ideas

- **`objective_fns` as a shorthand helper long-term** — keeping closures as a convenience API (engine calls `set_fitness_values()` internally) was considered but deferred. If user feedback shows the full `calculate_fitness()` migration is too cumbersome, this can be added in a future usability phase.
- **Blanket impl of `VectorFitness` for all `ChromosomeT`** — would eliminate the explicit opt-in requirement but was rejected; too much magic for default `fitness_values()` behavior, and MO engine bounds become trivially satisfied by all chromosomes.

</deferred>

---

*Phase: 55-rfc-multi-valued-fitness*
*Context gathered: 2026-05-29*
