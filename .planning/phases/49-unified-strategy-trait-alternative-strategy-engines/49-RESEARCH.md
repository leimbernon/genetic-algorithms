# Phase 49: Unified Strategy Trait + Alternative Strategy Engines — Research

**Researched:** 2026-05-22
**Domain:** Rust trait design, engine implementation patterns, observer wiring
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `Strategy<U>` trait exposes exactly `fn run(&mut self) -> Result<(), GaError>` and `fn best(&self) -> Option<&U>`. No builder methods on the trait — stays minimal and dyn-safe.
- **D-02:** `with_observer()` lives only on individual engine structs (concrete types), not on the trait. Observer wiring happens before boxing.
- **D-03:** The trait must be dyn-safe. Both methods are dyn-safe. No associated types, no generics on methods.
- **D-04:** Hill-climb observer hooks: `on_run_start`, `on_generation_start(iteration)`, `on_new_best(iteration, best)` (only on improvement), `on_generation_end(stats)`, `on_run_end`. GA-only hooks (`on_selection_complete`, `on_crossover_complete`, `on_mutation_complete`, `on_survivor_selection_complete`, `on_fitness_evaluation_complete`, `on_extension_triggered`, `on_stagnation`) do NOT fire.
- **D-05:** `PermutateEngine` fires same subset: `on_run_start`, `on_generation_start(candidate_index)`, `on_new_best`, `on_generation_end`, `on_run_end`. Each candidate evaluation is one "generation."
- **D-06:** Gate overflow uses `log::warn!(target = "ga_events", ...)` and returns `Ok(())`. No observer hook for gate overflow.
- **D-07:** `HillClimbEngine<U>` has one struct with `mode: HillClimbMode` enum field. `HillClimbMode::Stochastic` accepts first neighbor with higher fitness. `HillClimbMode::SteepestAscent` evaluates all, accepts global best.
- **D-08:** `neighbor_fn` stored as `Arc<dyn Fn(&U) -> Vec<U> + Send + Sync>`.
- **D-09:** `PermutateEngine<U>` accepts `Vec<U>` at build time, iterates lazily via `.iter()`.
- **D-10:** Safety gate is a configurable `usize` field on `PermutateConfiguration` (default 100,000). Evaluates up to the gate, then warns and returns.
- **D-11:** New engines land in `src/engines/hill_climb/` and `src/engines/permutate/` following `src/engines/de/` pattern. `Strategy<U>` trait lives in `src/traits/strategy.rs`, re-exported from `src/traits.rs`.
- **D-12:** `Ga<U>` implements `Strategy<U>` as an explicit impl — no changes to `Ga` internals.

### Claude's Discretion

None — all decisions were locked in discussion.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| STR-01 | `Strategy<U>` trait with `run()` / `best()` enabling `Box<dyn Strategy<U>>` runtime swapping | D-01 through D-03 + dyn-safety analysis below |
| STR-02 | `HillClimbEngine::Stochastic` — first-improving neighbor, configurable no-improvement limit, `GaObserver` hooks | D-04, D-07, D-08 + observer wiring pattern |
| STR-03 | `HillClimbEngine::SteepestAscent` — evaluate all neighbors, accept global best, `GaObserver` hooks | D-04, D-07, D-08 |
| STR-04 | `PermutateEngine` — exhaustive candidate eval up to safety gate, `GaObserver` hooks per candidate | D-05, D-06, D-09, D-10 |

</phase_requirements>

---

## Summary

Phase 49 adds a thin `Strategy<U>` trait that imposes a uniform `run()` / `best()` interface across all engines, enabling runtime algorithm swapping via `Box<dyn Strategy<U>>`. The trait is added to two new engines (`HillClimbEngine<U>` and `PermutateEngine<U>`) and retrofitted onto the existing `Ga<U>`. No existing engine internals change.

Both new engines follow the structural pattern established by `src/engines/de/` (three-file layout: `mod.rs`, `engine.rs`, `configuration.rs`) and wire `GaObserver<U>` exactly as `ga.rs` does (the `notify()` closure pattern). The critical constraint is WASM compatibility: no `Instant::now()` calls, no `par_iter()` in new engines. Both engines iterate sequentially over their work units — hill-climb neighbor lists and permutation candidate slices — so no WASM-specific branching is needed in their hot paths.

The `Ga<U>` impl of `Strategy<U>` is a three-line wrapper: `run()` calls `self.run()` internally (discarding the `&Population<U>` return) and `best()` reads `self.population.best_chromosome_is_set` then returns `Some(&self.population.best_chromosome)` or `None`.

