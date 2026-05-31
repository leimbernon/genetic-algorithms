//! Multi-range chromosome implementation.
//!
//! A [`MultiRangeChromosome<T>`] stores a vector of
//! [`MultiRangeGenotype<T>`](crate::genotypes::MultiRangeGenotype) genes, each
//! carrying its own `(lo, hi)` bounds and `mutation_rate`. This is suited for
//! real-valued optimization problems where each dimension has independent search
//! bounds and mutation step sizes (GEN-03).
//!
//! Unlike [`chromosomes::Range<T>`](crate::chromosomes::Range), no `Arc` indirection
//! is needed — bounds live directly on each gene (decision D-08). This allows
//! heterogeneous spaces: e.g., gene 0 in `[0.0, 1.0)` and gene 1 in `[10.0, 100.0)`.
//!
//! # OperatorCompat
//!
//! `MultiRangeChromosome<T>` imposes no operator restrictions. All standard
//! real-valued crossover and mutation operators (Gaussian, Creep, SinglePoint,
//! Uniform, etc.) are accepted. The empty `OperatorCompat` impl inherits the
//! default `None`-returning methods.

use crate::fitness::FitnessFnWrapper;
use crate::genotypes::MultiRangeGenotype;
use crate::operations::mutation::ValueMutable;
use crate::operations::mutation::gaussian::{multi_range_gaussian_mutation, GaussianConvertible};
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat, RealValued, VectorFitness};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;

/// A chromosome with per-gene independent bounds and mutation rates.
///
/// Each gene holds its own `(lo, hi)` bounds and `mutation_rate` as flat
/// fields — no `Arc` overhead. This is the correct type for real-valued
/// problems where each dimension occupies a different search space.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::chromosomes::MultiRangeChromosome;
/// use genetic_algorithms::traits::ChromosomeT;
///
/// let chromosome = MultiRangeChromosome::<f64>::default();
/// assert!(chromosome.fitness().is_nan());
/// assert!(chromosome.dna.is_empty());
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
pub struct MultiRangeChromosome<T: Sync + Send + Copy + Default + Debug> {
    pub dna: Vec<MultiRangeGenotype<T>>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<MultiRangeGenotype<T>>,
}

/// `MultiRangeChromosome<T>` imposes no operator restrictions — all crossovers
/// and mutations are accepted. The default `None`-returning methods are inherited
/// from the trait.
impl<T: Sync + Send + Copy + Default + Debug> OperatorCompat for MultiRangeChromosome<T> {}

impl<T: Sync + Send + Copy + Default + Debug> Default for MultiRangeChromosome<T> {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            fitness: f64::NAN,
            age: 0,
            fitness_values: Vec::new(),
            fitness_fn: FitnessFnWrapper::default(),
        }
    }
}

impl<T: Sync + Send + Copy + Default + Debug> MultiRangeChromosome<T> {
    /// Creates a new `MultiRangeChromosome` with default values (empty DNA, NaN fitness).
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a string representation of the chromosome's phenotype (gene values).
    pub fn phenotype(&self) -> String {
        self.dna
            .iter()
            .map(|gene| format!("{:?}", gene.value()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T: Sync + Send + Copy + Default + Debug + 'static> ChromosomeT for MultiRangeChromosome<T> {
    type Gene = MultiRangeGenotype<T>;

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

impl<T: Sync + Send + Copy + Default + Debug + 'static> VectorFitness for MultiRangeChromosome<T> {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl<T: Sync + Send + Copy + Default + Debug + 'static> LinearChromosome for MultiRangeChromosome<T> {
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
        F: Fn(&[MultiRangeGenotype<T>]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

impl<T: Sync + Send + Copy + Default + Debug + fmt::Display> fmt::Display
    for MultiRangeChromosome<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}

/// `RealValued` marker impl for `MultiRangeChromosome<T>`.
///
/// Enables `factory_multi_parent` dispatch for `Crossover::Undx`, `Crossover::Spx`,
/// and `Crossover::Pcx` on this type (Phase 51, Plan 02).
/// `SelfAdaptive` is intentionally omitted — that is scoped to Phase 48.
impl<T: Sync + Send + Copy + Default + Debug + 'static> RealValued for MultiRangeChromosome<T> {}

/// Per-gene Gaussian mutation for `MultiRangeChromosome<T>`.
///
/// Reads `gene.mutation_rate` as the noise scale (not the global `sigma`
/// argument) and clamps the result to `(gene.lo, gene.hi)` — decision D-10.
impl<T> ValueMutable for MultiRangeChromosome<T>
where
    T: Sync + Send + Copy + Default + Debug + 'static + PartialOrd + GaussianConvertible,
{
    fn gaussian_mutate(&mut self, _sigma: f64) {
        multi_range_gaussian_mutation(self, _sigma);
    }
}
