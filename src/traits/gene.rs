//! Gene trait definition.
//!
//! The [`GeneT`] trait is the smallest building block of a chromosome. Every
//! gene carries an integer ID (used for allele identity and duplicate
//! detection) and must be cloneable and thread-safe.

/// Trait that every gene type must implement.
///
/// A gene is a single element inside a chromosome's DNA. Implementations
/// must provide [`GeneT::set_id`]; the remaining methods have sensible
/// defaults.
///
/// # Required bounds
///
/// `Default + Clone + Sync + Send` — genes are created in bulk during
/// initialization and shared across threads by rayon.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::GeneT;
/// use genetic_algorithms::genotypes::Binary;
///
/// // Use a built-in gene type that implements GeneT:
/// let mut gene = <Binary as Default>::default();
/// gene.set_id(5);
/// assert_eq!(gene.id(), 5);
/// ```
pub trait GeneT: Default + Clone + Sync + Send {
    /// Creates a new gene using `Default::default()`.
    fn new() -> Self {
        Default::default()
    }
    /// Returns a gene with its ID reset to `-1` (sentinel / unset value).
    fn default(mut self) -> Self {
        self.set_id(-1);
        self
    }
    /// Returns the gene's integer identifier.
    ///
    /// The default implementation returns `0`; override this when your gene
    /// type carries meaningful identity (e.g., allele index).
    fn id(&self) -> i32 {
        0
    }
    /// Sets the gene's integer identifier and returns a mutable reference to `self`.
    fn set_id(&mut self, id: i32) -> &mut Self;
}