**Primary recommendation:** Follow `de/` layout exactly. Copy the `notify()` helper verbatim from `ga.rs`. Gate any `Instant` usage behind `#[cfg(not(target_arch = "wasm32"))]`. All new test files go under `tests/engines/hill_climb/` and `tests/engines/permutate/`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| `Strategy<U>` trait definition | `src/traits/strategy.rs` | re-export via `src/traits.rs` + `src/lib.rs` | Traits module owns all public trait contracts |
| `Ga<U>` Strategy impl | `src/engines/ga.rs` | — | Impl block on the existing struct; no structural change |
| `HillClimbEngine<U>` struct + loop | `src/engines/hill_climb/engine.rs` | — | Mirror of `de/engine.rs` |
| `HillClimbConfiguration` | `src/engines/hill_climb/configuration.rs` | — | Mirror of `de/configuration.rs` |
| `HillClimbMode` enum | `src/engines/hill_climb/configuration.rs` | — | Co-located with config; single-variant branch in engine loop |
| `PermutateEngine<U>` struct + loop | `src/engines/permutate/engine.rs` | — | Simpler than `de/` — no adaptive state |
| `PermutateConfiguration` | `src/engines/permutate/configuration.rs` | — | Safety gate + problem_solving |
| Observer dispatch | Engine struct `notify()` helper | `GaObserver<U>` trait | Zero-overhead when None |
| Public API surface | `src/lib.rs` | `src/traits.rs` | All new types re-exported at crate root |

---

## Standard Stack

No new external dependencies are introduced by this phase. [VERIFIED: CONTEXT.md / STATE.md]

All packages already in `Cargo.toml` are used as-is:

| Item | Already Present | Purpose in Phase 49 |
|------|----------------|---------------------|
| `rand` / `rand_chacha` | Yes | `make_rng()` for stochastic neighbor shuffling |
| `log` | Yes | `log::warn!(target = "ga_events", ...)` for gate overflow |
| `std::sync::Arc` | Yes | `neighbor_fn` + `observer` storage |

---

## Package Legitimacy Audit

No new packages are installed in this phase. This section is N/A.

---

## Architecture Patterns

### System Architecture Diagram

```
User code
   │
   ├── Box<dyn Strategy<U>>  ←── runtime dispatch
   │         │
   │    ┌────┴──────────────────────────────┐
   │    │                                   │
   │  Ga<U>::run()            HillClimbEngine<U>::run()
   │  Ga<U>::best()           HillClimbEngine<U>::best()
   │    │                           │
   │  existing GA loop         neighbor_fn(&current) → Vec<U>
   │    │                           │
   │    │                    mode branch:
   │    │                    ├── Stochastic: first-improving
   │    │                    └── SteepestAscent: global-best
   │    │                           │
   │    │                     observer.notify(on_generation_end)
   │    │
   │  PermutateEngine<U>::run()
   │  PermutateEngine<U>::best()
   │    │
   │  candidates.iter() (lazy, up to safety gate)
   │    │
   │  track running best
   │    │
   │  observer.notify(on_generation_end per candidate)
   │
GaObserver<U> hooks (subset: run_start, gen_start, new_best, gen_end, run_end)
```

### Recommended Project Structure

```
src/engines/
├── de/                         # existing
├── hill_climb/
│   ├── mod.rs                  # pub use re-exports
│   ├── engine.rs               # HillClimbEngine<U> struct + impl
│   └── configuration.rs        # HillClimbConfiguration, HillClimbMode
└── permutate/
    ├── mod.rs                  # pub use re-exports
    ├── engine.rs               # PermutateEngine<U> struct + impl
    └── configuration.rs        # PermutateConfiguration

src/traits/
├── strategy.rs                 # Strategy<U> trait (NEW)
└── (existing files unchanged)

tests/engines/
├── hill_climb/
│   └── test_hill_climb.rs
└── permutate/
    └── test_permutate.rs
```

### Pattern 1: Strategy Trait Definition

**What:** Minimal dyn-safe trait with exactly two methods.
**When to use:** Define in `src/traits/strategy.rs`.

```rust
// Source: CONTEXT.md D-01/D-03 + Rust reference on dyn-safety
use crate::error::GaError;
use crate::traits::ChromosomeT;

/// Common interface over all search strategy engines.
///
/// Enables runtime algorithm swapping via `Box<dyn Strategy<U>>`.
/// All implementations must wire `GaObserver` hooks before boxing.
pub trait Strategy<U: ChromosomeT> {
    /// Execute the search loop. Mutates internal state.
    fn run(&mut self) -> Result<(), GaError>;

    /// Returns the best candidate found, or `None` if `run()` has not been called.
    fn best(&self) -> Option<&U>;
}
```

