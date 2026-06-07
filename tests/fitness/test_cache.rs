use genetic_algorithms::fitness::cache::{hash_dna, wrap_with_cache, FitnessCache};
use genetic_algorithms::genotypes::Binary as BinaryGene;
use genetic_algorithms::traits::GeneT;
use std::sync::Arc;

// --- FitnessCache unit tests ---

#[test]
fn cache_new_is_empty() {
    let cache = FitnessCache::new(10);
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.hits(), 0);
    assert_eq!(cache.misses(), 0);
}

#[test]
fn cache_put_and_get() {
    let mut cache = FitnessCache::new(10);
    cache.put(42, 3.5);
    assert_eq!(cache.len(), 1);

    let result = cache.get(42);
    assert_eq!(result, Some(3.5));
    assert_eq!(cache.hits(), 1);
    assert_eq!(cache.misses(), 0);
}

#[test]
fn cache_miss_increments_counter() {
    let mut cache = FitnessCache::new(10);
    let result = cache.get(99);
    assert_eq!(result, None);
    assert_eq!(cache.misses(), 1);
    assert_eq!(cache.hits(), 0);
}

#[test]
fn cache_evicts_lru_when_full() {
    let mut cache = FitnessCache::new(3);
    cache.put(1, 1.0);
    cache.put(2, 2.0);
    cache.put(3, 3.0);
    assert_eq!(cache.len(), 3);

    // Insert a 4th entry — should evict key 1 (LRU)
    cache.put(4, 4.0);
    assert_eq!(cache.len(), 3);
    assert_eq!(cache.get(1), None); // evicted
    assert_eq!(cache.get(4), Some(4.0)); // present
}

#[test]
fn cache_access_updates_lru_order() {
    let mut cache = FitnessCache::new(3);
    cache.put(1, 1.0);
    cache.put(2, 2.0);
    cache.put(3, 3.0);

    // Access key 1 — it becomes most recently used
    cache.get(1);

    // Insert key 4 — should evict key 2 (now the LRU), not key 1
    cache.put(4, 4.0);
    assert_eq!(cache.get(1), Some(1.0)); // still present
    assert_eq!(cache.get(2), None); // evicted
}

#[test]
fn cache_update_existing_key() {
    let mut cache = FitnessCache::new(10);
    cache.put(1, 1.0);
    cache.put(1, 2.0);
    assert_eq!(cache.get(1), Some(2.0));
    assert_eq!(cache.len(), 1); // no duplicate
}

#[test]
fn cache_capacity_one() {
    let mut cache = FitnessCache::new(1);
    cache.put(1, 1.0);
    cache.put(2, 2.0);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(1), None);
    assert_eq!(cache.get(2), Some(2.0));
}

// --- hash_dna tests ---

#[test]
fn hash_dna_identical_produces_same_hash() {
    let dna1: Vec<BinaryGene> = vec![
        {
            let mut g = <BinaryGene as Default>::default();
            g.set_id(1);
            g.value = true;
            g
        },
        {
            let mut g = <BinaryGene as Default>::default();
            g.set_id(2);
            g.value = false;
            g
        },
    ];
    let dna2 = dna1.clone();
    assert_eq!(hash_dna(&dna1), hash_dna(&dna2));
}

#[test]
fn hash_dna_different_produces_different_hash() {
    let dna1: Vec<BinaryGene> = vec![{
        let mut g = <BinaryGene as Default>::default();
        g.set_id(1);
        g.value = true;
        g
    }];
    let dna2: Vec<BinaryGene> = vec![{
        let mut g = <BinaryGene as Default>::default();
        g.set_id(1);
        g.value = false;
        g
    }];
    assert_ne!(hash_dna(&dna1), hash_dna(&dna2));
}

#[test]
fn hash_dna_different_ids_produces_different_hash() {
    let dna1: Vec<BinaryGene> = vec![{
        let mut g = <BinaryGene as Default>::default();
        g.set_id(1);
        g
    }];
    let dna2: Vec<BinaryGene> = vec![{
        let mut g = <BinaryGene as Default>::default();
        g.set_id(2);
        g
    }];
    assert_ne!(hash_dna(&dna1), hash_dna(&dna2));
}

#[test]
fn hash_dna_empty_is_consistent() {
    let dna: Vec<BinaryGene> = vec![];
    assert_eq!(hash_dna(&dna), hash_dna(&dna));
}

// --- hash_dna with Range<f64> genes ---

#[test]
fn hash_dna_range_f64_identical() {
    use genetic_algorithms::genotypes::Range as RangeGene;
    let dna1 = vec![RangeGene::new(0, vec![(0.0, 10.0)], 5.5)];
    let dna2 = vec![RangeGene::new(0, vec![(0.0, 10.0)], 5.5)];
    assert_eq!(hash_dna(&dna1), hash_dna(&dna2));
}

