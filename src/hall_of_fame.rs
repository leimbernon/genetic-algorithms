//! Hall of Fame / Solution Archive.
//!
//! Provides a bounded archive of top-N unique solutions across a GA run,
//! with optional minimum-distance diversity filtering (fitness-space or
//! genotypic).
//!
//! The archive is implemented as an ordered `Vec<Entry<U>>` sorted by
//! fitness descending (best first). Insertion uses binary search for
//! O(log n) position finding with O(n) shift -- acceptable for the
//! typical archive sizes (10-100 entries).

use crate::traits::{ChromosomeT, GeneT};
use std::cmp::Ordering;

/// Distance metric for diversity filtering in the Hall of Fame.
///
/// Determines how "closeness" between two archived solutions is measured.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DistanceMetric {
    /// Euclidean distance in fitness space.
    ///
    /// For single-objective (default), this is `|f1 - f2|`.
    /// Chromosomes whose fitness is closer than `min_distance` to an
    /// existing entry are rejected.
    Fitness {
        /// Minimum allowed distance between two entries in fitness space.
        min_distance: f64,
    },
    /// Genotypic distance: fraction of differing gene positions (by `gene.id()`).
    ///
    /// Chromosomes whose DNA is closer than `min_distance` to an existing
    /// entry are rejected. Range: [0, 1].
    Genotypic {
        /// Minimum allowed genotypic distance (fraction of differing positions).
        min_distance: f64,
    },
}

impl Default for DistanceMetric {
    fn default() -> Self {
        // D-02: Default metric is Fitness-space
        // A value of 0.0 means no diversity filtering (pure top-N by fitness)
        DistanceMetric::Fitness { min_distance: 0.0 }
    }
}

/// Configuration for the Hall of Fame.
///
/// Controls the archive capacity and diversity filtering behavior.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HallOfFameConfig {
    /// Maximum number of archived solutions.
    pub capacity: usize,
    /// Distance metric for diversity filtering.
    ///
    /// Default: `Fitness { min_distance: 0.0 }` (no diversity filtering).
    pub distance_metric: DistanceMetric,
}

impl Default for HallOfFameConfig {
    fn default() -> Self {
        HallOfFameConfig {
            capacity: 100,
            distance_metric: DistanceMetric::default(),
        }
    }
}

/// A single archived entry in the Hall of Fame.
///
/// Stores the chromosome along with metadata about when and with what
/// fitness it was added.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entry<U: ChromosomeT> {
    /// The archived chromosome (clone of the original).
    pub chromosome: U,
    /// Generation number when this solution was added.
    pub generation_added: u64,
    /// Fitness value at the time of addition.
    pub fitness_at_addition: f64,
}

/// Bounded archive of top-N unique solutions across the entire GA run.
///
/// Entries are stored in descending order of fitness (best first).
/// When the archive is full and a new solution qualifies, the worst
/// entry is evicted (D-04).
///
/// # Type parameters
///
/// * `U` -- The chromosome type, must implement [`ChromosomeT`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HallOfFame<U: ChromosomeT> {
    /// Entries sorted by fitness descending (best first).
    entries: Vec<Entry<U>>,
    /// Maximum number of entries.
    capacity: usize,
    /// Distance metric for diversity filtering.
    distance_metric: DistanceMetric,
}

impl<U: ChromosomeT> HallOfFame<U> {
    /// Creates a new Hall of Fame with the given configuration.
    pub fn new(config: HallOfFameConfig) -> Self {
        HallOfFame {
            entries: Vec::new(),
            capacity: config.capacity,
            distance_metric: config.distance_metric,
        }
    }

