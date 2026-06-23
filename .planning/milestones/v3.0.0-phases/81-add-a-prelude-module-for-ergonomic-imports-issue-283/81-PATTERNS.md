# Phase 81: Add a Prelude Module for Ergonomic Imports - Pattern Map

**Mapped:** 2026-06-22
**Files analyzed:** 7 (2 new source, 2 new test, 3 modified)
**Analogs found:** 6 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/prelude.rs` | config (re-export module) | transform (compile-time) | `src/observer/mod.rs` re-export block | exact |
| `src/lib.rs` | config (module declaration) | transform | existing `pub mod` declarations in `src/lib.rs` lines 320-395 | exact |
| `examples/rastrigin.rs` | example | request-response | self (import section rewrite) | exact |
| `tests/test_prelude.rs` | test | request-response | `tests/observe/observer/test_observer_reexports.rs` | exact |
| `tests/test_prelude_minimal_ga.rs` | test | request-response | `tests/test_no_logger_installed.rs` | role-match |
| `README.md` | documentation | transform | existing Quick Start section (lines 110-153) | role-match |
| `docs/getting-started.md` | documentation | transform | existing First Run section (lines 53-80) | role-match |

## Pattern Assignments

### `src/prelude.rs` (config, transform — compile-time re-exports)

**Analog:** `src/observer/mod.rs` re-export block + `src/traits.rs` re-export block

This file is purely `pub use` re-exports. The closest structural analog is the observer module's re-export pattern and the traits module's grouped re-export pattern.

**Rustdoc header pattern** (from `src/traits.rs` lines 1-36):
```rust
//! # Prelude
//!
//! Convenient glob import for the most commonly used items from `genetic_algorithms`.
//!
//! This module re-exports engine entry points, core traits, operator enums,
//! configuration types, and error types so users can write:
//!
//! ```rust
//! use genetic_algorithms::prelude::*;
//! ```
//!
//! # What's included
//!
//! | Category | Items |
//! |----------|-------|
//! | Engines | `Ga`, `DeEngine`, `ScatterEngine`, `CellularEngine`, `AlpsEngine`, `IslandGa`, `GpGa`, `EdaEngine`, `EdaRealEngine`, `Nsga2Ga`, `Nsga3Ga`, `MoeaDGa`, `Spea2Ga`, `SmsEmoaGa`, `IbeaGa`, `CmaEngine`, `PsoEngine`, `HillClimbEngine`, `PermutateEngine` |
//! | Engine configs | `CmaConfiguration`, `PsoConfiguration`, `EdaConfiguration`, `AlpsConfiguration`, `HillClimbConfiguration`, `PermutateConfiguration`, `DeConfiguration`, `ScatterConfiguration`, `CellularConfiguration`, `GpConfiguration` |
//! | Core traits | `ChromosomeT`, `GeneT`, `LinearChromosome`, `ConfigurationT`, `CrossoverConfig`, `ElitismConfig`, `ExtensionConfig`, `LocalSearchConfig`, `MutationConfig`, `NichingConfig`, `SelectionConfig`, `StoppingConfig`, `SurvivorConfig` |
//! | Operator enums | `Selection`, `Crossover`, `Mutation`, `Survivor` |
//! | Config types | `ProblemSolving`, `ChromosomeLength` |
//! | Error | `GaError` |
//! | Observer | `GaObserver`, `NoopObserver` |
//!
//! Concrete chromosome/genotype types (`Binary`, `Range<T>`, `ListChromosome<T>`) and
//! initializer functions are intentionally excluded — they are problem-specific and should
//! be imported explicitly.
```

**Grouped re-export pattern** (composite from `src/traits.rs` lines 55-74 and `src/lib.rs` lines 397-437):

```rust
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
```

**Feature-gated re-export pattern** (from `src/lib.rs` lines 420-432):
```rust
// --- Feature-gated observers ---
#[cfg(feature = "logging")]
pub use crate::observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use crate::observer::MetricsObserver;
#[cfg(feature = "observer-tracing")]
pub use crate::observer::TracingObserver;
```

**Important notes:**
- All import paths use `crate::` prefix (absolute paths within the crate)
- The `pub use crate::traits::{...}` block uses a single multi-item import (pattern from `src/traits.rs` lines 57-60)
- Engine-specific configs are imported individually per engine module (pattern from `src/lib.rs` lines 410, 433-434)
- Feature gates must exactly mirror `src/lib.rs` lines 420-432 — same `#[cfg(feature = "...")]` annotations

---

### `src/lib.rs` (config, module declaration)

**Analog:** Existing `pub mod` declarations in `src/lib.rs` lines 320-395

