# Phase 81: Add a Prelude Module for Ergonomic Imports (Issue #283) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-22
**Phase:** 81-add-a-prelude-module-for-ergonomic-imports-issue-283
**Areas discussed:** Prelude breadth, Feature-gated items, Example to update

---

## Prelude Breadth

### Engine scope

| Option | Description | Selected |
|--------|-------------|----------|
| All engines | Every engine type: Ga, IslandGa, all multi-objective (6), all alt-metaheuristics (7), HillClimb, Permutate, GpGa | ✓ |
| Common subset only | Ga, IslandGa, Nsga2Engine, DeEngine, CmaEngine, PsoEngine | |
| Standard GA + multi-objective only | Ga, IslandGa, NSGA-II/III/MOEA-D/SPEA2/SMS-EMOA/IBEA only | |

**User's choice:** All engines
**Notes:** Dead code eliminated anyway — include everything so the prelude never forces an explicit engine import.

### Engine config structs

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — all engine configs | CmaConfiguration, PsoConfiguration, EdaConfiguration, AlpsConfiguration, etc. included | ✓ |
| No — only GaConfiguration traits | Only ConfigurationT + per-area builder traits | |
| You decide | Let researcher/planner resolve | |

**User's choice:** Yes — all engine configs

### Observer scope

| Option | Description | Selected |
|--------|-------------|----------|
| Core observers only — GaObserver + NoopObserver | GaObserver trait + NoopObserver; concrete impls excluded | ✓ |
| All observers | GaObserver, NoopObserver, LogObserver, CompositeObserver, AllObserver, engine-specific observers (~15 names) | |
| No observers in prelude | Observers are advanced usage; exclude entirely | |

**User's choice:** GaObserver trait + NoopObserver only

---

## Feature-gated items

| Option | Description | Selected |
|--------|-------------|----------|
| Re-export behind same cfg gates | #[cfg(feature="logging")] LogObserver, etc. in prelude.rs | ✓ |
| Exclude entirely from prelude | Keep prelude unconditionally stable | |
| You decide | Let researcher/planner resolve | |

**User's choice:** Re-export behind same cfg gates
**Notes:** Prelude should mirror lib.rs behavior — if the feature is enabled, the type appears in the glob.

---

## Example to update

### Showcase file

| Option | Description | Selected |
|--------|-------------|----------|
| Update rastrigin.rs | Convert 11 import lines to prelude glob; canonical simple-GA example | ✓ |
| New prelude_demo.rs | Dedicated example file for prelude | |
| Update onemax_binary.rs | Simplest possible GA example | |

**User's choice:** Update rastrigin.rs

### Documentation targets

| Option | Description | Selected |
|--------|-------------|----------|
| README + getting-started guide | Add section to README.md and docs/getting-started.md | ✓ |
| README only | Single block in README.md | |
| You decide | Let planner pick based on what exists | |

**User's choice:** README + getting-started guide

---

## Claude's Discretion

None — all areas had clear user decisions.

## Deferred Ideas

None — discussion stayed within phase scope.
