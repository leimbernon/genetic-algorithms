//! Binary chromosome implementation.
//!
//! A [`Binary`] chromosome stores a vector of [`genotypes::Binary`](crate::genotypes::Binary)
//! genes, each holding a boolean value. This is the natural choice for
//! problems like OneMax, knapsack, or any bit-string optimization.

use crate::error::GaError;
use crate::fitness::FitnessFnWrapper;
use crate::genotypes::Binary as BinaryGenotype;
use crate::operations::mutation::ValueMutable;
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat, VectorFitness};
use std::borrow::Cow;
use std::fmt;

/// A chromosome that uses a binary genotype.
///
/// This struct implements the `ChromosomeT` trait, allowing it to be used in genetic
/// algorithms. The `dna` field represents the sequence of genes, while the `fitness`
/// field represents the fitness score of the chromosome, and the `age` field represents
/// the age of the chromosome.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::traits::LinearChromosome;
///
/// let mut chrom = Binary::new();
/// chrom.dna_from_string("1010").expect("valid bit string");
/// assert_eq!(chrom.phenotype(), "1010");
/// assert_eq!(chrom.dna().len(), 4);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Binary {
    pub dna: Vec<BinaryGenotype>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<BinaryGenotype>,
}

impl ChromosomeT for Binary {
    type Gene = BinaryGenotype;

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

impl VectorFitness for Binary {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl LinearChromosome for Binary {
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
        F: Fn(&[BinaryGenotype]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

/// `BinaryChromosome` imposes no operator restrictions — all crossovers and
/// mutations are accepted. The default `None`-returning methods are inherited
/// from the trait.
impl OperatorCompat for Binary {}

impl Default for Binary {
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

impl Binary {
    /// Creates a new `Binary`.
    ///
    /// # Returns
    ///
    /// A new `Binary` with default values.
    pub fn new() -> Self {
        Self {
            dna: Vec::new(),
            fitness: f64::NAN,
            age: 0,
            fitness_values: Vec::new(),
            fitness_fn: FitnessFnWrapper::default(),
        }
    }

    /// Returns the phenotype of the chromosome as a string.
    ///
    /// # Returns
    ///
    /// A string representation of the chromosome's phenotype.
    pub fn phenotype(&self) -> String {
        self.dna
            .iter()
            .map(|gene| if gene.value { '1' } else { '0' })
            .collect()
    }

    /// Sets the DNA of the chromosome from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - A string representation of the DNA, where '1' represents a true value
    ///   and '0' represents a false value.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the string was parsed successfully, or `Err(GaError::ValidationError)`
    /// if the string contains characters other than '1' or '0'.
    pub fn dna_from_string(&mut self, s: &str) -> Result<(), GaError> {
        let mut dna = Vec::with_capacity(s.len());

        for (index, char) in s.chars().enumerate() {
            match char {
                '1' => dna.push(BinaryGenotype {
                    id: index as i32,
                    value: true,
                }),
                '0' => dna.push(BinaryGenotype {
                    id: index as i32,
                    value: false,
                }),
                _ => {
                    return Err(GaError::ValidationError(format!(
                        "Invalid character '{}' at position {}; only '1' and '0' are allowed",
                        char, index
                    )));
                }
            }
        }

        self.dna = dna;
        Ok(())
    }
}

impl ValueMutable for Binary {
    fn bit_flip_mutate(&mut self) {
        crate::operations::mutation::bit_flip::bit_flip(self);
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}
