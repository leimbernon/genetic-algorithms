# Phase 53: Tree Chromosome + GpGa Engine — Research

**Researched:** 2026-05-25
**Domain:** Genetic Programming — recursive tree data structures, GP operators, dedicated GA engine
**Confidence:** HIGH (all core decisions verified from codebase; GP algorithms verified from authoritative references)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `GpNode` trait on user enum (like `GeneT`): `fn arity(&self) -> usize`, `fn evaluate(&self, args: &[f64]) -> f64`, `fn is_terminal(&self) -> bool`, `fn sample_random_terminal(rng: &mut impl Rng) -> Self`
- **D-02:** ERCs are user-owned terminal variants; `sample_random_terminal` is the factory
- **D-03:** `GpChromosome<N: GpNode>` is the library's concrete type (like `BinaryChromosome`), stores `Box<Node<N>>`
- **D-04:** `TreeChromosome: ChromosomeT` is NOT a subtrait of `LinearChromosome`; `GpChromosome` must NOT implement `dna()` / `set_dna()` / `set_fitness_fn()`
- **D-05:** GP operators live in `GpConfiguration` only — NOT in main `Crossover`/`Mutation` enums
- **D-06:** `GpCrossover` enum with one variant: `SubtreeCrossover` — returns `GaError::TreeDepthExceeded` / `GaError::TreeSizeExceeded` on violations
- **D-07:** `GpMutation` enum with three variants: `SubtreeMutation`, `PointMutation`, `HoistMutation`
- **D-08:** Each `GpMutation` variant carries its own application probability
- **D-09:** `GpGa<U: TreeChromosome>` in `src/engines/gp/` — does NOT share code with `Ga<U: LinearChromosome>`
- **D-10:** `GpGa` reuses standard `Selection` enum and `Survivor` enum unchanged
- **D-11:** `GpGa` fires standard `GaObserver<U>` hooks: `on_run_start`, `on_generation_start`, `on_generation_end`, `on_new_best`, `on_run_end` (no `GpObserver` sub-trait in Phase 53)
- **D-12:** `avg_node_count: f64` added to `GenerationStats`; `avg_depth` is NOT added
- **D-13:** `serde_stacker` gated on existing `serde` feature flag; verify wasm32 compat first
- **D-14:** CI serde test with tree of depth >= 64 required
- **D-15:** `Display` for `GpChromosome<N>` — Lisp/prefix S-expression format: `(+ (* x 3) 2)`

### Claude's Discretion
- `fn all_functions() -> Vec<Self>` (or equivalent) API on `GpNode` for `PointMutation` — researcher should evaluate the right approach

### Deferred Ideas (OUT OF SCOPE)
- `GpObserver` sub-trait with `on_bloat_detected` / `on_tree_depth_exceeded`
- `avg_depth: f64` in `GenerationStats`
- Strongly-typed GP
- Grammar-guided / grammatical evolution
- Infix expression display
- `fn all_non_terminals() -> Vec<Self>` on `GpNode` (deferred from context — researcher to recommend the right API)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CHR-03 | User can define tree-structured chromosome by implementing `TreeChromosome: ChromosomeT` — separate from `LinearChromosome`; tree operators operate against this trait | §ChromosomeT Audit, §Architecture Patterns: TreeChromosome, §Standard Stack |
| CHR-04 | User can run GP optimization via `GpGa<U: TreeChromosome>` with ramped half-and-half init, configurable primitive set, and ERCs as terminals | §GP Algorithm Patterns, §GpGa Engine Structure, §Ramped Half-and-Half |
| CHR-05 | `max_depth` and `max_node_count` enforced post-crossover and post-mutation; violations return `GaError::TreeDepthExceeded` / `GaError::TreeSizeExceeded`; `avg_node_count` in `GenerationStats` | §GaError Additions, §GenerationStats Additions, §Bloat Control |
| CHR-06 | `serde` feature enables checkpoint/restore; `serde_stacker` prevents stack overflow on depth >= 64; CI test required | §Serde / Deep Trees, §Package Legitimacy Audit |
| CHR-07 | `GpChromosome` implements `Display` as Lisp/prefix S-expression | §Expression Display |
</phase_requirements>

---

## Summary

Phase 53 introduces a complete Genetic Programming subsystem as a self-contained module at `src/engines/gp/`. The implementation follows the codebase's established pattern: a user-facing trait (`GpNode`, analogous to `GeneT`), a library concrete chromosome type (`GpChromosome<N>`, analogous to `BinaryChromosome`), and a dedicated engine (`GpGa`, analogous to `DeEngine` or `CellularEngine`).

The most critical architectural constraint is that `ChromosomeT` currently enforces flat-slice methods (`dna()`, `set_dna()`, `set_fitness_fn()`) that have no meaning for a tree. Phase 47 (the `LinearChromosome` split) has not yet merged, so the Phase 53 plan must introduce `TreeChromosome: ChromosomeT` as a parallel supertrait that declares tree-specific methods while `GpChromosome<N>` implements only the subset of `ChromosomeT` that applies to trees — with stub implementations for the linear-slice methods that panic or are hidden behind the `TreeChromosome` trait boundary. This is the critical design tension for Wave 0.

The `serde_stacker` crate is authored by dtolnay and is legitimate. Its `stacker` dependency explicitly documents that wasm32 is unsupported because the WebAssembly callstack is not in a manipulatable address space — **but stacker fails open**: on unsupported platforms it compiles and runs as a no-op without panicking. This means `serde_stacker` will compile for wasm32 but will not actually grow the stack. For wasm32 users serializing deep trees, the library must document that serde of very deep trees may overflow the wasm32 stack (same limit as any recursive algorithm on wasm). This is acceptable because the CI wasm check only runs `cargo check --target wasm32-unknown-unknown --lib` (not integration tests) and the serde feature is already excluded from the wasm32 check workflow.

**Primary recommendation:** Implement Phase 53 in four waves: Wave 0 (types, traits, `GpChromosome` shell + `TreeChromosome` trait), Wave 1 (GP operators: crossover + mutations), Wave 2 (`GpGa` engine loop + ramped half-and-half init + bloat control), Wave 3 (serde, `Display`, `avg_node_count` stats, CI test).

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| GP tree representation | `src/engines/gp/chromosome.rs` | `src/engines/gp/node.rs` | GpChromosome<N> is the concrete type; Node<N> is the internal recursive enum |
| GpNode user trait | `src/engines/gp/node.rs` | — | User-facing trait separate from ChromosomeT machinery |
| Subtree crossover | `src/engines/gp/crossover.rs` | — | GP-specific; must not appear in `src/operations/crossover/` |
| GP mutations (3 variants) | `src/engines/gp/mutation.rs` | — | GP-specific; must not appear in `src/operations/mutation/` |
| Ramped half-and-half init | `src/engines/gp/init.rs` | — | Used only by GpGa, not reusable for linear chromosomes |
| GpGa engine loop | `src/engines/gp/engine.rs` | — | Separate from `ga.rs`; follows DeEngine pattern |
| GpConfiguration | `src/engines/gp/configuration.rs` | — | Embeds SelectionConfiguration + SurvivorConfiguration sub-configs |
| Observer hooks | `src/engines/gp/engine.rs` | `src/observe/observer/mod.rs` | GpGa calls standard GaObserver<U> hooks via notify() helper |
| GenerationStats | `src/stats.rs` | — | Add avg_node_count field here (shared with all engines) |
| GaError variants | `src/error.rs` | — | Add TreeDepthExceeded, TreeSizeExceeded here |
| Serde support | `src/engines/gp/chromosome.rs` | `Cargo.toml` | Gated behind existing `serde` feature flag; serde_stacker as conditional dep |
| Display/S-expression | `src/engines/gp/chromosome.rs` | — | impl fmt::Display for GpChromosome<N> |
| Public API exports | `src/lib.rs` | — | Add `pub mod gp;` with #[path = "engines/gp/mod.rs"] |

