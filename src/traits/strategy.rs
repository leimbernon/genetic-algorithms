//! Common interface over all search strategy engines.
//! Enables runtime algorithm swapping via `Box<dyn Strategy<U>>`.

use crate::error::GaError;
use crate::traits::ChromosomeT;

/// Common interface over all search strategy engines. Enables runtime algorithm swapping via `Box<dyn Strategy<U>>`.
pub trait Strategy<U: ChromosomeT> {
    /// Execute the search loop. Mutates internal state.
    fn run(&mut self) -> Result<(), GaError>;
    /// Returns the best candidate found, or `None` if `run()` has not been called.
    fn best(&self) -> Option<&U>;
}
