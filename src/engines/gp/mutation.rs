//! GP mutation operators.
//!
//! [`GpMutation`] is the tree-specific mutation enum for Genetic Programming.
//! It is dispatched directly by the `GpGa` engine (Wave 2) — it is NOT part of
//! the global `Mutation` factory chain used by `Ga`.
//!
//! # Variants
//!
//! | Variant | Description |
//! |---------|-------------|
//! | `SubtreeMutation` | Replaces a random subtree with a freshly grown random tree |
//! | `PointMutation` | Replaces each node's value with a same-arity alternative (tree shape preserved) |
//! | `HoistMutation` | Replaces a subtree with one of its own descendants (tree always shrinks) |
//!
//! # Bloat control
//!
//! `SubtreeMutation` checks depth/size limits via `check_limits` after generating
//! the new subtree. `PointMutation` and `HoistMutation` cannot increase tree size
//! so they require no bloat check.

use super::chromosome::GpChromosome;
use super::node::{check_limits, grow_tree, GpNode, Node};
use crate::error::GaError;
use rand::Rng;

// ---------------------------------------------------------------------------
// GpMutation enum
// ---------------------------------------------------------------------------

/// Tree mutation operator for Genetic Programming.
///
/// Dispatched directly by `GpGa`; not wired into the global `Mutation` enum.
#[derive(Clone, Debug)]
pub enum GpMutation {
    /// Replaces a randomly chosen subtree with a freshly grown random tree.
    ///
    /// The new subtree is grown using the "grow" method up to `mutation_max_depth`.
    /// After replacement the whole chromosome is checked against the engine limits.
    SubtreeMutation {
        /// Maximum depth for the newly generated replacement subtree.
        mutation_max_depth: usize,
    },
    /// Replaces each node's primitive value with a same-arity alternative.
    ///
    /// - For terminal nodes: replaces with a fresh terminal from
    ///   `N::sample_random_terminal`.
    /// - For function nodes: picks a random alternative from `N::all_functions()`
    ///   that has the same arity. If no alternative exists, the node is skipped
    ///   (no error, no panic).
    ///
    /// Tree shape (node count, depth) is always preserved.
    PointMutation {
        /// Per-node mutation probability in [0.0, 1.0].
        p_per_node: f64,
    },
    /// Replaces a randomly chosen Function subtree with one of its own descendants.
    ///
    /// The selected descendant (S2) is picked uniformly from the subtree rooted
    /// at S1 (a randomly chosen Function node). The parent's reference to S1 is
    /// then replaced with S2, causing the chromosome to shrink.
    ///
    /// On a terminal root, the mutation is a no-op and returns `Ok(())`.
    HoistMutation,
}

