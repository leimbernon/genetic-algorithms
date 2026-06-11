# Phase 40: Constraint Handling — Penalty Functions, Feasibility Rules, RepairOperator - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 40-constraint-handling-penalty-functions-feasibility-rules-repa
**Areas discussed:** Penalty application model, Feasibility rules scope, Multi-objective scope, Constraint function API

---

## Penalty Application Model

| Option | Description | Selected |
|--------|-------------|----------|
| A. Automatic | Ga applies penalty internally after fitness evaluation | |
| B. Manual | Ga provides strategy enums and helpers; user applies in fitness function | ✓ |
| C. Semi-automatic | Ga tracks violations; user configures whether penalty is auto-applied | |

**User's choice:** B
**Notes:** Clean separation of concerns. Ga provides the tools; user decides how to use them.

---

## Feasibility Rules Scope

| Option | Description | Selected |
|--------|-------------|----------|
| A. Selection only | Tournament selection checks feasibility before fitness | |
| B. Selection + Survivor | Both parent selection and population trimming | |
| C. Every context | Selection, survivor, elitism, best-chromosome tracking | ✓ |

**User's choice:** C
**Notes:** Full consistency across the GA loop. Comparison helper function to standardize.

---

## Multi-Objective Scope

| Option | Description | Selected |
|--------|-------------|----------|
| A. Ga only | Single-objective Ga only; MOEAs deferred | ✓ |
| B. Ga + NSGA-II | Most common constrained MOEA use case | |
| C. All 6 MOEAs | Deep scope including all MOEA engines | |

**User's choice:** A
**Notes:** Keep phase focused. MOEA constraint support deferred to future phase.

---

## Constraint Function API

| Option | Description | Selected |
|--------|-------------|----------|
| A. Single fn returning f64 | Simplest; works with all penalty strategies | |
| B. Vec of per-constraint fns | More granular; supports per-constraint breakdown | ✓ |
| C. Trait-based ConstraintSystem | Most flexible but heavier API | |

**User's choice:** B
**Notes:** Allows feasibility rules and reporting to show per-constraint breakdown.

---

## Claude's Discretion

- Adaptive penalty state management: planner decides approach (separate struct or Ga field)
- Observer hooks for constraint events: no new hooks (default)
- Validation details and error messages
- Serde derive attribute placement
- Exact mutation-site positions for repair operator

## Deferred Ideas

- Multi-objective constraint handling (NSGA-II through IBEA) — future phase
- Automatic penalty application mode — additive change, not breaking
- GaObserver hooks for constraint events — no immediate demand

---

*Context gathered: 2026-05-11*
