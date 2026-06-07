//! LRU fitness cache for avoiding redundant fitness evaluations.
//!
//! [`FitnessCache`] stores a bounded mapping from DNA hash → fitness value.
//! When the cache is full, the least recently used entry is evicted.
//!
//! This is especially useful when the fitness function is expensive and
//! the population contains many duplicate or near-duplicate chromosomes.

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use crate::traits::{FitnessFn, GeneT};

/// A bounded LRU (Least Recently Used) cache mapping DNA hashes to fitness values.
///
/// The cache uses a `HashMap` for O(1) lookups and a `VecDeque` for tracking
/// access order. When the cache exceeds its capacity, the least recently used
/// entry is evicted.
pub struct FitnessCache {
    map: HashMap<u64, f64>,
    order: VecDeque<u64>,
    capacity: usize,
    hits: u64,
    misses: u64,
}

impl FitnessCache {
    /// Creates a new cache with the given maximum capacity.
    pub fn new(capacity: usize) -> Self {
        FitnessCache {
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
            hits: 0,
            misses: 0,
        }
    }

    /// Looks up a fitness value by DNA hash, marking it as recently used.
    pub fn get(&mut self, key: u64) -> Option<f64> {
        if let Some(&value) = self.map.get(&key) {
            self.hits += 1;
            // Move to back (most recently used)
            self.order.retain(|&k| k != key);
            self.order.push_back(key);
            Some(value)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Inserts a fitness value into the cache, evicting the LRU entry if full.
    pub fn put(&mut self, key: u64, value: f64) {
        if let Some(existing) = self.map.get_mut(&key) {
            // Update existing entry and move to back
            *existing = value;
            self.order.retain(|&k| k != key);
            self.order.push_back(key);
            return;
        }

        // Evict if at capacity
        if self.map.len() >= self.capacity {
            if let Some(evicted) = self.order.pop_front() {
                self.map.remove(&evicted);
            }
        }

        self.map.insert(key, value);
        self.order.push_back(key);
    }

    /// Returns the number of cache hits.
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Returns the number of cache misses.
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Returns the current number of entries in the cache.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the cache contains no entries.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Computes a hash of the DNA slice using its `Debug` representation.
///
/// This approach works for all gene types (including `Range<f64>` which
/// does not implement `Hash`) since `Debug` is required by the `Ga` impl.
pub fn hash_dna<G: Debug>(dna: &[G]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    // Hash each gene's Debug representation to capture full state
    // (id, value, ranges, etc.)
    for gene in dna {
        let repr = format!("{:?}", gene);
        repr.hash(&mut hasher);
    }
    hasher.finish()
}

/// Wraps a fitness function with LRU caching.
///
/// Returns a tuple `(wrapped_fn, cache_handle)` where:
/// - `wrapped_fn` is a new fitness function that checks the cache before
///   calling the original, avoiding redundant evaluations for identical DNA.
/// - `cache_handle` is a shared `Arc<Mutex<FitnessCache>>` that callers can
///   use to read hit/miss statistics or reset the cache between runs.
///
/// The cache is shared across all chromosomes and threads via `Arc<Mutex<...>>`.
///
/// # Example
///
/// ```rust,ignore
/// let (cached_fn, cache) = wrap_with_cache(my_fitness_fn, 1024);
/// // After a run:
/// let stats = cache.lock().unwrap();
/// println!("hits={} misses={}", stats.hits(), stats.misses());
/// ```
pub fn wrap_with_cache<G>(
    fitness_fn: Arc<FitnessFn<G>>,
    cache_size: usize,
) -> (Arc<FitnessFn<G>>, Arc<Mutex<FitnessCache>>)
where
    G: GeneT + Debug + 'static,
{
    let cache = Arc::new(Mutex::new(FitnessCache::new(cache_size)));
    let cache_for_fn = Arc::clone(&cache);

    let wrapped = Arc::new(move |dna: &[G]| {
        let key = hash_dna(dna);

        // Try cache first
        {
            let mut cache = cache_for_fn.lock().expect("fitness cache lock poisoned");
            if let Some(fitness) = cache.get(key) {
                return fitness;
            }
        }

        // Cache miss — evaluate
        let fitness = fitness_fn(dna);

        // Store result
        {
            let mut cache = cache_for_fn.lock().expect("fitness cache lock poisoned");
            cache.put(key, fitness);
        }

        fitness
    });

    (wrapped, cache)
}
