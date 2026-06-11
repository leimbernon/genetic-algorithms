# Stack Research: v3.0.0 New Features

**Project:** genetic_algorithms v3.0.0 — Advanced Representations, Alternative Strategies & Architecture Simplification
**Researched:** 2026-05-19
**Confidence:** HIGH for all core recommendations (verified via Context7, docs.rs, and codebase inspection)
**Scope:** NEW stack needs only. Existing stack (rand 0.9.2, rand_distr 0.5.1, rayon 1.10, log, env_logger, serde, plotters, tracing, metrics, criterion) is unchanged and not re-documented here.

---

## Dependency Changes Summary

Zero new external crates required for most v3.0.0 features. One crate (`num-traits`) is a strong
candidate for multi-parent crossover generics. Tree chromosome representation is best served by an
internal enum-based design using `Box<T>` — no external arena crate needed.

The v3.0.0 feature set is primarily an **architectural and trait redesign** project, not a dependency
expansion project.

---

## New Dependencies Needed

| Crate | Version | Purpose | wasm32 Compatible | Justification |
|-------|---------|---------|-------------------|---------------|
| `num-traits` | `0.2` | Generic float arithmetic for UNDX/SPX/PCX multi-parent crossover (centroid computation, normal distribution sampling over generic types) | YES — pure `no_std` core; enable `libm` feature for transcendental ops if needed | The existing codebase uses raw `f64` everywhere; multi-parent crossover operators (UNDX, SPX, PCX) produce offspring via centroid/covariance arithmetic. Bounding this arithmetic behind `num_traits::Float` or `num_traits::real::Real` keeps operators generic over `f32`/`f64` gene types. The `Range<T>` gene is already bounded by `Copy + PartialOrd`; adding `num_traits::Float` at the crossover operator level is the minimum touch point. **MEDIUM confidence** that this is needed — if multi-parent crossover is restricted to `f64` genes only (which covers all practical Range<f64> use cases), this dep can be skipped entirely and raw f64 arithmetic used. Evaluate at implementation time. |

### Verdict on indextree

`indextree` 4.8.1 is an arena-based tree using a single `Vec` with `NodeId` index handles. It is
`Send + Sync`, supports `par_iter` via rayon, and has optional `deser` (serde) support. On paper it
fits the constraints.

**Recommendation: Do NOT add indextree.** Use an internal `Box<Node>` recursive enum instead:

1. Tree chromosomes in GP evolve under subtree crossover and grow/shrink mutation. The tree is
   cloned each generation (rayon produces offspring in parallel). `indextree`'s arena is a single
   `Vec<Node>` — cloning it clones the entire arena, not just the subtree. A `Box<Node>` enum
   clones exactly the subtree being copied with no wasted allocation.
2. Subtree crossover requires selecting a random node, detaching it, and grafting it onto another
   tree. With `Box<Node>` this is a pointer swap. With indextree it requires re-parenting indices
   across arenas, which is more complex.
3. The user-facing `GpNode` enum IS the chromosome representation — it is domain-specific. Hiding it
   inside an arena adds indirection without benefit.
4. wasm32 `par_iter` feature of indextree pulls in rayon, which must be cfg-gated anyway under the
   project's existing WASM rules. An internal enum has no such entanglement.

---

## Architecture Patterns

### 1. Tree Chromosome: Internal Recursive Enum

Do not model the tree as a flat `&[Gene]` slice. Instead, the `TreeChromosome<N>` holds a
`Box<N>` root where `N` is a user-defined node enum.

```
TreeChromosome<N> {
    root: Box<N>,
    fitness: f64,
    age: usize,
    fitness_fn: Option<Arc<dyn Fn(&N) -> f64 + Send + Sync>>,
}
```

The library provides a `GpNode` marker trait users implement on their own enum:

```rust
pub trait GpNode: Clone + Send + Sync + 'static {
    fn arity(&self) -> usize;      // 0 = terminal, >0 = internal node
    fn children(&self) -> &[Box<Self>];
    fn children_mut(&mut self) -> &mut Vec<Box<Self>>;
    fn depth(&self) -> usize;      // cached or computed recursively
    fn size(&self) -> usize;       // node count in subtree
}
```

This breaks the `dna() -> &[Gene]` assumption in `ChromosomeT`. The v3.0.0 `ChromosomeT` redesign
(architecture audit) must introduce a companion `TreeChromosomeT` trait or make `dna()` optional.

