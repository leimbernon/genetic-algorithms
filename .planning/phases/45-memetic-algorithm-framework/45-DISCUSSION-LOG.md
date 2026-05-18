# Phase 45: Memetic Algorithm Framework - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 45-Memetic Algorithm Framework
**Areas discussed:** Trait design & API, GA loop placement, Lamarckian vs Baldwinian, Parallel execution model, Interaction ordering

---

## Trait Design & API

| Option | Description | Selected |
|--------|-------------|----------|
| Full trait + enum + factory | Consistent with CrossoverOperator, MutationOperator, etc. Trait method receives &mut U + fitness_fn. | ✓ |
| Closure-only | Lighter approach: store as Arc<dyn Fn(...)>, no enum/factory, no serialization. | |
| Hybrid (trait + closure) | Both: full trait for built-ins + separate builder method for custom closures. | |

**User's choice:** Full trait + enum + factory
**Notes:** Consistent with every other operator pattern in the codebase. User wants users to be able to implement custom LocalSearch strategies via the trait.

---

## GA Loop Placement

| Option | Description | Selected |
|--------|-------------|----------|
| After offspring, before survivor | After crossover+mutation+fitness, before survivor selection. Offspring refined before competing. | ✓ |
| After survivor selection | After survivor selection and elite reinsertion. Refines next generation's breeding population. | |
| Per-strategy configurable | Placement varies per-application-strategy. More flexible but complex validation. | |

**User's choice:** After offspring, before survivor
**Notes:** Most common in memetic algorithm literature. Clean insertion point — offspring refined, then compete in survivor selection.

---

## Lamarckian vs Baldwinian

| Option | Description | Selected |
|--------|-------------|----------|
| Both modes, configurable | Config flag on LocalSearchConfiguration. Default Lamarckian. Both implemented since change is small. | ✓ |
| Lamarckian only | DNA + fitness updated. Learning inherits into genes. Standard approach. | |
| Baldwinian only | Only fitness updated, original DNA preserved. Learning guides selection but doesn't alter genes. | |

**User's choice:** Both modes, configurable
**Notes:** Default to Lamarckian (more common in literature). Config flag is simple — whether to call set_dna() or just set_fitness().

---

## Parallel Execution Model

| Option | Description | Selected |
|--------|-------------|----------|
| Trait method parameter | Pass fitness_fn as parameter to trait method. Ga loop uses Arc::clone to share across rayon. | ✓ |
| Stored in operator | Store fitness_fn ref in operator struct at construction time. Simpler per-call signature. | |
| Both (fallback) | Pass both: trait method receives fitness_fn AND operator can store a pre-configured one. | |

**User's choice:** Trait method parameter
**Notes:** Clean separation — operator doesn't hold fitness state. Arc::clone is zero-cost for sharing across rayon tasks.

---

## Interaction Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| LS after repair/constraints | Local search runs after repair + constraint penalty. Operates on already-valid individuals. | ✓ |
| LS before repair | Local search on possibly-invalid individuals, then repair might undo changes. | |

**User's choice:** LS after repair/constraints

| Option | Description | Selected |
|--------|-------------|----------|
| Skip LS on regrown | Extension regrown individuals don't get local search. | ✓ |
| Apply LS to regrown | Apply local search to regrown individuals too. | |
| Per-strategy decides | Let application strategy decide for regrown individuals. | |

**User's choice:** Skip LS on regrown
**Notes:** Extension is a diversity-rescue mechanism — applying local search to random regrowth is expensive and likely wasted effort.

## Claude's Discretion

- LocalSearch enum variant naming (HillClimbing as first built-in)
- Application strategy implementation details
- Trait method signature specifics (whether to pass strategy params per-call or store in struct)
- `LocalSearchConfiguration` struct fields and builder methods
- HillClimbing defaults (step_size=0.1, max_iterations=20 — from ScatterEngine reference)
- Factory function location and module structure
- Serde derives (established pattern)
- Whether to support user-supplied custom strategies via closures (future consideration)
- Ga struct field type (`Option<Box<dyn LocalSearchOperator<U>>>`)

## Deferred Ideas

None — discussion stayed within phase scope.
