//! Range chromosome implementation.
//!
//! A [`Range`] chromosome stores a vector of [`genotypes::Range`](crate::genotypes::Range)
//! genes, each holding a numeric value within defined bounds. This is suited
//! for continuous or integer optimization problems (function optimization,
//! parameter tuning, etc.).

use crate::fitness::FitnessFnWrapper;
use crate::genotypes::Range as RangeGenotype;
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;

/// A chromosome that uses a range genotype.
///
/// This struct implements the `ChromosomeT` trait, allowing it to be used in genetic
/// algorithms. The `dna` field represents the sequence of genes, while the `fitness`
/// field represents the fitness score of the chromosome, and the `age` field represents
/// the age of the chromosome.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::chromosomes::Range;
/// use crate::genetic_algorithms::traits::ChromosomeT;
/// use genetic_algorithms::genotypes::Range as RangeGenotype;
///
/// let mut chromosome = Range::<i32>::new();
/// chromosome.dna.push(RangeGenotype::new(0, vec![(0, 10)], 5));
/// chromosome.calculate_fitness();
/// println!("Fitness: {}", chromosome.fitness());
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
pub struct Range<T: Sync + Send + Copy + Default + Debug> {
    pub dna: Vec<RangeGenotype<T>>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<RangeGenotype<T>>,
}

/// `RangeChromosome<T>` imposes no operator restrictions — all crossovers and
/// mutations are accepted. The default `None`-returning methods are inherited
/// from the trait.
impl<T: Sync + Send + Copy + Default + Debug> OperatorCompat for Range<T> {}

impl<T: Sync + Send + Copy + Default + Debug> Default for Range<T> {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            fitness: f64::NAN,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        }
    }
}

impl<T: Sync + Send + Copy + Default + Debug> Range<T> {
    /// Creates a new `Range`.
    ///
    /// # Returns
    ///
    /// A new `Range` with default values.
    ///
    /// # Examples
    ///
    /// ```
    /// use genetic_algorithms::chromosomes::Range;
    ///
    /// let chromosome = Range::<i32>::new();
    /// println!("Chromosome: {:?}", chromosome);
    /// ```
    pub fn new() -> Self {
        Self {
            dna: Vec::new(),
            fitness: f64::NAN,
            age: 0,
            fitness_fn: FitnessFnWrapper::default(),
        }
    }

    /// Returns the phenotype of the chromosome as a string.
    ///
    /// # Returns
    ///
    /// A string representation of the chromosome's phenotype.
    ///
    /// # Examples
    ///
    /// ```
    /// use genetic_algorithms::chromosomes::Range;
    /// use genetic_algorithms::genotypes::Range as RangeGenotype;
    ///
    /// let mut chromosome = Range::<i32>::new();
    /// chromosome.dna.push(RangeGenotype::new(0, vec![(0, 10)], 5));
    /// let phenotype = chromosome.phenotype();
    /// println!("Phenotype: {}", phenotype);
    /// ```
    pub fn phenotype(&self) -> String {
        self.dna
            .iter()
            .map(|gene| format!("{:?}", gene.value()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T: Sync + Send + Copy + Default + Debug + 'static> ChromosomeT for Range<T> {
    type Gene = RangeGenotype<T>;

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

impl<T: Sync + Send + Copy + Default + Debug + 'static> LinearChromosome for Range<T> {
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
        F: Fn(&[RangeGenotype<T>]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

impl<T: Sync + Send + Copy + Default + Debug + fmt::Display> fmt::Display for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}