**Dyn-safety analysis:** Both methods are dyn-safe. [VERIFIED: Rust reference] `run(&mut self) -> Result<(), GaError>` — no generics, no `Self` in return position. `best(&self) -> Option<&U>` — `U` is the trait's type parameter, bound at `dyn Strategy<U>` site, not a method-level generic. This is the same dyn-safe pattern used by `GaObserver<U>`.

### Pattern 2: Ga<U> impl of Strategy<U>

**What:** Thin three-line wrapper on the existing `Ga<U>`. No changes to `Ga` internals.
**When to use:** Add an `impl Strategy<U> for Ga<U>` block in `src/engines/ga.rs`.

Key facts from code inspection:
- `Ga<U>::run(&mut self)` returns `Result<&Population<U>, GaError>` — the `Strategy` impl discards the reference.
- `population.best_chromosome` is `pub` on `Population<U>`.
- `population.best_chromosome_is_set` is `pub(crate)` — accessible inside the crate.

```rust
// Source: inspected ga.rs lines 1263, population.rs lines 46-47
impl<U> Strategy<U> for Ga<U>
where
    U: LinearChromosome + Send + Sync + 'static + Clone,
{
    fn run(&mut self) -> Result<(), GaError> {
        self.run().map(|_| ())
    }

    fn best(&self) -> Option<&U> {
        if self.population.best_chromosome_is_set {
            Some(&self.population.best_chromosome)
        } else {
            None
        }
    }
}
```

**Bound note:** `Ga<U>::run_with_callback` imposes `U: LinearChromosome + Send + Sync + 'static + Clone + MaybeDeserialize`. The `impl Strategy<U> for Ga<U>` block must carry at minimum `U: LinearChromosome` (the struct-level bound) plus whatever `run_with_callback` requires. Check if `MaybeDeserialize` needs to appear on the impl — if so, import it. [ASSUMED: the exact bound combination compiles without conflict; verify with `cargo check`.]

### Pattern 3: Observer Wiring (copy from ga.rs verbatim)

**What:** `observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>` field + `notify()` helper + `with_observer()` builder. This is the ONLY approved observer pattern — zero overhead when None.

```rust
// Source: ga.rs lines 278, 859-870
/// Optional structured lifecycle observer.
observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,

/// Attaches a lifecycle observer.
pub fn with_observer(mut self, observer: Arc<dyn GaObserver<U> + Send + Sync>) -> Self {
    self.observer = Some(observer);
    self
}

/// Dispatches an observer hook. No-op when observer is None.
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}
```

### Pattern 4: HillClimbEngine Run Loop

**What:** Single struct, two modes, shared loop body with a single branch at neighbor selection.

```rust
// Source: CONTEXT.md D-07/D-08, derived from de/engine.rs structure
pub struct HillClimbEngine<U: LinearChromosome> {
    config: HillClimbConfiguration,
    neighbor_fn: Arc<dyn Fn(&U) -> Vec<U> + Send + Sync>,
    current: Option<U>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: LinearChromosome + Clone> HillClimbEngine<U> {
    pub fn run(&mut self) -> Result<(), GaError> {
        // Initialize: current must be set before run() or return ConfigurationError
        let mut current = self.current.take()
            .ok_or_else(|| GaError::ConfigurationError("no initial solution".into()))?;

        self.notify(|obs| obs.on_run_start());

        let mut no_improvement_count = 0usize;
        let mut iteration = 0usize;
        let mut all_stats: Vec<GenerationStats> = Vec::new();

        loop {
            self.notify(|obs| obs.on_generation_start(iteration));

            let neighbors = (self.neighbor_fn)(&current);
            let improved = match self.config.mode {
                HillClimbMode::Stochastic => {
                    // First-improving: accept first neighbor with higher fitness
                    neighbors.into_iter().find(|n| self.is_better(n, &current))
                }
                HillClimbMode::SteepestAscent => {
                    // Evaluate all, accept global best only if better than current
                    neighbors.into_iter()
                        .filter(|n| self.is_better(n, &current))
                        .max_by(|a, b| self.cmp_fitness(a, b))
                }
            };

            if let Some(next) = improved {
                self.notify(|obs| obs.on_new_best(iteration, next.clone()));
                current = next;
                no_improvement_count = 0;
            } else {
                no_improvement_count += 1;
            }

            let stats = GenerationStats {
                generation: iteration,
                best_fitness: current.fitness(),
                worst_fitness: current.fitness(),
                avg_fitness: current.fitness(),
                fitness_std_dev: 0.0,
                population_size: 1,
                diversity: 0.0,
                dynamic_mutation_probability: None,
            };
            all_stats.push(stats.clone());
            self.notify(|obs| obs.on_generation_end(&stats));

            iteration += 1;

            // Stopping: SteepestAscent stops when no improvement found (no_improvement_count >= 1)
            // Stochastic stops when no_improvement_count >= config.no_improvement_limit
            if self.should_stop(no_improvement_count) {
                break;
            }
        }

        self.current = Some(current);
        self.notify(|obs| obs.on_run_end(TerminationCause::GenerationLimitReached, &all_stats));
        Ok(())
    }
}
```

