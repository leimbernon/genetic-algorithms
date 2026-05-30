//! Multi-group unique chromosome implementation.
//!
//! A [`MultiUniqueChromosome<T>`] stores a vector of [`UniqueGenotype<T>`] genes
//! that represent the concatenation of independent permutations — one permutation
//! per group. Each group has its own alphabet stored as `Arc<[T]>`. Group boundaries
//! are derived on-the-fly from the alphabet lengths via [`group_ranges()`].
//!
//! This chromosome type is suited for multi-group combinatorial optimization
//! problems such as multi-machine job scheduling, vehicle routing with vehicle
//! types, or any problem where the solution is a set of independent permutations
//! over disjoint value sets.
//!
//! # Group semantics
//!
//! Given groups of sizes `[3, 3, 2]`, the DNA has length 8 and the groups occupy:
//! - Group 0: indices `0..=2`
//! - Group 1: indices `3..=5`
//! - Group 2: indices `6..=7`
//!
//! [`group_ranges()`]: MultiUniqueChromosome::group_ranges
//!
//! # OperatorCompat restriction
//!
//! `MultiUniqueChromosome<T>` only accepts multi-group-aware crossover operators.
//! Standard `Pmx` and `Order` are **not** valid — they would corrupt group membership.
//! Configuring an incompatible operator causes `Ga::build()` to return
//! `Err(GaError::ConfigurationError)` before any generation runs.
//!
//! | Category | Allowed |
//! |----------|---------|
//! | Crossover | `MultiGroupPmx`, `MultiGroupOx`, `Clone`, `Rejuvenate` |
//! | Mutation | `Insertion`, `Swap`, `Inversion` |

use crate::fitness::FitnessFnWrapper;
use crate::genotypes::UniqueGenotype;
use crate::operations::mutation::ValueMutable;
use crate::operations::{Crossover, Mutation};
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat, VectorFitness};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

/// A chromosome whose DNA is the concatenation of independent permutations over disjoint groups.
///
/// Each group corresponds to an alphabet stored as `Arc<[T]>`. The DNA layout is:
///
/// ```text
/// [group_0_perm...][group_1_perm...][group_2_perm...]
/// ```
///
/// Call [`group_ranges()`] to retrieve `(start, end)` index pairs for each group.
///
/// # OperatorCompat restriction
///
/// Only `MultiGroupPmx`, `MultiGroupOx`, `Clone`, and `Rejuvenate` crossovers are
/// valid. Standard `Pmx` and `Order` are rejected at `Ga::build()` time.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::chromosomes::MultiUniqueChromosome;
///
/// let c = MultiUniqueChromosome::<i32>::new(vec![
///     vec![0, 1, 2],   // group 0: 3 genes
///     vec![10, 20, 30], // group 1: 3 genes
///     vec![100, 200],   // group 2: 2 genes
/// ]);
/// assert_eq!(c.group_ranges(), vec![(0, 2), (3, 5), (6, 7)]);
/// ```
///
/// [`group_ranges()`]: MultiUniqueChromosome::group_ranges
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct MultiUniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    /// Per-group alphabets. One `Arc<[T]>` per permutation group (D-12).
    /// Group sizes and boundaries are derived from alphabet lengths via [`group_ranges()`].
    ///
    /// [`group_ranges()`]: MultiUniqueChromosome::group_ranges
    pub groups: Vec<Arc<[T]>>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,
}

impl<T: Sync + Send + Clone + Default + Debug> MultiUniqueChromosome<T> {
    /// Creates a new `MultiUniqueChromosome` from a list of group alphabets.
    ///
    /// Each inner `Vec<T>` becomes one group. The DNA is initialized empty —
    /// populate it via an initialization closure or `set_dna()`.
    ///
    /// # Example
    ///
    /// ```
    /// use genetic_algorithms::chromosomes::MultiUniqueChromosome;
    ///
    /// let c = MultiUniqueChromosome::<i32>::new(vec![
    ///     vec![0, 1, 2],
    ///     vec![10, 20, 30],
    /// ]);
    /// assert_eq!(c.groups.len(), 2);
    /// ```
    pub fn new(groups: Vec<Vec<T>>) -> Self {
        let groups_arc: Vec<Arc<[T]>> = groups
            .into_iter()
            .map(|v| v.into_boxed_slice().into())
            .collect();
        Self {
            dna: Vec::new(),
            groups: groups_arc,
            fitness: f64::NAN,
            age: 0,
            fitness_values: Vec::new(),
            fitness_fn: FitnessFnWrapper::default(),
        }
    }

