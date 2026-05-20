use crate::traits::{ChromosomeT, GeneT};
use std::borrow::Cow;

/// Flat-slice chromosome contract.
///
/// `LinearChromosome` extends [`ChromosomeT`] with the flat-slice DNA surface
/// used by all traditional GA chromosomes (binary, real-valued, integer-valued,
/// list-based). Any type implementing this trait gains:
///
/// - `dna()` / `dna_mut()` / `set_dna()` — slice access and zero-copy DNA updates.
/// - `set_fitness_fn<F>()` — installs the user-provided fitness closure.
/// - `new_gene()` — factory method for default gene construction.
/// - **Default `set_gene()`** — in-place single-gene replacement with bounds checking.
/// - **Default `reset()`** — sets fitness to `NaN`, age to `0`, dna to empty.
///
/// # Supertrait relationship
///
/// `LinearChromosome: ChromosomeT`. All operators and engines that call
/// `dna()` / `dna_mut()` / `set_dna()` require `U: LinearChromosome`.
/// Operators that only use `fitness()` / `age()` remain generic over
/// `U: ChromosomeT` and work with any chromosome representation.
///
/// # WASM compatibility
///
/// The default impls contain no `Instant`, no `par_iter()`, and no
/// host-only stdlib — safe for `wasm32-unknown-unknown`.
pub trait LinearChromosome: ChromosomeT {
    /// Returns the DNA as an immutable slice.
    fn dna(&self) -> &[Self::Gene];

    /// Returns the DNA as a mutable slice for in-place edits.
    ///
    /// Prefer this over `set_dna` when mutating a single gene to avoid a
    /// full-DNA clone.
    fn dna_mut(&mut self) -> &mut [Self::Gene];

    /// Sets the DNA using `Cow` to avoid unnecessary copies.
    ///
    /// Semantics:
    /// - `Cow::Borrowed(&[Gene])`: clones into an internal `Vec<Gene>`.
    /// - `Cow::Owned(Vec<Gene>)`: moves the vector, avoiding an extra clone.
    ///
    /// The length and content of the DNA must match the chromosome semantics
    /// expected by operators and fitness functions.
    fn set_dna<'a>(&mut self, dna: Cow<'a, [Self::Gene]>) -> &mut Self;

    /// Installs the user-provided fitness function.
    ///
    /// The closure receives the chromosome's DNA slice and returns a scalar
    /// fitness value. The installed function is invoked by `calculate_fitness()`.
    fn set_fitness_fn<F>(&mut self, fitness_fn: F) -> &mut Self
    where
        F: Fn(&[Self::Gene]) -> f64 + Send + Sync + 'static;

    /// Creates a new default-initialized gene for this chromosome type.
    ///
    /// Default implementation delegates to `Self::Gene::new()`.
    fn new_gene() -> Self::Gene {
        Self::Gene::new()
    }

    /// Replaces the gene at `gene_index` with `gene`.
    ///
    /// If `gene_index` is out of bounds, this is a no-op and a warning is
    /// logged. Uses `dna_mut()` for an in-place edit, avoiding a full DNA clone.
    ///
    /// Returns `&mut Self` to support builder-style chaining.
    fn set_gene(&mut self, gene_index: usize, gene: Self::Gene) -> &mut Self {
        let len = self.dna().len();
        if gene_index >= len {
            log::warn!(
                "set_gene: index {} is out of bounds (DNA length {}), ignoring",
                gene_index,
                len
            );
            return self;
        }
        self.dna_mut()[gene_index] = gene;
        self
    }

    /// Resets the chromosome to a blank state.
    ///
    /// Sets:
    /// - `fitness` → `f64::NAN` (indicates unevaluated)
    /// - `age` → `0`
    /// - `dna` → empty slice via `Cow::Borrowed(&[])`
    ///
    /// Returns `&mut Self` to support builder-style chaining.
    fn reset(&mut self) -> &mut Self {
        self.set_fitness(f64::NAN);
        self.set_age(0);
        self.set_dna(Cow::Borrowed(&[]));
        self
    }
}
