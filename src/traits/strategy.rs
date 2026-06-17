//! Common interface over all search strategy engines.
//! Enables runtime algorithm swapping via `Box<dyn Strategy<U>>`.

use crate::error::GaError;
use crate::traits::ChromosomeT;

/// Common interface over all search strategy engines. Enables runtime algorithm swapping via `Box<dyn Strategy<U>>`.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::traits::Strategy;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::error::GaError;
///
/// struct MyStrategy;
///
/// impl Strategy<Binary> for MyStrategy {
///     fn run(&mut self) -> Result<(), GaError> { Ok(()) }
///     fn best(&self) -> Option<&Binary> { None }
/// }
/// ```
pub trait Strategy<U: ChromosomeT> {
    /// Execute the search loop. Mutates internal state.
    fn run(&mut self) -> Result<(), GaError>;
    /// Returns the best candidate found, or `None` if `run()` has not been called.
    fn best(&self) -> Option<&U>;
}
