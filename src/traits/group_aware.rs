//! Group-aware chromosome trait for multi-group crossover dispatch.
//!
//! The [`GroupAware`] trait exposes a `group_ranges()` method that returns
//! `(start, end)` index pairs for each independent permutation group in a
//! chromosome's DNA. This trait enables [`multi_group_pmx`] and
//! [`multi_group_ox`] crossover operators to apply PMX/OX within each group
//! independently, without crossing group boundaries.
//!
//! [`multi_group_pmx`]: crate::operations::crossover::multi_group_pmx
//! [`multi_group_ox`]: crate::operations::crossover::multi_group_ox
//!
//! # Design note
//!
//! This trait exists to enable generic dispatch in the crossover operator layer.
//! The `CrossoverOperator` trait is generic over `U: LinearChromosome`; to call
//! `group_ranges()` from a dispatch arm, the bound `U: LinearChromosome + GroupAware`
//! is added to the `multi_group_pmx` and `multi_group_ox` functions. Existing
//! chromosomes do NOT implement `GroupAware`, so they cannot accidentally invoke
//! the multi-group crossovers — `OperatorCompat` also guards against this at
//! `Ga::build()` time.

/// A chromosome that exposes its group boundaries for multi-group crossover.
///
/// Implement this trait on any chromosome type where the DNA is the concatenation
/// of independent permutations over disjoint groups. `group_ranges()` returns
/// the `(start, end)` index pairs (inclusive, 0-based) for each group.
///
/// For a chromosome with groups of sizes `[3, 3, 2]`:
///
/// ```text
/// group_ranges() → [(0, 2), (3, 5), (6, 7)]
/// ```
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::GroupAware;
/// use genetic_algorithms::chromosomes::MultiUniqueChromosome;
///
/// let c = MultiUniqueChromosome::<i32>::new(vec![vec![0, 1, 2], vec![10, 20, 30]]);
/// let ranges = c.group_ranges();
/// assert_eq!(ranges, vec![(0, 2), (3, 5)]);
/// ```
pub trait GroupAware {
    /// Returns `(start, end)` index pairs (inclusive) for each permutation group.
    ///
    /// - `start` is the index of the first gene in the group.
    /// - `end` is the index of the last gene in the group.
    /// - Empty groups are skipped.
    ///
    /// Returns an empty `Vec` if the chromosome has no groups.
    fn group_ranges(&self) -> Vec<(usize, usize)>;
}
