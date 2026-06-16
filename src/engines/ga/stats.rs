//! Extracted from src/engines/ga.rs in phase 69-04 — GenerationStats collection helpers.

use super::*;

/// Creates a new [`GenerationStats`] from the current generation's fitness values.
///
/// This is a thin wrapper around [`GenerationStats::from_fitness_values`] that
/// establishes the generation index and problem type.  The caller is responsible
/// for populating optional fields (`dynamic_mutation_probability`, `cache_hits`,
/// `cache_misses`, `true_fitness_calls`) after this call.
pub(crate) fn collect_generation_stats(
    generation: usize,
    fitness_values: &[f64],
    is_maximization: bool,
) -> GenerationStats {
    GenerationStats::from_fitness_values(generation, fitness_values, is_maximization)
}