**Module declaration to add** (alongside existing `pub mod` declarations, after line 345 `pub mod traits;`):
```rust
pub mod prelude;
```

**Pattern reference** — existing module declarations (`src/lib.rs` lines 320-346):
```rust
pub mod aos;
#[cfg(feature = "benchmarks")]
pub mod benchmarks;
#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;
#[path = "types/chromosomes/mod.rs"]
pub mod chromosomes;
pub mod configuration;
pub mod constraints;
pub mod error;
pub mod extension;
pub mod fitness;
#[path = "engines/ga/mod.rs"]
pub mod ga;
#[path = "types/genotypes/mod.rs"]
pub mod genotypes;
pub mod hall_of_fame;
pub mod initializers;
#[path = "observe/observer/mod.rs"]
pub mod observer;
pub mod operations;
pub mod population;
pub mod rng;
pub mod stats;
pub mod traits;
pub mod validators;
```

**Note:** `prelude.rs` lives directly in `src/` (not in a subdirectory), so it needs no `#[path = "..."]` attribute — just `pub mod prelude;`. Place it alphabetically among the other `pub mod` declarations (after `pub mod population;`, before `pub mod rng;`).

---

### `examples/rastrigin.rs` (example, import rewrite)

**Analog:** Self — only the import section (lines 23-37) changes.

**Current imports** (`examples/rastrigin.rs` lines 23-37):
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
use std::sync::Arc;
```

**Target imports** (after prelude):
```rust
use genetic_algorithms::prelude::*;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
// Non-prelude items used only in this example:
use genetic_algorithms::ga::TerminationCause;
use genetic_algorithms::operations::GaussianParams;
use genetic_algorithms::population::Population;
use genetic_algorithms::stats::GenerationStats;
use genetic_algorithms::{rng, CompositeObserver};
#[cfg(feature = "observer-metrics")]
use genetic_algorithms::MetricsObserver;
use std::sync::Arc;
```

**Pattern:** The prelude glob replaces all items that are in the prelude. Items NOT in the prelude (example-specific types) remain as explicit imports. The rest of the file (lines 38+) is unchanged.

---

### `tests/test_prelude.rs` (test, compile-check)

**Analog:** `tests/observe/observer/test_observer_reexports.rs`

This is the closest analog — a compile-check test verifying that re-exported types are accessible.

**Imports pattern** (from `tests/observe/observer/test_observer_reexports.rs` lines 1-6):
```rust
//! Compile-time verification that prelude types are re-exported and accessible.
//! Covers SC-1 (prelude re-exports all required items) and SC-4 (no name collisions).

use genetic_algorithms::prelude::*;
```

**Test structure pattern** (from `tests/observe/observer/test_observer_reexports.rs` lines 8-30):
```rust
#[test]
fn test_prelude_engines_accessible() {
    // Compile-time check: if this compiles, all engine types are accessible
    let _: fn() -> Ga<genetic_algorithms::chromosomes::Binary> = || unimplemented!();
    // Other engine types verified by the glob import succeeding
}

#[test]
fn test_prelude_traits_accessible() {
    // Compile-time check: trait bounds compile
    fn assert_traits<T: ChromosomeT + LinearChromosome>() {}
    fn assert_config<T: ConfigurationT>() {}
    let _ = (assert_traits::<genetic_algorithms::chromosomes::Binary>, assert_config::<genetic_algorithms::ga::Ga<genetic_algorithms::chromosomes::Binary>>);
}

#[test]
fn test_prelude_operator_enums_accessible() {
    let _sel = Selection::Tournament;
    let _cx = Crossover::Uniform;
    let _mut = Mutation::BitFlip;
    let _surv = Survivor::Fitness;
}

#[test]
fn test_prelude_config_types_accessible() {
    let _ps = ProblemSolving::Minimization;
    let _cl = ChromosomeLength::Fixed(5);
}

#[test]
fn test_prelude_error_accessible() {
    let _err = GaError::ConfigurationError("test".into());
}

#[test]
fn test_prelude_observer_accessible() {
    let _obs = NoopObserver;
}
```

**Key pattern:** Tests use `use genetic_algorithms::prelude::*;` at the top (glob import) and then verify individual items are accessible. Each test focuses on one category. If any name collision exists, the glob import itself will fail to compile.

---

### `tests/test_prelude_minimal_ga.rs` (test, integration)

**Analog:** `tests/test_no_logger_installed.rs` (builds and runs a minimal GA)

**Imports pattern** (adapted from `tests/test_no_logger_installed.rs` lines 30-38):
```rust
//! Integration test: a minimal GA can be built and run using only prelude imports
//! plus concrete chromosome/genotype types. Covers SC-3.

