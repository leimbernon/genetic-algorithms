//! GP population initializers.
//!
//! This module provides the two tree-generation methods used by the GP engine:
//!
//! - `full_tree` (crate-private) — generates a fully-populated tree where every
//!   branch reaches exactly `max_depth`. Produces bushy, large trees.
//! - `grow_tree` (crate-private) — generates a tree where branches can stop at
//!   any depth (50 % chance of a terminal at each level). Produces
//!   variable-size trees.
//! - [`ramped_half_and_half`] — standard GP initialization combining both
//!   methods across a range of depths, maximizing initial population diversity.

use super::chromosome::GpChromosome;
use super::node::{grow_tree, GpNode, Node};
use rand::Rng;

/// Generates a fully-populated tree of exactly `max_depth` levels.
///
/// Every internal branch is extended until `max_depth - 1` children reach
/// terminal nodes. If `max_depth <= 1` or the node type has no functions,
/// a single terminal is returned instead.
///
/// # Arguments
///
/// * `max_depth` — target depth for the generated tree
/// * `rng` — random number generator
pub(crate) fn full_tree<N: GpNode>(max_depth: usize, rng: &mut impl Rng) -> Node<N> {
    if max_depth <= 1 {
        return Node::Terminal(N::sample_random_terminal(rng));
    }
    let functions = N::all_functions();
    if functions.is_empty() {
        return Node::Terminal(N::sample_random_terminal(rng));
    }
    let idx = rng.random_range(0..functions.len());
    let f = functions[idx].clone();
    let arity = f.arity();
    let children: Vec<Box<Node<N>>> = (0..arity)
        .map(|_| Box::new(full_tree::<N>(max_depth - 1, rng)))
        .collect();
    Node::Function { value: f, children }
}

/// Standard GP initialization: ramped half-and-half.
///
/// Generates `pop_size` chromosomes by iterating over depths from 2 to
/// `init_max_depth` and producing two trees per depth per slot: one using the
/// `full` method (maximally bushy) and one using the `grow` method (variable
/// size). This combination maximizes initial population diversity in both size
/// and shape.
///
/// If `pop_size` is not evenly divisible by the number of depth levels, the
/// remainder is filled with `grow_tree(init_max_depth)` individuals.
///
/// # Arguments
///
/// * `pop_size` — desired population size (must be > 0)
/// * `init_max_depth` — maximum depth during initialization (must be >= 2 for
///   ramping to operate; depths in `2..=init_max_depth` are used)
/// * `rng` — random number generator
///
/// # Panics
///
/// Does not panic. Callers should use validated [`GpConfiguration`](super::GpConfiguration)
/// which enforces `init_max_depth > 0` and `population_size > 0`.
pub fn ramped_half_and_half<N: GpNode + Default>(
    pop_size: usize,
    init_max_depth: usize,
    rng: &mut impl Rng,
) -> Vec<GpChromosome<N>> {
    let depth_range = 2..=init_max_depth.max(2);
    let num_depths = depth_range.clone().count().max(1);
    let per_depth = (pop_size / num_depths).max(1);

    let mut pop: Vec<GpChromosome<N>> = Vec::with_capacity(pop_size);

    'outer: for d in depth_range {
        for i in 0..per_depth {
            if pop.len() >= pop_size {
                break 'outer;
            }
            let root = if i % 2 == 0 {
                full_tree::<N>(d, rng)
            } else {
                grow_tree::<N>(d, rng)
            };
            pop.push(GpChromosome::with_root(Box::new(root)));
        }
    }

    // Fill remainder if pop_size is not evenly divisible by num_depths.
    while pop.len() < pop_size {
        let root = grow_tree::<N>(init_max_depth.max(2), rng);
        pop.push(GpChromosome::with_root(Box::new(root)));
    }

    pop
}