**Breaking change vector:** `ChromosomeT::dna()` and `set_dna()` are linear-slice methods. Tree
chromosomes cannot implement them meaningfully. The v3.0.0 API simplification should split
`ChromosomeT` into two traits:
- `ChromosomeT` (core: fitness, age, clone, send+sync) — base for all chromosomes
- `LinearChromosomeT: ChromosomeT` — adds `dna()`, `set_dna()`, `dna_mut()`, `set_gene()`
- `TreeChromosomeT: ChromosomeT` — adds `root()`, `root_mut()`, `depth()`, `size()`

All existing engines continue to use `LinearChromosomeT` bounds. The new `GpEngine` uses
`TreeChromosomeT`.

**Bloat control** (critical for GP): enforce `max_depth` and `max_size` limits on every grow
operation and after subtree crossover. Research shows depth limits alone are effective; combining
with size limits is better. Both should be configurable on `GpConfiguration`.

### 2. Variable-Length Chromosomes

Variable-length is a relaxation of the linear DNA model: `Vec<Gene>` where len can differ between
individuals. This is simpler than trees.

**Pattern:** `VariableLengthChromosome<G>` wraps `Vec<G>` and implements `LinearChromosomeT`. The
distinction from current chromosomes is that:
- Crossover operators must handle mismatched parent lengths (use shorter-parent length, or splice)
- Mutation gains two new operations: `Grow` (append random gene) and `Shrink` (remove random gene)
- Survivor and selection operators already work on fitness — no changes needed there

**No new external dependency.** `Vec<G>` already supports grow/shrink. The length variation is an
operator-level concern, not a chromosome-level one.

**Minimum chromosome bounds:** Configurable `min_length` and `max_length` on the genotype config
to prevent degenerate zero-length or runaway-size chromosomes.

### 3. Unique<T> Genotype (Permutation)

A permutation chromosome is a `Vec<T>` where each element appears exactly once. This is a
constraint on the linear chromosome, not a structural change.

**Pattern:** `UniqueChromosome<T>` wraps `Vec<T>` where `T: Eq + Hash + Clone + Send + Sync`.
The existing `ListChromosome<T>` already does something similar for symbolic alphabets; `Unique`
adds the invariant that all elements are distinct.

Compatible operators: Order crossover (OX), PMX, Cycle crossover — all already implemented.
Incompatible operators that break uniqueness: Uniform crossover, most mutation operators except
Swap, Inversion, Scramble, Insertion — these already exist and preserve permutation validity.

**No new external dependency.**

### 4. MultiRange<T> and MultiUnique<T> Genotypes

`MultiRange<T>` is a chromosome where each gene position has its own `(min, max)` range, instead
of a shared range. Internally still `Vec<T>`, but mutation applies per-gene bounds.

`MultiUnique<T>` is multiple independent permutation groups concatenated in one chromosome.

Both are pure data-structure extensions of existing `Range<T>` and `List<T>` chromosomes.

**Pattern:** Store bounds as `Arc<[(T, T)]>` (same pattern as current `Range<T>` uses for its
`Arc<[(T,T)]>` shared slice — see Key Decision in PROJECT.md). Zero-copy across rayon threads.

**No new external dependency.**

### 5. Lexicase Selection

Lexicase selection requires per-case fitness values rather than a single scalar `f64`. This is a
**breaking change to `ChromosomeT`**.

**Breaking change vector:** `ChromosomeT::fitness() -> f64` is a scalar. Lexicase needs
`fitness_cases() -> &[f64]` where each element is performance on one training case.

**Pattern options:**

Option A (recommended): Add `fitness_cases() -> Option<&[f64]>` to `ChromosomeT` with a default
`None` impl. Lexicase selection checks `fitness_cases().is_some()` and falls back gracefully. The
scalar `fitness()` becomes the aggregate (mean or weighted sum) for non-lexicase operators that
need a single number (survivor, comparison).

Option B: Separate `LexicaseChromosomeT: ChromosomeT` trait. Cleaner separation, but forces users
to implement an extra trait.

Option A is better for the v3.0.0 "simplify, don't proliferate traits" goal.

**Algorithm:** O(n × m) where n = population size, m = case count. For each selection event:
shuffle case order → filter population by best-on-case iteratively → if pool reaches 1 select it,
else random from survivors. No new crate needed — standard `rand` shuffling.

### 6. Multi-Parent Crossover (UNDX, SPX, PCX)

All three operators work on real-valued (Range<f64>) genes and require:
- SPX: compute centroid of k parents, expand each parent relative to centroid by epsilon, sample
  offspring uniformly within the expanded simplex
- UNDX: centroid of k parents, normal distribution perpendicular to the primary direction
- PCX: child near each parent, covariance from difference vectors

**Math requirement:** centroid (mean over Vec<f64>), normal distribution sampling. Both are already
available: centroid via raw f64 arithmetic, normal distribution via `rand_distr::Normal` (already
in Cargo.toml).

