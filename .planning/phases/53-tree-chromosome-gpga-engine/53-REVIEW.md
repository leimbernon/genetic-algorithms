---
phase: 53-tree-chromosome-gpga-engine
reviewer: gsd-code-reviewer
depth: standard
reviewed: 2026-05-25T00:00:00Z
files_reviewed: 14
files_reviewed_list:
  - Cargo.toml
  - src/error.rs
  - src/stats.rs
  - src/lib.rs
  - src/engines/gp/mod.rs
  - src/engines/gp/node.rs
  - src/engines/gp/chromosome.rs
  - src/engines/gp/configuration.rs
  - src/engines/gp/primitives.rs
  - src/engines/gp/crossover.rs
  - src/engines/gp/mutation.rs
  - src/engines/gp/init.rs
  - src/engines/gp/engine.rs
  - tests/gp.rs
findings:
  critical: 3
  warning: 5
  info: 3
  total: 11
status: fixed
---

# Code Review — Phase 53: Tree Chromosome + GpGa Engine

**Reviewed:** 2026-05-25
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 53 delivers the GP subsystem: `GpNode` trait, `Node<N>` recursive tree, `GpChromosome<N>`, operators (`SubtreeCrossover`, `SubtreeMutation`, `PointMutation`, `HoistMutation`), `ramped_half_and_half`, the `GpGa` engine, and `serde_stacker` integration.

The architecture is sound. The iterative `Drop` implementation, WASM-gated rayon parallelism, bloat-limit design, and crossover index algebra are all correct. Three blockers prevent the code from shipping as-is: (1) `SubtreeMutation` corrupts chromosomes that violate bloat limits — the tree is modified before the check, so an error silently leaves the chromosome in a bloated state; (2) `MathNode::Var` always evaluates to `0.0` regardless of its index, making the type unusable for variable-injection without custom fitness-function workarounds; (3) `MathNode` and `BoolNode` do not implement `Default`, making `GpGa<MathNode>` and `GpGa<BoolNode>` uninstantiable despite being the only built-in node types.

---

## Critical Issues

### CR-01: SubtreeMutation corrupts chromosome on bloat-limit violation

**File:** `src/engines/gp/mutation.rs:119-123`

**Issue:** `subtree_mutation` replaces the target node in-place (line 120) *before* checking the bloat limits (line 123). When `check_limits` fails and returns `Err`, the chromosome root already contains the oversized subtree. The caller in `engine.rs` (lines 296-318) logs the error as a warning and continues — the bloated chromosome is pushed into offspring (line 321-322) and survives into the next generation. The hard depth/size limit is effectively not enforced for `SubtreeMutation`.

```rust
// CURRENT (buggy): mutate first, then check
replace_node_in_place(&mut chromosome.root, target, Box::new(new_subtree));
check_limits(&chromosome.root, max_depth, max_node_count)?;
```

**Fix:** Clone the chromosome root before replacement, check limits on the clone, and only commit if the check passes:

```rust
fn subtree_mutation<N: GpNode + Clone>(
    chromosome: &mut GpChromosome<N>,
    max_depth: usize,
    max_node_count: usize,
    mutation_max_depth: usize,
    rng: &mut impl Rng,
) -> Result<(), GaError> {
    let n = chromosome.root.node_count();
    let target = rng.random_range(0..n);
    let new_subtree = grow_tree::<N>(mutation_max_depth, rng);

    // Work on a clone; only commit if limits are satisfied.
    let mut candidate = chromosome.root.clone();
    replace_node_in_place(&mut candidate, target, Box::new(new_subtree));
    check_limits(&candidate, max_depth, max_node_count)?;
    chromosome.root = candidate;
    Ok(())
}
```

---

### CR-02: MathNode::Var always evaluates to 0.0 — variable injection is broken

**File:** `src/engines/gp/primitives.rs:83`

**Issue:** `Var` has arity 0, so the engine guarantees `args.len() == 0` when calling `evaluate`. `args.first().copied().unwrap_or(0.0)` therefore always returns `0.0` regardless of the index stored in `Var(usize)`. Any tree containing `Var(i)` nodes will silently produce incorrect results when the user relies on `GpNode::evaluate` for tree evaluation. The index field is completely ignored.

The comment on line 81-82 states "the GpGa fitness_fn is responsible for injecting variable values" — but there is no engine mechanism to inject per-node variable values during tree traversal, and no documented pattern for doing so. Users who use `MathNode` for symbolic regression (the stated purpose) will get wrong answers.

**Fix (Option A — document the constraint clearly):** Remove `Var` from `MathNode` and document that variable injection requires a custom `GpNode` implementation with context passed via the fitness function closure. This avoids false promises.

**Fix (Option B — provide a tree-walk evaluate helper):** Add a public `Node::eval_with_vars(vars: &[f64]) -> f64` method that recursively evaluates the tree and injects values for `Var(i)` nodes from the `vars` slice:

