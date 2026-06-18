# Phase 71: Per-Operator Mutation Parameters - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-18
**Phase:** 71-per-operator-mutation-params
**Areas discussed:** Named struct format, Mutation enum variant shape, factory_with_params cleanup, ValueMutable/RealValuedMutation trait scope

---

## Named struct format

| Option | Description | Selected |
|--------|-------------|----------|
| Option<f64> with defaults at dispatch | Matches current Mutation enum pattern; None = use default; minimal change surface | ✓ |
| f64 with defaults at construction | Cleaner API but forces explicit values; more breaking change surface | |
| You decide | Leave to planner/implementor | |

**User's choice:** Option<f64> with defaults at dispatch

**Sub-question: Where should structs live?**

| Option | Description | Selected |
|--------|-------------|----------|
| src/operations.rs alongside Mutation enum | Keeps params and enum in same file; logically cohesive | ✓ |
| src/operations/mutation/params.rs (new module) | Separate file; cleaner for growth; requires new mod + re-export | |
| You decide | Leave to planner | |

**User's choice:** src/operations.rs alongside the Mutation enum

---

## Mutation enum variant shape

| Option | Description | Selected |
|--------|-------------|----------|
| Tuple variants — Mutation::Gaussian(GaussianParams) | Clean, idiomatic; match arms update to Mutation::Gaussian(p); breaking change acceptable in v3.0.0 | ✓ |
| Keep inline struct fields, add named type aliases only | Zero breakage; structs are cosmetic, not usable as standalone types | |
| You decide | Leave to planner/implementor | |

**Sub-question: Which variants get param structs?**

| Option | Description | Selected |
|--------|-------------|----------|
| Only the parameterized variants | Creep, Gaussian, Polynomial, NonUniform, Differential, Cauchy, LevyFlight, SelfAdaptiveGaussian; unit variants stay unit | ✓ |
| All variants get a struct | Even Swap gets SwapParams{}; uniform but noisy | |

---

## factory_with_params cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Remove factory_with_params entirely | Redundant with factory(); clean v3.0.0 breaking change | ✓ |
| Remove just the _step/_sigma args | Leaves a function identical to factory() — confusing | |
| Keep as #[deprecated] stub | Removes call-site breakage; one-version migration path | |

**Sub-question: factory_with_chromosome_length**

| Option | Description | Selected |
|--------|-------------|----------|
| Remove _step/_sigma args only | Signature becomes factory_with_chromosome_length(mutation, individual, chromosome_length); 4 engine call sites drop None, None | ✓ |
| Remove function entirely | More churn at engine call sites | |

---

## ValueMutable/RealValuedMutation trait scope

| Option | Description | Selected |
|--------|-------------|----------|
| No — leave trait method signatures as raw f64 | Internal dispatch hooks, not user-facing API; narrower scope = less breakage | ✓ |
| Yes — wrap trait method args in param structs too | Consistent end-to-end; larger change affecting Range<T> impls too | |

---

## Claude's Discretion

- Exact derived trait impls on param structs (e.g., `#[derive(Debug, Clone, PartialEq)]`) — follow Mutation enum's existing derives
- Whether to implement `Default` on param structs (returning `None` for all fields)
- Serde `#[cfg_attr(feature = "serde", serde(default))]` annotations on struct fields

## Deferred Ideas

None — discussion stayed within phase scope.