**No new external dependency.** `rand_distr` (already present) handles the normal distribution.
`num-traits` is only needed if the operators must be generic over `T: Float`. Since all practical
use cases are `Range<f64>`, concrete f64 arithmetic is sufficient and avoids the extra dep.

**CrossoverOperator signature change:** Current trait takes `(&U, &U)`. Multi-parent crossover
needs `(&[&U])`. Two options:
- Add `CrossoverMultiParent` as a parallel trait (non-breaking addition)
- Change `CrossoverOperator` to take `&[&U]` and pass slice of length 2 for all current operators

The parallel trait is safer for v3.0.0 since it avoids breaking the 9 existing crossover
implementations.

### 7. Self-Adaptive Mutation

Strategy parameters (σ, step size) are stored inside each chromosome and mutated alongside the
object variables. This is the (μ/μ_I, λ)-σ-SA-ES pattern.

**Pattern:** `SelfAdaptiveChromosome<G>` wraps a `LinearChromosome<G>` and adds a `Vec<f64>`
strategy vector (one σ per gene, or a single global σ). Mutation first perturbs σ by
log-normal noise, then applies Gaussian mutation with the new σ.

```
struct SelfAdaptiveChromosome<G> {
    inner: LinearChromosome<G>,
    strategy_params: Vec<f64>,   // σ per gene, or [σ_global]
}
```

The `MutationOperator` for self-adaptive strategies receives the strategy_params from
`ChromosomeT`. This can be modeled as an optional method:

```rust
fn strategy_params(&self) -> Option<&[f64]> { None }
fn strategy_params_mut(&mut self) -> Option<&mut Vec<f64>> { None }
```

**No new external dependency.** `rand_distr::Normal` (already present) is sufficient for
log-normal perturbation of σ.

### 8. Unified Strategy Trait

The `Strategy` trait abstracts over `Ga<U>`, `HillClimb`, and `Permutate`, letting users
build a single `StrategyBuilder` and pick the strategy at runtime or compile time.