#[test]
fn hash_dna_range_f64_different_values() {
    use genetic_algorithms::genotypes::Range as RangeGene;
    let dna1 = vec![RangeGene::new(0, vec![(0.0, 10.0)], 5.5)];
    let dna2 = vec![RangeGene::new(0, vec![(0.0, 10.0)], 7.3)];
    assert_ne!(hash_dna(&dna1), hash_dna(&dna2));
}

// --- Integration: cache with GA builder ---

#[test]
fn ga_with_fitness_cache_builds_successfully() {
    use genetic_algorithms::chromosomes::Binary;
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::binary_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    let ga = Ga::<Binary>::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_initialization_fn(|n, _| binary_random_initialization(n, None))
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_fitness_cache_size(100)
        .build();

    assert!(ga.is_ok());
}

#[test]
fn ga_with_fitness_cache_runs_correctly() {
    use genetic_algorithms::chromosomes::Binary;
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::initializers::binary_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    let mut ga = Ga::<Binary>::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(4))
        .with_fitness_fn(|dna: &[BinaryGene]| dna.iter().filter(|g| g.value).count() as f64)
        .with_initialization_fn(|n, _| binary_random_initialization(n, None))
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::BitFlip)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_fitness_cache_size(200)
        .build()
        .unwrap();

    let result = ga.run();
    assert!(result.is_ok());
    let pop = result.unwrap();
    assert!(pop.size() > 0);
}

#[test]
fn ga_with_fitness_cache_range_chromosome() {
    use genetic_algorithms::chromosomes::Range as RangeChromosome;
    use genetic_algorithms::configuration::ProblemSolving;
    use genetic_algorithms::ga::Ga;
    use genetic_algorithms::genotypes::Range as RangeGene;
    use genetic_algorithms::initializers::range_random_initialization;
    use genetic_algorithms::operations::{Crossover, Mutation, Selection, Survivor};
    use genetic_algorithms::traits::{
        ConfigurationT, CrossoverConfig, MutationConfig, SelectionConfig, StoppingConfig,
    };

    let alleles = vec![RangeGene::new(0, vec![(0.0, 10.0)], 0.0)];
    let alleles_clone = alleles.clone();

    let mut ga = Ga::<RangeChromosome<f64>>::new()
        .with_population_size(20)
        .with_chromosome_length(genetic_algorithms::ChromosomeLength::Fixed(3))
        .with_alleles(alleles)
        .with_fitness_fn(|dna: &[RangeGene<f64>]| dna.iter().map(|g| g.value).sum::<f64>())
        .with_initialization_fn(move |n, _| {
            range_random_initialization(n, Some(&alleles_clone))
        })
        .with_selection_method(Selection::Tournament)
        .with_crossover_method(Crossover::Uniform)
        .with_mutation_method(Mutation::Value)
        .with_problem_solving(ProblemSolving::Maximization)
        .with_survivor_method(Survivor::Fitness)
        .with_max_generations(10)
        .with_fitness_cache_size(200)
        .build()
        .unwrap();

    let result = ga.run();
    assert!(result.is_ok());
}

// ─── Phase 60 foundation tests ────────────────────────────────────────────────

/// Verifies that wrap_with_cache returns a usable cache handle.
/// - Initial hits and misses are both 0.
/// - After two calls with identical DNA: hits == 1, misses == 1.
#[test]
fn wrap_with_cache_returns_handle() {
    let call_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let call_count_clone = Arc::clone(&call_count);

    #[allow(clippy::type_complexity)]
    let base_fn: Arc<dyn Fn(&[BinaryGene]) -> f64 + Send + Sync> = Arc::new(move |dna: &[BinaryGene]| {
        call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        dna.len() as f64
    });

    let (wrapped_fn, cache_handle) = wrap_with_cache(base_fn, 16);

    // Initial state: no calls made yet
    {
        let cache = cache_handle.lock().unwrap();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 0);
    }

    let dna: Vec<BinaryGene> = vec![{
        let mut g = <BinaryGene as Default>::default();
        g.set_id(1);
        g.value = true;
        g
    }];

    // First call — cache miss, base fn invoked
    let _ = wrapped_fn(&dna);
    {
        let cache = cache_handle.lock().unwrap();
        assert_eq!(cache.hits(), 0);
        assert_eq!(cache.misses(), 1);
    }

    // Second call with identical DNA — cache hit, base fn NOT invoked again
    let _ = wrapped_fn(&dna);
    {
        let cache = cache_handle.lock().unwrap();
        assert_eq!(cache.hits(), 1);
        assert_eq!(cache.misses(), 1);
    }

    // Base function was only called once (the first time)
    assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
}