    /// Returns `(start, end)` index pairs for each group in the concatenated DNA.
    ///
    /// Derived from `self.groups` alphabet lengths — no separate storage needed.
    /// Empty groups are skipped defensively.
    ///
    /// # Example
    ///
    /// ```
    /// use genetic_algorithms::chromosomes::MultiUniqueChromosome;
    ///
    /// let c = MultiUniqueChromosome::<i32>::new(vec![
    ///     vec![0, 1, 2],   // size 3 → (0, 2)
    ///     vec![10, 20, 30], // size 3 → (3, 5)
    ///     vec![100, 200],   // size 2 → (6, 7)
    /// ]);
    /// assert_eq!(c.group_ranges(), vec![(0, 2), (3, 5), (6, 7)]);
    /// ```
    pub fn group_ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = Vec::with_capacity(self.groups.len());
        let mut start = 0usize;
        for group in &self.groups {
            if group.is_empty() {
                continue;
            }
            let end = start + group.len().saturating_sub(1);
            ranges.push((start, end));
            start = end + 1;
        }
        ranges
    }

    /// Returns a string representation of the chromosome's phenotype (gene values).
    pub fn phenotype(&self) -> String {
        self.dna
            .iter()
            .map(|g| format!("{:?}", g.value))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T: Sync + Send + Clone + Default + Debug> Default for MultiUniqueChromosome<T> {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            groups: Vec::new(),
            fitness: f64::NAN,
            age: 0,
            fitness_values: Vec::new(),
            fitness_fn: FitnessFnWrapper::default(),
        }
    }
}

/// `MultiUniqueChromosome<T>` restricts operators to multi-group-aware crossovers
/// and permutation-safe mutations. Standard `Pmx` and `Order` crossovers are
/// excluded — they do not respect group boundaries. `Ga::build()` enforces this
/// via `operator_compat_check`.
impl<T: Sync + Send + Clone + Default + Debug + 'static> OperatorCompat
    for MultiUniqueChromosome<T>
{
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        Some(&[
            Crossover::MultiGroupPmx,
            Crossover::MultiGroupOx,
            Crossover::Clone,
            Crossover::Rejuvenate,
        ])
    }

    fn valid_mutations() -> Option<&'static [Mutation]> {
        Some(&[Mutation::Insertion, Mutation::Swap, Mutation::Inversion])
    }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> ChromosomeT
    for MultiUniqueChromosome<T>
{
    type Gene = UniqueGenotype<T>;

    fn calculate_fitness(&mut self) {
        self.fitness = self.fitness_fn.call(&self.dna);
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

impl<T: Sync + Send + Clone + Default + Debug + 'static> VectorFitness
    for MultiUniqueChromosome<T>
{
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> LinearChromosome
    for MultiUniqueChromosome<T>
{
    fn dna(&self) -> &[Self::Gene] {
        &self.dna
    }

    fn dna_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.dna
    }

    /// Sets the chromosome DNA.
    ///
    /// - `Cow::Borrowed`: clones into internal storage.
    /// - `Cow::Owned`: moves the provided vector into internal storage (no extra clone).
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self {
        self.dna = match dna {
            Cow::Borrowed(slice) => slice.to_vec(),
            Cow::Owned(vec) => vec,
        };
        self
    }

    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[UniqueGenotype<T>]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }

    /// Returns `(start, end)` index pairs for each permutation group in the DNA.
    ///
    /// Overrides the `LinearChromosome` default (empty vec). Derived from the
    /// `groups` field alphabet lengths — same semantics as the inherent `group_ranges()`.
    fn group_ranges(&self) -> Vec<(usize, usize)> {
        // Delegate to the inherent method — same logic, no duplication.
        MultiUniqueChromosome::group_ranges(self)
    }
}

impl<T: Sync + Send + Clone + Default + Debug> fmt::Display for MultiUniqueChromosome<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}

/// `MultiUniqueChromosome<T>` inherits the default fallback behavior from `ValueMutable`.
/// The `OperatorCompat` restriction ensures that incompatible mutations are rejected
/// at `Ga::build()` before any generation runs.
impl<T: Sync + Send + Clone + Default + Debug + 'static> ValueMutable
    for MultiUniqueChromosome<T>
{
}

