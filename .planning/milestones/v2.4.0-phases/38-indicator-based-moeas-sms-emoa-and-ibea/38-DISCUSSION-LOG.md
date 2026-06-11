# Phase 38: Indicator-based MOEAs — SMS-EMOA and IBEA - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 38-indicator-based-moeas-sms-emoa-and-ibea
**Areas discussed:** Phase ordering, Engine directory layout, Observer design, Example benchmarks

---

## Phase Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| Swap: do Phase 39 first | Build the shared quality-indicator library first. Then Phase 38 engines use it as a proper dependency — cleaner architecture, no refactoring later. | ✓ |
| Flip: build 38 with inline indicators | Bootstrap minimal hypervolume + epsilon-indicator inside Phase 38. Phase 39 later extracts+expands them. | |
| Merge: combine 38 + 39 | Build quality indicators AND both engines together in one larger phase. | |

**User's choice:** Do Phase 39 first. Phase 38 is blocked until quality indicator library is complete.

---

## Engine Directory Layout

| Option | Description | Selected |
|--------|-------------|----------|
| Separate dirs | sms_emoa/ and ibea/ each get their own directory with own configuration.rs. Follows established one-engine-per-dir pattern. | ✓ |
| Shared indicator_based/ dir | sms_emoa/ and ibea/ share a common parent with a shared config module. Tighter coupling but less code duplication. | |

**User's choice:** Separate directories per engine. Matches prior engine conventions exactly.

---

## Observer Design

| Option | Description | Selected |
|--------|-------------|----------|
| Separate trait per engine | SmsEmoaObserver<U> and IbeaObserver<U> each have engine-specific hooks. Follows the established pattern from all prior engines. | ✓ |
| One shared observer trait | A single IndicatorBasedObserver<U> trait shared by both engines. Less code but breaks the one-trait-per-engine convention. | |

**User's choice:** Separate observer traits per engine. Consistent with prior art.

---

## Example Benchmarks

| Option | Description | Selected |
|--------|-------------|----------|
| Both use ZDT1 | Same problem for both engines — fair comparison. ZDT1 is canonical (2-objective, 30 vars) used by SPEA2/NSGA2. | ✓ |
| ZDT1 for one, DTLZ2 for other | SMS-EMOA on ZDT1, IBEA on DTLZ2. Shows each engine in different contexts. | |

**User's choice:** Both engines use ZDT1. Users can directly compare SMS-EMOA vs IBEA vs SPEA2.

---

## Claude's Discretion

None — user made all decisions explicitly.

## Deferred Ideas

None — discussion stayed within phase scope.