---

## Standard Stack

### Core (no new dependencies for default feature)
| Crate | Version | Purpose | Why Standard |
|-------|---------|---------|--------------|
| `rand` | 0.9.2 (already in Cargo.toml) | RNG for random subtree selection, mutation point selection, terminal sampling | Already a dependency; `make_rng()` is the project's RNG entry point |

### Conditional (serde feature only)
| Crate | Version | Purpose | Why This |
|-------|---------|---------|---------|
| `serde_stacker` | 0.1.14 | Serde adapter that prevents stack overflow on deeply recursive structures via dynamic stack growth | Official dtolnay crate; used by projects like syn; no alternative that's simpler for recursive serde |

### Supporting (already present)
| Crate | Purpose | Already In Cargo.toml |
|-------|---------|----------------------|
| `serde` | Derive macros for checkpoint support | Yes, optional feature |
| `log` | `info!(target="ga_events", ...)` for engine events | Yes |
| `rayon` | `par_iter()` for parallel fitness evaluation (gated with cfg) | Yes |

**Installation (serde_stacker only):**
```toml
# In Cargo.toml [dependencies]:
serde_stacker = { version = "0.1.14", optional = true }

# In [features]:
serde = ["dep:serde", "dep:serde_json", "dep:serde_stacker"]
```

**Version verification:** [VERIFIED: crates.io registry] — `cargo info serde_stacker` returns version 0.1.14, published by dtolnay.

---

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| `serde_stacker` | crates.io | ~4 yrs | High (dtolnay ecosystem) | github.com/dtolnay/serde-stacker | N/A — slopcheck unavailable | [ASSUMED] — verified by official crates.io registry; dtolnay is the author of serde, serde_json, syn, quote; high-trust provenance |

