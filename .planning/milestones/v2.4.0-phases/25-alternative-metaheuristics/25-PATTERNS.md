# Phase 25: Directory Restructure - Pattern Map

**Mapped:** 2026-04-26
**Files analyzed:** 14 (lib.rs + 3 new group mod.rs files + 4 placeholder stubs + moved modules)
**Analogs found:** 14 / 14

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/lib.rs` | config | request-response | `src/lib.rs` (current) | exact — same file, surgical edit |
| `src/engines/ga.rs` | engine | batch | `src/ga.rs` | exact — file move only |
| `src/engines/island/` | engine | batch | `src/island/` | exact — directory move only |
| `src/engines/nsga2/` | engine | batch | `src/nsga2/` | exact — directory move only |
| `src/engines/de/mod.rs` | placeholder | — | `src/nsga2/mod.rs` (doc comment only pattern) | role-match |
| `src/engines/scatter/mod.rs` | placeholder | — | same | role-match |
| `src/engines/cellular/mod.rs` | placeholder | — | same | role-match |
| `src/engines/alps/mod.rs` | placeholder | — | same | role-match |
| `src/types/chromosomes.rs` + `chromosomes/` | types | — | `src/chromosomes.rs` + `chromosomes/` | exact — move only |
| `src/types/genotypes.rs` + `genotypes/` | types | — | `src/genotypes.rs` + `genotypes/` | exact — move only |
| `src/observe/observer/` | observer | event-driven | `src/observer/` | exact — move only |
| `src/observe/reporter/` | observer | event-driven | `src/reporter/` | exact — move only |
| `src/observe/visualization/` | observer | event-driven | `src/visualization/` | exact — move only |
| `src/observe/checkpoint.rs` | utility | file-I/O | `src/checkpoint.rs` | exact — move only |

---

## Pattern Assignments

### `src/lib.rs` (modified — add `#[path]` attributes)

**Analog:** `src/lib.rs` (current state, lines 70–107)

**Current `pub mod` block** (lines 70–93) — the block that gets surgically replaced:
```rust
extern crate core;

#[cfg(feature = "serde")]
pub mod checkpoint;
pub mod chromosomes;
pub mod configuration;
pub mod error;
pub mod extension;
pub mod fitness;
pub mod ga;
pub mod genotypes;
pub mod initializers;
pub mod observer;
pub mod operations;
pub mod population;
pub mod reporter;
pub mod rng;
pub mod stats;
pub mod traits;
pub mod validators;
#[cfg(feature = "visualization")]
pub mod visualization;

pub mod island;
pub mod niching;
pub mod nsga2;
```

**Target pattern — replace moved modules with `#[path]` declarations:**
```rust
extern crate core;

// Cross-cutting infrastructure — stays at src/ root, no #[path] needed
pub mod configuration;
pub mod error;
pub mod extension;
pub mod fitness;
pub mod initializers;
pub mod niching;
pub mod operations;
pub mod population;
pub mod rng;
pub mod stats;
pub mod traits;
pub mod validators;

// Engines group — files moved to src/engines/, paths preserved via #[path]
#[path = "engines/ga.rs"]
pub mod ga;
#[path = "engines/island/mod.rs"]
pub mod island;
#[path = "engines/nsga2/mod.rs"]
pub mod nsga2;

// Types group — files moved to src/types/, paths preserved via #[path]
#[path = "types/chromosomes.rs"]
pub mod chromosomes;
#[path = "types/genotypes.rs"]
pub mod genotypes;

// Observe group — files moved to src/observe/, paths preserved via #[path]
#[path = "observe/observer/mod.rs"]
pub mod observer;
#[path = "observe/reporter/mod.rs"]
pub mod reporter;
#[cfg(feature = "visualization")]
#[path = "observe/visualization/mod.rs"]
pub mod visualization;
#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;
```

**Top-level `pub use` block** (lines 95–107) — copy verbatim, zero changes needed:
```rust
pub use ga::TerminationCause;
pub use observer::AllObserver;
pub use observer::CompositeObserver;
pub use observer::ExtensionEvent;
pub use observer::GaObserver;
pub use observer::IslandGaObserver;
pub use observer::LogObserver;
#[cfg(feature = "observer-metrics")]
pub use observer::MetricsObserver;
pub use observer::NoopObserver;
pub use observer::Nsga2Observer;
#[cfg(feature = "observer-tracing")]
pub use observer::TracingObserver;
```

**Key constraint:** `pub use ga::TerminationCause` works as-is because `pub mod ga` (via `#[path]`) is still declared in lib.rs. No change to the `pub use` block is needed.

---

### `src/engines/ga.rs` (moved from `src/ga.rs`)

**Analog:** `src/ga.rs` — file move only, zero content changes.