**WASM note:** `neighbors.into_iter()` / `.iter()` only — never `.par_iter()`. No `Instant::now()`. [VERIFIED: CONTEXT.md]

### Pattern 5: PermutateEngine Run Loop

```rust
// Source: CONTEXT.md D-09/D-10, derived from de/engine.rs structure
pub struct PermutateEngine<U: ChromosomeT> {
    config: PermutateConfiguration,
    candidates: Vec<U>,
    best: Option<U>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

impl<U: ChromosomeT + Clone> PermutateEngine<U> {
    pub fn run(&mut self) -> Result<(), GaError> {
        self.notify(|obs| obs.on_run_start());

        let gate = self.config.safety_gate;
        let mut best: Option<U> = None;
        let mut all_stats: Vec<GenerationStats> = Vec::new();

        for (idx, candidate) in self.candidates.iter().enumerate() {
            if idx >= gate {
                log::warn!(
                    target = "ga_events",
                    "PermutateEngine: safety gate of {} reached, stopping evaluation",
                    gate
                );
                break;
            }

            self.notify(|obs| obs.on_generation_start(idx));

            let is_new_best = match &best {
                None => true,
                Some(b) => self.is_better(candidate.fitness(), b.fitness()),
            };

            if is_new_best {
                self.notify(|obs| obs.on_new_best(idx, candidate.clone()));
                best = Some(candidate.clone());
            }

            let stats = GenerationStats {
                generation: idx,
                best_fitness: best.as_ref().map(|b| b.fitness()).unwrap_or(f64::NAN),
                worst_fitness: candidate.fitness(),
                avg_fitness: candidate.fitness(),
                fitness_std_dev: 0.0,
                population_size: 1,
                diversity: 0.0,
                dynamic_mutation_probability: None,
            };
            all_stats.push(stats.clone());
            self.notify(|obs| obs.on_generation_end(&stats));
        }

        self.best = best;
        self.notify(|obs| obs.on_run_end(TerminationCause::GenerationLimitReached, &all_stats));
        Ok(())
    }
}
```

**Design note:** `PermutateEngine` iterates over a pre-built `Vec<U>`. Candidates must have fitness pre-evaluated by the caller before being passed in, OR `PermutateConfiguration` carries a fitness function to evaluate them inside the loop. The CONTEXT.md does not specify this — **the planner must resolve this: does `PermutateEngine` receive pre-evaluated candidates or does it carry a fitness function?** [ASSUMED: candidates are pre-evaluated by caller, consistent with D-09 "user materializes the candidate list". Needs confirmation during planning.]

### Pattern 6: Module Re-exports (mirror de/mod.rs exactly)

```rust
// src/engines/hill_climb/mod.rs — Source: de/mod.rs pattern
pub mod configuration;
pub mod engine;

pub use configuration::{HillClimbConfiguration, HillClimbMode};
pub use engine::HillClimbEngine;
```

```rust
// src/engines/permutate/mod.rs
pub mod configuration;
pub mod engine;

pub use configuration::PermutateConfiguration;
pub use engine::PermutateEngine;
```

### Pattern 7: lib.rs Registration

```rust
// Add to src/lib.rs — mirror of existing de/scatter/alps entries
#[path = "engines/hill_climb/mod.rs"]
pub mod hill_climb;

#[path = "engines/permutate/mod.rs"]
pub mod permutate;

// Flat re-exports at crate root
pub use hill_climb::{HillClimbEngine, HillClimbConfiguration, HillClimbMode};
pub use permutate::{PermutateEngine, PermutateConfiguration};
pub use traits::Strategy;
```

### Pattern 8: Strategy re-export in traits.rs

```rust
// Add to src/traits.rs
pub mod strategy;
pub use strategy::Strategy;
```

### Anti-Patterns to Avoid

