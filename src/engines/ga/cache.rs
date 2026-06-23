//! Extracted from src/engines/ga.rs in phase 69-04 — fitness cache helpers (D-07 snapshot/delta).

use super::*;

/// Snapshots the current cache hit/miss counters before a generation starts.
///
/// Returns `Ok((hits, misses))` from the active fitness cache, or `Ok((0, 0))` when no
/// cache is in use. Returns `Err(GaError::InternalError(...))` if the mutex was poisoned
/// by a panicking thread. These values are used by [`cache_fill_stats`] after the
/// generation completes to compute per-generation deltas (D-07).
pub(crate) fn cache_snapshot(
    fitness_cache: &Option<Arc<std::sync::Mutex<crate::fitness::cache::FitnessCache>>>,
) -> Result<(u64, u64), GaError> {
    match fitness_cache {
        Some(ch) => {
            let c = ch
                .lock()
                .map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?;
            Ok((c.hits(), c.misses()))
        }
        None => Ok((0, 0)),
    }
}

/// Fills per-generation cache delta statistics into `gen_stats`.
///
/// Called after the generation loop body completes.  Computes the difference
/// between the current cache counters and the snapshot taken at generation start
/// (via [`cache_snapshot`]), then stores the deltas in `gen_stats.cache_hits`
/// and `gen_stats.cache_misses`.  No-op when no cache is active.
/// Returns `Err(GaError::InternalError(...))` if the mutex was poisoned.
pub(crate) fn cache_fill_stats(
    fitness_cache: &Option<Arc<std::sync::Mutex<crate::fitness::cache::FitnessCache>>>,
    gen_stats: &mut GenerationStats,
    prev_hits: u64,
    prev_misses: u64,
) -> Result<(), GaError> {
    if let Some(ref ch) = fitness_cache {
        let c = ch
            .lock()
            .map_err(|_| GaError::InternalError("fitness cache mutex poisoned".to_string()))?;
        gen_stats.cache_hits = Some(c.hits().saturating_sub(prev_hits));
        gen_stats.cache_misses = Some(c.misses().saturating_sub(prev_misses));
    }
    Ok(())
}
