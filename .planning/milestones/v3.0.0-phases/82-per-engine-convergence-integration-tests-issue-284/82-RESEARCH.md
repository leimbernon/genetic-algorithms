# Phase 82: Per-Engine Convergence Integration Tests (Issue #284) - Research

**Researched:** 2026-06-22
**Domain:** Integration testing for genetic algorithm engines
**Confidence:** HIGH

## Summary

This phase adds end-to-end convergence tests for every single-objective engine (DeEngine, ScatterEngine, CellularEngine, AlpsEngine, CmaEngine, PsoEngine). Each test asserts the engine reaches a known optimum within tolerance on the Sphere function, preventing silent regressions in search dynamics.

The research confirms that all six engines already have test files with reusable `sphere` and `random_pop` helpers. The convergence pattern is well-established: configure engine with fixed RNG seed, run optimization, assert `best_fitness < threshold`. The CMA engine additionally supports IPOP/BIPOP restart testing via the existing `SpyObserver` pattern.

**Primary recommendation:** Add one convergence test function to each engine's existing test file, reusing the established helpers and patterns. For CMA, add a separate IPOP restart convergence test.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Convergence testing | Test suite | — | Tests validate engine behavior, not production code |
| RNG determinism | `src/rng.rs` | — | `set_seed(Some(seed))` ensures reproducible runs |
| Engine execution | `src/engines/<name>/` | — | Engines run optimization, tests verify results |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `genetic_algorithms::rng` | (internal) | Deterministic RNG seeding | `set_seed(Some(seed))` + `make_rng()` pattern |
| `genetic_algorithms::chromosomes::Range` | (internal) | Real-valued chromosome | Used by all engine tests |
| `genetic_algorithms::genotypes::Range` | (internal) | Real-valued gene | Used by all engine tests |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rand::Rng` | 0.9.x | Random number generation | Used in `random_pop` helper |
| `std::borrow::Cow` | (stdlib) | Clone-on-write DNA | Used in `set_dna` calls |
| `std::sync::Arc` | (stdlib) | Shared observer references | Used for SpyObserver |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Sphere function | Rastrigin | Sphere is simpler, already used in existing tests |
| Per-engine test files | Single shared test file | Per-engine files match existing structure |

## Package Legitimacy Audit

> No external packages installed in this phase — testing only.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Test Suite                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  tests/engines/<engine>/test_<engine>.rs            │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │    │
│  │  │ sphere()    │  │ random_pop()│  │ test_*_     │ │    │
│  │  │ helper      │  │ helper      │  │ convergence │ │    │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │    │
│  └─────────────────────────────────────────────────────┘    │
│                           │                                  │
│                           ▼                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Engine Configuration                                │    │
│  │  • population_size: 30                               │    │
│  │  • max_generations: 300                              │    │
│  │  • fitness_target: 1.0                               │    │
│  │  • problem_solving: Minimization                     │    │
│  └─────────────────────────────────────────────────────┘    │
│                           │                                  │
│                           ▼                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  rng::set_seed(Some(42))                            │    │
│  │  → Deterministic population initialization           │    │
│  │  → Reproducible optimization run                     │    │
│  └─────────────────────────────────────────────────────┘    │
│                           │                                  │
│                           ▼                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Assertion: best_fitness < 1.0                      │    │
│  │  → Sphere minimum is 0 at origin                    │    │
│  │  → Threshold loose enough for stochastic engines    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
tests/engines/
├── de/
│   └── test_de.rs          # Add: test_de_convergence
├── scatter/
│   └── test_scatter.rs     # Add: test_scatter_convergence
├── cellular/
│   └── test_cellular.rs    # Add: test_cellular_convergence
├── alps/
│   └── test_alps.rs        # Add: test_alps_convergence
├── cma/
│   └── test_cma.rs         # Add: test_cma_convergence, test_cma_ipop_convergence
└── pso/
    └── test_pso.rs         # Add: test_pso_convergence
```

### Pattern 1: Engine Convergence Test

**What:** Test that an engine converges to near-optimal fitness on Sphere function
**When to use:** For every single-objective engine
**Example:**
```rust
// Source: Existing pattern in test_de.rs, test_pso.rs
#[test]
fn test_<engine>_convergence() {
    rng::set_seed(Some(42));
    let config = <EngineConfiguration>::default()
        .with_population_size(30)
        .with_max_generations(300)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1.0);
    
    let mut engine = <Engine>::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere);
    let result = engine.run();
    
    assert!(
        result.best_fitness < 1.0,
        "Engine should converge to sphere minimum < 1.0; got {}",
        result.best_fitness
    );
}
```