- **Adding builder methods to `Strategy<U>`:** Destroys dyn-safety if any builder returns `Self`. Locked decision D-01.
- **Calling `par_iter()` in new engines:** WASM mandatory constraint. Use `.iter()` only.
- **Calling `Instant::now()` unconditionally:** Gate with `#[cfg(not(target_arch = "wasm32"))]` if timing is ever needed.
- **Putting tests inline with implementation:** Project rule — all tests in `tests/` directory.
- **Firing GA-only hooks in non-GA engines:** `on_selection_complete`, `on_crossover_complete`, etc. must NOT fire in hill-climb or permutation loops (D-04).
- **Using `Box<dyn Fn>` instead of `Arc<dyn Fn>` for neighbor_fn:** `Arc` is required for Clone-ability and consistency with observer storage pattern (D-08).
- **Modifying `Ga<U>` internals:** `impl Strategy<U> for Ga<U>` is purely additive — no field changes, no behavior changes to existing methods (D-12).

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random neighbor selection order | Custom shuffle | `make_rng()` + iterator ordering | Existing RNG already handles seeding/reproducibility |
| Observer dispatch boilerplate | Match arms | `notify()` closure pattern from ga.rs | Three lines, zero overhead when None |
| Fitness comparison (min/max) | Custom logic | `ProblemSolving` enum + `is_better()` helper | Already established in `de/engine.rs`; copy the pattern |
| Test chromosome types | New struct | `RangeChromosome<f64>` / `Binary` from `tests/engines/de/test_de.rs` | Existing test infra is ready |

**Key insight:** The `is_better()` helper in `de/engine.rs` (lines 237-249) is the canonical `ProblemSolving`-aware comparator. Copy this pattern into both new engines rather than writing ad-hoc comparisons.

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is a greenfield feature addition, not a rename/refactor/migration phase.

---

## Environment Availability

