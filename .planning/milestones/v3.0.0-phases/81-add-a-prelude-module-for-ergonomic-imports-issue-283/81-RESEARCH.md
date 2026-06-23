# Phase 81: Add a Prelude Module for Ergonomic Imports (Issue #283) - Research

**Researched:** 2026-06-22
**Domain:** Rust prelude pattern, library ergonomics, public API surface
**Confidence:** HIGH

## Summary

Phase 81 adds `src/prelude.rs` to the `genetic_algorithms` crate so users can write `use genetic_algorithms::prelude::*;` instead of 8–11 separate import lines. This is a purely additive change — no existing public API changes. The prelude is a single file of `pub use` re-exports that mirrors what every user needs to set up a GA: engine entry points, configuration traits, operator enums, error types, and core traits.

The primary risk is **name collisions** when glob-imported. All engine types, config structs, operator enums, and trait names are already unique across the crate — no two public types share the same identifier. The prelude file is purely mechanical: compose re-exports from existing paths, add `pub mod prelude;` to `lib.rs`, update one example, and document.

**Primary recommendation:** Create `src/prelude.rs` as a flat list of `pub use crate::...` re-exports grouped by category (engines, config traits, operator enums, core types, observers). Mirror feature-gated items with the same `#[cfg]` gates already used in `lib.rs`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Prelude re-exports | `src/prelude.rs` | `src/lib.rs` (module declaration) | Prelude is a convenience layer over the existing public API |
| Module declaration | `src/lib.rs` | — | Must add `pub mod prelude;` alongside existing module declarations |
| Example showcase | `examples/rastrigin.rs` | — | D-06: replace 11 imports with prelude glob |
| Documentation | `README.md`, `docs/getting-started.md` | rustdoc on prelude module | D-07: document prelude in both locations |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| No new dependencies | — | Prelude is pure `pub use` re-exports | Zero dependency overhead; standard Rust pattern |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None | — | — | — |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Separate `prelude.rs` file | Inline `pub mod prelude { ... }` in `lib.rs` | Separate file is cleaner for a module with 50+ re-exports; matches crate convention |

## Package Legitimacy Audit

> No new external packages are installed in this phase. The prelude is purely internal re-exports.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| (none) | — | — | — | — | — | — |

**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

## Architecture Patterns

### System Architecture Diagram

```
User code
  │
  ▼
use genetic_algorithms::prelude::*;   ← glob import
  │
  ▼
src/prelude.rs                         ← flat pub use re-exports
  │
  ├──► crate::ga::Ga                  (engine)
  ├──► crate::de::DeEngine            (engine)
  ├──► crate::cma::CmaEngine          (engine)
  ├──► ... (13 engines total)
  ├──► crate::traits::ConfigurationT  (trait)
  ├──► crate::traits::ChromosomeT     (trait)
  ├──► crate::traits::GeneT           (trait)
  ├──► crate::traits::LinearChromosome (trait)
  ├──► crate::operations::Selection   (enum)
  ├──► crate::operations::Crossover   (enum)
  ├──► crate::operations::Mutation    (enum)
  ├──► crate::operations::Survivor    (enum)
  ├──► crate::configuration::ProblemSolving (enum)
  ├──► crate::chromosomes::ChromosomeLength (enum)
  ├──► crate::error::GaError          (enum)
  ├──► crate::observer::GaObserver    (trait)
  ├──► crate::observer::NoopObserver  (struct)
  └──► #[cfg(feature)] conditional observers
```

### Recommended Project Structure

```
src/
├── lib.rs              # Add: pub mod prelude;
├── prelude.rs          # NEW: flat pub use re-exports
└── ... (unchanged)
```

### Pattern 1: Rust Prelude Convention

**What:** A `prelude` module re-exports the most commonly needed items from a crate so users can write a single glob import.

**When to use:** When a crate has 5+ types that nearly every user needs. Standard Rust convention (`std::prelude`, `tokio::prelude`, `serde::prelude`, `bevy::prelude`).

