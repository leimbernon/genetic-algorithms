use crate::traits::GeneT;

/// Minimal evaluation contract for a chromosome.
///
/// `ChromosomeT` covers only the fitness/age evaluation surface — the smallest
/// set of methods that every chromosome type must provide regardless of its
/// internal representation (flat-slice, tree, variable-length, etc.).
///
/// # Flat-slice chromosomes
///
/// Types backed by a contiguous DNA slice (the traditional GA representation)
/// should implement [`LinearChromosome`](crate::traits::LinearChromosome) in
/// addition to this trait. `LinearChromosome: ChromosomeT` extends the contract
/// with `dna()`, `dna_mut()`, `set_dna()`, `set_fitness_fn()`, and provides
/// default implementations of `set_gene()` and `reset()`.
///
/// # Custom / non-linear chromosomes
///
/// Tree chromosomes, variable-length chromosomes, and other non-flat types
/// only need to implement `ChromosomeT`. They are compatible with any operator
/// or engine that is generic over `U: ChromosomeT`.
///
/// # Required bounds
///
/// `Clone + Default + Send + Sync + 'static` — chromosomes are cloned during
/// reproduction and moved across rayon threads during parallel evaluation.
pub trait ChromosomeT: Clone + Default + Send + Sync + 'static {
    /// Gene type used by this chromosome.
    type Gene: GeneT;

    /// Constructs a new chromosome using `Default`.
    fn new() -> Self {
        Default::default()
    }

    /// Recomputes and sets the chromosome fitness using the installed fitness function.
    fn calculate_fitness(&mut self);

    /// Returns the latest fitness value.
    fn fitness(&self) -> f64;

    /// Sets the fitness value. Useful for caching or test scaffolding.
    ///
    /// Returns `&mut Self` to support builder-style chaining.
    fn set_fitness(&mut self, fitness: f64) -> &mut Self;

    /// Sets the age (typically, number of generations since creation).
    ///
    /// Returns `&mut Self` to support builder-style chaining.
    fn set_age(&mut self, age: usize) -> &mut Self;

    /// Returns the age.
    fn age(&self) -> usize;

    /// Absolute distance to a target fitness value.
    ///
    /// Useful for stopping criteria and diagnostics. The default implementation
    /// returns `|fitness_target - self.fitness()|`.
    fn fitness_distance(&self, fitness_target: &f64) -> f64 {
        (fitness_target - self.fitness()).abs()
    }
}
