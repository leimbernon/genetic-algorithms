//! GP subtree crossover operator.
//!
//! [`GpCrossover`] is the tree-specific crossover enum for Genetic Programming.
//! It is dispatched directly by the `GpGa` engine (Wave 2) — it is NOT part of
//! the global `Crossover` factory chain used by `Ga`.
//!
//! # Variants
//!
//! | Variant | Description |
//! |---------|-------------|
//! | `SubtreeCrossover` | Swaps a randomly chosen subtree between two parent chromosomes |
//!
//! # Bloat control
//!
//! After each swap the resulting offspring is checked against `max_depth` and
//! `max_node_count`. If either limit is violated the operator returns
//! [`GaError::TreeDepthExceeded`] or [`GaError::TreeSizeExceeded`] respectively.

use super::chromosome::GpChromosome;
use super::node::{check_limits, GpNode, Node};
use crate::error::GaError;
use rand::Rng;

// ---------------------------------------------------------------------------
// GpCrossover enum
// ---------------------------------------------------------------------------

/// Tree crossover operator for Genetic Programming.
///
/// Dispatched directly by `GpGa`; not wired into the global `Crossover` enum.
#[derive(Clone, Debug)]
pub enum GpCrossover {
    /// Standard GP subtree crossover: swaps randomly selected subtrees between
    /// two parent chromosomes and enforces depth/size bloat limits.
    SubtreeCrossover,
}

impl GpCrossover {
    /// Applies the crossover operator to two parent chromosomes.
    ///
    /// # Parameters
    ///
    /// - `p1`, `p2` — parent chromosomes (immutable borrows; cloned internally)
    /// - `max_depth` — maximum allowed depth for the offspring trees
    /// - `max_node_count` — maximum allowed node count for the offspring trees
    /// - `rng` — random number generator
    ///
    /// # Errors
    ///
    /// Returns [`GaError::TreeDepthExceeded`] or [`GaError::TreeSizeExceeded`] if
    /// either child exceeds the configured limits.
    pub fn apply<N: GpNode + Clone>(
        &self,
        p1: &GpChromosome<N>,
        p2: &GpChromosome<N>,
        max_depth: usize,
        max_node_count: usize,
        rng: &mut impl Rng,
    ) -> Result<(GpChromosome<N>, GpChromosome<N>), GaError> {
        match self {
            GpCrossover::SubtreeCrossover => {
                subtree_crossover(p1, p2, max_depth, max_node_count, rng)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Subtree traversal helpers
// ---------------------------------------------------------------------------

/// Extracts (clones) the subtree at pre-order index `target` from `root`.
///
/// Pre-order traversal: root first, then children left-to-right.
/// Index 0 = root itself.
fn clone_subtree_at_index<N: GpNode + Clone>(root: &Node<N>, target: usize) -> Option<Node<N>> {
    let mut current_index = 0usize;
    clone_at_index_impl(root, target, &mut current_index)
}

fn clone_at_index_impl<N: GpNode + Clone>(
    node: &Node<N>,
    target: usize,
    current_index: &mut usize,
) -> Option<Node<N>> {
    let my_index = *current_index;
    *current_index += 1;

    if my_index == target {
        return Some(node.clone());
    }

    match node {
        Node::Terminal(_) => None,
        Node::Function { children, .. } => {
            for child in children {
                if let Some(found) = clone_at_index_impl(child, target, current_index) {
                    return Some(found);
                }
            }
            None
        }
    }
}

/// Stateful replacer that traverses a tree and swaps the node at `target`
/// with `replacement`.
///
/// Using a struct avoids moving `replacement` in a loop (Rust borrow rules).
struct Replacer<N: GpNode> {
    target: usize,
    replacement: Option<Box<Node<N>>>,
}

impl<N: GpNode> Replacer<N> {
    fn new(target: usize, replacement: Box<Node<N>>) -> Self {
        Replacer {
            target,
            replacement: Some(replacement),
        }
    }

    /// Performs the replacement starting from `node` at pre-order position `*current_index`.
    fn run(&mut self, node: &mut Box<Node<N>>, current_index: &mut usize) {
        // Once replaced, short-circuit the remaining traversal.
        if self.replacement.is_none() {
            return;
        }

        let my_index = *current_index;
        *current_index += 1;

        if my_index == self.target {
            let replacement = self.replacement.take().unwrap();
            // Assign directly; the evicted subtree is dropped here (no need to retain it).
            *node = replacement;
            return;
        }

        if let Node::Function { children, .. } = node.as_mut() {
            for child in children.iter_mut() {
                self.run(child, current_index);
                if self.replacement.is_none() {
                    return;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SubtreeCrossover implementation
// ---------------------------------------------------------------------------

fn subtree_crossover<N: GpNode + Clone>(
    p1: &GpChromosome<N>,
    p2: &GpChromosome<N>,
    max_depth: usize,
    max_node_count: usize,
    rng: &mut impl Rng,
) -> Result<(GpChromosome<N>, GpChromosome<N>), GaError> {
    let n1 = p1.root.node_count();
    let n2 = p2.root.node_count();

    // Pick random crossover points (pre-order indices).
    let i1 = rng.random_range(0..n1);
    let i2 = rng.random_range(0..n2);

    // Clone both parent roots to produce mutable offspring.
    let mut child1_root = p1.root.clone();
    let mut child2_root = p2.root.clone();

    // Extract (clone) the subtrees that will be exchanged.
    let sub1 = clone_subtree_at_index(&p1.root, i1).expect("i1 is within p1.root bounds");
    let sub2 = clone_subtree_at_index(&p2.root, i2).expect("i2 is within p2.root bounds");

    // Swap: insert sub2 into child1 at position i1.
    {
        let mut replacer = Replacer::new(i1, Box::new(sub2));
        let mut idx = 0usize;
        replacer.run(&mut child1_root, &mut idx);
    }
    // Swap: insert sub1 into child2 at position i2.
    {
        let mut replacer = Replacer::new(i2, Box::new(sub1));
        let mut idx = 0usize;
        replacer.run(&mut child2_root, &mut idx);
    }

    // Bloat control — depth checked before size (per check_limits contract).
    check_limits(&child1_root, max_depth, max_node_count)?;
    check_limits(&child2_root, max_depth, max_node_count)?;

    Ok((
        GpChromosome::with_root(child1_root),
        GpChromosome::with_root(child2_root),
    ))
}
