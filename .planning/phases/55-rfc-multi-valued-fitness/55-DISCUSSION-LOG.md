# Phase 55: RFC Multi-Valued Fitness - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-29
**Phase:** 55-rfc-multi-valued-fitness
**Areas discussed:** Trait topology, MO engine integration, Semantic separation, Migration strategy

---

## Trait Topology

| Option | Description | Selected |
|--------|-------------|----------|
| No — ChromosomeT stays scalar | ChromosomeT keeps fitness() -> f64; MultiCaseFitness stays separate opt-in supertrait | |
| Yes — add fitness_values() to ChromosomeT | ChromosomeT gains fn fitness_values() -> &[f64] with default impl | |
| Merge MultiCaseFitness into ChromosomeT | case_fitness()/set_case_fitness() move into ChromosomeT as required methods | |
| One trait, two use cases (clarify in docs) | Keep/rename MultiCaseFitness; MO engines and lexicase both use it | ✓ |
| Two traits, same shape | MultiCaseFitness stays for lexicase; new MultiObjectiveFitness for MO engines | |
| One supertrait with marker subtrait | VectorFitness base + marker subtrait per engine type | |

**User's choice:** One trait, two use cases — same trait (`VectorFitness`) serves lexicase and MO engines; semantic difference handled by engine behavior and docs.

**Notes:** User initiated a clarifying discussion: "so the idea would be that MultiCaseFitness should also be used by the MO engines instead of external closures?" This confirmed the intent of the RFC — unify both vector-fitness use cases under one trait.

---

## MO Engine Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Switch MO engines to read from trait | Users implement calculate_fitness(); NSGA-II reads fitness_values() instead of calling objective_fns | ✓ |
| Keep closures, add trait as alternative | MO engines still accept objective_fns but also accept chromosomes implementing the trait | |
| Trait-only for new users, closures stay for compat | Closures deprecated but not removed in v3.0.0; prefer fitness_values() if bound is met | |

**User's choice:** Switch MO engines to read from trait — full migration, no dual-track.

---

## Semantic Separation (Naming)

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — rename to VectorFitness | MultiCaseFitness → VectorFitness; case_fitness() → fitness_values() | ✓ |
| Keep the name MultiCaseFitness | Name stays; docs explain dual use | |
| ObjectiveFitness | MultiCaseFitness → ObjectiveFitness | |
| fitness_values() / set_fitness_values() | Method names mirror existing fitness() / set_fitness() pattern | ✓ |
| objective_values() / set_objective_values() | More explicit about MO intent | |
| Keep case_fitness() / set_case_fitness() | No method rename, only trait rename | |

**User's choice:** Rename to `VectorFitness` with methods `fitness_values()` / `set_fitness_values()`.

---

## Migration Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Hard rename in v3.0.0 | MultiCaseFitness removed; VectorFitness is replacement | ✓ |
| Type alias bridge | pub type MultiCaseFitness = VectorFitness; kept for one release cycle | |
| Keep MultiCaseFitness, add VectorFitness as supertrait | Old impls still compile; supertrait layer must be removed later | |
| objective_fns removed entirely | Users move all objective logic into calculate_fitness() + set_fitness_values() | ✓ |
| objective_fns deprecated but kept | Marked #[deprecated] in v3.0.0; removed in v4.0.0 | |
| objective_fns kept as shorthand helper | Closures still accepted; engine populates fitness_values internally | |
| Default impl returning &[self.fitness()] | VectorFitness::fitness_values() defaults to scalar-wrapping | ✓ |
| No default impl | Explicit impl always required | |
| Blanket impl on ChromosomeT | All ChromosomeT impls get VectorFitness automatically | |

**User's choice:** Hard rename + objective_fns removed + default impl for fitness_values(). Clean v3.0.0 break with MIGRATION.md documentation.

---

## Claude's Discretion

- Default impl strategy for `fitness_values() -> &[f64]`: returning a reference to a by-value `f64` from `fitness()` has lifetime implications — planner should verify whether the default impl is achievable or if it requires storing a `Vec<f64>` field. Flagged in CONTEXT.md specifics.

## Deferred Ideas

- `objective_fns` as long-term convenience helper (closures as shorthand that auto-populate `fitness_values`) — deferred pending user feedback after v3.0.0 ships
- Blanket `VectorFitness` impl for all `ChromosomeT` — rejected during discussion (too magical, undermines explicit opt-in)
