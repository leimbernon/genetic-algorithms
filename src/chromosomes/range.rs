use std::fmt::Debug;
use std::borrow::Cow;
use crate::fitness::FitnessFnWrapper;
use crate::genotypes::Range as RangeGenotype;
use crate::traits::ChromosomeT;

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
/// println!("Fitness: {}", chromosome.get_fitness());
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Range<T: Sync + Send + Clone + Default + Debug> {
    pub dna: Vec<RangeGenotype<T>>,
    pub fitness: f64,
    pub age: i32,
    pub fitness_fn: FitnessFnWrapper<RangeGenotype<T>>,
}

impl<T: Sync + Send + Clone + Default + Debug> Range<T> {
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
            fitness: 0.0,
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
            .map(|gene| format!("{:?}", gene.get_value()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T: Sync + Send + Clone + Default + Debug + 'static> ChromosomeT for Range<T> {
    type Gene = RangeGenotype<T>;

    fn get_dna(&self) -> &[Self::Gene] {
        &self.dna
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

    fn calculate_fitness(&mut self) {
        self.fitness = self.fitness_fn.call(&self.dna);
    }

    fn get_fitness(&self) -> f64 {
        self.fitness
    }

    fn set_fitness(&mut self, fitness: f64) -> &mut Self {
        self.fitness = fitness;
        self
    }

    fn set_age(&mut self, age: i32) -> &mut Self {
        self.age = age;
        self
    }

    fn get_age(&self) -> i32 {
        self.age
    }
}