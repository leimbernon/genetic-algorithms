# Phase 43: Adaptive Operator Selection (AOS) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 43-Adaptive Operator Selection (AOS)
**Areas discussed:** Credit reward model, Portfolio structure, GA loop integration, Interaction with existing Adaptive GA

---

## Credit Reward Model

| Option | Description | Selected |
|--------|-------------|----------|
| Offspring vs parent | Compare offspring fitness to parent fitness. Positive delta = reward | ✓ |
| Best-fitness improvement | Only global best improvement generation gives credit | |
| Offspring rank-based | Rank all offspring, map to sliding scale | |

| Option | Description | Selected |
|--------|-------------|----------|
| Raw improvement delta | reward = max(0, parent_fitness - offspring_fitness) | |
| Binary win/loss | 1.0 if offspring beats parent, 0.0 otherwise | |
| Normalized improvement | (parent_fitness - offspring_fitness) / best_fitness | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| Sliding window | Fixed-size window of recent W rewards per operator | ✓ |
| Exponential recency-weighted average | Alpha-weighted moving average | |
| Cumulative all-time sum | All rewards since start | |

| Option | Description | Selected |
|--------|-------------|----------|
| Default 50, configurable | W=50 with .with_reward_window(usize) | ✓ |
| Fixed W=50 | No user override | |
| Population-proportional default | Scoped to pop_size * offspring count | |

---

## Portfolio Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Separate crossover + mutation portfolios | Independent AOS state per operator type | ✓ |
| Combined flat list | Mixed crossover + mutation in one portfolio | |
| Paired strategies | Each arm = (crossover, mutation) pair | |

| Option | Description | Selected |
|--------|-------------|----------|
| Vec of existing operator enums | .with_crossover_portfolio(vec![Crossover::SBX, ...]) | ✓ |
| New wrapper enum | AosOperator wrapping Crossover/Mutation | |
| Vec of Box<dyn Operator> | Runtime dynamic dispatch via factory | |

| Option | Description | Selected |
|--------|-------------|----------|
| Minimum 2 per portfolio | AOS needs >1 to be meaningful | ✓ |
| No minimum | Any length OK | |

| Option | Description | Selected |
|--------|-------------|----------|
| Portfolio replaces single operator | .with_crossover() ignored when portfolio set | ✓ |
| Coexist with fallback | Standard operator used during exploration | |
| Exclusive mode enum | Explicit AosMode vs StandardMode | |

---

## GA Loop Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Per offspring couple | Each parent pair gets best estimated operator | ✓ |
| Per generation | One operator for all offspring | |
| Per individual offspring | New operator per offspring | |

| Option | Description | Selected |
|--------|-------------|----------|
| Replace operator calls in offspring loop | AOS wraps operator dispatch | ✓ |
| Pre-loop selection step | Select operator before loop starts | |
| AOS as meta-operator wrapper | Loop calls AosCrossover that internally dispatches | |

| Option | Description | Selected |
|--------|-------------|----------|
| Thread-safe accumulator, batch update | Mutex<Vec<(idx, reward)>>, batch after parallel block | ✓ |
| Lock-free atomics | Atomic operations inside parallel loop | |
| Per-thread local buffers | Local per-thread, merged after | |

| Option | Description | Selected |
|--------|-------------|----------|
| Initial exploration phase | First W/2 gen uniform selection | ✓ |
| No explicit exploration | Strategy handles early phase inherently | |

---

## Interaction with Existing Adaptive GA

| Option | Description | Selected |
|--------|-------------|----------|
| Coexist | AOS for WHICH, Adaptive for rates | ✓ |
| AOS replaces Adaptive | AOS is the only adaptive mechanism | |
| Mutually exclusive mode enum | User picks one or the other | |

| Option | Description | Selected |
|--------|-------------|----------|
| Configure-time only | AOS params set at build time | ✓ |
| Self-adaptive via strategy internals | Strategy's internal adaptation handles changes | |

| Option | Description | Selected |
|--------|-------------|----------|
| Ga only | Standard Ga engine | ✓ |
| Ga + Nsga2Ga | Both single and multi-objective GA | |

---

## Claude's Discretion

- AosState struct design (separate per-type or single generic)
- Strategy parameter defaults (PM: alpha=0.8, learning_rate=0.3; AP: beta=0.5, C=1.5; MAB: C=1.0, epsilon=0.1)
- Reward normalization reference and clamping
- Exploration uniform selection implementation
- AOS controller placement on Ga struct
- Serialization derives on state structs
- GaObserver integration for AOS events (optional)

## Deferred Ideas

- AOS for non-Ga engines (Nsga2Ga, De, Scatter, Cellular, Alps) — future phase
- Dynamically adjustable AOS parameters mid-run — no immediate demand
- Combined flat-list portfolios — would need unified AosOperator enum
- GaObserver hooks for AOS events — no immediate demand