```rust
impl Node<MathNode> {
    pub fn eval_with_vars(&self, vars: &[f64]) -> f64 {
        match self {
            Node::Terminal(MathNode::Var(i)) => {
                vars.get(*i).copied().unwrap_or(0.0)
            }
            Node::Terminal(other) => other.evaluate(&[]),
            Node::Function { value, children } => {
                let child_vals: Vec<f64> =
                    children.iter().map(|c| c.eval_with_vars(vars)).collect();
                value.evaluate(&child_vals)
            }
        }
    }
}
```

Either option is acceptable; the current code must not ship in its current form since `MathNode` is advertised as suitable for symbolic regression with variables.

---

### CR-03: MathNode and BoolNode do not implement Default — GpGa<MathNode> is uninstantiable

**File:** `src/engines/gp/primitives.rs:30-45`, `src/engines/gp/engine.rs:95`

**Issue:** `GpGa<N>` has the trait bound `N: GpNode + Default + Clone + Send + Sync + 'static`. `MathNode` and `BoolNode` do not derive or implement `Default`. Any user who follows the documented primary constructor (`GpGa::with_ramped_half_and_half`) with either built-in node type will get a compile error. This also affects `GpChromosome<N>` (`Default for GpChromosome<N>` is gated on `N: Default`), and the doc example in `engine.rs` (line 83) shows `MathNode` being used with `GpGa` — which would fail to compile if the `ignore` attribute were removed.

**Fix:** Add `Default` derives to both built-in node types with sensible defaults:

```rust
// MathNode — sensible default is a zero constant
#[derive(Clone, Debug, Default)]
pub enum MathNode {
    Add, Sub, Mul, ProtectedDiv,
    #[default]
    Const(f64),   // Default::default() = Const(0.0)
    Var(usize),
}

// BoolNode — sensible default is And (arity 2 matches most contexts)
#[derive(Clone, Debug, Default)]
pub enum BoolNode {
    #[default]
    And,
    Or, Not, Gt, Lt,
}
```

Note: `f64` does not implement `Default` in a meaningful way for `Const`; use `#[default]` on the variant to specify `Const(0.0)` rather than relying on field defaults.

---

## Warnings

### WR-01: max_generations = 0 not validated — silent zero-generation run

**File:** `src/engines/gp/configuration.rs:277-323`

**Issue:** `GpConfiguration::build()` validates `max_depth`, `max_node_count`, `init_max_depth`, `population_size`, and `mutations`, but does not validate `max_generations > 0`. When `max_generations == 0`, `GpGa::run` silently skips the main loop entirely (line 244: `for gen in 0..0`), evaluates the initial population, then immediately returns a `GpResult` with `generations: 0`. The `best` chromosome is correctly set from the initial population, but the user gets no warning that their configuration produced a no-op run.

**Fix:**

```rust
if self.max_generations == 0 {
    return Err(GaError::ConfigurationError(
        "max_generations must be greater than 0".to_string(),
    ));
}
```

---

### WR-02: Mutation probability values are not validated

**File:** `src/engines/gp/configuration.rs:317-320`

**Issue:** `build()` checks that the `mutations` list is non-empty but does not validate that each probability `p` satisfies `0.0 <= p <= 1.0`. A negative probability or `NaN` will silently pass validation, produce incorrect stochastic behavior (`rng.random::<f64>() < NaN` is always false; negative probability means mutations never apply), and produce no error or warning at runtime.

**Fix:**

```rust
for (i, (_, prob)) in self.mutations.iter().enumerate() {
    if !prob.is_finite() || *prob < 0.0 || *prob > 1.0 {
        return Err(GaError::ConfigurationError(format!(
            "mutations[{}]: probability {} is not in [0.0, 1.0]",
            i, prob
        )));
    }
}
```

---

### WR-03: Observer hooks missing — on_stagnation, on_selection_complete, on_crossover_complete, on_mutation_complete, on_fitness_evaluation_complete, on_survivor_selection_complete

**File:** `src/engines/gp/engine.rs`

**Issue:** The `GaObserver` trait defines 12 lifecycle hooks. `GpGa` calls only 4: `on_run_start`, `on_generation_start`, `on_new_best`, `on_generation_end`, `on_run_end`. The following hooks are defined and called by the standard `Ga` engine but are never called by `GpGa`:

- `on_stagnation` (line 349: stagnation_count increments but no hook fires)
- `on_selection_complete`
- `on_crossover_complete`
- `on_mutation_complete`
- `on_fitness_evaluation_complete`
- `on_survivor_selection_complete`

CLAUDE.md (Observability initiative): "All changes to the GA execution flow must preserve observability hooks. The `GaObserver` trait is the foundation — never remove or bypass observer notification points."

Users attaching observers that use these hooks (e.g., `MetricsObserver`, `TracingObserver`) will get silent gaps in their telemetry for GP runs.