**Example:**
```rust
// Source: https://doc.rust-lang.org/std/prelude/index.html
// std::prelude re-exports Option, Result, Vec, String, etc.

// genetic_algorithms prelude follows the same pattern:
// src/prelude.rs
pub use crate::ga::Ga;
pub use crate::error::GaError;
pub use crate::traits::{ChromosomeT, GeneT, LinearChromosome, ConfigurationT};
// ... etc
```

### Anti-Patterns to Avoid

- **Re-exporting concrete chromosome/genotype types:** `Binary`, `Range<T>`, `ListChromosome<T>` are problem-specific — users should choose explicitly. Including them in a glob import risks confusion.
- **Re-exporting functions (e.g., `range_random_initialization`):** Initializer functions are varied and problem-specific; they don't belong in a convenience glob.
- **Re-exporting observer sub-traits (e.g., `Nsga2Observer`, `CmaObserver`):** Advanced/optional; users who need them already know the import path. Only `GaObserver` + `NoopObserver` are general-purpose (D-04).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Prelude module | Custom macro or build script | Plain `pub use` re-exports | Standard Rust pattern; zero runtime cost; compiler-resolved |

**Key insight:** A prelude is purely a developer-ergonomics module. It has zero runtime cost (compiler resolves imports at compile time) and zero binary size impact (re-exports are aliases, not copies).

## Common Pitfalls

### Pitfall 1: Name Collisions in Glob Import

**What goes wrong:** If two re-exported items share the same name, `use prelude::*` produces a compile error.

**Why it happens:** Rust does not allow two items with the same identifier in scope from a glob import.

**How to avoid:** Audit all re-exported names for uniqueness. In this crate, all engine types, config structs, operator enums, and trait names are already unique. The only shared name is `Binary` (genotype vs chromosome), but neither is in the prelude.

**Warning signs:** `cargo check` fails with "the name `X` is defined multiple times" after adding the prelude.

**Verification:** After creating `prelude.rs`, compile a test file that does `use genetic_algorithms::prelude::*;` and nothing else — must succeed with zero errors.

### Pitfall 2: Feature-Gated Items Missing `#[cfg]`

**What goes wrong:** `LogObserver` is only available when `feature = "logging"` is enabled. If re-exported unconditionally, `cargo check --no-default-features` fails.

**Why it happens:** Forgetting to mirror the `#[cfg]` gate from `lib.rs`.

**How to avoid:** Copy the exact `#[cfg(feature = "...")]` annotations from `lib.rs` lines 420–432.

**Warning signs:** `cargo check --no-default-features` fails with "could not find `LogObserver`".

### Pitfall 3: Stale Prelude After New Engine Addition

**What goes wrong:** A future phase adds a new engine but forgets to add it to `prelude.rs`.

**Why it happens:** No automated check that the prelude is a superset of the roadmap-listed items.

**How to avoid:** Add a doc-comment in `prelude.rs` listing the expected contents, and a `// TODO:` note in the engine-addition workflow template (AGENTS.md §7) to update the prelude.

## Code Examples

### Minimal GA with Prelude (Target UX)

```rust
// After Phase 81, a minimal GA setup looks like this:
use genetic_algorithms::prelude::*;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;

let fitness_fn = |dna: &[RangeGenotype<f64>]| -> f64 {
    dna.iter().map(|g| g.value.powi(2)).sum()
};

let alleles = vec![RangeGenotype::new(0, vec![(-5.12, 5.12)], 0.0_f64)];
let alleles_clone = alleles.clone();

let mut ga: Ga<RangeChromosome<f64>> = Ga::new()
    .with_chromosome_length(ChromosomeLength::Fixed(5))
    .with_population_size(100)
    .with_initialization_fn(move |n, _| range_random_initialization(n, Some(&alleles_clone)))
    .with_fitness_fn(fitness_fn)
    .with_selection_method(Selection::Tournament)
    .with_crossover_method(Crossover::Uniform)
    .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
    .with_survivor_method(Survivor::Fitness)
    .with_problem_solving(ProblemSolving::Minimization)
    .with_max_generations(500)
    .build()
    .expect("Invalid configuration");

let population = ga.run().expect("GA run failed");
```