    /// Attempts to insert a chromosome into the Hall of Fame.
    ///
    /// A chromosome is admitted if:
    /// 1. It does not have NaN fitness (T-41-01)
    /// 2. The archive is not full, or its fitness is >= the worst entry
    /// 3. Its DNA is not already in the archive (genotypic uniqueness, D-07)
    /// 4. It passes the distance filter (if Genotypic with `min_distance > 0.0`)
    ///
    /// Returns `true` if the chromosome was admitted.
    pub fn try_insert(&mut self, chromosome: &U, generation: u64) -> bool {
        let fitness = chromosome.fitness();
        let dna = chromosome.dna();

        // Skip NaN fitness (T-41-01)
        if fitness.is_nan() {
            log::debug!("Skipping chromosome with NaN fitness");
            return false;
        }

        // Capacity zero: never admit any chromosome
        if self.capacity == 0 {
            return false;
        }

        // Check 1: Fitness threshold (D-06)
        // An admission is only possible if the archive is not full, or the
        // new chromosome's fitness is at least as good as the worst entry.
        let meets_threshold = if self.entries.len() < self.capacity {
            true
        } else {
            // Archive is full -- must beat or match the worst (last) entry's fitness.
            // Since entries are sorted descending, the last has the worst fitness.
            fitness >= self.entries.last().unwrap().fitness_at_addition
        };
        if !meets_threshold {
            return false;
        }

        // Check 2: Genotypic uniqueness (D-07)
        // Linear scan over entries comparing DNA slices via gene.id().
        // Uses the same max_len/unwrap_or(-1) pattern as niching (ga.rs:1045-1056).
        if self.entries.iter().any(|e| {
            let existing_dna = e.chromosome.dna();
            let max_len = dna.len().max(existing_dna.len());
            if max_len == 0 {
                return true; // both empty are equal
            }
            (0..max_len).all(|i| {
                let id_a = dna.get(i).map(|g| g.id()).unwrap_or(-1);
                let id_b = existing_dna.get(i).map(|g| g.id()).unwrap_or(-1);
                id_a == id_b
            })
        }) {
            return false; // DNA already in archive
        }

        // Check 3: Distance filter (D-01, D-03)
        if let DistanceMetric::Genotypic { min_distance } = self.distance_metric {
            if min_distance > 0.0 {
                for entry in self.entries.iter() {
                    let existing_dna = entry.chromosome.dna();
                    let distance = genotypic_distance(dna, existing_dna);
                    if distance < min_distance {
                        return false; // Too close to an existing entry
                    }
                }
            }
        }

        // Insert at correct sorted position (descending by fitness).
        // binary_search_by expects ascending order, so we reverse the comparison.
        let pos = self
            .entries
            .binary_search_by(|e| {
                e.fitness_at_addition
                    .partial_cmp(&fitness)
                    .unwrap_or(Ordering::Equal)
                    .reverse() // descending: higher fitness first
            })
            .unwrap_or_else(|e| e);

        self.entries.insert(
            pos,
            Entry {
                chromosome: chromosome.clone(),
                generation_added: generation,
                fitness_at_addition: fitness,
            },
        );

        // Enforce capacity (D-04: evict worst).
        // The last element is the worst (descending order).
        if self.entries.len() > self.capacity {
            self.entries.pop();
        }

        true
    }

    /// Returns all archived entries, sorted best-first.
    ///
    /// Each entry includes the chromosome, the generation when it was added,
    /// and the fitness value at the time of addition.
    pub fn solutions(&self) -> &[Entry<U>] {
        &self.entries
    }

    /// Returns the top `k` entries (or all, if fewer than `k`).
    ///
    /// The returned slice is sorted by fitness descending (best first).
    pub fn top(&self, k: usize) -> &[Entry<U>] {
        let end = k.min(self.entries.len());
        &self.entries[..end]
    }

    /// Returns `true` if the given chromosome would qualify for admission
    /// based on the current fitness threshold.
    ///
    /// This checks only the fitness threshold criterion -- it does NOT
    /// check genotypic uniqueness or distance filtering. Use this to
    /// pre-filter chromosomes before computing expensive genotypic checks.
    pub fn would_qualify(&self, chromosome: &U) -> bool {
        if self.entries.len() < self.capacity {
            true
        } else {
            chromosome.fitness() >= self.entries.last().unwrap().fitness_at_addition
        }
    }

    /// Returns the number of entries in the archive.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the archive is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over the archived entries with metadata.
    ///
    /// Each item is an `&Entry<U>` containing the chromosome, the generation
    /// when it was added, and the fitness at addition time.
    pub fn iter(&self) -> impl Iterator<Item = &Entry<U>> {
        self.entries.iter()
    }

    /// Returns a reference to the internal entries vector.
    ///
    /// Entries are sorted by fitness descending (best first).
    pub fn entries(&self) -> &[Entry<U>] {
        &self.entries
    }
}

/// Computes the genotypic distance between two DNA sequences.
///
/// Returns the fraction of positions that differ (range [0, 1]).
/// Missing positions (when lengths differ) count as differing, with
/// the missing gene's ID treated as `-1`.
///
/// Reuses the `.id()` comparison pattern from niching (ga.rs:1045-1056).
pub fn genotypic_distance<G: GeneT>(dna_a: &[G], dna_b: &[G]) -> f64 {
    let max_len = dna_a.len().max(dna_b.len());
    if max_len == 0 {
        return 0.0;
    }
    let mut diff = 0usize;
    for i in 0..max_len {
        let id_a = dna_a.get(i).map(|g| g.id()).unwrap_or(-1);
        let id_b = dna_b.get(i).map(|g| g.id()).unwrap_or(-1);
        if id_a != id_b {
            diff += 1;
        }
    }
    diff as f64 / max_len as f64
}