**Why zero changes:** All internal `crate::` references inside `ga.rs` resolve from lib.rs (the crate root). Since lib.rs still declares `pub mod observer`, `pub mod island`, etc. (either directly or via `#[path]`), every `crate::observer::…`, `crate::island::…` inside the moved file resolves correctly without any edits.

---

### `src/engines/island/` (moved from `src/island/`)

**Analog:** `src/island/mod.rs` (lines 27–35) — internal `crate::` imports pattern to confirm no edits needed:
```rust
use crate::configuration::{GaConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::island::configuration::IslandConfiguration;
use crate::island::migration::migrate;
use crate::observer::IslandGaObserver;
use crate::operations::mutation;
use crate::population::Population;
use crate::stats::GenerationStats;
use crate::traits::{ChromosomeT, FitnessFn, InitializationFn};
```

All references use `crate::` (not `super::` or relative paths). After the move, lib.rs still exposes `pub mod island` (via `#[path]`), so `crate::island::configuration::IslandConfiguration` resolves correctly. **No edits to `mod.rs` or any sub-file of `island/`.**

**Self-referencing pattern inside island/mod.rs** (lines 30, 31) — `crate::island::…` is used to reference the module's own submodules. This is safe because lib.rs's `#[path = "engines/island/mod.rs"] pub mod island;` makes `crate::island` the canonical path for the moved directory.

---

### `src/engines/nsga2/` (moved from `src/nsga2/`)

**Analog:** `src/nsga2/mod.rs` (lines 35–49) — internal imports pattern:
```rust
use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::nsga2::configuration::{Nsga2Configuration, ObjectiveDirection};
use crate::nsga2::crowding_distance::assign_crowding_distance;
use crate::nsga2::non_dominated_sort::{…};
use crate::nsga2::pareto::{ParetoFront, ParetoIndividual};
use crate::observer::Nsga2Observer;
use crate::operations::mutation;
use crate::traits::{ChromosomeT, InitializationFn};
```

Same pattern as island — all `crate::` paths, no edits needed after move.

---

### Placeholder stubs: `src/engines/de/mod.rs`, `src/engines/scatter/mod.rs`, `src/engines/cellular/mod.rs`, `src/engines/alps/mod.rs`

**Analog:** The module-level doc comment pattern from any existing `mod.rs`.

**Pattern — doc-comment-only stub (no `pub` items, never compiled until declared):**
```rust
//! Differential Evolution engine — placeholder for Phase 26.
```
```rust
//! Scatter Search engine — placeholder for Phase 27.
```
```rust
//! Cellular Genetic Algorithm engine — placeholder for Phase 28.
```
```rust
//! Age-Layered Population Structure (ALPS) engine — placeholder for Phase 29.
```

**Critical:** Nothing declares `mod de;` (or scatter/cellular/alps) anywhere. The files exist on disk but are never compiled. This means zero public API surface and zero Clippy/doc warnings.

---

### `src/types/chromosomes.rs` + `src/types/chromosomes/` (moved from `src/`)

**Analog:** `src/chromosomes.rs` (lines 1–15) — copy verbatim:
```rust
//! Built-in chromosome types.
//!
//! This module provides ready-to-use chromosome implementations:
//!
//! - [`Binary`] — a chromosome whose DNA is a vector of [`genotypes::Binary`](crate::genotypes::Binary) genes.
//! - [`Range`] — a chromosome whose DNA is a vector of [`genotypes::Range`](crate::genotypes::Range) genes.
//! - [`ListChromosome`] — a chromosome whose DNA is a vector of [`genotypes::List`](crate::genotypes::List) genes.

pub mod binary;
pub mod list;
mod range;

pub use binary::Binary;
pub use list::ListChromosome;
pub use range::Range;
```

File move only — zero content changes. All `crate::` paths in sub-files resolve unchanged.

---

### `src/types/genotypes.rs` + `src/types/genotypes/` (moved from `src/`)

**Analog:** `src/genotypes.rs` (lines 1–17) — copy verbatim:
```rust
//! Built-in gene (genotype) types.
//!
//! This module provides ready-to-use [`GeneT`](crate::traits::GeneT) implementations:
//!
//! - [`Binary`] — a gene that holds a boolean value (`true`/`false`).
//! - [`Range`] — a gene that holds a numeric value within an interval.
//! - [`List`] — a gene that holds a value drawn from a finite set of alleles.

pub mod binary;
pub mod list;
pub mod range;

pub use binary::Binary;
pub use list::List;
pub use range::Range;
```

File move only — zero content changes.

---

### `src/observe/observer/` (moved from `src/observer/`)