impl GpMutation {
    /// Applies the mutation operator to a chromosome in place.
    ///
    /// # Parameters
    ///
    /// - `chromosome` — chromosome to mutate (modified in place)
    /// - `max_depth` — maximum allowed depth after mutation
    /// - `max_node_count` — maximum allowed node count after mutation
    /// - `rng` — random number generator
    ///
    /// # Errors
    ///
    /// Only `SubtreeMutation` can return an error (bloat limit exceeded).
    pub fn apply<N: GpNode + Clone>(
        &self,
        chromosome: &mut GpChromosome<N>,
        max_depth: usize,
        max_node_count: usize,
        rng: &mut impl Rng,
    ) -> Result<(), GaError> {
        match self {
            GpMutation::SubtreeMutation { mutation_max_depth } => subtree_mutation(
                chromosome,
                max_depth,
                max_node_count,
                *mutation_max_depth,
                rng,
            ),
            GpMutation::PointMutation { p_per_node } => {
                point_mutation(chromosome, *p_per_node, rng);
                Ok(())
            }
            GpMutation::HoistMutation => {
                hoist_mutation(chromosome, rng);
                Ok(())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SubtreeMutation
// ---------------------------------------------------------------------------

fn subtree_mutation<N: GpNode + Clone>(
    chromosome: &mut GpChromosome<N>,
    max_depth: usize,
    max_node_count: usize,
    mutation_max_depth: usize,
    rng: &mut impl Rng,
) -> Result<(), GaError> {
    let n = chromosome.root.node_count();
    let target = rng.random_range(0..n);

    // Generate the replacement subtree.
    let new_subtree = grow_tree::<N>(mutation_max_depth, rng);

    // Work on a clone; only commit the replacement if limits are satisfied.
    // This prevents chromosome.root from being left in a bloated state when
    // check_limits returns Err (CR-01).
    let mut candidate = chromosome.root.clone();
    replace_node_in_place(&mut candidate, target, Box::new(new_subtree));
    check_limits(&candidate, max_depth, max_node_count)?;
    chromosome.root = candidate;

    Ok(())
}

// ---------------------------------------------------------------------------
// PointMutation
// ---------------------------------------------------------------------------

fn point_mutation<N: GpNode + Clone>(
    chromosome: &mut GpChromosome<N>,
    p_per_node: f64,
    rng: &mut impl Rng,
) {
    point_mutation_node(&mut chromosome.root, p_per_node, rng);
}

fn point_mutation_node<N: GpNode + Clone>(
    node: &mut Box<Node<N>>,
    p_per_node: f64,
    rng: &mut impl Rng,
) {
    if rng.random::<f64>() < p_per_node {
        match node.as_mut() {
            Node::Terminal(v) => {
                *v = N::sample_random_terminal(rng);
            }
            Node::Function { value, .. } => {
                let arity = value.arity();
                let alternatives: Vec<N> = N::all_functions()
                    .into_iter()
                    .filter(|f| f.arity() == arity)
                    .collect();
                if !alternatives.is_empty() {
                    let idx = rng.random_range(0..alternatives.len());
                    *value = alternatives[idx].clone();
                }
                // If no alternatives exist, silently skip (no panic, no error).
            }
        }
    }

    // Recurse into children regardless of whether this node was mutated.
    if let Node::Function { children, .. } = node.as_mut() {
        for child in children.iter_mut() {
            point_mutation_node(child, p_per_node, rng);
        }
    }
}

// ---------------------------------------------------------------------------
// HoistMutation
// ---------------------------------------------------------------------------

fn hoist_mutation<N: GpNode + Clone>(chromosome: &mut GpChromosome<N>, rng: &mut impl Rng) {
    // Collect pre-order indices of all Function nodes.
    let mut function_indices: Vec<usize> = Vec::new();
    collect_function_indices(&chromosome.root, &mut function_indices, &mut 0);

    if function_indices.is_empty() {
        // Terminal root — nothing to hoist; no-op.
        return;
    }

    // Pick a random Function node S1.
    let s1_idx_in_list = rng.random_range(0..function_indices.len());
    let s1_preorder = function_indices[s1_idx_in_list];

    // Clone the subtree at S1 to work with it.
    let s1_node =
        clone_subtree_at_index_local(&chromosome.root, s1_preorder).expect("s1_preorder is valid");

    // Count nodes in S1 to pick a random descendant S2.
    let s1_size = s1_node.node_count();

    // S2 is a random node within S1's subtree (including S1 itself would be a
    // no-op since S1 would replace S1). Pick from [1, s1_size) to skip S1
    // itself; if S1 has no descendants (arity=0 function — shouldn't happen but
    // be safe), just use 0.
    let s2_local_idx = if s1_size > 1 {
        rng.random_range(1..s1_size)
    } else {
        0
    };

    // Clone S2 (relative index within S1's subtree).
    let s2_node = clone_subtree_at_index_local(&s1_node, s2_local_idx)
        .expect("s2_local_idx is valid within s1_node");

    // Replace S1 with S2 in the full chromosome tree.
    replace_node_in_place(&mut chromosome.root, s1_preorder, Box::new(s2_node));
}

/// Collects pre-order indices of all Function nodes in the tree.
fn collect_function_indices<N: GpNode>(
    node: &Node<N>,
    indices: &mut Vec<usize>,
    current_index: &mut usize,
) {
    let my_index = *current_index;
    *current_index += 1;

    match node {
        Node::Terminal(_) => {}
        Node::Function { children, .. } => {
            indices.push(my_index);
            for child in children {
                collect_function_indices(child, indices, current_index);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared traversal helpers (local to this module)
// ---------------------------------------------------------------------------

/// Clones the subtree at pre-order index `target` in `root`.
fn clone_subtree_at_index_local<N: GpNode + Clone>(
    root: &Node<N>,
    target: usize,
) -> Option<Node<N>> {
    let mut idx = 0usize;
    clone_at_idx(root, target, &mut idx)
}

fn clone_at_idx<N: GpNode + Clone>(
    node: &Node<N>,
    target: usize,
    current: &mut usize,
) -> Option<Node<N>> {
    let my_index = *current;
    *current += 1;

    if my_index == target {
        return Some(node.clone());
    }

    match node {
        Node::Terminal(_) => None,
        Node::Function { children, .. } => {
            for child in children {
                if let Some(found) = clone_at_idx(child, target, current) {
                    return Some(found);
                }
            }
            None
        }
    }
}

/// Replaces the node at pre-order index `target` in `root` with `replacement` in place.
fn replace_node_in_place<N: GpNode>(
    root: &mut Box<Node<N>>,
    target: usize,
    replacement: Box<Node<N>>,
) {
    let mut replacer = NodeReplacer::new(target, replacement);
    let mut idx = 0usize;
    replacer.run(root, &mut idx);
}

/// Stateful helper that performs a single in-place subtree replacement.
struct NodeReplacer<N: GpNode> {
    target: usize,
    replacement: Option<Box<Node<N>>>,
}

impl<N: GpNode> NodeReplacer<N> {
    fn new(target: usize, replacement: Box<Node<N>>) -> Self {
        NodeReplacer {
            target,
            replacement: Some(replacement),
        }
    }

    fn run(&mut self, node: &mut Box<Node<N>>, current_index: &mut usize) {
        if self.replacement.is_none() {
            return;
        }

        let my_index = *current_index;
        *current_index += 1;

        if my_index == self.target {
            let replacement = self.replacement.take().unwrap();
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
