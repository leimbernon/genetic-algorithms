//! Extracted from src/engines/ga.rs in phase 69-04 — Observer hook dispatch (notify fn).

use super::*;

/// Dispatches an observer hook if an observer is attached.
///
/// This is the free-function implementation of `Ga::notify`. The method on `Ga<U>`
/// delegates here, keeping the dispatch logic in one place.  No-op when `observer`
/// is `None` (zero overhead per the observer mandate in CLAUDE.md).
///
/// All `notify(|obs| obs.on_*)` call-sites in `mod.rs` are preserved verbatim
/// (observability mandate — CLAUDE.md §Observability).
#[inline]
pub(crate) fn dispatch<U, F>(observer: &Option<Arc<dyn GaObserver<U> + Send + Sync>>, f: F)
where
    U: LinearChromosome,
    F: FnOnce(&dyn GaObserver<U>),
{
    if let Some(ref obs) = observer {
        f(obs.as_ref());
    }
}
