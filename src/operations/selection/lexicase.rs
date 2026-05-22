//! Lexicase and epsilon-lexicase selection operators.
//!
//! Lexicase selection filters candidates by iterating over test cases in a
//! shuffled order, retaining only those that are at least as good as the
//! current-case best. This promotes specialist preservation.
//!
//! Epsilon-lexicase relaxes the filter: candidates within `epsilon` of the
//! per-case maximum are retained. When `epsilon = None`, uses the dynamic
//! per-case Median Absolute Deviation (MAD).
//!
//! WASM note: This implementation uses sequential `.iter()` throughout.
//! The shrinking-pool state in the filter cascade cannot be parallelised.

use crate::traits::{ChromosomeT, MultiCaseFitness};
use log::{debug, trace};
use rand::Rng;

// WASM: intentionally sequential — lexicase inner loop uses a shrinking pool state
// that cannot be parallelized. No Rayon iterators are used in this file.

/// Returns per-case MAD epsilons for dynamic epsilon-lexicase.
///
/// For each test case i, computes the Median Absolute Deviation (MAD) of the
/// case scores across all chromosomes. This provides a data-driven adaptive
/// tolerance that scales with the spread of scores on each case.
fn compute_mad_epsilons<U: MultiCaseFitness>(chromosomes: &[U], num_cases: usize) -> Vec<f64> {
    (0..num_cases)
        .map(|case_i| {
            let mut scores: Vec<f64> = chromosomes
                .iter()
                .map(|c| c.case_fitness()[case_i])
                .collect();
            scores.sort_unstable_by(|a, b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });

            let n = scores.len();
            let median = if n % 2 == 1 {
                scores[n / 2]
            } else {
                (scores[n / 2 - 1] + scores[n / 2]) / 2.0
            };

            let mut abs_devs: Vec<f64> = scores.iter().map(|&s| (s - median).abs()).collect();
            abs_devs.sort_unstable_by(|a, b| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });

            if n % 2 == 1 {
                abs_devs[n / 2]
            } else {
                (abs_devs[n / 2 - 1] + abs_devs[n / 2]) / 2.0
            }
        })
        .collect()
}

/// Selects one winner from the pool via lexicase case-by-case filtering.
///
/// Cases are iterated in a randomly shuffled order (Fisher-Yates). At each
/// case, only candidates within `per_case_epsilon[case]` of the case-best
/// score are retained. Returns the index (into `chromosomes`) of the winner.
fn select_one_winner<U: MultiCaseFitness>(
    chromosomes: &[U],
    num_cases: usize,
    per_case_epsilon: &[f64],
    rng: &mut impl Rng,
) -> usize {
    // Fisher-Yates shuffle of case order
    let mut case_order: Vec<usize> = (0..num_cases).collect();
    for i in (1..num_cases).rev() {
        let j = rng.random_range(0..=i);
        case_order.swap(i, j);
    }

    let mut pool: Vec<usize> = (0..chromosomes.len()).collect();

    for &case in &case_order {
        if pool.len() <= 1 {
            break;
        }
        let best = pool
            .iter()
            .map(|&i| chromosomes[i].case_fitness()[case])
            .fold(f64::NEG_INFINITY, f64::max);
        let eps = per_case_epsilon[case];
        pool.retain(|&i| chromosomes[i].case_fitness()[case] >= best - eps);
        debug_assert!(
            !pool.is_empty(),
            "Lexicase pool became empty — case {} best={} eps={}",
            case,
            best,
            eps
        );
    }

    let winner = pool[rng.random_range(0..pool.len())];
    trace!(target = "selection_events", method = "lexicase"; "Winner: index={}", winner);
    winner
}

/// Lexicase selection: selects parent pairs by per-case filtering with shuffled case order.
///
/// Requires chromosomes implementing [`MultiCaseFitness`]. Each parent is
/// independently selected by iterating through test cases in a random order,
/// retaining only those that match the case-best score exactly.
///
/// # Arguments
///
/// * `chromosomes` - Population slice (must have at least 2 individuals with non-empty case scores).
/// * `number_of_couples` - Number of parent pairs to produce.
///
/// # Returns
///
/// `Vec<(usize, usize)>` of parent index pairs. Returns empty vec if population
/// has fewer than 2 individuals or case scores are empty.
pub fn lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
) -> Vec<(usize, usize)>
where
    U: ChromosomeT + MultiCaseFitness,
{
    debug!(target = "selection_events", method = "lexicase"; "Starting lexicase selection with number_of_couples={}", number_of_couples);

    if chromosomes.len() < 2 || chromosomes[0].case_fitness().is_empty() {
        return Vec::new();
    }

    let num_cases = chromosomes[0].case_fitness().len();
    let zero_eps = vec![0.0f64; num_cases];
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(number_of_couples);

    while mating.len() < number_of_couples {
        let p1 = select_one_winner(chromosomes, num_cases, &zero_eps, &mut rng);
        let p2 = select_one_winner(chromosomes, num_cases, &zero_eps, &mut rng);
        mating.push((p1, p2));
        trace!(target = "selection_events", method = "lexicase"; "Pair: ({}, {})", p1, p2);
    }

    debug!(target = "selection_events", method = "lexicase"; "Lexicase selection finished: {} pairs", mating.len());
    mating
}

/// Epsilon-lexicase selection: like lexicase but with relaxed per-case retention.
///
/// When `epsilon = Some(e)`, candidates within `e` of the case-best score are
/// retained. When `epsilon = None`, a dynamic per-case MAD (Median Absolute
/// Deviation) is used as the tolerance, adapting to each case's score distribution.
///
/// # Arguments
///
/// * `chromosomes` - Population slice (must have at least 2 individuals with non-empty case scores).
/// * `number_of_couples` - Number of parent pairs to produce.
/// * `epsilon` - Fixed tolerance (`Some(e)`) or dynamic MAD (`None`).
///
/// # Returns
///
/// `Vec<(usize, usize)>` of parent index pairs. Returns empty vec if population
/// has fewer than 2 individuals or case scores are empty.
pub fn epsilon_lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
    epsilon: Option<f64>,
) -> Vec<(usize, usize)>
where
    U: ChromosomeT + MultiCaseFitness,
{
    debug!(target = "selection_events", method = "epsilon_lexicase"; "Starting epsilon-lexicase selection with number_of_couples={} epsilon={:?}", number_of_couples, epsilon);

    if chromosomes.len() < 2 || chromosomes[0].case_fitness().is_empty() {
        return Vec::new();
    }

    let num_cases = chromosomes[0].case_fitness().len();
    let per_case_eps = match epsilon {
        Some(e) => vec![e; num_cases],
        None => compute_mad_epsilons(chromosomes, num_cases),
    };
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(number_of_couples);

    while mating.len() < number_of_couples {
        let p1 = select_one_winner(chromosomes, num_cases, &per_case_eps, &mut rng);
        let p2 = select_one_winner(chromosomes, num_cases, &per_case_eps, &mut rng);
        mating.push((p1, p2));
        trace!(target = "selection_events", method = "epsilon_lexicase"; "Pair: ({}, {})", p1, p2);
    }

    debug!(target = "selection_events", method = "epsilon_lexicase"; "Epsilon-lexicase selection finished: {} pairs", mating.len());
    mating
}
