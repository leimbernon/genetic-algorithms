# Phase 44: Standard Benchmark Functions Suite - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 44-standard-benchmark-functions-suite
**Areas discussed:** Module structure, API design, Scope & coverage, Integration & migration, Multi-objective interface

---

## Module Structure

| Option | Description | Selected |
|--------|-------------|----------|
| By family (recommended) | Separate modules per function family | ✓ |
| Flat single file | All functions in one file | |
| Per-function files | One file per function | |

**User's choice:** By family (recommended)
**Notes:** Following existing sub-module architecture patterns

## API Design

| Option | Description | Selected |
|--------|-------------|----------|
| Structs with metadata (recommended) | BenchmarkFn trait with name/bounds/optimum/evaluate | ✓ |
| Free functions only | Plain pub fn pointers | |
| Trait + free fn shims | Both trait and convenience functions | |

**User's choice:** Structs with metadata (recommended)
**Notes:** Trait-driven approach consistent with library architecture

## Scope & Coverage

| Option | Description | Selected |
|--------|-------------|----------|
| Stick to the roadmap (recommended) | Sphere, Rastrigin, Ackley, ZDT1-6, DTLZ1-7 | ✓ |
| Add Rosenbrock + Schwefel | Two more classic single-objective benchmarks | |
| Include all classics | Full set including Griewank and CEC subset | |

**User's choice:** Stick to the roadmap (recommended)
**Notes:** ~17 functions exceeds the 15+ target

## Integration & Migration

| Option | Description | Selected |
|--------|-------------|----------|
| New code only | Create library, leave existing code as-is | |
| Migrate benches (de.rs) | Only refactor the DE benchmark | |
| Migrate all | Refactor ALL benches and examples to use shared library | ✓ |

**User's choice:** Migrate all
**Notes:** Most consistent approach, carry regression awareness

## Multi-objective Interface

| Option | Description | Selected |
|--------|-------------|----------|
| Unified Vec<f64> (recommended) | Same trait, SO returns vec![val], MO returns vec![f1, f2,...] | ✓ |
| Two separate traits | BenchmarkFn and MoBenchmarkFn | |
| Dimension-parameterized | Generic over objective count at call site | |

**User's choice:** Unified Vec<f64> (recommended)
**Notes:** Simplest and most flexible

## Claude's Discretion

- Dimension defaults for ZDT (30 vars) and DTLZ (n vars, m objectives)
- Exact struct naming conventions
- Convenience constants (optima, bounds)
- Test strategy
- serde derives on benchmark structs

## Deferred Ideas

None — discussion stayed within phase scope.
