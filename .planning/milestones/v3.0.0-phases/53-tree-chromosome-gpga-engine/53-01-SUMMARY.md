---
phase: 53-tree-chromosome-gpga-engine
plan: "01"
subsystem: engines/gp
tags:
  - genetic-programming
  - tree-chromosome
  - gp-node
  - gp-chromosome

dependency_graph:
  requires: []
  provides:
    - GpNode trait (src/engines/gp/node.rs)
    - Node<N> recursive enum with iterative Drop, depth(), node_count()
    - GpChromosome<N> implementing ChromosomeT (dna() panics, set_fitness_fn no-op)
    - TreeChromosome supertrait with tree(), tree_mut(), depth(), node_count()
    - GpChromosome Display as Lisp prefix S-expression
    - GpConfiguration shell with max_depth/max_node_count validation
    - GaError::TreeDepthExceeded / TreeSizeExceeded variants
    - GenerationStats::avg_node_count field (serde(default))
    - grow_tree / check_limits helpers in node.rs
    - primitives module (MathNode, BoolNode)
    - lib.rs re-export via pub mod gp
    - tests/gp.rs with 7 active tests + 8 ignored stubs
  affects:
    - src/engines/gp/mod.rs (new)
    - src/engines/gp/node.rs (new)
    - src/engines/gp/chromosome.rs (new)
    - src/engines/gp/configuration.rs (new)
    - src/engines/gp/primitives.rs (new)
    - src/error.rs (new variants)
    - src/stats.rs (avg_node_count field)
    - src/lib.rs (pub mod gp)
    - tests/gp.rs (new)

tech_stack:
  added: []
  patterns:
    - "Iterative Drop for Node<N> to avoid stack overflow on deep trees"
    - "TreeChromosome supertrait over ChromosomeT for GP-specific tree access"
    - "dna() panics on GpChromosome — GP chromosomes are not linear; tree() is the correct accessor"
    - "serde(default) on avg_node_count for backward-compatible stats extension"

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

key-decisions:
  - "dna() panics intentionally on GpChromosome — GP trees are not linear DNA sequences; callers must use tree()/tree_mut() instead"
  - "Iterative Drop implemented for Node<N> to prevent stack overflow when dropping deep trees (right-spine chains of 1000+ nodes)"
  - "TreeChromosome defined as a supertrait of ChromosomeT rather than a separate trait — GP engine can require TreeChromosome and get ChromosomeT bounds for free"
  - "avg_node_count added to GenerationStats with serde(default = 0.0) — backward-compatible; old checkpoints deserialize without error"

requirements-completed:
  - CHR-03
  - CHR-04

duration: 25min
completed: 2026-05-25
---

# Phase 53 Plan 01: GP Subsystem Core Types

**GP subsystem core types established: `GpNode` trait, `Node<N>` recursive enum with iterative Drop, `GpChromosome<N>` implementing `ChromosomeT`, `TreeChromosome` supertrait, `GpConfiguration`, primitives, and 7 active + 8 ignored tests in `tests/gp.rs`**

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | GpNode, Node<N>, GpChromosome, TreeChromosome, GpConfiguration, primitives | cdb9b2f | src/engines/gp/\* (new), src/lib.rs |
| 2 | GaError tree variants, GenerationStats avg_node_count, tests/gp.rs stubs | cdb9b2f | src/error.rs, src/stats.rs, tests/gp.rs |

## Decisions Made

1. **`dna()` panics on `GpChromosome`** — GP chromosomes are tree-structured, not linear. Calling `dna()` is a logic error; the intentional panic surfaces it at development time rather than silently returning an empty slice.

2. **Iterative `Drop` for `Node<N>`** — A recursive `Drop` impl would stack-overflow on deep trees (right-spine chains of depth 1000+). The iterative implementation uses an explicit `Vec` stack to traverse and drop nodes without OS stack growth.

3. **`TreeChromosome` as supertrait of `ChromosomeT`** — GP engines need both tree access (`tree()`, `depth()`) and the standard chromosome interface (`fitness()`, `age()`). Supertrait composition gives both bounds in one constraint.

4. **`avg_node_count` in `GenerationStats` with `serde(default)`** — Adds GP-relevant stats without breaking existing checkpoint files. Old checkpoints missing the field deserialize to `0.0`.

## Files Created/Modified

- `src/engines/gp/node.rs` — `GpNode` trait, `Node<N>` enum, iterative `Drop`, `depth()`, `node_count()`, `grow_tree()`, `check_limits()`, `Display` (Lisp S-expression prefix)
- `src/engines/gp/chromosome.rs` — `GpChromosome<N>`: `TreeChromosome` + `ChromosomeT` impls; `dna()` panic, `set_fitness_fn()` no-op, fitness/age fields
- `src/engines/gp/configuration.rs` — `GpConfiguration` shell with `max_depth`, `max_node_count` hard-cap validation
- `src/engines/gp/primitives.rs` — `MathNode` and `BoolNode` built-in node enums
- `src/engines/gp/mod.rs` — pub re-exports; wires submodules
- `src/error.rs` — `GaError::TreeDepthExceeded` and `GaError::TreeSizeExceeded` variants added
- `src/stats.rs` — `avg_node_count: f64` field with `#[serde(default)]`
- `src/lib.rs` — `pub mod gp` added via `#[path = "engines/gp/mod.rs"]`
- `tests/gp.rs` — 7 active tests (GpNode trait, Node construction, GpChromosome, MathNode, BoolNode) + 8 ignored stubs for Wave 2–4

## Self-Check: PASSED

- `cargo test --test gp`: 7 passed, 8 ignored
- `cargo test --features serde`: all serde tests pass (avg_node_count backward-compatible)
- `cargo check --target wasm32-unknown-unknown`: 0 errors