**Analog:** `src/observer/mod.rs` (lines 189–203) — feature-gated sub-module declaration pattern:
```rust
mod log;
pub use log::LogObserver;

#[cfg(feature = "observer-tracing")]
mod tracing_observer;
#[cfg(feature = "observer-tracing")]
pub use tracing_observer::TracingObserver;

#[cfg(feature = "observer-metrics")]
mod metrics_observer;
#[cfg(feature = "observer-metrics")]
pub use metrics_observer::MetricsObserver;

mod composite;
pub use composite::CompositeObserver;
```

File move only. The `crate::ga::TerminationCause` reference at line 31 resolves because lib.rs still declares `pub mod ga` (via `#[path]`).

---

### `src/observe/reporter/` (moved from `src/reporter/`)

**Analog:** `src/reporter/mod.rs` (lines 1–51) — internal `crate::ga::TerminationCause` reference:
```rust
use crate::ga::TerminationCause;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
```

File move only — the `crate::ga` reference resolves through lib.rs's `#[path]` shim.

---

### `src/observe/visualization/` (moved from `src/visualization/`)

**Analog:** `src/visualization/mod.rs` — directory move only.

**Feature gate preserved in lib.rs:**
```rust
#[cfg(feature = "visualization")]
#[path = "observe/visualization/mod.rs"]
pub mod visualization;
```

The `#[cfg]` annotation must appear before the `#[path]` attribute, both before `pub mod`.

---

### `src/observe/checkpoint.rs` (moved from `src/checkpoint.rs`)

**Analog:** `src/checkpoint.rs` (lines 1–16) — serde dependency pattern:
```rust
use crate::configuration::GaConfiguration;
use crate::error::GaError;
use crate::population::Population;
use crate::stats::GenerationStats;
use crate::traits::ChromosomeT;
use serde::{Deserialize, Serialize};
use std::path::Path;
```

File move only. Feature gate preserved in lib.rs:
```rust
#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;
```

---

## Shared Patterns

### `#[path]` Attribute Declaration Order

**Apply to:** Every moved module declaration in lib.rs.

The `#[cfg]` attribute (if any) must come before `#[path]`, which must come before `pub mod`:
```rust
// Feature-gated + path: cfg first, path second, then pub mod
#[cfg(feature = "serde")]
#[path = "observe/checkpoint.rs"]
pub mod checkpoint;

// Path only (no feature gate):
#[path = "engines/ga.rs"]
pub mod ga;
```

Inverting the order (`#[path]` before `#[cfg]`) is also valid Rust, but the `cfg`-first convention matches the current lib.rs style (lines 70–71: `#[cfg(feature = "serde")] pub mod checkpoint;`).

---

### Internal `crate::` Reference — No Changes Needed

**Source:** Verified from `src/island/mod.rs` lines 27–35 and `src/nsga2/mod.rs` lines 35–49.

**Apply to:** Every moved file (`ga.rs`, `island/**`, `nsga2/**`, `observer/**`, `reporter/**`, `visualization/**`, `checkpoint.rs`, `chromosomes/**`, `genotypes/**`).

`crate::X` resolves from lib.rs regardless of where the source file physically lives. As long as lib.rs declares `pub mod X` (directly or via `#[path]`), all `crate::X::…` paths inside moved files compile without modification.

**Anti-pattern to avoid:** Changing `crate::observer::GaObserver` to `crate::observe::observer::GaObserver` inside moved files — the `observe` group is invisible to downstream; `crate::observer` is the canonical path.

---

### Feature-Gate Propagation

**Source:** `src/lib.rs` lines 70–71 and 88–89 (current):
```rust
#[cfg(feature = "serde")]
pub mod checkpoint;

#[cfg(feature = "visualization")]
pub mod visualization;
```

**Apply to:** The corresponding `#[path]` declarations in the updated lib.rs. Both `#[cfg]` annotations must transfer exactly — dropping either causes unconditional compilation of a feature-gated module.

---

### Module Doc Comment Convention

**Source:** `src/island/mod.rs` lines 1–20, `src/nsga2/mod.rs` lines 1–28, `src/observer/mod.rs` lines 1–30.

All `mod.rs` files begin with a `//!` module-level doc comment describing the module's purpose and optionally providing a `\`\`\`ignore` usage example. New `mod.rs` files (if created) must follow this pattern.

---

## No Analog Found

No files in this phase lack a codebase analog. This is a pure file-move restructure — every file already exists; new files are limited to placeholder stubs with doc-comment-only content.

---

## Metadata

**Analog search scope:** `src/` (all modules), `src/lib.rs`, `src/island/mod.rs`, `src/nsga2/mod.rs`, `src/observer/mod.rs`, `src/reporter/mod.rs`, `src/chromosomes.rs`, `src/genotypes.rs`, `src/checkpoint.rs`
**Files read:** 9 source files
**Pattern extraction date:** 2026-04-26