**Pattern (modeled on genetic_algorithm crate's proven approach):**

```rust
pub trait Strategy {
    type Output;
    fn run(&mut self) -> Result<Self::Output, GaError>;
    fn best(&self) -> Option<&Self::Output>;
}

impl Strategy for Ga<U> { ... }
impl Strategy for HillClimb<U> { ... }
impl Strategy for Permutate<U> { ... }
```

The trait is lightweight — no new logic, just a unifying interface. Existing engines remain
unchanged internally.

**HillClimb variants:**
- `Stochastic`: accept any neighbor that improves fitness; stop on no improvement or iteration limit
- `SteepestAscent`: enumerate all neighbors (requires `neighbouring_population_size` concept), pick
  the best; suitable only for small discrete search spaces

**Permutate:** exhaustive iteration using a permutation iterator over the genotype's allele space.
Suitable for fewer than ~12 genes (12! = 479M, hard limit). Should emit a warning when
`chromosome_permutations_size() > configurable_limit`.

**No new external dependency.**

---

## What NOT to Add

| Anti-Pattern | Why | What to Do Instead |
|---|---|---|
| `indextree` or any arena tree crate | Cloning an arena is O(arena size), not O(subtree size). Subtree crossover becomes an index-remapping problem. Adds a non-trivial dep with rayon entanglement. | Internal `Box<N>` recursive enum — clones only what is needed |
| `nalgebra` | Full linear algebra library; UNDX/SPX/PCX only need centroid + normal sampling | raw f64 arithmetic + `rand_distr::Normal` (already present) |
| `petgraph` | Graph chromosome is not in scope for v3.0.0 | Not needed |
| `ego-tree` or `slab` | Same arena-clone problem as indextree; ego-tree is unmaintained | Internal Box<N> enum |
| `derive_more` or similar derive crates | Convenience for trait derives; adds a proc-macro dep | Implement Clone/Debug/Default manually — they are simple for chromosome wrappers |
| `bigdecimal` / `num-bigint` | Permutate strategy size reporting uses BigUint in `genetic_algorithm` crate; our Permutate only needs a `u64` safety gate on max permutation count | Plain `u64` with a saturating max check |
| New feature flags for individual GP/VarLen features | Feature flag proliferation makes CI matrix and documentation unwieldy | GP engine behind `gp` feature flag ONLY IF it pulls a new dep. Otherwise unconditional module like `src/island/` |

---

## Integration Considerations

### WASM (wasm32-unknown-unknown)

All new features must gate any `std::time::Instant` and `rayon::par_iter` behind
`#[cfg(not(target_arch = "wasm32"))]`. Specific concerns for v3.0.0:

- **TreeChromosome subtree operations**: recursive tree traversal is single-threaded by default.
  Rayon parallel offspring generation applies at the population level (each offspring tree is
  evaluated independently) — existing cfg pattern covers this.
- **Permutate strategy**: exhaustive permutation iteration is inherently sequential — no rayon,
  no WASM issue.
- **HillClimb strategy**: same as Permutate, sequential by design.
- **num-traits (if added)**: pure no_std, wasm32 compatible. Use `features = ["i128"]` if i128
  gene types are needed; otherwise default features suffice.
- **rand_distr (already present)**: already WASM-compatible in project's existing configuration.

### Serde / Checkpoint

- `TreeChromosome<N>` is serializable only if `N: Serialize + Deserialize`. Gate serde impls behind
  the existing `serde` feature flag. The `Box<N>` structure serializes naturally with serde's
  derive macros when `N` is serde-annotated.
- `SelfAdaptiveChromosome<G>` adds a `Vec<f64>` field — trivially serializable.
- `VariableLengthChromosome<G>`: already a `Vec<G>` — trivially serializable.

### Rayon Parallelism

- All chromosome types (TreeChromosome, VariableLengthChromosome, SelfAdaptiveChromosome,
  UniqueChromosome) must be `Send + Sync`. The `Box<N>` recursive enum is `Send + Sync` if `N`
  is, which it will be (user's node enum must implement `GpNode: Send + Sync`).
- Multi-parent crossover takes `&[&U]` — all parents are immutable borrows during crossover.
  rayon evaluates offsprings independently, same pattern as current two-parent crossover.

### Breaking Changes in ChromosomeT

v3.0.0 allows breaking changes. The architectural audit should split `ChromosomeT`:

```
ChromosomeT (base)
├── LinearChromosomeT  ← all current engines use this bound
└── TreeChromosomeT    ← GpEngine uses this bound
```

Adding `fitness_cases() -> Option<&[f64]>` to `ChromosomeT` base (with default `None`) is
non-breaking at the trait level but does change the trait signature. Since v3.0.0 is a major
bump, this is acceptable. Users implementing `ChromosomeT` get the default `None` impl
automatically — no migration burden for non-lexicase users.

### Existing Operator Compatibility

| New Feature | Crossover Compat | Mutation Compat | Selection Compat | Survivor Compat |
|---|---|---|---|---|
| UniqueChromosome | OX, PMX, Cycle, ERX ✓ | Swap, Inversion, Scramble, Insertion ✓ | All ✓ | All ✓ |
| MultiRange<T> | SinglePoint, MultiPoint, Uniform ✓ | Value, Creep, Gaussian, Polynomial ✓ | All ✓ | All ✓ |
| MultiUnique<T> | OX, PMX within each group | Swap/Inversion within group | All ✓ | All ✓ |
| VariableLength | needs length-aware variants | + Grow/Shrink new ops | All ✓ | All ✓ |
| TreeChromosome | Subtree crossover (new) | Subtree mutation, Point mutation (new) | All ✓ | All ✓ |
| SelfAdaptive | All (σ updated before gene mutation) | SelfAdaptiveGaussian (new variant) | All ✓ | All ✓ |
| LexicaseSelection | All ✓ | All ✓ | Lexicase (new), all existing ✓ | All ✓ |

---

## Sources

- indextree 4.8.1 docs: https://docs.rs/indextree/latest/indextree/ (HIGH confidence — official docs)
- indextree Context7: /saschagrunert/indextree (HIGH)
- num-traits 0.2 WASM/no_std: https://github.com/rust-num/num-traits/issues/75 (MEDIUM — GitHub issue, verified by num_traits::float::FloatCore being always available)
- genetic_algorithm crate Strategy/HillClimb/Permutate/UniqueGenotype patterns: Context7 /websites/rs_genetic_algorithm_0_27_1_genetic_algorithm (HIGH — official docs)
- Lexicase selection algorithm: https://lexicase.ai/ and https://arxiv.org/pdf/1709.05394 (HIGH — original research)
- SPX/UNDX/PCX: https://waterprogramming.wordpress.com/2018/11/26/introduction-to-borg-operators-part-1-simplex-crossover-spx/ (MEDIUM — blog, verified against ACM DL paper)
- Bloat control depth/size limits: https://link.springer.com/chapter/10.1007/0-387-28111-8_15 (HIGH — Springer, original GP research)
- Self-adaptive ES σ pattern: https://algorithmafternoon.com/strategies/self_adaptive_evolution_strategy/ and Scholarpedia ES article (MEDIUM)
- Existing codebase: ChromosomeT trait, GeneT trait, Cargo.toml (HIGH — direct inspection)