### Pattern 2: CMA IPOP Restart Convergence Test

**What:** Test that CMA-ES with IPOP restart strategy converges
**When to use:** For CMA engine restart path testing
**Example:**
```rust
// Source: Existing pattern in test_cma.rs (CMA-12 through CMA-16)
#[test]
fn test_cma_ipop_convergence() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 10,
            max_restarts: 3,
        });
    
    let spy = Arc::new(SpyObserver::default());
    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .with_observer(spy.clone());
    
    let result = engine.run().expect("engine run should succeed");
    
    assert!(
        result.best_fitness < 1.0,
        "CMA with IPOP should converge; got {}",
        result.best_fitness
    );
    assert!(
        spy.restart_count.load(Ordering::SeqCst) >= 1,
        "IPOP should trigger at least one restart"
    );
}
```

### Anti-Patterns to Avoid

- **Don't use loose thresholds:** Threshold of 10.0 or higher doesn't prove convergence — use 1.0 for 5D Sphere
- **Don't skip RNG seeding:** Without `set_seed(Some(seed))`, tests are non-deterministic and flaky
- **Don't use Rastrigin for all engines:** Sphere is simpler and already established in existing tests
- **Don't duplicate helpers:** Reuse existing `sphere` and `random_pop` functions in each test file

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random population generation | Custom initializer | Existing `random_pop` helper | Already tested, consistent across engines |
| Sphere fitness evaluation | New fitness function | Existing `sphere` helper | Already in each test file |
| RNG determinism | Manual seed management | `rng::set_seed(Some(seed))` | Thread-safe, counter-based deduplication |
| Observer spy for restarts | New observer type | Existing `SpyObserver` in test_cma.rs | Already handles all restart event fields |

## Common Pitfalls

### Pitfall 1: Non-Deterministic Test Failures
**What goes wrong:** Tests pass locally but fail in CI due to different RNG seeds
**Why it happens:** Missing `rng::set_seed(Some(seed))` call before population initialization
**How to avoid:** Always call `rng::set_seed(Some(42))` before `random_pop`
**Warning signs:** Test failures that vary between runs

### Pitfall 2: Threshold Too Tight
**What goes wrong:** Tests fail intermittently because threshold is too close to actual convergence
**Why it happens:** Stochastic engines may not always reach very tight thresholds
**How to avoid:** Use threshold of 1.0 for 5D Sphere (loose enough for stochastic, tight enough to prove convergence)
**Warning signs:** Tests that pass 90% of the time but fail occasionally

### Pitfall 3: CMA Result Type Difference
**What goes wrong:** Using `engine.run()` directly for CMA without `.expect()`
**Why it happens:** CMA returns `Result<CmaResult, GaError>` while other engines return result directly
**How to avoid:** Use `engine.run().expect("engine run should succeed")` for CMA
**Warning signs:** Compilation errors about Result types

## Code Examples

Verified patterns from official sources:

### DE Convergence Test (from test_de.rs)
```rust
// Source: tests/engines/de/test_de.rs lines 60-71
#[test]
fn test_de_rand1_binomial_converges() {
    let mut engine = sphere_engine(DeMutationStrategy::Rand1, DeCrossoverMode::Binomial);
    let result = engine.run();
    assert!(
        result.best_fitness < 5.0,
        "DE/rand/1 binomial should reduce sphere fitness; got {}",
        result.best_fitness
    );
    assert!(result.generations > 0);
    assert!(!result.population.is_empty());
}
```

### PSO Convergence Test (from test_pso.rs)
```rust
// Source: tests/engines/pso/test_pso.rs lines 346-362
#[test]
fn test_pso_sphere_converges() {
    rng::set_seed(Some(42));
    let init_pop = random_pop(30, 10, -5.12, 5.12, 42);
    let config = PsoConfiguration::default()
        .with_population_size(30)
        .with_max_generations(500)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_fitness_target(1e-2);
    let mut engine = PsoEngine::new(config, move |_n| init_pop.clone(), sphere);
    let result = engine.run().expect("engine run should succeed");
    assert!(
        result.best_fitness < 1e-2 || result.generations < 500,
        "PSO must converge on 10D Sphere: best_fitness={:.6} after {} generations",
        result.best_fitness,
        result.generations
    );
}
```

