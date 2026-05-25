//! GP tree node trait and recursive node enum.
//!
//! [`GpNode`] is the user-facing trait: implement it on your own enum to define
//! the function and terminal set for genetic programming.
//!
//! [`Node<N>`] is the library-provided recursive tree structure. It stores a
//! tree of `GpNode` values and supports depth computation, node counting, and
//! an iterative `Drop` implementation to prevent stack overflows on deep trees.

use rand::Rng;
use std::mem;

/// Trait that every GP primitive set must implement.
///
/// Implement `GpNode` on your own enum to define the function set (non-terminals
/// with arity > 0) and the terminal set (leaves with arity == 0). The engine uses
/// this trait to build, evaluate, and mutate expression trees.
///
/// # Required methods
///
/// | Method | Description |
/// |--------|-------------|
/// | `arity` | Number of child arguments this node consumes |
/// | `evaluate` | Evaluate the node given pre-evaluated child values |
/// | `sample_random_terminal` | Produce a fresh terminal node (for ERC support) |
/// | `all_functions` | Enumerate all non-terminal variants (for point mutation) |
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Clone, Debug)]
/// enum MyNode { Add, Mul, Const(f64) }
///
/// impl GpNode for MyNode {
///     fn arity(&self) -> usize { match self { MyNode::Add | MyNode::Mul => 2, MyNode::Const(_) => 0 } }
///     fn evaluate(&self, args: &[f64]) -> f64 { match self { MyNode::Add => args[0] + args[1], MyNode::Mul => args[0] * args[1], MyNode::Const(v) => *v } }
///     fn sample_random_terminal(rng: &mut impl Rng) -> Self { MyNode::Const(rng.gen_range(-1.0..=1.0)) }
///     fn all_functions() -> Vec<Self> { vec![MyNode::Add, MyNode::Mul] }
/// }
/// ```
pub trait GpNode: Clone + Send + Sync + 'static {
    /// Returns the number of child arguments this node requires.
    ///
    /// 0 means this is a terminal (leaf); > 0 means it is a function node.
    fn arity(&self) -> usize;

    /// Evaluates this node given the already-evaluated values of its children.
    ///
    /// `args.len()` equals `self.arity()`. The engine guarantees this invariant
    /// during tree evaluation.
    fn evaluate(&self, args: &[f64]) -> f64;

    /// Returns `true` if this node is a terminal (leaf).
    ///
    /// The default implementation returns `self.arity() == 0`.
    fn is_terminal(&self) -> bool {
        self.arity() == 0
    }

    /// Produces a fresh terminal node, optionally using the provided RNG.
    ///
    /// Used during ramped half-and-half initialization and subtree mutation when
    /// a new terminal is needed. For ephemeral random constants (ERCs), generate
    /// the constant value here (e.g., `MyNode::Const(rng.gen_range(-1.0..=1.0))`).
    ///
    /// # Panics
    ///
    /// Implementations that have no terminal variants should panic with
    /// `unreachable!("This node type has no terminals")` — calling this method
    /// on a purely functional node type is a programming error.
    fn sample_random_terminal(rng: &mut impl Rng) -> Self;

    /// Returns all function (non-terminal) variants of this node type.
    ///
    /// Used by point mutation to find a compatible replacement for an existing
    /// function node (same arity). The engine iterates this list and filters by
    /// matching arity.
    fn all_functions() -> Vec<Self>;
}

/// A recursive expression tree node.
///
/// `Node<N>` stores a tree of `N: GpNode` values. Function nodes hold their
/// children in a `Vec<Box<Node<N>>>` (one child per `N::arity()`). Terminal
/// nodes hold a single leaf value.
///
/// # Memory safety
///
/// `Node<N>` implements a custom iterative [`Drop`] to avoid stack overflow
/// when dropping very deep trees. The default recursive drop would overflow the
/// stack for trees with hundreds of thousands of nodes.
#[derive(Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "N: serde::Serialize",
        deserialize = "N: for<'de2> serde::Deserialize<'de2>"
    ))
)]
pub enum Node<N: GpNode> {
    /// A function node with child sub-trees.
    Function {
        /// The function primitive.
        value: N,
        /// Child sub-trees. Length must equal `value.arity()`.
        children: Vec<Box<Node<N>>>,
    },
    /// A terminal (leaf) node.
    Terminal(N),
}

impl<N: GpNode> Node<N> {
    /// Returns the depth of this tree.
    ///
    /// A single terminal has depth 1. A function node's depth is
    /// `1 + max(child depths)`.
    pub fn depth(&self) -> usize {
        match self {
            Node::Terminal(_) => 1,
            Node::Function { children, .. } => {
                1 + children
                    .iter()
                    .map(|c| c.depth())
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    /// Returns the total number of nodes in this tree.
    ///
    /// A terminal counts as 1. A function node counts as `1 + sum(child counts)`.
    pub fn node_count(&self) -> usize {
        match self {
            Node::Terminal(_) => 1,
            Node::Function { children, .. } => {
                1 + children.iter().map(|c| c.node_count()).sum::<usize>()
            }
        }
    }
}

/// Custom iterative `Drop` to prevent stack overflow on very deep trees.
///
/// The default recursive drop would overflow the call stack for trees with
/// depth in the thousands. This implementation drains children iteratively
/// using a worklist.
impl<N: GpNode> Drop for Node<N> {
    fn drop(&mut self) {
        // We only need special handling for Function nodes that own children.
        // Terminal nodes have no heap-allocated children to drain.
        if let Node::Function { children, .. } = self {
            // Move current node's children into a worklist (avoids drain+collect).
            let mut worklist: Vec<Box<Node<N>>> = mem::take(children);
            while let Some(mut node) = worklist.pop() {
                if let Node::Function { children, .. } = &mut *node {
                    // Move grandchildren onto the worklist before `node` is dropped.
                    worklist.append(children);
                }
                // `node` is dropped here — it is now a Terminal or an empty
                // Function, so its own drop will not recurse further.
            }
        }
    }
}