### Prelude File Structure

```rust
// src/prelude.rs
//!
//! # Prelude
//!
//! Convenient glob import for the most commonly used items.
//!
//! ```rust
//! use genetic_algorithms::prelude::*;
//! ```

// --- Engine entry points ---
pub use crate::ga::Ga;
pub use crate::de::DeEngine;
pub use crate::scatter::ScatterEngine;
pub use crate::cellular::CellularEngine;
pub use crate::alps::AlpsEngine;
pub use crate::island::IslandGa;
pub use crate::gp::GpGa;
pub use crate::eda::{EdaEngine, EdaRealEngine};
pub use crate::nsga2::Nsga2Ga;
pub use crate::nsga3::Nsga3Ga;
pub use crate::moead::MoeaDGa;
pub use crate::spea2::Spea2Ga;
pub use crate::sms_emoa::SmsEmoaGa;
pub use crate::ibea::IbeaGa;
pub use crate::cma::CmaEngine;
pub use crate::pso::PsoEngine;
pub use crate::hill_climb::HillClimbEngine;
pub use crate::permutate::PermutateEngine;

// --- Engine-specific configuration structs ---
pub use crate::cma::CmaConfiguration;
pub use crate::pso::PsoConfiguration;
pub use crate::eda::EdaConfiguration;
pub use crate::alps::AlpsConfiguration;
pub use crate::hill_climb::HillClimbConfiguration;
pub use crate::permutate::PermutateConfiguration;
pub use crate::de::DeConfiguration;
pub use crate::scatter::ScatterConfiguration;
pub use crate::cellular::CellularConfiguration;
pub use crate::gp::GpConfiguration;

// --- Core traits ---
pub use crate::traits::{
    ChromosomeT, ConfigurationT, GeneT, LinearChromosome,
    CrossoverConfig, ElitismConfig, ExtensionConfig, LocalSearchConfig,
    MutationConfig, NichingConfig, SelectionConfig, StoppingConfig, SurvivorConfig,
};

// --- Operator enums ---
pub use crate::operations::{Crossover, Mutation, Selection, Survivor};

// --- Configuration types ---
pub use crate::configuration::ProblemSolving;
pub use crate::chromosomes::ChromosomeLength;

// --- Error ---
pub use crate::error::GaError;

// --- Observer (minimal) ---
pub use crate::observer::{GaObserver, NoopObserver};

