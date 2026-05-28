//! GP chromosome types: [`GpGene`] marker, [`TreeChromosome`] supertrait,
//! and the concrete [`GpChromosome<N>`] implementation.
//!
//! `GpChromosome<N>` implements [`ChromosomeT`] — NOT `LinearChromosome`.
//! Calling `dna()`, `dna_mut()`, or `set_dna()` on a `GpChromosome` panics
//! with an explicit error message, because tree chromosomes have no flat DNA
//! slice representation.

use super::node::{GpNode, Node};
use crate::traits::{ChromosomeT, GeneT, LinearChromosome};
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

/// Type alias for the tree-based fitness function stored inside [`GpChromosome`].
type TreeFitnessFn<N> = Option<Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>>;

// ---------------------------------------------------------------------------
// GpGene — marker gene required by ChromosomeT::Gene associated type
// ---------------------------------------------------------------------------

/// Marker gene type for GP chromosomes.
///
/// `GpChromosome<N>` must satisfy `ChromosomeT`, which requires an associated
/// `Gene: GeneT`. Since tree chromosomes have no flat DNA slice, `GpGene` is
/// a zero-sized marker type that fulfills this bound without carrying data.
///
/// Do not use `GpGene` directly — it exists solely to satisfy the trait system.
#[derive(Clone, Default, Debug)]
pub struct GpGene;

impl GeneT for GpGene {
    fn set_id(&mut self, _id: i32) -> &mut Self {
        self
    }
}

// ---------------------------------------------------------------------------
// TreeChromosome supertrait
// ---------------------------------------------------------------------------

/// Supertrait of [`ChromosomeT`] for tree-structured chromosomes.
///
/// Implementors provide access to the root [`Node<N>`] and convenience
/// methods for querying tree properties. This trait does NOT extend
/// `LinearChromosome` — tree chromosomes have no flat DNA slice.
///
/// The only built-in implementor is [`GpChromosome<N>`].
pub trait TreeChromosome: ChromosomeT {
    /// The GP node type stored in this chromosome's expression tree.
    type GpNodeType: GpNode;

    /// Returns a reference to the root node of the expression tree.
    fn tree(&self) -> &Node<Self::GpNodeType>;

    /// Returns a mutable reference to the root node of the expression tree.
    fn tree_mut(&mut self) -> &mut Node<Self::GpNodeType>;

    /// Returns the depth of the expression tree.
    fn depth(&self) -> usize;

    /// Returns the total number of nodes in the expression tree.
    fn node_count(&self) -> usize;
}

// ---------------------------------------------------------------------------
// GpChromosome<N>
// ---------------------------------------------------------------------------

/// A chromosome that stores a GP expression tree.
///
/// `GpChromosome<N>` is the library-provided concrete type for Genetic
/// Programming. Users implement [`GpNode`] on their own enum and instantiate
/// `GpChromosome<MyNode>` — the same pattern as `BinaryChromosome` /
/// `RangeChromosome<T>` for linear chromosomes.
///
/// # Tree fitness
///
/// The fitness function must accept a `&Node<N>` (not a DNA slice). Install it
/// via the GP engine (not via `set_fitness_fn` which is a no-op). The engine
/// stores the fitness function inside the chromosome as an `Arc` so cloning is
/// cheap.
///
/// # Linear DNA methods
///
/// `dna()`, `dna_mut()`, and `set_dna()` all **panic** with an explicit message.
/// Do not pass `GpChromosome` to operators that call these methods. Use `GpGa`,
/// not `Ga`, for tree chromosomes.
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "N: serde::Serialize",
        deserialize = "N: for<'de2> serde::Deserialize<'de2>"
    ))
)]
pub struct GpChromosome<N: GpNode> {
    /// The root of the expression tree.
    pub root: Box<Node<N>>,
    fitness: f64,
    age: usize,
    /// Tree fitness function. Skipped during serde — the engine re-installs it
    /// after deserialization via `GpGa::run`.
    #[cfg_attr(feature = "serde", serde(skip, default = "default_fitness_fn"))]
    fitness_fn: TreeFitnessFn<N>,
}

#[cfg(feature = "serde")]
fn default_fitness_fn<N: GpNode>() -> TreeFitnessFn<N> {
    None
}

// Manual Debug because Arc<dyn Fn> does not implement Debug.
impl<N: GpNode + std::fmt::Debug> std::fmt::Debug for GpChromosome<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpChromosome")
            .field("root", &self.root)
            .field("fitness", &self.fitness)
            .field("age", &self.age)
            .finish_non_exhaustive()
    }
}

