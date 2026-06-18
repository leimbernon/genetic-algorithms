//! List chromosome implementation.
//!
//! A [`ListChromosome<T>`] chromosome stores a vector of
//! [`genotypes::List`](crate::genotypes::List) genes, each holding a value
//! drawn from a finite set of alleles. This is suited for combinatorial and
//! symbolic optimization problems (routing, scheduling, sequence problems, etc.).

use crate::fitness::FitnessFnWrapper;
use crate::genotypes::List as ListGenotype;
use crate::traits::{ChromosomeT, LinearChromosome, OperatorCompat, VectorFitness};
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;

/// A chromosome that uses a list genotype.
///
/// This struct implements the `ChromosomeT` trait, allowing it to be used in
/// genetic algorithms. The `dna` field represents the sequence of genes, while
/// the `fitness` field represents the fitness score of the chromosome, and the
/// `age` field represents the age of the chromosome.
///
/// # Examples
///
/// ```
/// use genetic_algorithms::chromosomes::ListChromosome;
/// use genetic_algorithms::traits::ChromosomeT;
/// use genetic_algorithms::genotypes::List;
///
/// let mut chromosome = ListChromosome::<char>::new();
/// let gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
/// chromosome.dna.push(gene);
/// chromosome.set_fitness(1.0);
/// println!("Fitness: {}", chromosome.fitness());
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "T: serde::Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))
)]
pub struct ListChromosome<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<ListGenotype<T>>,
    pub fitness: f64,
    pub age: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub fitness_values: Vec<f64>,
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<ListGenotype<T>>,
}

/// `ListChromosome<T>` imposes no operator restrictions — all crossovers and
/// mutations are accepted. The default `None`-returning methods are inherited
/// from the trait.
impl<T: Sync + Send + Clone + Default + Debug> OperatorCompat for ListChromosome<T> {}

impl<T: Sync + Send + Clone + Default + Debug> Default for ListChromosome<T> {
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

impl<T: Sync + Send + Clone + Default + Debug> ListChromosome<T> {
    /// Creates a new `ListChromosome` with default values.
    ///
    /// # Returns
    ///
    /// A new `ListChromosome` with empty DNA, NaN fitness, and age 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use genetic_algorithms::chromosomes::ListChromosome;
    ///
    /// let chromosome = ListChromosome::<char>::new();
    /// println!("Chromosome: {:?}", chromosome);
    /// ```
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
    /// Each gene's value is formatted with `{:?}` and joined by `", "`.
    ///
    /// # Returns
    ///
    /// A string representation of the chromosome's phenotype.
    ///
    /// # Examples
    ///
    /// ```
    /// use genetic_algorithms::chromosomes::ListChromosome;
    /// use genetic_algorithms::genotypes::List;
    ///
    /// let mut chromosome = ListChromosome::<char>::new();
    /// chromosome.dna.push(List::new(0, vec!['a', 'b', 'c'], 'a').unwrap());
    /// chromosome.dna.push(List::new(2, vec!['a', 'b', 'c'], 'a').unwrap());
    /// let phenotype = chromosome.phenotype();
    /// println!("Phenotype: {}", phenotype);
    /// ```
    pub fn phenotype(&self) -> String {
        self.dna
            .iter()
            .map(|g| format!("{:?}", g.value))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> ChromosomeT for ListChromosome<T> {
    type Gene = ListGenotype<T>;

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

impl<T: Sync + Send + Clone + Default + Debug + 'static> VectorFitness for ListChromosome<T> {
    fn fitness_values(&self) -> &[f64] {
        &self.fitness_values
    }

    fn set_fitness_values(&mut self, values: Vec<f64>) {
        self.fitness_values = values;
    }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> LinearChromosome for ListChromosome<T> {
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
        F: Fn(&[ListGenotype<T>]) -> f64 + Send + Sync + 'static,
    {
        self.fitness_fn = FitnessFnWrapper::new(fitness_fn);
        self
    }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> crate::traits::RealValuedMutation
    for ListChromosome<T>
{
}

impl<T: Sync + Send + Clone + Default + Debug> fmt::Display for ListChromosome<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}
