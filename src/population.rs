//! Population — chromosome container for evolving generations.
//!
//! A [`Population`] holds the current generation of chromosomes, tracks the
//! best-so-far individual, and computes aggregate fitness statistics used by
//! adaptive GA operators.
//!
//! The struct supports standard Rust collection traits (`IntoIterator`,
//! `FromIterator`, `Index`, `IndexMut`), and implements `Clone`, `Debug`,
//! and optional `serde` serialization behind the `serde` feature flag.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`Population<U>`] | Main container, generic over chromosome type |
//! | [`Population::best_chromosome`] | The best-so-far individual |
//! | `Population::fitness_stats` | Aggregate fitness statistics |
//!
//! # When to use
//! Used by all engines as the primary solution container. It is the return
//! type of `ga.run()` and provides access to the evolved solutions and their
//! fitness statistics.

use crate::configuration::ProblemSolving;
use crate::traits::ChromosomeT;
use rayon::prelude::*;
use std::fmt;
use std::ops::{Index, IndexMut};

/// Population of chromosomes with aggregate statistics and best tracking.
///
/// Responsibilities:
/// - Store the evolving set of chromosomes.
/// - Maintain best chromosome according to the configured problem objective.
/// - Compute average (f_avg) and maximum (f_max) fitness (used by adaptive GA).
/// - Parallel fitness calculation to leverage multiple threads.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::population::Population;
/// use genetic_algorithms::chromosomes::Binary;
///
/// // Create an empty population, then populate it.
/// let chromosomes: Vec<Binary> = Vec::new();
/// let pop = Population::new(chromosomes);
/// assert_eq!(pop.chromosomes.len(), 0);
///
/// // An empty population for use before the first generation:
/// let empty: Population<Binary> = Population::new_empty();
/// assert_eq!(empty.chromosomes.len(), 0);
/// ```
pub struct Population<U>
where
    U: ChromosomeT,
{
    /// The chromosomes (members) of the population.
    pub chromosomes: Vec<U>,

    /// Best chromosome in the population (by fitness, according to `ProblemSolving`).
    pub best_chromosome: U,
    pub(crate) best_chromosome_is_set: bool,

    /// Average fitness across the population.
    pub f_avg: f64,

    /// Largest fitness value in the population.
    pub f_max: f64,
}

