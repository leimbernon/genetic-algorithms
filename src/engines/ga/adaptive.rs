//! Extracted from src/engines/ga.rs in phase 69-04 — adaptive crossover/mutation probability recomputation.

use super::*;

/// Updates the dynamic mutation probability based on current population diversity.
///
/// Called once per generation when `dynamic_mutation` is enabled in the mutation
/// configuration.  Uses `mutation::dynamic_probability` to recompute the probability
/// and writes the result back into both `dynamic_mutation_probability` (for the next
/// generation's `parent_crossover` call) and `gen_stats.dynamic_mutation_probability`
/// (for observability / statistics reporting).
///
/// Returns the updated probability (same value written to `*dynamic_mutation_probability`).
pub(crate) fn update_dynamic_mutation(
    mutation_config: &crate::configuration::MutationConfiguration,
    dynamic_mutation_probability: &mut f64,
    gen_stats: &mut GenerationStats,
) {
    if !mutation_config.dynamic_mutation {
        return;
    }

    let target = mutation_config.target_cardinality.unwrap_or(0.5);
    let step = mutation_config.probability_step.unwrap_or(0.01);
    let p_max = mutation_config.probability_max.unwrap_or(1.0);
    let p_min = mutation_config.probability_min.unwrap_or(0.0);

    *dynamic_mutation_probability = mutation::dynamic_probability(
        *dynamic_mutation_probability,
        gen_stats.diversity,
        target,
        step,
        p_max,
        p_min,
    );

    // Set the field directly on gen_stats before push (no last_mut needed)
    gen_stats.dynamic_mutation_probability = Some(*dynamic_mutation_probability);
}
