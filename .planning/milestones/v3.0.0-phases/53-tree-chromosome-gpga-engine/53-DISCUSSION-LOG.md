# Phase 53: Tree Chromosome + GpGa Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 53-tree-chromosome-gpga-engine
**Areas discussed:** GpNode API, Operator placement, GpGa engine scope, Expression display

---

## GpNode API

### Q1: How does the user define their GP primitive and terminal sets?

| Option | Description | Selected |
|--------|-------------|----------|
| Trait-on-user-enum | User defines an enum and implements `GpNode` trait: `fn arity()`, `fn evaluate()`, `fn is_terminal()`. Full type safety, zero allocations. Same pattern as GeneT. | ✓ |
| PrimitiveSet builder | User calls `PrimitiveSet::new().add_function(...)`. Library owns GpNode type internally. Less boilerplate but box-allocated closures. | |
| Hybrid: built-in GpNode + user extension point | Library provides `GpNode<T>` with standard built-ins. User supplies T via associated type. | |

**User's choice:** Trait-on-user-enum
**Notes:** Consistent with GeneT pattern; zero allocations; full user control.

---

### Q2: How do ephemeral random constants (ERCs) fit in?

| Option | Description | Selected |
|--------|-------------|----------|
| User-owned terminal variant | User includes `Const(f64)` in their enum. `GpNode` trait gets `fn sample_random_terminal(rng: &mut impl Rng) -> Self`. Engine calls it during init and mutation. | ✓ |
| ERC marker + library sampling | User marks a terminal with `fn is_erc() -> bool`; engine calls `sample_random_terminal()` only when `is_erc()` is true. | |
| You decide | Defer to Claude. | |

**User's choice:** User-owned terminal variant
**Notes:** No special ERC type needed; user controls sampling completely.

---

### Q3: What does GpChromosome look like?

| Option | Description | Selected |
|--------|-------------|----------|
| GpChromosome<N: GpNode> as library's concrete type | Library ships `GpChromosome<N>` implementing `TreeChromosome + ChromosomeT`. Users instantiate `GpChromosome<MyNode>`. Same pattern as BinaryChromosome / RangeChromosome. | ✓ |
| TreeChromosome as trait only; user provides struct | Library defines `TreeChromosome: ChromosomeT` trait only; user implements both GpNode and their own chromosome struct. | |

**User's choice:** GpChromosome<N: GpNode> as library's concrete type
**Notes:** Consistent with existing chromosome pattern; minimal user boilerplate.

---

## Operator Placement

### Q1: Where do tree-specific operators live?

| Option | Description | Selected |
|--------|-------------|----------|
| GpConfiguration owns tree operators directly | GpConfiguration has `crossover: GpCrossover` and `mutations: Vec<GpMutation>`. Never in main Crossover/Mutation enums. Clean separation. | ✓ |
| Add GP variants to main Crossover/Mutation enums | Add `Crossover::SubtreeCrossover`, `Mutation::SubtreeMutation` etc. Consistent pattern but pollutes enums for non-GP users. | |
| You decide | Defer to Claude. | |

**User's choice:** GpConfiguration owns tree operators directly
**Notes:** Avoids dead cases in main operator enums for non-GP users.

---

### Q2: Which tree mutation operators should be in Phase 53?

| Option | Description | Selected |
|--------|-------------|----------|
| All three: subtree, point, hoist | SubtreeMutation, PointMutation, HoistMutation — canonical three from GP literature. Each with its own probability. | ✓ |
| Subtree mutation only | Only SubtreeMutation (most common). Point and hoist added later. | |
| Subtree + point only | Skip hoist; max_depth/max_node_count handles bloat. | |

**User's choice:** All three: subtree, point, hoist
**Notes:** Complete canonical GP mutation set; hoist is a bloat-reduction tool that complements the hard limits.

---

## GpGa Engine Scope

### Q1: Does GpGa reuse standard Selection and Survivor enums?

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse Selection + Survivor enums | Selection and Survivor work on scalar `fitness()` — don't touch DNA. GpConfiguration embeds SelectionConfiguration and SurvivorConfiguration. | ✓ |
| GpGa defines its own selection/survivor logic | Implement tournament selection directly inside GpGa. Simpler but duplicates logic. | |
| You decide | Defer to Claude. | |

**User's choice:** Reuse Selection + Survivor enums
**Notes:** Fitness-based operators work on any ChromosomeT; no duplication needed.

---

### Q2: Does GpGa fire GaObserver<U> or need a GpObserver sub-trait?

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse GaObserver<U> directly | Standard hooks (on_start, on_generation_end, on_new_best, on_finish). No GP-specific events in Phase 53. | ✓ |
| Add GpObserver sub-trait now | GpObserver<U>: GaObserver<U> with on_bloat_detected, on_tree_depth_exceeded. Adds scope. | |

**User's choice:** Reuse GaObserver<U> directly
**Notes:** GpObserver deferred; consistent with Phase 30 observer-wiring pattern for alt-engines.

---

### Q3: What GenerationStats additions does GpGa need?

| Option | Description | Selected |
|--------|-------------|----------|
| avg_node_count + avg_depth (both) | CHR-05 required: avg_node_count. avg_depth adds diagnostic value at minimal cost. | |
| avg_node_count only (minimal) | Only the CHR-05 required field. avg_depth inferrable from Display. | ✓ |
| You decide | Defer to Claude. | |

**User's choice:** avg_node_count only
**Notes:** Minimum viable stats; avg_depth can be added later if users request it.

---

## Expression Display

### Q1: What expression format does GpChromosome's Display produce?

| Option | Description | Selected |
|--------|-------------|----------|
| Lisp/prefix S-expression | `(+ (* x 3) 2)`. Standard in GP literature, unambiguous, simple recursive walk. | ✓ |
| Infix with parentheses | `((x * 3) + 2)`. More natural but requires operator-precedence logic; boolean ops look odd. | |
| Both — prefix default, infix via wrapper | `Display` as prefix; add `to_infix()` separately. More scope. | |

**User's choice:** Lisp/prefix S-expression
**Notes:** Standard in GP literature; simplest Display implementation; avoids precedence logic.

---

## Claude's Discretion

None — all areas had clear user choices.

## Deferred Ideas

- `GpObserver` sub-trait with `on_bloat_detected` and `on_tree_depth_exceeded` — GP-specific observability for a future phase
- `avg_depth: f64` in `GenerationStats` — can be added when users request it
- Infix expression display — can be added as `.to_infix()` helper in a future phase
- `fn all_non_terminals() -> Vec<Self>` on `GpNode` for PointMutation — researcher should evaluate the right API (noted as open design question in specifics)