**Fix:** Add the missing `notify` calls at the appropriate points in the generation loop. At minimum, `on_stagnation` must fire when `stagnation_count` increments (line ~349):

```rust
} else {
    stagnation_count += 1;
    let sc = stagnation_count;
    self.notify(|obs| obs.on_stagnation(gen, sc));
}
```

The timing-based hooks (`on_selection_complete`, etc.) require `Instant` — these must be gated with `#[cfg(not(target_arch = "wasm32"))]` per project WASM policy if added.

---

### WR-04: depth() and node_count() are recursive — stack overflow on trees exceeding max_depth

**File:** `src/engines/gp/node.rs:134-157`

**Issue:** `Node::depth()` and `Node::node_count()` use recursive descent. The custom iterative `Drop` implementation correctly avoids stack overflow when dropping trees, but these two traversal methods are still recursive. For a tree at `max_depth = 1000` (the hard cap enforced by `build()`), a right-spine tree of depth 1000 would require a recursion depth of 1000 calls to `depth()`. Default stack sizes on Linux (8 MB) can handle ~10,000–100,000 recursive Rust frames, so depth 1000 is safe in practice. However, the same logic is used in `check_limits` *after* crossover/mutation, which means even before limits are enforced these recursive calls could blow the stack if a pre-check tree exceeds the cap due to crossover with seeds that produce very deep intermediate trees.

This is a latent risk rather than an immediate crash, but worth documenting. The iterative Drop pattern already in the file should be extended to these methods.

**Fix:** Convert `depth()` and `node_count()` to iterative implementations using an explicit stack (similar to the `Drop` worklist pattern already in node.rs), or document the stack-safety boundary explicitly for users setting `max_depth` close to 1000.

---

### WR-05: Replacer::old field is set but never read — dead code

**File:** `src/engines/gp/crossover.rs:113, 138`

**Issue:** The `Replacer` struct has an `old: Option<Box<Node<N>>>` field that is populated with the evicted subtree during `run()` (line 138: `self.old = Some(old)`) but is never read. The subtrees are cloned upfront via `clone_subtree_at_index` (lines 176-179), making the `old` capture redundant. This dead code wastes allocation for every crossover.

**Fix:** Remove the `old` field from `Replacer` and drop the `let old = mem::replace(...)` / `self.old = Some(old)` lines. The `std::mem::replace` can be replaced by a direct assignment:

```rust
if my_index == self.target {
    let replacement = self.replacement.take().unwrap();
    *node = replacement;  // old node is dropped here — no need to store it
    return;
}
```

---

## Info

### IN-01: MathNode::Var evaluate comment contradicts the GpNode trait contract

**File:** `src/engines/gp/primitives.rs:81-83`

**Issue:** The comment says "the GpGa fitness_fn is responsible for injecting variable values." This contradicts the `GpNode::evaluate` docstring which states "args.len() equals self.arity()" — for `Var` (arity 0), args is always empty. There is no engine mechanism to inject values into `evaluate`, so the comment describes a pattern that does not exist in the engine. After CR-02 is resolved, this comment should be updated to describe the actual pattern.

---

### IN-02: doc example in engine.rs uses MathNode without Default — misleading after fix

**File:** `src/engines/gp/engine.rs:82-94`

**Issue:** The `GpGa` struct doc example (marked `ignore`) shows `MathNode` used with `GpGa`. After CR-03 is fixed by adding `Default` to `MathNode`, this example should be updated from `ignore` to a compilable doc test to verify correctness.

---

### IN-03: Configuration validation does not validate mutation_max_depth > 0

**File:** `src/engines/gp/configuration.rs:277-323`

**Issue:** `SubtreeMutation { mutation_max_depth: 0 }` passes `build()` validation. With `mutation_max_depth = 0`, `grow_tree(0, rng)` produces a terminal node (this is the defined safe behavior in `grow_tree`), effectively making subtree mutation always produce a single terminal replacement. This is not wrong per se, but it is probably not what the user intends. A validation note or minimum constraint (e.g., `mutation_max_depth >= 1`) would prevent silent misconfiguration.

---

## Verdict

**REVISE**

Three blockers must be resolved before merge:

1. **CR-01** — SubtreeMutation leaves chromosomes in a bloated state on limit violation; the chromosome must be modified only after the check passes (clone-check-commit pattern).
2. **CR-02** — `MathNode::Var` is non-functional for its documented purpose; fix requires either removing the variant or providing a tree-walk evaluation helper with variable injection.
3. **CR-03** — `MathNode` and `BoolNode` lack `Default`, making `GpGa` unusable with the two built-in node types the module exports as ready-to-use.

WR-01 through WR-05 should be addressed before merge. WR-03 (observer hooks) is required by project policy (CLAUDE.md Observability initiative).

---

_Reviewed: 2026-05-25_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
