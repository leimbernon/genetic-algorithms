# Phase 70: Replace Operator Downcasting - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-17
**Phase:** 70-replace-operator-downcasting
**Areas discussed:** Dispatch mechanism, Operator signatures, Error behavior

---

## Dispatch mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| New trait on chromosomes | Add RealValuedMutation trait with methods chromosomes implement | ✓ |
| Consolidated macro dispatch | Single generic dispatch function trying all 4 types | |
| Compile-time type routing | Associated types or const generics on LinearChromosome | |

**User's choice:** New trait on chromosomes
**Notes:** User selected "Single trait, optional methods" for trait design — one trait with 5 optional methods, default impls return Err.

---

## Operator signatures

| Option | Description | Selected |
|--------|-------------|----------|
| Keep signatures as-is | Phase 70 only removes downcasting; Phase 71 cleans up signatures | ✓ |
| Clean up signatures too | Also wrap parameters in structs in this phase | |

**User's choice:** Keep signatures as-is
**Notes:** Clean separation of concerns — downcasting removal in Phase 70, parameter struct refactor in Phase 71.

---

## Error behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Return error (current) | GaError::MutationError with clear message | ✓ |
| Silent skip | Return Ok(()) when unsupported | |
| Fall back to swap | Match ValueMutable default pattern | |

**User's choice:** Return error (current behavior)
**Notes:** Matches existing behavior, downstream code already handles this pattern.

---

## Agent's Discretion

- Exact error message strings for default trait implementations
- Whether to use `#[inline]` on trait default methods
- Trait placement (src/traits/ following existing pattern)

## Deferred Ideas

None
