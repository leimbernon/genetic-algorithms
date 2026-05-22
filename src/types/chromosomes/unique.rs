//! Unique chromosome implementation.
//!
//! A [`UniqueChromosome<T>`] chromosome stores a vector of
//! [`UniqueGenotype<T>`](crate::genotypes::UniqueGenotype) genes that form a
//! permutation of a shared alphabet. No gene value appears more than once —
//! all alphabet elements are present.
//!
//! This chromosome type is suited for combinatorial optimization problems
//! where the solution is a permutation: job scheduling, TSP, routing, etc.
//!
//! The shared alphabet is stored as an `Arc<[T]>` field on the chromosome,
//! keeping gene structs lightweight (`id + value` only). Cloning a chromosome
//! is O(n) for the DNA vector and O(1) for the alphabet (atomic refcount).

use crate::fitness::FitnessFnWrapper;
use crate::genotypes::UniqueGenotype;
use crate::operations::mutation::ValueMutable;
use crate::operations::{Crossover, Mutation};
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

/// A chromosome whose DNA is a permutation of a shared alphabet.
///
/// Each gene holds a value drawn from the `alphabet` — no duplicates, all
/// elements present. The alphabet is shared across all chromosomes in the
/// population via `Arc<[T]>`, so cloning is inexpensive.
///
/// # OperatorCompat restriction
///
/// `UniqueChromosome<T>` only accepts permutation-safe operators.
/// Configuring an incompatible operator causes `Ga::build()` to return
/// `Err(GaError::ConfigurationError)` before any generation runs.
///
/// | Category | Allowed |
/// |----------|---------|
/// | Crossover | `Pmx`, `Order`, `EdgeRecombination`, `Clone`, `Rejuvenate` |
/// | Mutation | `Insertion`, `Swap`, `Inversion` |
///
/// # Examples
///
/// ```
/// use genetic_algorithms::chromosomes::UniqueChromosome;
/// use genetic_algorithms::traits::ChromosomeT;
///
/// let chromosome = UniqueChromosome::<i32>::default();
/// assert!(chromosome.fitness().is_nan());
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct UniqueChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<UniqueGenotype<T>>,
    /// Shared alphabet for this chromosome — all permutation values come from here.
    pub alphabet: Arc<[T]>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<UniqueGenotype<T>>,
}

/// `UniqueChromosome<T>` restricts operators to permutation-safe crossovers and
/// mutations. `Ga::build()` enforces this at build time via `operator_compat_check`.
impl<T: Sync + Send + Clone + Default + Debug + 'static> OperatorCompat for UniqueChromosome<T> {
    fn valid_crossovers() -> Option<&'static [Crossover]> {
        Some(&[
            Crossover::Pmx,
            Crossover::Order,
            Crossover::EdgeRecombination,
            Crossover::Clone,
            Crossover::Rejuvenate,
        ])
    }

    fn valid_mutations() -> Option<&'static [Mutation]> {
        Some(&[Mutation::Insertion, Mutation::Swap, Mutation::Inversion])
    }
}

impl<T: Sync + Send + Clone + Default + Debug> Default for UniqueChromosome<T> {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            alphabet: Arc::from([]),
            fitness: f64::NAN,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        }
    }
}

impl<T: Sync + Send + Clone + Default + Debug> UniqueChromosome<T> {
    /// Creates a new `UniqueChromosome` with default values (empty DNA, empty alphabet).
    pub fn new() -> Self {
        Self::default()
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

impl<T: Sync + Send + Clone + Default + Debug + 'static> ChromosomeT for UniqueChromosome<T> {
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

impl<T: Sync + Send + Clone + Default + Debug + 'static> LinearChromosome for UniqueChromosome<T> {
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
}

impl<T: Sync + Send + Clone + Default + Debug> fmt::Display for UniqueChromosome<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}

/// `UniqueChromosome<T>` inherits the default fallback behavior from `ValueMutable`
/// (all non-permutation mutations fall back to swap). The `OperatorCompat` restriction
/// ensures that incompatible mutations are rejected at `Ga::build()` before any
/// generation runs.
impl<T: Sync + Send + Clone + Default + Debug + 'static> ValueMutable for UniqueChromosome<T> {}