Step 2.6: No external tool dependencies beyond the existing Rust toolchain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | build + test | Yes | (project's Rust toolchain) | — |
| `cargo check --target wasm32-unknown-unknown` | WASM gate | Yes | requires wasm32 target installed | install: `rustup target add wasm32-unknown-unknown` |

**Verify wasm32 target is installed before implementation:**
```bash
rustup target list --installed | grep wasm32-unknown-unknown
```

---

## Common Pitfalls

### Pitfall 1: Ga<U> impl requires extra bounds beyond struct definition

**What goes wrong:** `Ga<U>` struct is bounded by `U: LinearChromosome`. Its `run_with_callback` method adds `U: ... + Clone + MaybeDeserialize`. If the `Strategy<U>` impl for `Ga<U>` calls `run_with_callback`, the impl block must carry those extra bounds or the compiler rejects it.

**Why it happens:** Method-level bounds are not inherited by impl blocks in Rust — each impl block must declare everything it needs.

**How to avoid:** Write the impl block with the full bound set that `run_with_callback` requires. Alternatively, expose a separate internal method that does not require `MaybeDeserialize` if that bound is inconvenient. [ASSUMED: `MaybeDeserialize` is a serde feature gate — verify if it can be excluded when `serde` feature is off.]

**Warning signs:** `error[E0277]: the trait bound ... is not satisfied` when compiling `impl Strategy<U> for Ga<U>`.

### Pitfall 2: `best()` before `run()` returns stale data

**What goes wrong:** `Ga<U>::best()` reads `population.best_chromosome_is_set`. If a user calls `best()` before `run()`, the flag is `false` and `best()` returns `None`. This is correct behavior — but the `HillClimbEngine` and `PermutateEngine` must handle the same pre-run state.

**How to avoid:** Both new engines store `current: Option<U>` / `best: Option<U>` as `None` by default and return `None` from `best()` before `run()` is called. This is consistent and correct.

### Pitfall 3: SteepestAscent stopping condition

**What goes wrong:** `SteepestAscent` stops when the best neighbor is not better than current. If `neighbors` is empty (user's `neighbor_fn` returned `Vec::new()`), the engine must not panic and should treat this as "no improvement."

**Why it happens:** `.max_by()` on an empty iterator returns `None`, which is then treated as "no improvement" — correct behavior if the code checks `Option` properly.

**How to avoid:** The `improved.is_none()` path increments `no_improvement_count` regardless of whether the neighbor list was empty or all-worse. No special case needed, but add a test covering an empty neighbor list.

### Pitfall 4: PermutateEngine candidate fitness not pre-evaluated

**What goes wrong:** If candidates in `Vec<U>` have `NaN` fitness (default state), the `is_better()` comparisons all return `false` (NaN comparisons are always false in IEEE 754), and `best` remains `None` after the run.

**Why it happens:** `ChromosomeT::new()` sets fitness to `NaN` by convention (see `LinearChromosome::reset()`). A caller who builds chromosomes without calling `calculate_fitness()` will silently produce wrong results.

**How to avoid:** Document clearly in `PermutateEngine`'s doc comment that candidates must have fitness set. Optionally, `PermutateConfiguration` can carry an optional fitness function (like `DeConfiguration` carries one) and the engine evaluates if present. **This design choice must be made during planning** — current CONTEXT.md is silent on whether `PermutateEngine` evaluates fitness internally.

### Pitfall 5: `on_run_end` requires `TerminationCause`

**What goes wrong:** `GaObserver::on_run_end` takes `(cause: TerminationCause, all_stats: &[GenerationStats])`. New engines must decide which `TerminationCause` variant to pass. There is no "NoImprovementLimit" variant in the current `TerminationCause` enum.

**Why it happens:** `TerminationCause` was designed for `Ga<U>`. Hill-climb stopping is semantically different (no-improvement threshold or SteepestAscent convergence). Permutation engine stopping is "all candidates evaluated" or "gate reached."

**How to avoid:** Use `TerminationCause::GenerationLimitReached` as the closest semantic match for both new engines. If a new variant (e.g., `NoImprovementLimitReached`) is desired, add it to `GaError` — but this is a non-breaking addition only if `TerminationCause` does not appear in match arms in user code. **The planner should decide: reuse `GenerationLimitReached` or add variants.** [ASSUMED: reuse existing variants to avoid any match-exhaustion breakage for users; this is the least-risk path.]

### Pitfall 6: `mod.rs` required for nested modules with submodules

**What goes wrong:** If `src/engines/hill_climb/` needs any nested submodule in the future, the directory form (`mod.rs`) is required, not the single-file form. Since all existing engine directories use the `mod.rs` pattern, follow the same.

**How to avoid:** Always use `src/engines/hill_climb/mod.rs`, not `src/engines/hill_climb.rs`. This is confirmed by the STATE.md v2.3.0 decision.

---

## Code Examples

### Complete HillClimbConfiguration

```rust
// Source: CONTEXT.md D-07/D-08/D-10 + de/configuration.rs pattern
use crate::configuration::ProblemSolving;
use std::sync::Arc;
use crate::traits::LinearChromosome;

#[derive(Debug, Clone, PartialEq)]
pub enum HillClimbMode {
    /// Accept the first neighbor with higher fitness (early exit).
    Stochastic,
    /// Evaluate all neighbors; accept the global best if better than current.
    SteepestAscent,
}

#[derive(Clone)]
pub struct HillClimbConfiguration {
    /// Which mode to use for neighbor selection.
    pub mode: HillClimbMode,
    /// Stochastic mode: stop after this many consecutive iterations with no improvement.
    pub no_improvement_limit: usize,
    /// Whether to minimize or maximize fitness.
    pub problem_solving: ProblemSolving,
}

impl Default for HillClimbConfiguration {
    fn default() -> Self {
        Self {
            mode: HillClimbMode::Stochastic,
            no_improvement_limit: 100,
            problem_solving: ProblemSolving::Minimization,
        }
    }
}

impl HillClimbConfiguration {
    pub fn with_mode(mut self, mode: HillClimbMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn with_no_improvement_limit(mut self, limit: usize) -> Self {
        self.no_improvement_limit = limit;
        self
    }
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self {
        self.problem_solving = ps;
        self
    }
}
```

### Complete PermutateConfiguration

```rust
// Source: CONTEXT.md D-10 + de/configuration.rs pattern
use crate::configuration::ProblemSolving;

#[derive(Debug, Clone)]
pub struct PermutateConfiguration {
    /// Maximum number of candidates to evaluate before stopping.
    pub safety_gate: usize,
    /// Whether to minimize or maximize fitness.
    pub problem_solving: ProblemSolving,
}

impl Default for PermutateConfiguration {
    fn default() -> Self {
        Self {
            safety_gate: 100_000,
            problem_solving: ProblemSolving::Minimization,
        }
    }
}

impl PermutateConfiguration {
    pub fn with_safety_gate(mut self, gate: usize) -> Self {
        self.safety_gate = gate;
        self
    }
    pub fn with_problem_solving(mut self, ps: ProblemSolving) -> Self {
        self.problem_solving = ps;
        self
    }
}
```

### `is_better()` helper (copy from de/engine.rs)

```rust
// Source: de/engine.rs lines 237-249
fn is_better(&self, candidate: f64, current: f64) -> bool {
    match self.config.problem_solving {
        ProblemSolving::Minimization => candidate < current,
        ProblemSolving::Maximization => candidate > current,
        ProblemSolving::FixedFitness => {
            if let Some(t) = self.config.fitness_target {
                (candidate - t).abs() < (current - t).abs()
            } else {
                candidate < current
            }
        }
    }
}
```

**Note:** `HillClimbConfiguration` should include `fitness_target: Option<f64>` if `is_better` with `FixedFitness` is needed. `PermutateConfiguration` should also include it. [ASSUMED: add `fitness_target: Option<f64>` to both configs for consistency with `DeConfiguration`.]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Reporter<U>` trait for hooks | `GaObserver<U>` with 12 hooks | v3.0.0 (Phase 47) | New engines use `GaObserver` only — `Reporter` is removed |
| Engine-specific observer sub-traits added to `AllObserver` | Separate per-engine observer sub-traits (not in `AllObserver`) | v2.4.0+ | `HillClimbEngine` uses `GaObserver` directly — no new sub-trait needed |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `impl Strategy<U> for Ga<U>` can use `GenerationLimitReached` for `on_run_end` without adding new `TerminationCause` variants | Pitfall 5 / Pattern 2 | Low — adding a variant is non-breaking but requires a decision; reusing existing is safe |
| A2 | `PermutateEngine` candidates are pre-evaluated (fitness set by caller before passing `Vec<U>`) | Pattern 5 / Pitfall 4 | MEDIUM — if wrong, `best()` silently returns `None` or the wrong candidate; planner must resolve |
| A3 | `MaybeDeserialize` bound on `Ga::run_with_callback` is feature-gated via `serde` and does not appear on non-serde builds, allowing a simpler `impl Strategy<U> for Ga<U>` bound | Pitfall 1 / Pattern 2 | MEDIUM — if wrong, the Strategy impl requires importing `MaybeDeserialize` or using a cfg gate |
| A4 | Both new configs include `fitness_target: Option<f64>` for `FixedFitness` parity with `DeConfiguration` | Code Examples | Low — omitting it is also valid; planner can decide |

**If this table is empty:** N/A — assumptions are documented above.

---

## Open Questions

1. **Does `PermutateEngine` evaluate fitness internally or require pre-evaluated candidates?**
   - What we know: D-09 says "user materializes the candidate list"; the context is silent on fitness evaluation.
   - What's unclear: If candidates are built via chromosome constructors (default NaN fitness), the engine silently produces wrong output.
   - Recommendation: Add an optional `fitness_fn: Option<Arc<FitnessFn<U::Gene>>>` to `PermutateConfiguration` (consistent with `DeConfiguration`). If present, evaluate each candidate inside the loop; if absent, trust the caller's pre-evaluated fitness. Document this in `PermutateEngine`'s doc comment.

2. **Should `TerminationCause` gain new variants for hill-climb/permutation termination reasons?**
   - What we know: Existing variants are `GenerationLimitReached`, `FitnessTargetReached`, `StagnationReached`, `ConvergenceReached`, `TimeLimitReached`, `CallbackRequested`, `NotTerminated`.
   - What's unclear: Whether a `NoImprovementLimitReached` variant adds user value or is premature.
   - Recommendation: Reuse `GenerationLimitReached` for both new engines in v3.0.0. Adding variants in a future phase is non-breaking.

3. **Should `HillClimbEngine` require an initial solution at build time or via a separate method?**
   - What we know: D-07/D-08 define `HillClimbEngine<U>` with `mode`, `neighbor_fn`, but do not specify how the initial solution is provided.
   - Recommendation: Accept the initial solution as a required parameter of `HillClimbEngine::new()` — mirrors how `PermutateEngine::new()` takes `candidates: Vec<U>`.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`cargo test`) |
| Config file | `Cargo.toml` (workspace-level) |
| Quick run command | `cargo test --test test_hill_climb -- --nocapture` |
| Full suite command | `cargo test && cargo test --features serde && cargo clippy` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STR-01 | `Box<dyn Strategy<U>>` compiles and dispatches to `Ga`, `HillClimbEngine`, `PermutateEngine` | integration | `cargo test --test test_strategy_trait` | No — Wave 0 |
| STR-01 | `Strategy<U>` is dyn-safe (compilation test) | compile | `cargo build` | No — Wave 0 |
| STR-02 | `HillClimbEngine::Stochastic` finds improving solution on a simple landscape | integration | `cargo test --test test_hill_climb stochastic` | No — Wave 0 |
| STR-02 | Stochastic stops after `no_improvement_limit` consecutive non-improving iterations | unit | `cargo test --test test_hill_climb stochastic_stops` | No — Wave 0 |
| STR-02 | Observer hooks fire in correct order during stochastic run | integration | `cargo test --test test_hill_climb observer_hooks` | No — Wave 0 |
| STR-03 | `SteepestAscent` converges on a simple unimodal landscape | integration | `cargo test --test test_hill_climb steepest_ascent` | No — Wave 0 |
| STR-03 | `SteepestAscent` stops when best neighbor is not better than current | unit | `cargo test --test test_hill_climb steepest_stops` | No — Wave 0 |
| STR-04 | `PermutateEngine` evaluates all candidates and returns the best | integration | `cargo test --test test_permutate basic` | No — Wave 0 |
| STR-04 | Safety gate triggers `log::warn` and stops early when exceeded | unit | `cargo test --test test_permutate gate_overflow` | No — Wave 0 |
| STR-04 | Observer hooks fire per candidate | integration | `cargo test --test test_permutate observer_hooks` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test && cargo clippy`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy && cargo check --target wasm32-unknown-unknown`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- `tests/engines/hill_climb/test_hill_climb.rs` — covers STR-02, STR-03
- `tests/engines/permutate/test_permutate.rs` — covers STR-04
- `tests/engines/test_strategy_trait.rs` — covers STR-01 (dyn dispatch + compilation)

*(If any of the three engine directories are created without a `mod.rs` test module entry, the test runner will not pick up the files automatically — add mod declarations to `tests/engines/` integration test entry points.)*

---

## Security Domain

This phase introduces no authentication, session management, input validation from external sources, cryptography, or network I/O. The only user-provided input is:
- `neighbor_fn: Fn(&U) -> Vec<U>` — trusted user closure, no sanitization needed.
- `candidates: Vec<U>` — owned by the caller, no injection surface.

Security domain: NOT APPLICABLE.

---

## Project Constraints (from CLAUDE.md)

These directives are mandatory and must be honored by every task in the plan:

| Directive | Impact on Phase 49 |
|-----------|-------------------|
| WASM mandatory — no `Instant::now()` unconditionally | New engines must not call `Instant::now()`. Gate with `#[cfg(not(target_arch = "wasm32"))]` if timing is ever added. |
| WASM mandatory — no `par_iter()` unconditionally | `HillClimbEngine` iterates neighbor lists with `.iter()`. `PermutateEngine` iterates candidates with `.iter()`. |
| Tests in `tests/` directory only — never inline | All tests for new engines go under `tests/engines/hill_climb/` and `tests/engines/permutate/`. |
| No breaking changes by default | `impl Strategy<U> for Ga<U>` adds a new impl block — purely additive, non-breaking. |
| `log!` macros use `target = "ga_events"` | Gate overflow warn: `log::warn!(target = "ga_events", ...)`. |
| Branch from milestone branch, not main | Feature work goes on `feat/<issue>-<description>` from `milestone/v3.0.0`. |
| `cargo check --target wasm32-unknown-unknown` before considering a feature complete | Run at end of every wave. |
| Observer hooks must be preserved | New engines wire the approved subset of `GaObserver<U>` hooks (D-04/D-05). |

---

## Sources

### Primary (HIGH confidence)

- `src/engines/ga.rs` lines 278, 859-870 — observer field, `with_observer()`, `notify()` pattern [inspected directly]
- `src/engines/ga.rs` lines 1263-1269 — `Ga::run()` exact signature (`Result<&Population<U>, GaError>`) [inspected directly]
- `src/engines/de/engine.rs` — `find_best()`, `is_better()`, `reached_target()` helpers; struct layout [inspected directly]
- `src/engines/de/configuration.rs` — builder pattern reference [inspected directly]
- `src/engines/de/mod.rs` — module re-export pattern [inspected directly]
- `src/observe/observer/mod.rs` — `GaObserver<U>` all 12 hooks, `on_run_end` signature [inspected directly]
- `src/traits/strategy.rs` — does not exist yet; defined by this phase
- `src/error.rs` — `GaError` enum variants (all reusable; `ConfigurationError(String)` covers missing initial solution) [inspected directly]
- `src/stats.rs` — `GenerationStats` struct (all fields; `from_fitness_values` constructor) [inspected directly]
- `src/population.rs` lines 46-47 — `best_chromosome: U` (pub), `best_chromosome_is_set: bool` (pub(crate)) [inspected directly]
- `src/traits/linear_chromosome.rs` — `LinearChromosome` supertrait, required by new engines [inspected directly]
- `src/traits.rs` — current re-exports, insertion point for `Strategy<U>` [inspected directly]
- `src/lib.rs` — `#[path]` pattern for engine registration, insertion points [inspected directly]
- `.planning/phases/49-unified-strategy-trait-alternative-strategy-engines/49-CONTEXT.md` — all locked decisions [inspected directly]

### Secondary (MEDIUM confidence)

- `tests/engines/de/test_de.rs` — test structure reference for new engine tests [inspected directly]

### Tertiary (LOW confidence)

None — all findings verified against source code.

---

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — no new dependencies; all existing crates verified in source
- Architecture: HIGH — all patterns read directly from source files, not inferred
- Pitfalls: HIGH (Pitfall 1-3, 6) / MEDIUM (Pitfall 4-5) — NaN fitness and TerminationCause choices are design questions, not bugs

**Research date:** 2026-05-22
**Valid until:** 2026-06-22 (stable domain — 30-day window)
