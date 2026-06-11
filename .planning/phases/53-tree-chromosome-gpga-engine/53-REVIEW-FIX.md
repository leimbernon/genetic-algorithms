---
phase: 53-tree-chromosome-gpga-engine
fixed_at: 2026-05-25T00:00:00Z
review_path: .planning/phases/53-tree-chromosome-gpga-engine/53-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 53: Code Review Fix Report

**Fixed at:** 2026-05-25
**Source review:** `.planning/phases/53-tree-chromosome-gpga-engine/53-REVIEW.md`
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (3 Critical + 5 Warning)
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: SubtreeMutation corrupts chromosome on bloat-limit violation

**Files modified:** `src/engines/gp/mutation.rs`
**Commit:** clone-check-commit pattern for SubtreeMutation bloat guard
**Applied fix:** `subtree_mutation` now clones `chromosome.root` into a `candidate`, calls `replace_node_in_place` on the candidate, then calls `check_limits` on the candidate. Only if the check passes does `chromosome.root = candidate` execute. The original root is never modified until the limit check succeeds, preventing bloated chromosomes from leaking into the population.

---

### CR-02: MathNode::Var always evaluates to 0.0

**Files modified:** `src/engines/gp/primitives.rs`
**Commit:** add eval_with_vars helper so MathNode::Var injection works correctly
**Applied fix:** Added `Node::<MathNode>::eval_with_vars(&[f64]) -> f64` which recursively evaluates the tree and substitutes `vars[i]` for each `Var(i)` terminal. The `Var` evaluate arm now returns `0.0` explicitly (not the misleading `args.first().copied().unwrap_or(0.0)` which implied args could be non-empty). The `Var` doc comment is updated to reference `eval_with_vars` as the correct evaluation path for trees containing variable terminals.

Note: Var arity remains 0 (terminal). The `eval_with_vars` helper (Option B from review) was chosen over Option A (arity=1) because making Var a function-node would remove it from the terminal set, making it inaccessible via `grow_tree` and `sample_random_terminal`. The `eval_with_vars` approach is semantically correct without breaking the node categorization.

---

### CR-03: MathNode and BoolNode lack Default

**Files modified:** `src/engines/gp/primitives.rs`
**Commit:** implement Default for MathNode and BoolNode
**Applied fix:** Added manual `impl Default for MathNode { fn default() -> Self { MathNode::Const(0.0) } }` (manual impl required because `Const(f64)` is a tuple variant and `#[default]` only works on unit variants). Added `#[derive(Default)]` with `#[default]` on `BoolNode::And`. Both defaults are placeholder values used only to satisfy the `N: Default` bound on `GpGa<N>` and `GpChromosome<N>` — the engine always overwrites nodes before use.

---

### WR-01: max_generations = 0 not validated

**Files modified:** `src/engines/gp/configuration.rs`
**Commit:** validate max_generations > 0 and mutation probability range in GpConfiguration::build()
**Applied fix:** Added check for `self.max_generations == 0` in `GpConfiguration::build()` returning `ConfigurationError`. Updated doc comment for `build()` to list the new constraints.

---

### WR-02: Mutation probability not validated

**Files modified:** `src/engines/gp/configuration.rs`
**Commit:** validate max_generations > 0 and mutation probability range in GpConfiguration::build()
**Applied fix:** Added iteration over `self.mutations` in `build()` checking that each probability is finite and in `[0.0, 1.0]`. NaN, negative, or out-of-range values return `ConfigurationError` with the index and value.

---

### WR-03: Missing observer hook calls in GpGa

**Files modified:** `src/engines/gp/engine.rs`
**Commit:** add missing observer hook calls in GpGa::run()
**Applied fix:** Added the following hooks to `GpGa::run()`:
- `on_selection_complete(gen, elapsed, pairs.len())` — after parent selection
- `on_crossover_complete(gen, elapsed, offspring_count)` — after the crossover+mutation loop
- `on_mutation_complete(gen, elapsed, pop_size)` — after the crossover+mutation loop (same elapsed, per ga.rs convention)
- `on_fitness_evaluation_complete(gen, elapsed, pop_size)` — after offspring fitness evaluation
- `on_survivor_selection_complete(gen, elapsed, pop_size)` — after survivor selection
- `on_stagnation(gen, stagnation_count)` — when stagnation_count increments

Timing hooks use `Option<Instant>` with `cfg(not(target_arch = "wasm32"))` gating on `Instant::now()` calls (the `Instant` type annotation itself is available on WASM; only `::now()` is gated). A follow-up commit corrected the import to `use std::time::Instant` (unconditional) matching the `ga.rs` convention.

---

### WR-04: depth() and node_count() risk stack overflow

**Files modified:** `src/engines/gp/node.rs`
**Commit:** convert Node::depth() and node_count() to iterative implementations
**Applied fix:** Both methods converted from recursive to explicit-stack iterative traversal. `depth()` uses a `Vec<(&Node<N>, usize)>` stack tracking depth per node. `node_count()` uses a `Vec<&Node<N>>` stack counting nodes. Both follow the same worklist pattern as the existing iterative `Drop` implementation.

---

### WR-05: Dead field Replacer::old in crossover.rs

**Files modified:** `src/engines/gp/crossover.rs`
**Commit:** remove dead Replacer::old field from crossover.rs
**Applied fix:** Removed the `old: Option<Box<Node<N>>>` field from the `Replacer` struct and removed the `std::mem::replace` call that populated it. The replacement node is now assigned via `*node = replacement`, which drops the evicted subtree immediately. The `std::mem` import was not explicitly imported in this file (it was used via the full path `std::mem::replace`), so no import cleanup was needed.

---

## Verification

All fixes were verified:

```
cargo test --test gp  →  15 passed (1 suite)
cargo clippy -- -D warnings  →  No issues found
cargo check --target wasm32-unknown-unknown  →  Finished (0 errors)
```

---

_Fixed: 2026-05-25_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
