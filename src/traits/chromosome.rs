use crate::traits::GeneT;
use std::borrow::Cow;

/// Core trait representing a chromosome for the genetic algorithm.
///
/// A chromosome is a sequence of genes (DNA) along with evaluation metadata (fitness, age).
/// Implementors should store their DNA internally and provide efficient ways to set the DNA
/// either by borrowing (no copy at call site) or by moving (no extra clone) using `Cow`.
///
/// Performance:
/// - `set_dna(Cow<[Gene]>)` enables zero-copy moves when the caller owns the DNA,
///   and minimal-copy behavior when passing a borrowed slice.
/// - Prefer editing DNA in-place when possible, or using `set_gene` for single-gene changes.
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    /// Gene type used by this chromosome.
    type Gene: GeneT;

    /// Constructs a new chromosome using `Default`.
    fn new() -> Self {
        Default::default()
    }

    /// Resets common chromosome state (fitness, age, DNA).
    fn default(mut self) -> Self {
        self.set_fitness(f64::NAN);
        self.set_age(0);
        self.set_dna(Cow::Borrowed(&[]));
        self
    }

    /// Creates a new default-initialized gene for this chromosome.
    fn new_gene() -> Self::Gene {
        Self::Gene::new()
    }

    /// Returns the DNA as an immutable slice.
    fn get_dna(&self) -> &[Self::Gene];

    /// Sets the DNA using `Cow` to avoid unnecessary copies.
    ///
    /// Semantics:
    /// - `Cow::Borrowed(&[Gene])`: clones into an internal `Vec<Gene>`.
    /// - `Cow::Owned(Vec<Gene>)`: moves the vector, avoiding an extra clone.
    ///
    /// Contract:
    /// - The length and content of the DNA must match the chromosome semantics expected by
    ///   operators and fitness functions.
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;

    /// Replaces the gene at `gene_index` with `gene`.
    ///
    /// If `gene_index` is out of bounds, this is a no-op and a warning is logged.
    ///
    /// Implementation detail:
    /// - Builds a temporary owned `Vec<Gene>` from the current DNA and moves it back using `Cow::Owned`.
    /// - Implementors may override this with a more efficient in-place edit if their storage allows.
    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self {
        let mut dna_temp = self.get_dna().to_vec();
        if gene_index >= dna_temp.len() {
            log::warn!(
                "set_gene: index {} is out of bounds (DNA length {}), ignoring",
                gene_index,
                dna_temp.len()
            );
            return self;
        }
        dna_temp[gene_index] = gene;
        // Move the vector to avoid an extra clone
        self.set_dna(Cow::Owned(dna_temp));
        self
    }

    /// Installs the user-provided fitness function.
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;

    /// Recomputes and sets the chromosome fitness using the installed fitness function.
    fn calculate_fitness(&mut self);

    /// Returns the latest fitness value.
    fn get_fitness(&self) -> f64;

    /// Sets the fitness value. Useful for caching or test scaffolding.
    fn set_fitness(&mut self, fitness: f64) -> &mut Self;

    /// Sets the age (usually, generation count since creation).
    fn set_age(&mut self, age: usize) -> &mut Self;

    /// Returns the age.
    fn get_age(&self) -> usize;

    /// Absolute distance to a target fitness, helpful for stopping criteria or diagnostics.
    fn get_fitness_distance(&self, fitness_target: &f64) -> f64 {
        (fitness_target - self.get_fitness()).abs()
    }
}
