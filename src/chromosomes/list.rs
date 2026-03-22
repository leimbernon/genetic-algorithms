//! List chromosome implementation.
//!
//! A [`ListChromosome<T>`] chromosome stores a vector of
//! [`genotypes::List`](crate::genotypes::List) genes, each holding a value
//! drawn from a finite set of alleles. This is suited for combinatorial and
//! symbolic optimization problems (routing, scheduling, sequence problems, etc.).

use crate::fitness::FitnessFnWrapper;
use crate::genotypes::List as ListGenotype;
use crate::traits::ChromosomeT;
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
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub fitness_fn: FitnessFnWrapper<ListGenotype<T>>,
}

impl<T: Sync + Send + Clone + Default + Debug> Default for ListChromosome<T> {
    fn default() -> Self {
        Self {
            dna: Vec::new(),
            fitness: f64::NAN,
            age: 0,
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

impl<T: Sync + Send + Clone + Default + Debug> fmt::Display for ListChromosome<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] fitness={:.6}", self.phenotype(), self.fitness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::GeneT;
    use std::borrow::Cow;

    fn make_gene(id: i32, alleles: Vec<char>) -> ListGenotype<char> {
        ListGenotype::new(id, alleles, 'a').unwrap()
    }

    // ── new / default ─────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_new_has_empty_dna_nan_fitness_age_zero() {
        let c = ListChromosome::<char>::new();
        assert!(c.dna().is_empty());
        assert!(c.fitness().is_nan());
        assert_eq!(c.age(), 0);
    }

    #[test]
    fn list_chromosome_default_same_as_new() {
        let c: ListChromosome<char> = Default::default();
        assert!(c.dna().is_empty());
        assert!(c.fitness().is_nan());
        assert_eq!(c.age(), 0);
    }

    // ── set_dna / dna ────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_set_dna_owned() {
        let mut c = ListChromosome::<char>::new();
        let genes = vec![make_gene(0, vec!['a', 'b']), make_gene(1, vec!['a', 'b'])];
        c.set_dna(Cow::Owned(genes.clone()));
        assert_eq!(c.dna().len(), 2);
        assert_eq!(c.dna()[0].id(), 0);
        assert_eq!(c.dna()[1].id(), 1);
    }

    #[test]
    fn list_chromosome_set_dna_borrowed() {
        let mut c = ListChromosome::<char>::new();
        let genes = vec![make_gene(0, vec!['x', 'y', 'z'])];
        c.set_dna(Cow::Borrowed(&genes));
        assert_eq!(c.dna().len(), 1);
        assert_eq!(c.dna()[0].value(), 'x');
    }

    // ── dna_mut ──────────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_dna_mut_modifications_visible() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(0, vec!['a', 'b', 'c']));
        c.dna_mut()[0].set_id(2);
        assert_eq!(c.dna()[0].value(), 'c');
    }

    // ── fitness ───────────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_set_fitness_and_get() {
        let mut c = ListChromosome::<char>::new();
        c.set_fitness(42.0);
        assert_eq!(c.fitness(), 42.0);
    }

    // ── age ───────────────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_set_age_and_get() {
        let mut c = ListChromosome::<char>::new();
        c.set_age(5);
        assert_eq!(c.age(), 5);
    }

    // ── set_fitness_fn + calculate_fitness ────────────────────────────────────

    #[test]
    fn list_chromosome_calculate_fitness_using_fn() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(0, vec!['a', 'b']));
        c.dna.push(make_gene(1, vec!['a', 'b']));
        c.set_fitness_fn(|dna| dna.len() as f64);
        c.calculate_fitness();
        assert_eq!(c.fitness(), 2.0);
    }

    // ── Clone ─────────────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_clone_is_independent() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(0, vec!['a', 'b', 'c']));
        c.set_fitness(1.0);
        let mut cloned = c.clone();
        cloned.set_fitness(99.0);
        cloned.dna.push(make_gene(1, vec!['a', 'b', 'c']));
        assert_eq!(c.fitness(), 1.0);
        assert_eq!(c.dna().len(), 1);
    }

    // ── phenotype ─────────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_phenotype_formats_values() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(0, vec!['a', 'b', 'c'])); // value = 'a'
        c.dna.push(make_gene(2, vec!['a', 'b', 'c'])); // value = 'c'
        let p = c.phenotype();
        assert_eq!(p, "'a', 'c'");
    }

    // ── Display ───────────────────────────────────────────────────────────────

    #[test]
    fn list_chromosome_display_format() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(0, vec!['a', 'b'])); // value 'a'
        c.set_fitness(std::f64::consts::PI);
        let s = format!("{}", c);
        assert!(s.contains("fitness="), "display was: {}", s);
    }

    // ── serde (feature-gated) ─────────────────────────────────────────────────

    #[cfg(feature = "serde")]
    #[test]
    fn list_chromosome_serde_roundtrip() {
        let mut c = ListChromosome::<char>::new();
        c.dna.push(make_gene(1, vec!['a', 'b', 'c']));
        c.set_fitness(7.0);
        c.set_age(3);
        let json = serde_json::to_string(&c).expect("serialize");
        let restored: ListChromosome<char> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.dna.len(), 1);
        assert_eq!(restored.dna[0].id(), 1);
        assert_eq!(restored.fitness(), 7.0);
        assert_eq!(restored.age(), 3);
        // fitness_fn is skipped — that's OK
    }
}
