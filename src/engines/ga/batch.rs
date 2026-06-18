//! Extracted from src/engines/ga.rs in phase 69-04 — `batch_evaluate` helper.

use super::*;

/// Evaluates `pop` using a batch evaluator, with optional LRU cache partitioning.
///
/// Free function used by the GA run loop to avoid Rust's borrow checker conflict
/// when `pop` is a field of the same struct that also owns `batch_evaluator` and
/// `fitness_cache`.
///
/// # Cases
///
/// - `cache_opt = None`: evaluates every chromosome via `evaluator.evaluate_batch(pop)`.
/// - `cache_opt = Some(cache)`: performs the D-06 hit/miss partition — only cache misses
///   are sent to `evaluate_batch`; hits are returned from the cache directly. The cache
///   `Mutex` is released before the `evaluate_batch` call (Pitfall 2 / T-60-05).
pub(crate) fn batch_evaluate<U>(
    evaluator: Arc<dyn crate::fitness::BatchFitnessEvaluator<U> + Send + Sync>,
    cache_opt: Option<Arc<std::sync::Mutex<crate::fitness::cache::FitnessCache>>>,
    pop: &mut [U],
) -> Result<(), GaError>
where
    U: LinearChromosome + Clone,
    U::Gene: Debug,
{
    if pop.is_empty() {
        return Ok(());
    }

    match cache_opt {
        None => {
            // Case B: evaluator set, no cache — batch-evaluate everything
            let values = evaluator.evaluate_batch(pop);
            debug_assert_eq!(
                values.len(),
                pop.len(),
                "evaluate_batch returned {} values for {} chromosomes (T-60-01)",
                values.len(),
                pop.len()
            );
            for (i, chromosome) in pop.iter_mut().enumerate() {
                chromosome.set_fitness(values[i]);
            }
        }
        Some(cache_handle) => {
            // Case C: evaluator + cache — D-06 partition algorithm
            let mut fitness_values: Vec<f64> = vec![0.0; pop.len()];
            let mut miss_indices: Vec<usize> = Vec::new();

            // Step 1: check cache for each chromosome; collect misses
            {
                let mut cache = cache_handle.lock().expect("fitness cache lock poisoned");
                for (i, chromosome) in pop.iter().enumerate() {
                    let key = crate::fitness::cache::hash_dna(chromosome.dna());
                    match cache.get(key) {
                        Some(f) => fitness_values[i] = f,
                        None => miss_indices.push(i),
                    }
                }
            } // Lock released here (Pitfall 2 — never hold lock across evaluate_batch)

            if !miss_indices.is_empty() {
                // Step 2: clone only the miss chromosomes (D-01: evaluate_batch takes &[U])
                let miss_chromosomes: Vec<U> = miss_indices
                    .iter()
                    .map(|&orig_i| pop[orig_i].clone())
                    .collect();

                // Step 3: batch-evaluate misses
                let miss_values = evaluator.evaluate_batch(&miss_chromosomes);
                debug_assert_eq!(
                    miss_values.len(),
                    miss_indices.len(),
                    "evaluate_batch returned {} values for {} miss chromosomes (T-60-01)",
                    miss_values.len(),
                    miss_indices.len()
                );

                // Step 4: re-acquire cache and store miss results
                {
                    let mut cache = cache_handle.lock().expect("fitness cache lock poisoned");
                    for (pos, &orig_i) in miss_indices.iter().enumerate() {
                        let f = miss_values[pos];
                        fitness_values[orig_i] = f;
                        let key = crate::fitness::cache::hash_dna(pop[orig_i].dna());
                        cache.put(key, f);
                    }
                } // Lock released
            }

            // Step 5: write all fitness values (hits + misses) back into the population
            for (i, chromosome) in pop.iter_mut().enumerate() {
                chromosome.set_fitness(fitness_values[i]);
            }
        }
    }

    Ok(())
}
