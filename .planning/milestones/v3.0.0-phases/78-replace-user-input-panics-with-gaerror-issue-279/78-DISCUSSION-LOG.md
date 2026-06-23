# Phase 78: Replace User-Input Panics with GaError (Issue #279) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-19
**Phase:** 78-replace-user-input-panics-with-gaerror-issue-279
**Areas discussed:** GP chromosome trait panics, Mutex poison handling scope, Cellular/ALPS validation placement, Audit scope boundary

---

## GP Chromosome Trait Panics

| Option | Description | Selected |
|--------|-------------|----------|
| Keep as panics | Trait method signatures can't return Result. Add/keep # Panics doc comment. | ✓ |
| Add a fallible helper method | Add try_dna() -> Result alongside existing dna() | |
| You decide | Claude picks | |

**User's choice:** Keep as panics

**Follow-up — improve panic messages?**

| Option | Description | Selected |
|--------|-------------|----------|
| Current messages are fine | Already say "not supported — use GpGa, not Ga" | ✓ |
| Add 'Did you mean GpGa<N>?' hint | More explicit suggestion in panic message | |

**User's choice:** Current messages are fine

**Notes:** GpChromosome panics are intentional misuse-panics. Cannot be converted due to trait signature constraint.

---

## Mutex Poison Handling Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Propagate as GaError | generation loop returns Result<(), GaError>; PoisonError → GaError::InternalError | ✓ |
| Recover gracefully (log + continue) | unwrap_or_else(|e| e.into_inner()); accepts poisoned lock | |
| You decide | Claude picks | |

**User's choice:** Propagate as GaError

**Follow-up — which GaError variant?**

| Option | Description | Selected |
|--------|-------------|----------|
| New variant: GaError::InternalError | Dedicated variant, clearly distinct, grep-able | ✓ |
| GaError::MutationError | Re-uses existing variant | |
| GaError::ConfigurationError | Misrepresents the failure | |

**User's choice:** New variant GaError::InternalError

**Follow-up — convert fitness cache locks too?**

| Option | Description | Selected |
|--------|-------------|----------|
| Convert cache locks too | Consistent approach across all mutex failures | ✓ |
| Leave cache expects as-is | Cache lock poisoning = run already toast; expect message is clear | |

**User's choice:** Convert cache locks too

**Notes:** One new variant `InternalError(String)` added to GaError. All mutex failures (AOS state + fitness cache) use it.

---

## Cellular/ALPS Validation Placement

| Option | Description | Selected |
|--------|-------------|----------|
| Move to build() / new() | new() returns Result<Self, GaError>; fail-fast at construction | ✓ |
| Convert run() panics to GaError in run() | Keep validation in run() but return Err instead of panic | |

**User's choice:** Move to new()

**Follow-up — breaking change acceptable?**

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — change new() to return Result | v3.0.0 is semver-breaking anyway; right API | ✓ |
| Add separate validate() method | Less breaking but awkward separation | |

**User's choice:** Yes, change new() to return Result<Self, GaError>

**Notes:** Both CellularEngine::new() and AlpsEngine::new() change signature. Callers need .unwrap() or ?.

---

## Audit Scope Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Exclude internal invariants | Keep expects/unwraps with clear proof they're unreachable from user input | ✓ |
| Convert all unwrap/expect | Maximum safety; larger diff | |
| You decide | Claude draws the line | |

**User's choice:** Exclude internal invariants

**Follow-up — Lexicase wrong-factory panic?**

| Option | Description | Selected |
|--------|-------------|----------|
| Convert to GaError::SelectionError | User-input-reachable (user configured Lexicase + called wrong factory) | ✓ |
| Keep as panic | Programming error; panics are fine for this case | |

**User's choice:** Convert to GaError::SelectionError

**Notes:** Exclusion criteria: (a) proof in surrounding code that the call is unreachable from valid user input, AND (b) expect/unwrap message clearly states why. Included cases: crossover downcast expects, gp/crossover index expects, ga/mod.rs stats.last().unwrap().

---

## Claude's Discretion

- Exact error messages for each new `GaError::InternalError` and `GaError::InitializationError` instance
- Whether to add a helper macro/function for the mutex poison conversion pattern (if it repeats 10+ times)
- Order of validation checks within new() for Cellular and ALPS

## Deferred Ideas

None — discussion stayed within phase scope.
