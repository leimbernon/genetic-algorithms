# Phase 40 Research: Constraint Handling

## Current State Assessment

Phase 40 (Constraint Handling) is significantly pre-implemented. The core implementation code compiles correctly, but the test file has compilation errors.

### What Already Works (Implementation)

| Component | Status | Location |
|-----------|--------|----------|
| `PenaltyStrategy` enum (None, Static, Dynamic, Adaptive) | ✅ Complete | `src/constraints.rs:14-47` |
| `ConstraintHandling` enum (FeasibilityRules) | ✅ Complete | `src/constraints.rs:51-57` |
| `total_violation()` helper | ✅ Complete | `src/constraints.rs:63-65` |
| `apply_static_penalty()` helper | ✅ Complete | `src/constraints.rs:70-72` |
| `apply_dynamic_penalty()` helper | ✅ Complete | `src/constraints.rs:77-88` |
| `validate_penalty_strategy()` | ✅ Complete | `src/constraints.rs:94-143` |
| `GaError::InvalidConstraintConfiguration` | ✅ Complete | `src/error.rs:49` |
| `GaError::RepairError` | ✅ Complete | `src/error.rs:51` |
| `RepairOperator` trait (Send + Sync) | ✅ Complete | `src/traits/operators.rs:231-238` |
| `Ga` struct constraint fields | ✅ Complete | `src/engines/ga.rs:141-158` |
| Builder methods (4 methods) | ✅ Complete | `src/engines/ga.rs:635-678` |
| `process_constraints_population()` | ✅ Complete | `src/engines/ga.rs:1443-1471` |
| `apply_feasibility_rules()` | ✅ Complete | `src/engines/ga.rs:1476-1526` |
| `apply_penalty_to_chromosomes()` (all 4 strategies) | ✅ Complete | `src/engines/ga.rs:1529-1595` |
| Repair + penalty in GA run loop | ✅ Complete | `src/engines/ga.rs:927-974` |
| Adaptive penalty tracking | ✅ Complete | `src/engines/ga.rs:1549-1586` |
| lib.rs re-exports | ✅ Complete | `src/lib.rs:147-148` |
| NSGA-II `with_constraint_fns()` | ✅ Complete | `src/engines/nsga2/mod.rs:140-146` |
| `constrained_dominates()` | ✅ Complete | `src/engines/multi_objective/pareto.rs` |
| `non_dominated_sort_constrained()` | ✅ Complete | `src/engines/multi_objective/non_dominated_sort.rs` |
| NSGA-II constraint integration in run loop | ✅ Complete | `src/engines/nsga2/mod.rs:227-507` |

### What's Broken

**`tests/test_constraints.rs`** — 10 compilation errors:

| Error | Cause | Fix |
|-------|-------|-----|
| `ProblemSolving` import not found | path changed | Use `configuration::ProblemSolving` |
| `id_values()` not found | API changed | Use `value()` on RangeGene |
| `n as i32` → `usize` mismatch | type mismatch | `.try_into().unwrap()` |
| `.dna()` is field not method | ChromosomeT not in scope | Import `ChromosomeT` trait |
| `.set_dna()` not found | ChromosomeT not in scope | Import `ChromosomeT` trait |
| `max(0.0)` on ambiguous float | type inference | Add type annotation |
| Unused imports | stale imports | Clean up |

### What's Missing

1. **No `ConstraintHandling` integration test** — feasibility rules with GA run (`test_constraint_handling_ga_with_static_penalty` exists but no equivalent for `ConstraintHandling::FeasibilityRules`)
2. **No `PenaltyStrategy::Adaptive` integration test** — adaptive penalty with GA run
3. **No multi-objective constraint test** — NSGA-II with constraint functions tested
4. **No constrained optimization example** — no runnable example showing constrained optimization (e.g., G1 benchmark)
5. **No `ConstraintHandling::None` variant** — only `FeasibilityRules` exists; no way to "disable" once set
6. **Original fitness not preserved** — issue #212 mentions "Original (unpenalized) fitness accessible for reporting" but penalty modifies fitness in-place
7. **Other MOE engines lack constraints** — NSGA-III, MOEA/D, SPEA2, SMS-EMOA, IBEA don't have constraint_fns

### Constraints Not Required for This Phase

Per the roadmap, this phase covers issues #212, #213, #214 (single-objective GA constraint handling). The following are explicitly out of scope:
- Multi-objective engine constraints beyond NSGA-II (already exists) — deferred to engine-specific issues
- `ConstraintHandling::None` variant — not needed; default when `constraint_handling` is `None`

## Verification Strategy

1. Fix `test_constraints.rs` compilation errors
2. Add adaptive penalty integration test
3. Add feasibility rules integration test  
4. Add NSGA-II constraint integration test
5. Add constrained optimization example (G1)
6. Verify `cargo test`, `cargo clippy`, `cargo doc --no-deps` all pass
7. Verify WASM: `cargo check --target wasm32-unknown-unknown` (pre-existing `getrandom` issue expected)