### CMA IPOP Restart Test (from test_cma.rs)
```rust
// Source: tests/engines/cma/test_cma.rs lines 401-433
#[test]
fn test_cma_ipop_restarts() {
    let config = CmaConfiguration::default_for_dim(5)
        .with_max_generations(50)
        .with_problem_solving(ProblemSolving::Minimization)
        .with_sigma0(0.3)
        .with_restart_strategy(RestartStrategy::Ipop {
            population_scale: 2.0,
            stagnation_threshold: 5,
            max_restarts: 2,
        });
    
    let spy = Arc::new(SpyObserver::default());
    let mut engine = CmaEngine::new(config, |n| random_pop(n, 5, -5.0, 5.0, 42), sphere)
        .with_observer(spy.clone());
    
    let result = engine.run().expect("engine run should succeed");
    
    assert!(
        spy.restart_count.load(Ordering::SeqCst) >= 1,
        "on_restart should fire at least once"
    );
    assert!(
        result.total_restarts >= 1,
        "total_restarts should be >= 1 after IPOP restart"
    );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No convergence tests | Per-engine convergence tests | Phase 82 | Prevents silent regressions in search dynamics |
| Loose fitness thresholds (< 20.0) | Tight convergence threshold (< 1.0) | Phase 82 | Proves actual convergence, not just improvement |

**Deprecated/outdated:**
- Loose thresholds (10.0, 20.0): Don't prove convergence — use 1.0 for 5D Sphere

## Assumptions Log

> All claims in this research were verified against existing code — no assumptions needed.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| — | (none) | — | — |

## Open Questions

1. **Should we extract shared `sphere` helper to a common module?**
   - What we know: `sphere` is duplicated in 6 test files
   - What's unclear: Whether extraction would break existing test organization
   - Recommendation: Keep helpers in each file — matches existing pattern, avoids cross-file dependencies

2. **What seed value should we use?**
   - What we know: Existing tests use seeds 42, 99, 1, 2, etc.
   - What's unclear: Whether seed 42 is optimal for all engines
   - Recommendation: Use seed 42 consistently — matches existing DE and CMA tests

## Environment Availability

> Skip this section — no external dependencies needed for testing.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | none — standard Rust test harness |
| Quick run command | `cargo test engines::de::test_de::test_de_convergence` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | DeEngine convergence | integration | `cargo test engines::de::test_de::test_de_convergence` | ✅ (add function) |
| SC-2 | ScatterEngine convergence | integration | `cargo test engines::scatter::test_scatter::test_scatter_convergence` | ✅ (add function) |
| SC-3 | CellularEngine convergence | integration | `cargo test engines::cellular::test_cellular::test_cellular_convergence` | ✅ (add function) |
| SC-4 | AlpsEngine convergence | integration | `cargo test engines::alps::test_alps::test_alps_convergence` | ✅ (add function) |
| SC-5 | CmaEngine convergence | integration | `cargo test engines::cma::test_cma::test_cma_convergence` | ✅ (add function) |
| SC-6 | CmaEngine IPOP restart convergence | integration | `cargo test engines::cma::test_cma::test_cma_ipop_convergence` | ✅ (add function) |
| SC-7 | PsoEngine convergence | integration | `cargo test engines::pso::test_pso::test_pso_convergence` | ✅ (add function) |
| SC-8 | Fixed RNG seed determinism | integration | All above tests use `rng::set_seed(Some(42))` | ✅ |
| SC-9 | Tests in correct location | structural | `ls tests/engines/*/test_*.rs` | ✅ |

### Sampling Rate

- **Per task commit:** `cargo test engines::<engine>::test_<engine>::test_<engine>_convergence`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

None — existing test infrastructure covers all phase requirements. Each engine already has a test file with `sphere` and `random_pop` helpers.

## Security Domain

> Not applicable — this phase adds tests only, no security-sensitive code.

## Sources

### Primary (HIGH confidence)
- `tests/engines/de/test_de.rs` — Existing DE convergence pattern (lines 60-71)
- `tests/engines/pso/test_pso.rs` — Existing PSO convergence pattern (lines 346-362)
- `tests/engines/cma/test_cma.rs` — Existing CMA IPOP restart pattern (lines 401-433)
- `src/rng.rs` — Deterministic RNG seeding mechanism

### Secondary (MEDIUM confidence)
- `.planning/phases/82-per-engine-convergence-integration-tests-issue-284/82-CONTEXT.md` — Locked decisions

### Tertiary (LOW confidence)
- None — all findings verified against codebase

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all components exist in codebase
- Architecture: HIGH — pattern established in existing tests
- Pitfalls: HIGH — derived from existing test patterns

**Research date:** 2026-06-22
**Valid until:** 2026-07-22 (30 days — stable testing patterns)