// Manual Clone because Arc<dyn Fn> is Clone via Arc::clone.
impl<N: GpNode> Clone for GpChromosome<N> {
    fn clone(&self) -> Self {
        GpChromosome {
            root: self.root.clone(),
            fitness: self.fitness,
            age: self.age,
            fitness_fn: self.fitness_fn.clone(),
        }
    }
}

impl<N: GpNode + Default> Default for GpChromosome<N> {
    /// Creates a default chromosome with a single terminal root.
    ///
    /// The root is `Node::Terminal(N::default())`. `GpGa` always overwrites the
    /// root before use; this `Default` impl exists only to satisfy trait bounds.
    fn default() -> Self {
        GpChromosome {
            root: Box::new(Node::Terminal(N::default())),
            fitness: 0.0,
            age: 0,
            fitness_fn: None,
        }
    }
}

impl<N: GpNode> GpChromosome<N> {
    /// Creates a new chromosome with the given root node.
    pub fn with_root(root: Box<Node<N>>) -> Self {
        GpChromosome {
            root,
            fitness: 0.0,
            age: 0,
            fitness_fn: None,
        }
    }

    /// Installs a tree-based fitness function.
    ///
    /// This is the correct way to set the fitness function for GP chromosomes.
    /// The standard `ChromosomeT::set_fitness_fn` is a no-op (see trait impl).
    pub fn set_tree_fitness_fn<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&Node<N>) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = Some(Arc::new(f) as Arc<dyn Fn(&Node<N>) -> f64 + Send + Sync>);
        self
    }
}

impl<N: GpNode + Default> ChromosomeT for GpChromosome<N> {
    type Gene = GpGene;

    fn calculate_fitness(&mut self) {
        if let Some(ref f) = self.fitness_fn {
            self.fitness = f(&self.root);
        }
    }

    fn fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn set_age(&mut self, age: usize) -> &mut Self {
        self.age = age;
        self
    }

    fn age(&self) -> usize {
        self.age
    }
}

impl<N: GpNode + Default> TreeChromosome for GpChromosome<N> {
    type GpNodeType = N;

    fn tree(&self) -> &Node<N> {
        &self.root
    }

    fn tree_mut(&mut self) -> &mut Node<N> {
        &mut self.root
    }

    fn depth(&self) -> usize {
        self.root.depth()
    }

    fn node_count(&self) -> usize {
        self.root.node_count()
    }
}

// ---------------------------------------------------------------------------
// Display — Lisp prefix S-expression
// ---------------------------------------------------------------------------

fn write_node<N: GpNode + fmt::Display>(node: &Node<N>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl<N: GpNode + fmt::Display + Default> fmt::Display for GpChromosome<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_node(&self.root, f)
    }
}

// ---------------------------------------------------------------------------
// LinearChromosome impl — required by some operator factories that bound on it
// (e.g., `survivor::factory`). All methods panic because `GpChromosome` has no
// flat DNA representation; use `GpGa<N>` (which dispatches GP-specific operators)
// instead of routing tree chromosomes through linear-chromosome code paths.
// ---------------------------------------------------------------------------

impl<N: GpNode + Default> LinearChromosome for GpChromosome<N> {
    /// # Panics
    ///
    /// Always panics. `GpChromosome` has no flat DNA slice — use `GpGa<N>`.
    fn dna(&self) -> &[Self::Gene] {
        panic!(
            "GpChromosome::dna() is not supported — use GpChromosome with GpGa, not Ga. \
             TreeChromosome has no flat DNA slice."
        )
    }

    /// # Panics
    ///
    /// Always panics. `GpChromosome` has no flat DNA slice — use `GpGa<N>`.
    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        panic!(
            "GpChromosome::dna_mut() is not supported — use GpChromosome with GpGa, not Ga. \
             TreeChromosome has no flat DNA slice."
        )
    }

    /// # Panics
    ///
    /// Always panics. `GpChromosome` has no flat DNA slice — use `GpGa<N>`.
    fn set_dna<'a>(&mut self, _dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        panic!(
            "GpChromosome::set_dna() is not supported — use GpChromosome with GpGa, not Ga. \
             TreeChromosome has no flat DNA slice."
        )
    }

    /// No-op.
    ///
    /// `GpGa` owns the fitness function, not the chromosome. Use
    /// [`GpChromosome::set_tree_fitness_fn`] to install a tree-based fitness
    /// function directly, or let `GpGa` install it for the whole population.
    fn set_fitness_fn<F>(&mut self, _fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static,
    {
        // no-op: GpGa owns the fitness fn, not the chromosome
        self
    }
}
