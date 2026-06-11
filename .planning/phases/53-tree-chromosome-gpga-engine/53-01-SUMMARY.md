---
phase: 53-tree-chromosome-gpga-engine
plan: "01"
subsystem: gp
tags:
  - genetic-programming
  - tree-chromosome
  - api-contract
  - wave-0
dependency_graph:
  requires: []
  provides:
    - gp::GpNode
    - gp::Node<N>
    - gp::GpChromosome<N>
    - gp::TreeChromosome
    - gp::GpConfiguration
    - gp::MathNode
    - gp::BoolNode
  affects:
    - src/error.rs (GaError new variants)
    - src/stats.rs (GenerationStats new field)
    - src/lib.rs (new pub mod gp)
tech_stack:
  added:
    - "rand::Rng trait used in GpNode::sample_random_terminal (rand 0.9 — random_range API)"
    - "std::sync::Arc for tree fitness function storage in GpChromosome"
    - "std::mem::take for iterative Drop in Node<N>"
  patterns:
    - "Trait-on-user-type: GpNode follows same design philosophy as GeneT"
    - "super:: relative imports within src/engines/gp/ (not crate::engines::gp::)"
    - "Type alias TreeFitnessFn<N> to satisfy clippy::type_complexity"
    - "cfg(feature = serde) gating on default_fitness_fn to suppress dead_code warning"
key_files:
  created:
    - src/engines/gp/mod.rs
    - src/engines/gp/node.rs
    - src/engines/gp/chromosome.rs
    - src/engines/gp/configuration.rs
    - src/engines/gp/primitives.rs
    - tests/gp.rs
  modified:
    - src/error.rs
    - src/stats.rs
    - src/lib.rs
decisions:
  - "Used super:: relative imports within engines/gp/ because lib.rs exports pub mod gp via #[path], so the crate-level path is crate::gp, not crate::engines::gp"
  - "Extracted TreeFitnessFn<N> type alias from the complex Arc<dyn Fn(&Node<N>)> type to satisfy clippy::type_complexity"
  - "Gated default_fitness_fn behind #[cfg(feature = serde)] to suppress dead_code warning when serde is disabled"
  - "Used std::mem::take + Vec::append in iterative Drop instead of drain().collect() + extend(drain(..)) to satisfy clippy::drain_collect and clippy::extend_with_drain"
  - "Used rand::Rng::random_range (rand 0.9 API) not the deprecated gen_range"
  - "GpConfiguration::build() enforces max_depth <= 1000 and max_node_count <= 100_000 hard caps per threat model T-53-02 and T-53-03"
metrics:
  duration: "20m 54s"
  completed: "2026-05-25"
  tasks_completed: 3
  files_modified: 8
---

# Phase 53 Plan 01: GP Subsystem Wave 0 API Contract Summary

Wave 0 establishes the complete GP type system: `GpNode` trait, `Node<N>` recursive enum with iterative `Drop`, `GpChromosome<N>` implementing `ChromosomeT` with panicking linear stubs, `TreeChromosome` supertrait, `GpConfiguration` shell with validated depth/size limits, `MathNode`/`BoolNode` built-in primitives, and a compiling `tests/gp.rs` stub file.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Core types (GpNode, Node, GpChromosome, TreeChromosome, GpConfiguration, primitives) | 6322881 | src/engines/gp/{mod,node,chromosome,configuration,primitives}.rs, src/lib.rs |
| 2 | Wire GaError variants, GenerationStats field, tests/gp.rs stub | 1b9c6d8 | src/error.rs, src/stats.rs, tests/gp.rs |
| 3 | Primitives MathNode/BoolNode tests (done in Task 1) | 6322881 | src/engines/gp/primitives.rs, tests/gp.rs |

## Verification Results

- `cargo check` — zero errors
- `cargo test --test gp` — 7 passed, 8 ignored (0 failures)
- `cargo test --features serde` — 0 FAILED (no regressions)
- `cargo clippy -- -D warnings` — no warnings
- `cargo check --target wasm32-unknown-unknown` — passes (pure type definitions, no rayon/std::time)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Module path `crate::engines::gp` is invalid**
- **Found during:** Task 1 (first `cargo check` after lib.rs export added)
- **Issue:** `src/engines/gp/chromosome.rs` and `primitives.rs` used `use crate::engines::gp::node::...`. Since lib.rs exports the gp module with `#[path = "engines/gp/mod.rs"] pub mod gp`, the crate-level path is `crate::gp`, not `crate::engines::gp`. This is not a real `engines` module.
- **Fix:** Replaced `crate::engines::gp::node::` with `super::node::` (relative within the module tree).
- **Files modified:** src/engines/gp/chromosome.rs, src/engines/gp/primitives.rs
- **Commit:** 6322881

**2. [Rule 1 - Bug] Deprecated `rand::Rng::gen_range` in rand 0.9**
- **Found during:** Task 1 (cargo check warning)
- **Issue:** `rng.gen_range(-1.0_f64..=1.0_f64)` triggers deprecation warning in rand 0.9; renamed to `random_range`.
- **Fix:** Changed to `rng.random_range(-1.0_f64..=1.0_f64)`.
- **Files modified:** src/engines/gp/primitives.rs
- **Commit:** 6322881

**3. [Rule 1 - Bug] Three clippy violations in new code**
- **Found during:** Task 1 (cargo clippy -- -D warnings)
- **Issues:**
  - `clippy::type_complexity` on `Option<Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>>`
  - `clippy::drain_collect` on `children.drain(..).collect()`
  - `clippy::extend_with_drain` on `worklist.extend(children.drain(..))`
- **Fix:** Extracted `TreeFitnessFn<N>` type alias; replaced drain+collect with `std::mem::take`; replaced extend+drain with `Vec::append`.
- **Files modified:** src/engines/gp/chromosome.rs, src/engines/gp/node.rs
- **Commit:** 6322881

**4. [Rule 2 - Missing critical functionality] GpConfiguration hard caps not in original action spec**
- **Found during:** Task 1 (threat model review)
- **Issue:** Threat model T-53-02 and T-53-03 require `max_depth <= 1000` and `max_node_count <= 100_000` hard caps. The task action spec only mentioned `max_depth > 0` and `max_node_count >= max_depth`.
- **Fix:** Added both hard caps to `GpConfiguration::build()` with descriptive error messages.
- **Files modified:** src/engines/gp/configuration.rs
- **Commit:** 6322881

## WASM Compatibility

All new code is pure type definitions with no `rayon` or `std::time::Instant` usage. WASM check passes. The only external dependency used is `rand::Rng` (in GpNode trait bounds) which is WASM-compatible.

## Self-Check: PASSED

Files exist:
- FOUND: src/engines/gp/mod.rs
- FOUND: src/engines/gp/node.rs
- FOUND: src/engines/gp/chromosome.rs
- FOUND: src/engines/gp/configuration.rs
- FOUND: src/engines/gp/primitives.rs
- FOUND: tests/gp.rs

Commits exist:
- FOUND: 6322881 (Task 1)
- FOUND: 1b9c6d8 (Task 2)