impl<U> Population<U>
where
    U: ChromosomeT,
{
    /// Creates a new empty `Population`.
    pub fn new_empty() -> Population<U> {
        Population {
            chromosomes: vec![],
            f_avg: 0.0,
            f_max: 0.0,
            best_chromosome: U::new(),
            best_chromosome_is_set: false,
        }
    }

    /// Creates a `Population` with the given chromosomes as members.
    pub fn new(chromosomes: Vec<U>) -> Population<U> {
        Population {
            chromosomes,
            f_avg: 0.0,
            f_max: 0.0,
            best_chromosome: U::new(),
            best_chromosome_is_set: false,
        }
    }

    /// Recalculate `f_avg` and `f_max` (used by adaptive GA probabilities).
    pub fn recalculate_aga(&mut self) {
        self.f_max = f64::NEG_INFINITY;
        self.f_avg = 0.0;
        for chromosome in self.chromosomes.as_slice() {
            self.f_max = if chromosome.fitness() > self.f_max {
                chromosome.fitness()
            } else {
                self.f_max
            };
            self.f_avg += chromosome.fitness();
        }
        self.f_avg /= self.chromosomes.len() as f64;
    }

    /// Appends a list of chromosomes into the current population.
    pub fn add_chromosomes(&mut self, chromosomes: &mut Vec<U>) {
        self.chromosomes.append(chromosomes);
    }

    /// Returns the number of chromosomes in the population.
    pub fn size(&self) -> usize {
        self.chromosomes.len()
    }

    /// Computes fitness for unevaluated chromosomes in parallel and updates the best chromosome.
    ///
    /// Uses rayon's parallel iterators for efficient work distribution.
    /// A chromosome is considered unevaluated if its fitness is `NaN`.
    pub fn fitness_calculation(
        &mut self,
        _number_of_threads: usize,
        problem_solving: ProblemSolving,
    ) {
        crate::log_debug!(target="population_events", method="fitness_calculation"; "Started the population fitness calculation");

        // Calculate fitness in parallel for chromosomes that have not yet been evaluated.
        // NaN fitness indicates a chromosome whose fitness has never been computed.
        self.chromosomes.par_iter_mut().for_each(|chromosome| {
            if chromosome.fitness().is_nan() {
                chromosome.calculate_fitness();
            }
        });

        // Update best chromosome: scan for the best fitness among all
        // chromosomes and clone only the winner (instead of cloning every
        // chromosome in the loop).
        if let Some(best_idx) = find_best_index(&self.chromosomes, problem_solving) {
            if !self.best_chromosome_is_set {
                self.best_chromosome = self.chromosomes[best_idx].clone();
                self.best_chromosome_is_set = true;
            } else {
                let candidate_fitness = self.chromosomes[best_idx].fitness();
                let current_fitness = self.best_chromosome.fitness();
                let candidate_is_better = match problem_solving {
                    ProblemSolving::Maximization | ProblemSolving::FixedFitness => {
                        candidate_fitness > current_fitness
                    }
                    ProblemSolving::Minimization => candidate_fitness < current_fitness,
                };
                if candidate_is_better {
                    self.best_chromosome = self.chromosomes[best_idx].clone();
                }
            }
        }

        crate::log_debug!(target="ga_events", method="population_fitness_calculation"; "Population fitness calculation finished");
    }

    /// Update the best chromosome given a candidate, according to the problem objective.
    pub fn decide_best_chromosome(&mut self, new_chromosome: &U, problem_solving: ProblemSolving) {
        crate::log_debug!(target="population_events", method="decide_best_chromosome"; "Started the best chromosome method");

        if !self.best_chromosome_is_set {
            self.best_chromosome = new_chromosome.clone();
            self.best_chromosome_is_set = true;
        } else {
            crate::log_trace!(target="population_events", method="decide_best_chromosome"; "Best chromosome fitness: {} - New chromosome fitness: {}",
                self.best_chromosome.fitness(), new_chromosome.fitness());

            let is_self_better = match problem_solving {
                ProblemSolving::Maximization => {
                    self.best_chromosome.fitness() >= new_chromosome.fitness()
                }
                ProblemSolving::Minimization => {
                    self.best_chromosome.fitness() < new_chromosome.fitness()
                }
                _ => self.best_chromosome.fitness() >= new_chromosome.fitness(),
            };

            if !is_self_better {
                self.best_chromosome = new_chromosome.clone();
            };
        }

        crate::log_debug!(target="chromosome_events", method="get_best_chromosome"; "Best chromosome method finished");
    }
}

/// Finds the index of the best chromosome in a slice according to `problem_solving`.
///
/// Returns `None` for an empty slice.
fn find_best_index<U: ChromosomeT>(
    chromosomes: &[U],
    problem_solving: ProblemSolving,
) -> Option<usize> {
    if chromosomes.is_empty() {
        return None;
    }
    let mut best_idx = 0;
    let mut best_fit = chromosomes[0].fitness();
    for (i, c) in chromosomes.iter().enumerate().skip(1) {
        let fit = c.fitness();
        let is_better = match problem_solving {
            ProblemSolving::Maximization | ProblemSolving::FixedFitness => fit > best_fit,
            ProblemSolving::Minimization => fit < best_fit,
        };
        if is_better {
            best_idx = i;
            best_fit = fit;
        }
    }
    Some(best_idx)
}

// ---------------------------------------------------------------------------
// Standard trait implementations for Population<U>
// ---------------------------------------------------------------------------

impl<U: ChromosomeT + Clone> Clone for Population<U> {
    fn clone(&self) -> Self {
        Population {
            chromosomes: self.chromosomes.clone(),
            best_chromosome: self.best_chromosome.clone(),
            best_chromosome_is_set: self.best_chromosome_is_set,
            f_avg: self.f_avg,
            f_max: self.f_max,
        }
    }
}

impl<U: ChromosomeT + fmt::Debug> fmt::Debug for Population<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Population")
            .field("size", &self.chromosomes.len())
            .field("f_avg", &self.f_avg)
            .field("f_max", &self.f_max)
            .field("best_chromosome_is_set", &self.best_chromosome_is_set)
            .finish()
    }
}

impl<U: ChromosomeT> Default for Population<U> {
    fn default() -> Self {
        Population::new_empty()
    }
}