// --- Feature-gated observers ---
#[cfg(feature = "logging")]
pub use crate::observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use crate::observer::MetricsObserver;
#[cfg(feature = "observer-tracing")]
pub use crate::observer::TracingObserver;
```

### Rastrigin Example Before/After (D-06)

**Before (11 imports):**
```rust
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::{Ga, TerminationCause};
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
};
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;
use genetic_algorithms::{rng, ChromosomeLength, CompositeObserver, LogObserver};
```

**After (4 imports):**
```rust
use genetic_algorithms::prelude::*;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
// Plus non-prelude items used only in this example:
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::operations::GaussianParams;
use genetic_algorithms::{rng, CompositeObserver};
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;
```

**Note:** The example uses several types not in the prelude (`TerminationCause`, `Population`, `GenerationStats`, `GaussianParams`, `CompositeObserver`, `MetricsObserver`, `rng`). These are example-specific and don't belong in the general prelude. The ergonomic win is still significant: the 4 core import lines replace 6, and the prelude glob covers all the high-frequency builder types.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 8-11 separate `use` lines per example | `use genetic_algorithms::prelude::*;` + 2-4 concrete type imports | Phase 81 | Reduced boilerplate for new users |

**Deprecated/outdated:**
- None — this is purely additive

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | All engine type names are unique across the crate (no two public structs share the same identifier) | Architecture Patterns | Name collision in glob import — compile error |
| A2 | `GaError` is accessible as `crate::error::GaError` (not re-exported at crate root) | Code Examples | Wrong import path in prelude |
| A3 | Engine-specific config structs are accessible via their engine module path (e.g., `crate::cma::CmaConfiguration`) | Code Examples | Wrong import path in prelude |
| A4 | `CompositeObserver` and `MetricsObserver` are NOT in the prelude per D-04 | Code Examples | Unintended scope expansion |

**All assumptions verified against source code in this session.**

## Open Questions (RESOLVED)

1. RESOLVED: **No** — mutation params are operator-specific and varied (6 param structs). Users constructing `Mutation::Gaussian(GaussianParams { ... })` already know they need the params type. Keep it out of the prelude; users import from `genetic_algorithms::operations::GaussianParams`.

2. RESOLVED: **No** — `TerminationCause` is only needed for match arms on `ga.run()` results and observer implementations. Not part of the GA builder pattern.

3. RESOLVED: **No** per D-04 — `GaObserver` trait + `NoopObserver` only. `CompositeObserver` is optional/advanced.

## Environment Availability

> Skip — this phase has no external dependencies (pure code changes).

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| (none) | — | — | — | — |

**Missing dependencies with no fallback:** none
**Missing dependencies with fallback:** none

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Built-in `cargo test` + `cargo test --doc` |
| Config file | `Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test && cargo test --doc && cargo bench --no-run` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SC-1 | `src/prelude.rs` re-exports all required items | compile-check | `cargo check` | ❌ Wave 0 |
| SC-3 | Minimal GA can be written with prelude glob + concrete types | integration test | `cargo test test_prelude_minimal_ga` | ❌ Wave 0 |
| SC-4 | No glob-import name collisions | compile-check | `cargo check` with test file using `prelude::*` | ❌ Wave 0 |
| SC-7 | `cargo doc --no-deps` clean | doc-build | `cargo doc --no-deps` | ✅ (CI) |

### Sampling Rate

- **Per task commit:** `cargo test && cargo clippy`
- **Per wave merge:** `cargo test && cargo test --doc && cargo bench --no-run`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/test_prelude.rs` — compile-check test that `use genetic_algorithms::prelude::*;` succeeds and key types are accessible
- [ ] `tests/test_prelude_minimal_ga.rs` — integration test that a minimal GA can be built and run using only prelude imports + concrete types

## Security Domain

> Not applicable — this phase adds no authentication, input validation, cryptography, or access control. It is a pure re-export module with zero runtime behavior.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | no | — |
| V6 Cryptography | no | — |

## Sources

### Primary (HIGH confidence)
- `src/lib.rs` lines 397–437 — current `pub use` re-exports (verified by reading source)
- `src/traits.rs` lines 53–74 — current trait re-exports (verified by reading source)
- `src/operations.rs` — operator enums `Selection`, `Crossover`, `Mutation`, `Survivor` (verified by reading source)
- `src/configuration.rs` — `ProblemSolving`, `GaConfiguration` (verified by reading source)
- `src/error.rs` — `GaError` enum (verified by reading source)
- `src/observe/observer/mod.rs` — `GaObserver`, `NoopObserver`, feature-gated observers (verified by reading source)
- `src/engines/*/` — all 17 engine struct names verified via grep

### Secondary (MEDIUM confidence)
- https://doc.rust-lang.org/std/prelude/index.html — Rust standard prelude pattern [CITED: doc.rust-lang.org]

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; pure re-exports
- Architecture: HIGH — standard Rust prelude pattern; all paths verified in source
- Pitfalls: HIGH — name collision risk is zero (verified all public type names are unique)

**Research date:** 2026-06-22
**Valid until:** 2026-07-22 (stable — prelude pattern is well-established)