use genetic_algorithms::prelude::*;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
```

**Test pattern** (adapted from `tests/test_no_logger_installed.rs` lines 70-91):
```rust
fn count_ones(genes: &[Binary]) -> f64 {
    genes.iter().filter(|g| g.value).count() as f64
}

#[test]
fn test_prelude_minimal_ga() {
    let mut ga: Ga<BinaryChromosome> = Ga::new()
        .with_population_size(4)
        .with_chromosome_length(ChromosomeLength::Fixed(4))
        .with_initialization_fn(binary_random_initialization)
        .with_fitness_fn(count_ones)
        .with_selection_method(Selection::Random)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_survivor_method(Survivor::Fitness)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_max_generations(1)
        .build()
        .expect("valid configuration");

    let result = ga.run();
    assert!(result.is_ok(), "GA run should complete using prelude imports");
}
```

**Key pattern:** Uses `use genetic_algorithms::prelude::*;` as the primary import, then explicit imports only for concrete types not in the prelude (Binary chromosome/genotype, initializer function). All builder methods (`with_*`), operator enums, config types, and error types come from the prelude glob.

---

### `README.md` (documentation)

**Analog:** Existing Quick Start section (`README.md` lines 110-153)

**Current Quick Start** uses 8 import lines (lines 115-123). The update should add an "Ergonomic Imports" subsection after the Quick Start code block showing the prelude alternative.

**Pattern to add** (after line 153, before `## Features`):
```markdown
### Ergonomic Imports

Instead of multiple import lines, use the prelude for a single glob import:

```rust
use genetic_algorithms::prelude::*;
use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::initializers::range_random_initialization;
```

The prelude includes all engine entry points, configuration traits, operator enums,
core traits, and error types. Concrete chromosome/genotype types and initializer
functions are imported separately since they are problem-specific.
```

---

### `docs/getting-started.md` (documentation)

**Analog:** Existing First Run section (`docs/getting-started.md` lines 53-80)

**Pattern to add** (after the First Run code example, before the next section):
```markdown
## Using the Prelude

For a cleaner import experience, use the prelude module:

```rust
use genetic_algorithms::prelude::*;
use genetic_algorithms::chromosomes::Binary as BinaryChromosome;
use genetic_algorithms::genotypes::Binary;
use genetic_algorithms::initializers::binary_random_initialization;
```

The prelude re-exports all high-frequency items: engine entry points, core traits,
operator enums, configuration types, and error types. Concrete types and initializer
functions remain explicit imports.
```

---

## Shared Patterns

### Feature-Gated Re-exports
**Source:** `src/lib.rs` lines 420-432
**Apply to:** `src/prelude.rs` — observer section

```rust
#[cfg(feature = "logging")]
pub use crate::observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use crate::observer::MetricsObserver;
#[cfg(feature = "observer-tracing")]
pub use crate::observer::TracingObserver;
```

This exact pattern must be copied to `prelude.rs` so that `use genetic_algorithms::prelude::*;` respects the same feature gates as `use genetic_algorithms::LogObserver;`.

### Module Declaration Convention
**Source:** `src/lib.rs` lines 320-346
**Apply to:** `src/lib.rs` — add `pub mod prelude;`

All public modules in `lib.rs` follow the same pattern: `pub mod name;` with optional `#[cfg(feature = "...")]` and `#[path = "..."]` attributes. `prelude.rs` is a simple top-level file, so just `pub mod prelude;`.

### Test Compile-Check Pattern
**Source:** `tests/observe/observer/test_observer_reexports.rs`
**Apply to:** `tests/test_prelude.rs`

Integration test files that verify re-exports use the pattern: import the item, construct or reference it in a test function, and let compilation prove accessibility. Each test function covers one category of re-exported items.

### Integration Test GA-Builder Pattern
**Source:** `tests/test_no_logger_installed.rs` lines 70-91
**Apply to:** `tests/test_prelude_minimal_ga.rs`

Build a minimal GA using the builder pattern, call `.run()`, assert success. This pattern is used across many test files (`test_stopping_config.rs`, `test_chromosome_length.rs`, etc.) and is the standard way to verify end-to-end functionality.

## No Analog Found

Files with no close match in the codebase (planner should use RESEARCH.md patterns instead):

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| (none) | — | — | All files have adequate analogs |

## Metadata

**Analog search scope:** `src/`, `tests/`, `examples/`, `docs/`, `README.md`
**Files scanned:** 12 (lib.rs, traits.rs, observer/mod.rs, test_observer_reexports.rs, test_no_logger_installed.rs, test_error.rs, test_chromosome_length.rs, test_stopping_config.rs, rastrigin.rs, README.md, getting-started.md, ROADMAP.md)
**Pattern extraction date:** 2026-06-22