impl<U: ChromosomeT> IntoIterator for Population<U> {
    type Item = U;
    type IntoIter = std::vec::IntoIter<U>;

    fn into_iter(self) -> Self::IntoIter {
        self.chromosomes.into_iter()
    }
}

impl<'a, U: ChromosomeT> IntoIterator for &'a Population<U> {
    type Item = &'a U;
    type IntoIter = std::slice::Iter<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.chromosomes.iter()
    }
}

impl<'a, U: ChromosomeT> IntoIterator for &'a mut Population<U> {
    type Item = &'a mut U;
    type IntoIter = std::slice::IterMut<'a, U>;

    fn into_iter(self) -> Self::IntoIter {
        self.chromosomes.iter_mut()
    }
}

impl<U: ChromosomeT> FromIterator<U> for Population<U> {
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let chromosomes: Vec<U> = iter.into_iter().collect();
        Population::new(chromosomes)
    }
}

impl<U: ChromosomeT> Index<usize> for Population<U> {
    type Output = U;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chromosomes[index]
    }
}

impl<U: ChromosomeT> IndexMut<usize> for Population<U> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.chromosomes[index]
    }
}

// ---------------------------------------------------------------------------
// Serde support (behind the "serde" feature flag)
// ---------------------------------------------------------------------------

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::de::{self, MapAccess, Visitor};
    use serde::ser::SerializeStruct;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::marker::PhantomData;

    impl<U> Serialize for Population<U>
    where
        U: ChromosomeT + Serialize,
    {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut state = serializer.serialize_struct("Population", 5)?;
            state.serialize_field("chromosomes", &self.chromosomes)?;
            state.serialize_field("best_chromosome", &self.best_chromosome)?;
            state.serialize_field("best_chromosome_is_set", &self.best_chromosome_is_set)?;
            state.serialize_field("f_avg", &self.f_avg)?;
            state.serialize_field("f_max", &self.f_max)?;
            state.end()
        }
    }

    impl<'de, U> Deserialize<'de> for Population<U>
    where
        U: ChromosomeT + Deserialize<'de>,
    {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            #[derive(serde::Deserialize)]
            #[serde(field_identifier, rename_all = "snake_case")]
            enum Field {
                Chromosomes,
                BestChromosome,
                BestChromosomeIsSet,
                FAvg,
                FMax,
            }

            struct PopulationVisitor<U>(PhantomData<U>);

            impl<'de, U> Visitor<'de> for PopulationVisitor<U>
            where
                U: ChromosomeT + Deserialize<'de>,
            {
                type Value = Population<U>;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("struct Population")
                }

                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut chromosomes: Option<Vec<U>> = None;
                    let mut best_chromosome: Option<U> = None;
                    let mut best_chromosome_is_set: Option<bool> = None;
                    let mut f_avg: Option<f64> = None;
                    let mut f_max: Option<f64> = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Chromosomes => {
                                chromosomes = Some(map.next_value()?);
                            }
                            Field::BestChromosome => {
                                best_chromosome = Some(map.next_value()?);
                            }
                            Field::BestChromosomeIsSet => {
                                best_chromosome_is_set = Some(map.next_value()?);
                            }
                            Field::FAvg => {
                                f_avg = Some(map.next_value()?);
                            }
                            Field::FMax => {
                                f_max = Some(map.next_value()?);
                            }
                        }
                    }

                    Ok(Population {
                        chromosomes: chromosomes
                            .ok_or_else(|| de::Error::missing_field("chromosomes"))?,
                        best_chromosome: best_chromosome
                            .ok_or_else(|| de::Error::missing_field("best_chromosome"))?,
                        best_chromosome_is_set: best_chromosome_is_set
                            .ok_or_else(|| de::Error::missing_field("best_chromosome_is_set"))?,
                        f_avg: f_avg.ok_or_else(|| de::Error::missing_field("f_avg"))?,
                        f_max: f_max.ok_or_else(|| de::Error::missing_field("f_max"))?,
                    })
                }
            }

            const FIELDS: &[&str] = &[
                "chromosomes",
                "best_chromosome",
                "best_chromosome_is_set",
                "f_avg",
                "f_max",
            ];
            deserializer.deserialize_struct("Population", FIELDS, PopulationVisitor(PhantomData))
        }
    }
}
