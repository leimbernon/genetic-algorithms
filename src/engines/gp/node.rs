//! GP tree node trait and recursive node enum.
//!
//! [`GpNode`] is the user-facing trait: implement it on your own enum to define
//! the function and terminal set for genetic programming.
//!
//! [`Node<N>`] is the library-provided recursive tree structure. It stores a
//! tree of `GpNode` values and supports depth computation, node counting, and
//! an iterative `Drop` implementation to prevent stack overflows on deep trees.

use crate::error::GaError;
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
///     fn evaluate(&self, args: &[f64]) -> f64 {
///         match self {
///             MyNode::Add => args[0] + args[1],
///             MyNode::Mul => args[0] * args[1],
///             MyNode::Const(v) => *v,
///         }
///     }
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
///
/// # Serde and deep trees
///
/// When the `serde` feature is enabled, `Node<N>` derives `Serialize` and
/// `Deserialize` via the standard serde derive macros. For trees up to depth 64
/// (the GP engine's recommended maximum), stack usage is well within typical
/// system limits. For extremely deep trees (depth > ~500), use
/// `serde_stacker::Serializer` / `serde_stacker::Deserializer` wrappers at the
/// serialization call site to grow the stack dynamically.
///
/// # Security note
///
/// Checkpoint files should be treated as trusted input. Deserialization does
/// not bound allocation — an attacker-controlled JSON blob with an unbounded
/// `children` array could exhaust process memory.
#[derive(Clone, Debug)]
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
    ///
    /// Uses an explicit stack (iterative) to avoid call-stack overflow on deep
    /// trees, matching the iterative [`Drop`] implementation below.
    pub fn depth(&self) -> usize {
        let mut stack: Vec<(&Node<N>, usize)> = vec![(self, 1)];
        let mut max_depth = 0;
        while let Some((node, d)) = stack.pop() {
            if d > max_depth {
                max_depth = d;
            }
            if let Node::Function { children, .. } = node {
                for child in children {
                    stack.push((child, d + 1));
                }
            }
        }
        max_depth
    }

    /// Returns the total number of nodes in this tree.
    ///
    /// A terminal counts as 1. A function node counts as `1 + sum(child counts)`.
    ///
    /// Uses an explicit stack (iterative) to avoid call-stack overflow on deep
    /// trees, matching the iterative [`Drop`] implementation below.
    pub fn node_count(&self) -> usize {
        let mut stack: Vec<&Node<N>> = vec![self];
        let mut count = 0;
        while let Some(node) = stack.pop() {
            count += 1;
            if let Node::Function { children, .. } = node {
                for child in children {
                    stack.push(child);
                }
            }
        }
        count
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (pub(crate)) used by crossover.rs and mutation.rs
// ---------------------------------------------------------------------------

/// Checks that `node` does not violate the configured depth or size limits.
///
/// Returns [`GaError::TreeDepthExceeded`] if `node.depth() > max_depth` or
/// [`GaError::TreeSizeExceeded`] if `node.node_count() > max_node_count`.
///
/// Depth is checked first: a tree that is too deep is also too large, so the
/// depth error gives a more precise diagnosis.
pub(crate) fn check_limits<N: GpNode>(
    node: &Node<N>,
    max_depth: usize,
    max_node_count: usize,
) -> Result<(), GaError> {
    let d = node.depth();
    if d > max_depth {
        return Err(GaError::TreeDepthExceeded(format!(
            "depth {} exceeds max_depth {}",
            d, max_depth
        )));
    }
    let n = node.node_count();
    if n > max_node_count {
        return Err(GaError::TreeSizeExceeded(format!(
            "{} nodes exceeds max_node_count {}",
            n, max_node_count
        )));
    }
    Ok(())
}

/// Grows a random tree up to `max_depth` using the "grow" method.
///
/// At each level there is a 50% probability of generating a terminal (leaf)
/// unless `max_depth <= 1`, in which case a terminal is always returned.
/// If `N::all_functions()` returns an empty list, a terminal is always returned
/// regardless of depth.
///
/// # Arguments
///
/// * `max_depth` — maximum allowed depth for the generated subtree
/// * `rng` — random number generator
pub(crate) fn grow_tree<N: GpNode>(max_depth: usize, rng: &mut impl Rng) -> Node<N> {
    if max_depth <= 1 {
        return Node::Terminal(N::sample_random_terminal(rng));
    }
    let functions = N::all_functions();
    if functions.is_empty() {
        return Node::Terminal(N::sample_random_terminal(rng));
    }
    let p_terminal: f64 = 0.5;
    if rng.random::<f64>() < p_terminal {
        return Node::Terminal(N::sample_random_terminal(rng));
    }
    let idx = rng.random_range(0..functions.len());
    let f = functions[idx].clone();
    let arity = f.arity();
    let children: Vec<Box<Node<N>>> = (0..arity)
        .map(|_| Box::new(grow_tree::<N>(max_depth - 1, rng)))
        .collect();
    Node::Function { value: f, children }
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