**slopcheck status:** slopcheck could not be installed in this environment. All packages tagged `[ASSUMED]` above.
**Packages removed due to [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none
**Note:** `serde_stacker` is a dtolnay crate in the official serde ecosystem. The planner should include a `checkpoint:human-verify` step before adding it to Cargo.toml per protocol, but the risk is extremely low given authorship.

---

## Architecture Patterns

### System Architecture Diagram

```
User defines MyNode enum → implements GpNode
                            ↓
                   GpChromosome<MyNode> (library concrete type)
                   - root: Box<Node<MyNode>>
                   - fitness: f64
                   - age: usize
                   implements TreeChromosome: ChromosomeT
                            ↓
                   GpGa<GpChromosome<MyNode>>.run()
                            ↓
           ┌────────────────────────────────────┐
           │  Ramped half-and-half init (Wave 2) │
           │  → grow_tree() / full_tree()        │
           └────────────────┬───────────────────┘
                            │ initial population
                            ▼
           ┌────────────────────────────────────────────────┐
           │  Generation loop                                │
           │  1. on_generation_start hook                   │
           │  2. Selection::factory() → parent pairs        │
           │  3. GpCrossover::SubtreeCrossover              │
           │     → bloat check → GaError if exceeded        │
           │  4. GpMutation (per-variant probability roll)  │
           │     SubtreeMutation / PointMutation / Hoist    │
           │     → bloat check → GaError if exceeded        │
           │  5. fitness_fn(tree) → set_fitness()           │
           │  6. Survivor::factory()                        │
           │  7. best tracking → on_new_best hook           │
           │  8. GenerationStats (incl. avg_node_count)     │
           │  9. on_generation_end hook                     │
           │ 10. stopping criteria check                    │
           └────────────────────────────────────────────────┘
                            │
                   on_run_end hook
                   GpResult<U> { population, best, generations }
```

### Recommended Project Structure
```
src/engines/gp/
├── mod.rs              # pub re-exports: GpGa, GpConfiguration, GpNode, GpChromosome, etc.
├── node.rs             # GpNode trait definition + Node<N> recursive enum
├── chromosome.rs       # GpChromosome<N> impl + TreeChromosome trait + Display
├── engine.rs           # GpGa<U: TreeChromosome> engine loop + GpResult
├── configuration.rs    # GpConfiguration struct + builder methods
├── crossover.rs        # GpCrossover enum + SubtreeCrossover impl
├── mutation.rs         # GpMutation enum + SubtreeMutation/PointMutation/HoistMutation
└── init.rs             # ramped_half_and_half(), grow_tree(), full_tree()

src/
├── error.rs            # ADD: TreeDepthExceeded, TreeSizeExceeded
├── stats.rs            # ADD: avg_node_count: f64 to GenerationStats
└── lib.rs              # ADD: #[path = "engines/gp/mod.rs"] pub mod gp;

tests/engines/gp/
├── test_gp_chromosome.rs      # GpChromosome unit tests: depth, node_count, Display
├── test_gp_crossover.rs       # SubtreeCrossover: depth/size limits, valid offspring
├── test_gp_mutation.rs        # All three mutations: shrink, arity, valid offspring
├── test_gp_init.rs            # Ramped half-and-half: population structure
├── test_gp_engine.rs          # GpGa end-to-end: convergence, observer hooks
└── test_gp_serde.rs           # #[cfg(feature = "serde")] depth-64 tree round-trip
```

### Pattern 1: GpNode Trait (analogous to GeneT)
**What:** User-implemented trait on their own enum to define the GP primitive set
**When to use:** Always — this is the user's entry point for GP
**Example:**
```rust
// Source: CONTEXT.md §D-01 + GeneT pattern from src/traits/gene.rs
use rand::Rng;

pub trait GpNode: Clone + Send + Sync + 'static {
    /// Number of arguments this node takes (0 = terminal, >0 = function).
    fn arity(&self) -> usize;
    /// Evaluate this node given its computed child arguments.
    fn evaluate(&self, args: &[f64]) -> f64;
    /// Returns true if this node is a terminal (leaf).
    fn is_terminal(&self) -> bool { self.arity() == 0 }
    /// Produce a fresh terminal (leaf) node, possibly with random value.
    /// Used for initialization and mutation.
    fn sample_random_terminal(rng: &mut impl Rng) -> Self;
    /// Return all function (non-terminal) variants of the primitive set.
    /// Used by PointMutation to find same-arity replacements.
    fn all_functions() -> Vec<Self>;
}
```

**Note on `all_functions()`:** This is the recommended API for PointMutation (see §PointMutation section). It is a static method returning all non-terminal primitives. The engine filters by `arity() == node.arity()` to find valid replacements. If the user returns an empty vec or no same-arity alternative exists, PointMutation silently skips that node. This is simpler than a `fn with_arity(usize) -> Vec<Self>` approach because the caller always needs to filter anyway.

### Pattern 2: Node<N> Recursive Enum (internal tree representation)
**What:** Library-internal recursive tree node type — users never touch this directly
**When to use:** All internal GP tree operations
**Example:**
```rust
// Source: CONTEXT.md §D-03 (Box<N> for tree nodes, locked in STATE.md)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Node<N: GpNode> {
    /// A non-terminal (function) node with `N` children.
    Function {
        value: N,
        children: Vec<Box<Node<N>>>,
    },
    /// A terminal (leaf) node with no children.
    Terminal(N),
}

impl<N: GpNode> Node<N> {
    /// Returns the depth of the subtree rooted at this node.
    pub fn depth(&self) -> usize {
        match self {
            Node::Terminal(_) => 1,
            Node::Function { children, .. } => {
                1 + children.iter().map(|c| c.depth()).max().unwrap_or(0)
            }
        }
    }

    /// Returns the total node count of the subtree.
    pub fn node_count(&self) -> usize {
        match self {
            Node::Terminal(_) => 1,
            Node::Function { children, .. } => {
                1 + children.iter().map(|c| c.node_count()).sum::<usize>()
            }
        }
    }
}
```

**Stack overflow risk:** Recursive `depth()`, `node_count()`, `evaluate()`, and `Display` all use O(depth) stack. At depth 64 (CI requirement), stack usage is negligible (64 frames). At depth >= ~5000–10000 (default thread stack ~8MB, each frame ~hundreds of bytes), stack overflow becomes a risk. Since `max_depth` in `GpConfiguration` is user-controlled, recommend a hard upper limit in validation (e.g., `max_depth <= 1000`) with documentation. The CI depth-64 test exercises the serde path specifically.

**Iterative Drop:** The auto-generated `drop` for deeply nested `Box<Node<N>>` is recursive and can overflow at extreme depths. Implement a custom `Drop` for `Node<N>` that uses an explicit work queue:
```rust
// Source: matklad.github.io/2022/11/18/if-a-tree-falls-in-a-forest-does-it-overflow-the-stack.html
impl<N: GpNode> Drop for Node<N> {
    fn drop(&mut self) {
        let mut stack: Vec<Box<Node<N>>> = Vec::new();
        // Drain children into the stack to avoid recursive drop
        if let Node::Function { ref mut children, .. } = self {
            stack.extend(children.drain(..));
        }
        while let Some(mut node) = stack.pop() {
            if let Node::Function { ref mut children, .. } = *node {
                stack.extend(children.drain(..));
            }
            // `node` is dropped here with no children (children already drained)
        }
    }
}
```

### Pattern 3: GpChromosome<N> implementing TreeChromosome: ChromosomeT
**What:** Library concrete chromosome type. Implements `ChromosomeT` but not `LinearChromosome`.
**Critical:** `ChromosomeT` currently mandates `dna()`, `set_dna()`, `dna_mut()`, `set_fitness_fn()` — methods that don't apply to trees. Since Phase 47 (the split) has not merged, `GpChromosome` must provide stub implementations:
- `dna()` → `panic!("GpChromosome is a tree chromosome; use tree() instead")`
- `set_dna()` → `panic!("...")`
- `dna_mut()` → `panic!("...")`
- `set_fitness_fn()` → store the fn but call it differently

**Recommended approach:** Store fitness as `f64` and the user's fitness function as `Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>`. For `calculate_fitness()`, call the tree-based fn. The `Gene` associated type can be a `PhantomData` marker or a unit struct — something that satisfies the `GeneT` bound without ever appearing in a slice.

```rust
// Source: codebase analysis of ChromosomeT in src/traits/chromosome.rs
/// Marker gene type for GpChromosome — never stored in a slice.
#[derive(Clone, Default)]
pub struct GpGene;
impl GeneT for GpGene {
    fn set_id(&mut self, _id: i32) -> &mut Self { self }
}

pub struct GpChromosome<N: GpNode> {
    pub root: Box<Node<N>>,
    pub fitness: f64,
    pub age: usize,
    // Tree-based fitness function — takes the tree root, not a DNA slice
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: Option<Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>>,
}
```

The `TreeChromosome` supertrait adds tree-specific methods:
```rust
pub trait TreeChromosome: ChromosomeT {
    type Node;
    fn tree(&self) -> &Node<Self::Node>;
    fn tree_mut(&mut self) -> &mut Node<Self::Node>;
    fn depth(&self) -> usize;
    fn node_count(&self) -> usize;
}
```

### Pattern 4: GpGa Engine Structure (follow DeEngine pattern)
**What:** Dedicated engine in `src/engines/gp/engine.rs` — does not inherit from `ga.rs`
**When to use:** This is THE engine for GP

```rust
// Source: DeEngine pattern from src/engines/de/engine.rs
pub struct GpGa<U: TreeChromosome> {
    config: GpConfiguration,
    fitness_fn: Arc<dyn Fn(&Node<U::GpNodeType>) -> f64 + Send + Sync>,
    observer: Option<Arc<dyn GaObserver<U> + Send + Sync>>,
}

// Key observer notify helper (same as ga.rs pattern):
impl<U: TreeChromosome> GpGa<U> {
    #[inline]
    fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
        if let Some(ref obs) = self.observer {
            f(obs.as_ref());
        }
    }
}
```

**Engine loop outline:**
```rust
pub fn run(&mut self) -> GpResult<U> {
    self.notify(|obs| obs.on_run_start());
    let mut pop = self.init_population();  // ramped half-and-half
    self.evaluate_population(&mut pop);    // set fitness for all

    for gen in 0..self.config.max_generations {
        self.notify(|obs| obs.on_generation_start(gen));

        // Selection (reuses Selection::factory())
        let parents = selection::factory(&pop, self.config.selection_config, 1)?;

        // Crossover + mutation → offspring
        let mut offspring = self.apply_crossover_mutation(&pop, &parents, gen)?;

        // Evaluate offspring
        self.evaluate_population(&mut offspring);

        // Merge and survive
        pop.extend(offspring);
        survivor::factory(&mut pop, self.config.population_size, ...)?;

        // Stats (including avg_node_count)
        let stats = self.compute_stats(&pop, gen);
        // best tracking → on_new_best
        self.notify(|obs| obs.on_generation_end(&stats));

        // Stopping criteria
        if self.should_stop(&stats) { break; }
    }
    self.notify(|obs| obs.on_run_end(cause, &all_stats));
    GpResult { population: pop, best, generations }
}
```

### Anti-Patterns to Avoid
- **Implementing LinearChromosome on GpChromosome:** `GpChromosome` must never implement `LinearChromosome`. The flat-slice methods (`dna()`, `set_dna()`) must panic or be unreachable — not silently return empty slices. This gives compile-time safety once Phase 47 merges.
- **Adding GP variants to global Crossover/Mutation enums:** GP crossover lives in `GpConfiguration` only. The global `Crossover` enum is for linear chromosomes.
- **Forgetting iterative Drop:** Auto-generated recursive drop can overflow at depth ~5000+ trees. Always use the explicit drop worklist pattern.
- **Blocking wasm32 with serde_stacker:** `serde_stacker` compiles on wasm32 as a no-op (stacker is a no-op on wasm32 per psm documentation). Do NOT add a `#[cfg(not(target_arch = "wasm32"))]` gate around it — just add it to the `serde` feature and let it compile everywhere.
- **Using `par_iter()` without wasm32 gate:** `GpGa` fitness evaluation must gate rayon with `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` exactly as `ga.rs` does.

---

## GP Algorithm Patterns

### Ramped Half-and-Half Initialization
**Authoritative reference:** Koza (1992), "Genetic Programming" + Wikipedia [CITED: en.wikipedia.org/wiki/Genetic_programming]

**Algorithm:**
```
Input: population_size P, init_max_depth D (typically 2..6)
Output: Vec<GpChromosome<N>> of length P

1. Divide population into (D - 1) size-groups (depths 2..=D).
   Each group has P / (D - 1) individuals.
   
2. For each depth d in 2..=D:
   - Create half the group with FULL method (all leaves at exactly depth d)
   - Create other half with GROW method (random early termination)
   
   FULL(d, rng):
     if d == 1: return Terminal(N::sample_random_terminal(rng))
     choose random function node f from primitive set
     children = (0..f.arity()).map(|_| FULL(d-1, rng)).collect()
     return Function { value: f, children }
   
   GROW(d, rng):
     if d == 1: return Terminal(N::sample_random_terminal(rng))
     // With some probability, choose terminal (stops early)
     let p_terminal = n_terminals / (n_terminals + n_functions)
     if rng.random() < p_terminal || d == 1:
       return Terminal(N::sample_random_terminal(rng))
     choose random function node f from primitive set
     children = (0..f.arity()).map(|_| GROW(d-1, rng)).collect()
     return Function { value: f, children }
```

**Configuration fields in `GpConfiguration`:**
- `init_max_depth: usize` — max depth for initialization (separate from runtime `max_depth`)
- `max_depth: usize` — runtime enforcement per CHR-05
- `max_node_count: usize` — runtime enforcement per CHR-05

### SubtreeCrossover
**Authoritative reference:** Field Guide to GP §2.4 [CITED: digitalcommons.morris.umn.edu/cs_facpubs/1/]

**Algorithm:**
```
Input: parent1, parent2 (both GpChromosome<N>), max_depth, max_node_count
Output: (child1, child2) or GaError

1. Collect all nodes in parent1 as a flat list with their paths (indices)
2. Pick a random crossover point p1 in parent1
3. Collect all nodes in parent2 with their paths
4. Pick a random crossover point p2 in parent2 (compatible arity is NOT required in standard GP)
5. child1 = parent1 with subtree at p1 replaced by parent2's subtree at p2
6. child2 = parent2 with subtree at p2 replaced by parent1's subtree at p1
7. If child1.depth() > max_depth → return Err(GaError::TreeDepthExceeded(...))
8. If child1.node_count() > max_node_count → return Err(GaError::TreeSizeExceeded(...))
9. Same checks for child2
10. Return Ok((child1, child2))

Implementation note: "collecting all nodes" requires traversal to build an index
(node count before subtree = offset into pre-order traversal). Use a recursive
traversal that fills a Vec<&mut Node<N>> or Vec<(path, &Node<N>)>.
For swap, clone the target subtree, replace the source subtree.
```

**Selecting a random crossover point:** Pick a random integer in `[0, node_count)` then traverse in pre-order, counting nodes until the target index is reached. This is O(n) per crossover which is acceptable.

### SubtreeMutation
**Authoritative reference:** Field Guide to GP §5.2 [CITED: digitalcommons.morris.umn.edu/cs_facpubs/1/]

**Algorithm:**
```
Input: chromosome, mutation_max_depth, max_depth, max_node_count, rng
1. Pick a random mutation point p in the tree
2. Generate a new random subtree T using GROW(mutation_max_depth, rng)
3. Replace subtree at p with T
4. Check depth/node_count limits → GaError if exceeded
5. Return Ok(()) or Err(GaError::TreeSizeExceeded/TreeDepthExceeded)
```

### PointMutation
**Authoritative reference:** Field Guide to GP §5.2 + Wikipedia [CITED: en.wikipedia.org/wiki/Genetic_programming]

**Algorithm:**
```
Input: chromosome, rng, primitive set (via N::all_functions())
1. Traverse all nodes in the tree
2. For each node: with probability p_mutation_per_node:
   - If node is a terminal: replace with N::sample_random_terminal(rng)
   - If node is a function with arity k:
       candidates = N::all_functions().filter(|f| f.arity() == k)
       if candidates.is_empty(): skip (no same-arity alternative)
       replace node.value with candidates[rng.random_range(0..candidates.len())]
       (children are PRESERVED — arity is unchanged)
3. PointMutation does NOT change tree size or shape → no bloat check needed
   (same arity guarantees identical structure)
```

**`all_functions()` API decision:** [ASSUMED] The method signature `fn all_functions() -> Vec<Self>` on `GpNode` is the recommended approach over `fn with_arity(n: usize) -> Vec<Self>`. Rationale:
- The engine only ever calls it once per mutation and then filters — caching is simple
- User implements it once for their entire primitive set
- Simpler API: user returns all variants, engine filters by arity
- If the user wants to exclude a function from point mutation (e.g., a dangerous division), they simply omit it from `all_functions()`

### HoistMutation
**Authoritative reference:** Field Guide to GP §5.2 [CITED: digitalcommons.morris.umn.edu/cs_facpubs/1/]

**Algorithm:**
```
Input: chromosome, rng
1. Pick a random subtree S1 in the tree (cannot be the root terminal case)
2. Pick a random subtree S2 within S1 (a descendant of S1)
3. Replace S1 with S2 in the full tree
4. Offspring is guaranteed smaller than parent (S2 ⊆ S1 in size)
5. No bloat check needed (tree always shrinks)
6. Edge case: if S1 is a terminal, skip (no children to hoist)
```

### Bloat Control Pattern
Post-crossover and post-mutation: check before accepting offspring:
```rust
fn check_limits(node: &Node<N>, max_depth: usize, max_node_count: usize)
    -> Result<(), GaError>
{
    if node.depth() > max_depth {
        return Err(GaError::TreeDepthExceeded(format!(
            "tree depth {} exceeds max_depth {}", node.depth(), max_depth
        )));
    }
    if node.node_count() > max_node_count {
        return Err(GaError::TreeSizeExceeded(format!(
            "tree has {} nodes, exceeds max_node_count {}", node.node_count(), max_node_count
        )));
    }
    Ok(())
}
```

When an offspring violates limits, the engine can either: (a) discard and retry (simple), or (b) return the error to the caller. Per CHR-05, the operator *returns* `GaError::TreeDepthExceeded` / `GaError::TreeSizeExceeded`. The engine handles this error by discarding the offspring (logging a warning) rather than propagating it to the user. This keeps the run loop alive despite bloat events.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Stack-safe recursive serde | Custom serde Visitor with manual stack | `serde_stacker` (dtolnay) | Handles arbitrary depth; works with existing `#[derive(Serialize, Deserialize)]` |
| Parallel population evaluation | `std::thread::spawn` per individual | `rayon` (already in Cargo.toml) with cfg gate | Thread pool management, work stealing, wasm32 compatibility |
| RNG in GP operators | `rand::thread_rng()` | `crate::rng::make_rng()` | Seedable, reproducible, project-standard |
| Selection in GpGa | Custom selection loop | `selection::factory()` from `src/operations/selection.rs` | Reuses all existing selection variants including Tournament, Roulette, etc. |
| Survivor selection | Custom fitness-rank loop | `survivor::factory()` from `src/operations/survivor.rs` | Reuses existing Truncation, Fitness, MuPlusLambda, etc. |
| Observer dispatch | Explicit if-let in every hook call site | `fn notify<F>(f: F)` helper pattern from `ga.rs` line 892 | Zero overhead when observer is None; consistent with all other engines |

**Key insight:** GP operators are GP-specific (subtree crossover is fundamentally different from uniform crossover), but selection, survival, RNG, observer dispatch, and parallel evaluation are fully reusable from the existing infrastructure.

---

## ChromosomeT Audit

**Source:** [VERIFIED: src/traits/chromosome.rs] — current (pre-Phase-47) `ChromosomeT`

The current `ChromosomeT` trait mandates these methods, which `GpChromosome` must implement (or stub):

| Method | Nature | GpChromosome action |
|--------|--------|---------------------|
| `dna(&self) -> &[Gene]` | Flat-slice | **PANIC stub** — not applicable to trees |
| `dna_mut(&mut self) -> &mut [Gene]` | Flat-slice | **PANIC stub** |
| `set_dna(Cow<[Gene]>)` | Flat-slice | **PANIC stub** |
| `set_fitness_fn<F>(F)` | Fitness fn | **Store as `Option<Arc<fn(&Node<N>) -> f64>>`** — but ChromosomeT expects `Fn(&[Gene]) -> f64`. This requires a workaround: store the tree-fitness fn separately; `set_fitness_fn` is a no-op or unreachable |
| `calculate_fitness()` | Compute | **Uses tree-based fn** stored internally |
| `fitness() -> f64` | Read | Standard `self.fitness` |
| `set_fitness(f64)` | Write | Standard `self.fitness = f` |
| `set_age(usize)` | Write | Standard |
| `age() -> usize` | Read | Standard |

**The `type Gene` problem:** `ChromosomeT` has `type Gene: GeneT`. `GpChromosome<N>` must declare a `Gene` associated type even though it has no flat gene slice. The recommended approach: declare `type Gene = GpGene` where `GpGene` is a zero-sized marker that implements `GeneT`. This satisfies the bound without semantic meaning.

**The `set_fitness_fn` problem:** `ChromosomeT::set_fitness_fn<F>` expects `F: Fn(&[Self::Gene]) -> f64`. This is incompatible with tree-based evaluation. Recommended: implement `set_fitness_fn` as a no-op on `GpChromosome` (the method body does nothing). The `GpGa` engine never calls `chromosome.set_fitness_fn()` — it owns the fitness function directly and calls `chromosome.set_fitness(fitness_fn(&chromosome.root))`.

**Phase 47 forward compatibility:** When Phase 47 eventually merges and `LinearChromosome` becomes the flat-slice supertrait, `GpChromosome` simply stops implementing `LinearChromosome` and the panicking stubs are removed. The `TreeChromosome` supertrait and all GP operators remain unchanged.

---

## GaError Additions

**Source:** [VERIFIED: src/error.rs]

Current variants (verified): `ConfigurationError`, `ValidationError`, `CrossoverError`, `MutationError`, `InitializationError`, `SelectionError`, `InvalidIslandConfiguration`, `InvalidNichingConfiguration`, `InvalidNsga2Configuration`, `InvalidNsga3Configuration`, `InvalidMoeaDConfiguration`, `InvalidSpea2Configuration`, `InvalidSmsEmoaConfiguration`, `InvalidIbeaConfiguration`, `InvalidConstraintConfiguration`, `InvalidIndicatorConfiguration`, `MigrationError`, `CheckpointError`, `LocalSearchError`.

**Add two variants:**
```rust
/// A tree crossover or mutation would exceed the configured max_depth limit.
TreeDepthExceeded(String),
/// A tree crossover or mutation would exceed the configured max_node_count limit.
TreeSizeExceeded(String),
```

Both require updating:
- `impl fmt::Display for GaError` — add two match arms
- `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` already on the enum — no additional change needed

---

## GenerationStats Additions

**Source:** [VERIFIED: src/stats.rs]

Current fields: `generation`, `best_fitness`, `worst_fitness`, `avg_fitness`, `fitness_std_dev`, `population_size`, `diversity`, `dynamic_mutation_probability`.

**Add one field (CHR-05):**
```rust
/// Average node count across the surviving population.
/// Only populated by GpGa; all other engines set this to 0.0.
#[cfg_attr(feature = "serde", serde(default))]
pub avg_node_count: f64,
```

**`serde(default)` is required** so existing checkpoint files (without this field) still deserialize correctly — same pattern as `diversity` and `dynamic_mutation_probability`.

**`from_fitness_values()` change:** The current static constructor does not know about node counts. `GpGa` must set `avg_node_count` after calling `from_fitness_values()` (or use a separate builder pattern). Simplest: call `from_fitness_values()` then `stats.avg_node_count = ...`. Alternatively, add a `GpGenerationStats` method — but the locked decision (D-12) specifies adding to `GenerationStats` directly.

**Backward compatibility:** The `serde(default)` attribute ensures all existing serde tests continue to pass. All other engines simply leave `avg_node_count` at its default value of `0.0`.

---

## Observer Hook Wiring

**Source:** [VERIFIED: src/engines/ga.rs lines 892-896, 1522, 1538, 2116, 2192-2194, 2259]
**Source:** [VERIFIED: src/observe/observer/mod.rs — full GaObserver<U> trait]

The `GaObserver<U>` trait has 12 hooks. `GpGa` fires a subset (matching D-11):

| Hook | When GpGa fires it |
|------|--------------------|
| `on_run_start` | Before generation 0 |
| `on_generation_start(gen)` | Top of each generation loop |
| `on_new_best(gen, best.clone())` | When best fitness improves |
| `on_generation_end(&stats)` | After stats computed, end of generation |
| `on_run_end(cause, &all_stats)` | After loop exits |

Hooks `GpGa` does NOT fire (no GP-specific timing data yet): `on_selection_complete`, `on_crossover_complete`, `on_mutation_complete`, `on_fitness_evaluation_complete`, `on_survivor_selection_complete`, `on_stagnation`, `on_extension_triggered`.

**Implementation pattern (copy from ga.rs):**
```rust
// Source: ga.rs line 892 — the notify helper
#[inline]
fn notify<F: FnOnce(&dyn GaObserver<U>)>(&self, f: F) {
    if let Some(ref obs) = self.observer {
        f(obs.as_ref());
    }
}

// Usage pattern:
self.notify(|obs| obs.on_run_start());
self.notify(|obs| obs.on_generation_start(gen));
self.notify(|obs| obs.on_new_best(gen, best.clone()));
self.notify(|obs| obs.on_generation_end(&stats));
self.notify(|obs| obs.on_run_end(termination_cause, &all_stats));
```

`TerminationCause` is in `src/engines/ga.rs` and re-exported from `src/lib.rs`. Import via `use crate::ga::TerminationCause`.

---

## Serde / Deep Trees

**Source:** [VERIFIED: crates.io registry, dtolnay/serde-stacker GitHub, rust-lang/stacker psm README]

### serde_stacker behavior on wasm32
- `serde_stacker` depends on `stacker` crate (v0.1.15+)
- `stacker` depends on `psm` (Portable Stack Manipulation)
- `psm` README explicitly states: "This library is not applicable to the [WASM] target. WASM hasn't a specified C ABI, the callstack is not even in an address space and does not appear to be manipulatable."
- **However:** stacker README states "On all unsupported platforms this library is a noop. It should compile and run, but it won't actually grow the stack."
- **Conclusion:** `serde_stacker` compiles for wasm32-unknown-unknown without errors. It is a no-op: deep-tree serde on wasm32 will not grow the stack, so a sufficiently deep tree may overflow the wasm32 stack. This is acceptable for Phase 53 because:
  1. The CI wasm32 check (`cargo check --target wasm32-unknown-unknown --lib --features serde`) only checks compilation, not runtime
  2. `max_depth` limits in `GpConfiguration` prevent trees deep enough to be practically problematic
  3. Document the limitation in the rustdoc for serde support

### serde on Node<N>
Standard `#[derive(Serialize, Deserialize)]` on `Node<N>` will work for shallow trees. For deep trees, wrap deserialization:
```rust
// Source: docs.rs/serde_stacker pattern
#[cfg(feature = "serde")]
impl<'de, N: GpNode + Deserialize<'de>> Deserialize<'de> for Node<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        serde_stacker::deserialize(d)
    }
}

#[cfg(feature = "serde")]
impl<N: GpNode + Serialize> Serialize for Node<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serde_stacker::serialize(self, s)
    }
}
```

**CI test (CHR-06):**
```rust
// tests/engines/gp/test_gp_serde.rs
#[cfg(feature = "serde")]
#[test]
fn test_serde_depth_64_no_overflow() {
    // Build a right-spine tree of depth 64 (worst case for stack)
    let root = build_right_spine(64);
    let chromosome = GpChromosome::from_root(root);
    let json = serde_json::to_string(&chromosome).expect("serialize");
    let restored: GpChromosome<TestNode> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.depth(), 64);
}
```

---

## Runtime State Inventory

Phase 53 is a greenfield feature addition. No rename, refactor, or migration involved.

**Nothing found in any category — verified by codebase inspection:**
- Stored data: None — no existing GP chromosomes in any datastore
- Live service config: None
- OS-registered state: None
- Secrets/env vars: None
- Build artifacts: None

---

## Common Pitfalls

### Pitfall 1: Flat ChromosomeT Methods Called on GpChromosome
**What goes wrong:** A linear-chromosome operator (e.g., `CrossoverOperator::crossover(&U, &U)`) calls `chromosome.dna()` on a `GpChromosome`, causing a panic.
**Why it happens:** `GpChromosome` implements `ChromosomeT` (required for trait bounds), and any code with `U: ChromosomeT` can call `dna()`.
**How to avoid:** Make the panic messages explicit: `panic!("GpChromosome::dna() called — use GpChromosome with GpGa, not Ga. This is a tree chromosome and has no flat DNA slice.")`. After Phase 47 merges, the compile-time bound change from `ChromosomeT` to `LinearChromosome` prevents this at compile time.
**Warning signs:** Any test using `GpChromosome` with standard `Crossover` variants.

### Pitfall 2: PointMutation Changing Tree Arity (Breaks Children Count)
**What goes wrong:** Replacing a function node with a different-arity function leaves the wrong number of children attached.
**Why it happens:** Only the node's `value` (the `N` variant) changes in PointMutation — children must be left unchanged. Changing arity would require adding/removing children.
**How to avoid:** PointMutation ONLY changes `node.value` to a same-arity node from `N::all_functions()`. It never adds or removes children. Enforce this in the implementation by filtering `all_functions()` to exactly `node.arity()`.
**Warning signs:** Test failures where trees have wrong node_count after point mutation.

### Pitfall 3: Forgetting Iterative Drop for Deep Trees
**What goes wrong:** Stack overflow when `GpChromosome` goes out of scope for deeply evolved trees.
**Why it happens:** Rust's auto-generated `drop` for `Box<Node<N>>` is recursive — each `Box<Node>` drop triggers its children's drop, consuming O(depth) stack frames.
**How to avoid:** Implement `Drop for Node<N>` with an explicit work queue (see §Pattern 2 code example).
**Warning signs:** Stack overflow in test teardown, not in the operator call itself.

### Pitfall 4: avg_node_count Missing serde(default)
**What goes wrong:** All existing serde tests fail because old checkpoint JSON doesn't have `avg_node_count`.
**Why it happens:** New field added to `GenerationStats` without `#[cfg_attr(feature = "serde", serde(default))]`.
**How to avoid:** Add `serde(default)` to the new field — same pattern as `diversity` and `dynamic_mutation_probability` in the current `stats.rs`.
**Warning signs:** `cargo test --features serde` failures in `test_ga.rs` or `test_observe/test_serde.rs`.

### Pitfall 5: wasm32 CI Failure from serde_stacker
**What goes wrong:** `cargo check --target wasm32-unknown-unknown --lib --features serde` fails.
**Why it happens:** If `serde_stacker` is added to the `serde` feature but `stacker/psm` has a transitive dependency that doesn't compile for wasm32.
**How to avoid:** Run `cargo check --target wasm32-unknown-unknown --lib --features serde` locally before committing. If it fails (unexpectedly — given stacker is documented as a no-op on unsupported platforms), fall back to custom iterative serde (D-13 fallback from CONTEXT.md).
**Warning signs:** CI wasm-check job fails on the `--features serde` step.

### Pitfall 6: Not Gating par_iter() in GpGa
**What goes wrong:** `cargo check --target wasm32-unknown-unknown` fails with "rayon not available".
**Why it happens:** `GpGa::evaluate_population()` uses `par_iter()` without the wasm32 cfg gate.
**How to avoid:** Apply the exact pattern from `ga.rs` lines 1135-1153: `#[cfg(not(target_arch = "wasm32"))] iter.par_iter()` / `#[cfg(target_arch = "wasm32")] iter.iter()`.
**Warning signs:** wasm32 CI failure on the first push.

---

## Code Examples

### GpNode User Implementation Example
```rust
// Source: CONTEXT.md §D-01 design + GeneT pattern (verified in src/traits/gene.rs)
use genetic_algorithms::gp::GpNode;
use rand::Rng;

#[derive(Clone, Debug)]
enum MyNode {
    // Functions (arity > 0)
    Add,
    Mul,
    // Terminals (arity == 0)
    X,
    Const(f64),  // ERC — ephemeral random constant
}

impl GpNode for MyNode {
    fn arity(&self) -> usize {
        match self {
            MyNode::Add | MyNode::Mul => 2,
            MyNode::X | MyNode::Const(_) => 0,
        }
    }

    fn evaluate(&self, args: &[f64]) -> f64 {
        match self {
            MyNode::Add => args[0] + args[1],
            MyNode::Mul => args[0] * args[1],
            MyNode::X => args[0],  // passed via fitness_fn closure
            MyNode::Const(c) => *c,
        }
    }

    fn sample_random_terminal(rng: &mut impl Rng) -> Self {
        if rng.random::<bool>() {
            MyNode::X
        } else {
            MyNode::Const(rng.random::<f64>() * 10.0 - 5.0)
        }
    }

    fn all_functions() -> Vec<Self> {
        vec![MyNode::Add, MyNode::Mul]
    }
}
```

### GpChromosome Display (Lisp S-expression)
```rust
// Source: CONTEXT.md §D-15 — Lisp/prefix S-expression format
impl<N: GpNode + fmt::Display> fmt::Display for GpChromosome<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_node(&self.root, f)
    }
}

fn write_node<N: GpNode + fmt::Display>(
    node: &Node<N>,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match node {
        Node::Terminal(v) => write!(f, "{}", v),
        Node::Function { value, children } => {
            write!(f, "({}", value)?;
            for child in children {
                write!(f, " ")?;
                write_node(child, f)?;
            }
            write!(f, ")")
        }
    }
}
// Output: (+ (* x 3.0) 2.0) for (x*3)+2
```

### lib.rs Addition Pattern (verified from existing engine pattern)
```rust
// Source: src/lib.rs lines 279-307 — existing engine export pattern
#[path = "engines/gp/mod.rs"]
pub mod gp;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Arena-indexed GP trees | `Box<N>` recursive enum | Phase 53 decision (STATE.md) | Simpler code; clone is O(subtree) not O(arena); no index-remapping |
| Inline `#[cfg(test)]` modules | Tests in `tests/` directory | Project feedback (memory) | All GP tests go in `tests/engines/gp/` |
| Reporter trait | `GaObserver<U>` trait | v2.2.0 | GP engine uses observer, not reporter |
| GP as extension of linear GA | `GpGa` as separate engine | Phase 53 design | Cleaner separation; no linear-operator contamination |

**Deprecated/outdated:**
- `Reporter<U>`: removed in v3.0.0 (ARCH-03); do not use in `GpGa`
- Inline `#[derive(Serialize, Deserialize)]` without `serde_stacker` on recursive types: still works but overflows at depth >= ~5000

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `all_functions() -> Vec<Self>` is the right API for PointMutation on GpNode | §Pattern 1, §PointMutation | Low — if a different API is chosen (e.g., `fn with_arity(n) -> Vec<Self>`), it's a minor rename with no downstream impact on the engine |
| A2 | `serde_stacker` compiles for wasm32 as a no-op without errors | §Serde / Deep Trees | Medium — if psm has a non-compiling dependency on wasm32, the CI wasm check fails; D-13 fallback (iterative serde) must be implemented instead |
| A3 | `serde_stacker` version 0.1.14 is sufficient; dtolnay provenance is high-trust | §Package Legitimacy Audit | Very Low — dtolnay is the serde author; package is well-established |
| A4 | Implementing `dna()` etc. as panicking stubs is acceptable until Phase 47 merges | §ChromosomeT Audit | Low — Phase 47 is planned (it's the next unstarted architecture phase); panicking stubs are clearly documented |
| A5 | Iterative Drop is needed for trees with `max_depth <= 1000` (typical use) | §Common Pitfalls | Low — at depth 64 (CI test), recursion is trivially safe; custom Drop is a correctness precaution for adversarial inputs |

---

## Open Questions

1. **`GpGa` fitness function signature**
   - What we know: ChromosomeT expects `Fn(&[Gene]) -> f64` but tree evaluation needs `Fn(&Node<N>) -> f64`
   - What's unclear: Whether `GpGa::new(config, fitness_fn)` should accept the tree-based signature directly (bypassing `ChromosomeT::set_fitness_fn`) or adapt somehow
   - Recommendation: `GpGa` owns the fitness function directly as `Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>` and calls `chromosome.set_fitness(fn(&chromosome.root))` in the evaluation loop. `GpChromosome::set_fitness_fn` is a no-op (never called by `GpGa`).

2. **What happens to bloat-rejected offspring?**
   - What we know: CHR-05 says violations "return GaError::..."; D-06 confirms this
   - What's unclear: Does the engine propagate the error to the user (crashing the run) or does it silently discard the rejected offspring?
   - Recommendation: Discard and log a warning via `log::warn!(target = "gp_events", ...)`. Re-run crossover with a different random point (with a max retry count of 3). If all retries fail, keep the better parent unchanged. This matches common GP implementations and avoids crashing production runs due to bloat.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | Yes | 1.94.1 | — |
| wasm32-unknown-unknown target | CI wasm check, WASM compat | Yes | (rustup installed) | — |
| cargo | Build, test | Yes | 1.94.1 | — |
| `serde_stacker` crate | CHR-06 serde checkpoint | Available on crates.io | 0.1.14 | Iterative serde (D-13 fallback) if wasm32 fails |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** `serde_stacker` — if wasm32 compilation fails, D-13 specifies an iterative serde approach as the fallback.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`#[test]`) — same as all other tests in codebase |
| Config file | None needed — standard cargo test |
| Quick run command | `cargo test gp` |
| Full suite command | `cargo test && cargo test --features serde` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CHR-03 | `TreeChromosome: ChromosomeT` — NOT `LinearChromosome`; tree methods available | unit | `cargo test test_gp_chromosome` | No — Wave 0 gap |
| CHR-03 | `GpChromosome::depth()`, `node_count()` correct | unit | `cargo test test_gp_chromosome::test_depth_node_count` | No — Wave 0 gap |
| CHR-03 | `GpChromosome` with linear operator produces compile error (not runtime panic) | compile-fail | Manual — not automated in Phase 53 (Phase 47 handles this) | N/A |
| CHR-04 | `GpGa` runs end-to-end with simple arithmetic primitive set | integration | `cargo test test_gp_engine::test_gpga_converges` | No — Wave 2 gap |
| CHR-04 | Ramped half-and-half produces expected depth distribution | unit | `cargo test test_gp_init::test_ramped_half_and_half_distribution` | No — Wave 2 gap |
| CHR-04 | ERC terminals produce random values via `sample_random_terminal` | unit | `cargo test test_gp_chromosome::test_erc_sampling` | No — Wave 0 gap |
| CHR-05 | SubtreeCrossover returns `TreeDepthExceeded` when limit violated | unit | `cargo test test_gp_crossover::test_depth_limit_enforcement` | No — Wave 1 gap |
| CHR-05 | SubtreeCrossover returns `TreeSizeExceeded` when node count violated | unit | `cargo test test_gp_crossover::test_size_limit_enforcement` | No — Wave 1 gap |
| CHR-05 | `avg_node_count` in GenerationStats is non-zero after GpGa run | integration | `cargo test test_gp_engine::test_avg_node_count_populated` | No — Wave 2 gap |
| CHR-06 | GpChromosome with depth 64 serializes/deserializes without stack overflow | integration | `cargo test --features serde test_gp_serde::test_serde_depth_64_no_overflow` | No — Wave 3 gap |
| CHR-07 | `GpChromosome::to_string()` produces valid Lisp S-expression | unit | `cargo test test_gp_chromosome::test_display_sexpr` | No — Wave 0 gap |
| CHR-07 | Nested expression `(+ (* x 3) 2)` renders correctly | unit | `cargo test test_gp_chromosome::test_display_nested` | No — Wave 0 gap |

### Sampling Rate
- **Per task commit:** `cargo test gp`
- **Per wave merge:** `cargo test && cargo test --features serde && cargo clippy`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/engines/gp/test_gp_chromosome.rs` — covers CHR-03, CHR-07
- [ ] `tests/engines/gp/test_gp_crossover.rs` — covers CHR-05 crossover
- [ ] `tests/engines/gp/test_gp_mutation.rs` — covers CHR-05 mutation variants
- [ ] `tests/engines/gp/test_gp_init.rs` — covers CHR-04 initialization
- [ ] `tests/engines/gp/test_gp_engine.rs` — covers CHR-04, CHR-05 (avg_node_count)
- [ ] `tests/engines/gp/test_gp_serde.rs` — covers CHR-06 (feature-gated)

---

## Wave Breakdown Recommendation

**4 waves, each a mergeable PR:**

### Wave 0 — Types, Traits, Shells (API contract)
Files: `src/engines/gp/` (mod.rs, node.rs, chromosome.rs with stubs, configuration.rs shell), `src/error.rs` (2 new variants), `src/stats.rs` (avg_node_count), `src/lib.rs` (pub mod gp), `tests/engines/gp/` (all test files with stubs)
Deliverables: `GpNode` trait, `Node<N>` enum + iterative Drop, `GpChromosome<N>` with `TreeChromosome: ChromosomeT`, `GpGene` marker, `Display` impl, `GaError::TreeDepthExceeded/TreeSizeExceeded`, `GenerationStats::avg_node_count`, `GpConfiguration` shell
Tests green: CHR-03 (trait structure), CHR-07 (Display)

### Wave 1 — GP Operators
Files: `src/engines/gp/crossover.rs`, `src/engines/gp/mutation.rs`
Deliverables: `GpCrossover::SubtreeCrossover` with bloat enforcement, `GpMutation::{SubtreeMutation, PointMutation, HoistMutation}` with probability application
Tests green: CHR-05 operator-level

### Wave 2 — GpGa Engine + Ramped Init
Files: `src/engines/gp/engine.rs`, `src/engines/gp/init.rs`
Deliverables: `GpGa<U>` struct + `run()` loop with observer hooks + stopping criteria, `ramped_half_and_half()` init, `GpResult<U>`, `avg_node_count` populated in stats
Tests green: CHR-04 (end-to-end), CHR-05 (avg_node_count)

### Wave 3 — Serde Checkpoint + wasm32 Verification
Files: `Cargo.toml` (serde_stacker dep), `src/engines/gp/chromosome.rs` (Serialize/Deserialize with serde_stacker), `.github/workflows/wasm-check.yml` (already covers serde feature check)
Deliverables: `serde_stacker` conditional dependency, `Serialize`/`Deserialize` on `Node<N>`, depth-64 CI test
Tests green: CHR-06 (`--features serde` test suite)

---

## Security Domain

Phase 53 adds no authentication, session management, access control, or cryptography. The only applicable ASVS category is V5 (Input Validation):

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Yes (partial) | `max_depth` and `max_node_count` validation in `GpConfiguration::build()` prevents resource exhaustion from unconstrained tree growth |
| V6 Cryptography | No | — |

**V5 note:** User-provided `init_max_depth`, `max_depth`, and `max_node_count` must be validated at configuration build time (e.g., `max_depth > 0`, `max_node_count >= max_depth`, `init_max_depth <= max_depth`). A tree with `max_depth = usize::MAX` and no node count limit could consume unbounded memory.

---

## Sources

### Primary (HIGH confidence)
- `src/traits/chromosome.rs` — current ChromosomeT trait (all methods verified)
- `src/error.rs` — GaError enum (all variants verified)
- `src/stats.rs` — GenerationStats struct (all fields verified)
- `src/engines/ga.rs` — observer hook pattern: `notify()` helper line 892; hook call sites lines 1522, 1538, 2116, 2194, 2259
- `src/engines/de/engine.rs` — DeEngine pattern: engine struct, `run()` loop structure
- `src/engines/cellular/engine.rs` — CellularEngine pattern: alternative engine reference
- `src/observe/observer/mod.rs` — GaObserver<U> trait: all 12 hooks verified
- `src/types/chromosomes/binary.rs` — BinaryChromosome pattern: concrete chromosome impl reference
- `src/lib.rs` — lib.rs module structure and export pattern
- `Cargo.toml` — current dependencies and features confirmed
- `.github/workflows/wasm-check.yml` — CI wasm32 check configuration (both default and serde features)
- `src/rng.rs` — make_rng() function

### Secondary (MEDIUM confidence)
- [crates.io: serde_stacker 0.1.14](https://crates.io/crates/serde_stacker) — version and authorship
- [psm README (rust-lang/stacker)](https://github.com/rust-lang/stacker/blob/master/psm/README.mkd) — "not applicable to WASM" (wasm32 behavior of stacker)
- [stacker crate README](https://github.com/rust-lang/stacker) — "noop on unsupported platforms"
- [Wikipedia: Genetic Programming](https://en.wikipedia.org/wiki/Genetic_programming) — ramped half-and-half, subtree crossover, hoist mutation algorithms
- [matklad: If a Tree Falls in a Forest](https://matklad.github.io/2022/11/18/if-a-tree-falls-in-a-forest-does-it-overflow-the-stack.html) — iterative Drop implementation pattern

### Tertiary (LOW confidence / training knowledge)
- Genetic Programming algorithm details (subtree crossover depth enforcement, point mutation same-arity semantics, hoist mutation algorithm) — cross-referenced with Wikipedia and Field Guide references above

---

## Metadata

**Confidence breakdown:**
- Standard Stack: HIGH — all crates verified from crates.io registry; no new crates except serde_stacker (dtolnay, HIGH-trust)
- Architecture: HIGH — all patterns derived from codebase (verified source files)
- GP Algorithms: MEDIUM — canonical GP algorithms; cross-referenced from Wikipedia and field guide references; exact pseudocode is standard across GP literature
- serde_stacker wasm32: MEDIUM — psm "noop on unsupported platforms" confirmed from GitHub; no test run performed

**Research date:** 2026-05-25
**Valid until:** 2026-06-25 (stable domain; GP algorithms don't change; codebase patterns are locked by Phase 52 completion)
