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

use crate::traits::{ChromosomeT, VectorFitness};
use log::{debug, trace};
use rand::Rng;

// WASM: intentionally sequential — lexicase inner loop uses a shrinking pool state
// that cannot be parallelized. No Rayon iterators are used in this file.

/// Returns per-case MAD epsilons for dynamic epsilon-lexicase.
///
/// For each test case i, computes the Median Absolute Deviation (MAD) of the
/// case scores across all chromosomes. This provides a data-driven adaptive
/// tolerance that scales with the spread of scores on each case.
fn compute_mad_epsilons<U: VectorFitness>(chromosomes: &[U], num_cases: usize) -> Vec<f64> {
    (0..num_cases)
        .map(|case_i| {
            let mut scores: Vec<f64> = chromosomes
                .iter()
                .map(|c| c.fitness_values()[case_i])
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
fn select_one_winner<U: VectorFitness>(
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
            .map(|&i| chromosomes[i].fitness_values()[case])
            .fold(f64::NEG_INFINITY, f64::max);
        let eps = per_case_epsilon[case];
        pool.retain(|&i| chromosomes[i].fitness_values()[case] >= best - eps);
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

/// Lexicase selection: selects parent groups by per-case filtering with shuffled case order.
///
/// Requires chromosomes implementing [`VectorFitness`]. Each parent is
/// independently selected by iterating through test cases in a random order,
/// retaining only those that match the case-best score exactly.
///
/// # Arguments
///
/// * `chromosomes` - Population slice (must have at least 2 individuals with non-empty case scores).
/// * `number_of_couples` - Number of parent groups to produce.
/// * `num_parents` - Number of parents per group (must be >= 2).
///
/// # Returns
///
/// `Vec<Vec<usize>>` of parent index groups. Returns empty vec if population
/// has fewer than 2 individuals or case scores are empty.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::selection::lexicase_selection;
/// use genetic_algorithms::chromosomes::Binary;
/// let population: Vec<Binary> = vec![Binary::new(); 10];
/// let pairs = lexicase_selection(&population, 5, 2);
/// ```
pub fn lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
    num_parents: usize,
) -> Vec<Vec<usize>>
where
    U: ChromosomeT + VectorFitness,
{
    let num_parents = num_parents.max(2);
    debug!(target = "selection_events", method = "lexicase"; "Starting lexicase selection with number_of_couples={}", number_of_couples);

    if chromosomes.len() < 2 || chromosomes[0].fitness_values().is_empty() {
        return Vec::new();
    }

    let num_cases = chromosomes[0].fitness_values().len();
    if chromosomes.iter().any(|c| c.fitness_values().len() != num_cases) {
        log::warn!(target: "selection_events", "lexicase: fitness_values length mismatch — returning empty selection");
        return Vec::new();
    }
    let zero_eps = vec![0.0f64; num_cases];
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(number_of_couples);

    while mating.len() < number_of_couples {
        let mut group = Vec::with_capacity(num_parents);
        for _ in 0..num_parents {
            group.push(select_one_winner(chromosomes, num_cases, &zero_eps, &mut rng));
        }
        trace!(target = "selection_events", method = "lexicase"; "Group: {:?}", group);
        mating.push(group);
    }

    debug!(target = "selection_events", method = "lexicase"; "Lexicase selection finished: {} groups", mating.len());
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
/// * `number_of_couples` - Number of parent groups to produce.
/// * `epsilon` - Fixed tolerance (`Some(e)`) or dynamic MAD (`None`).
/// * `num_parents` - Number of parents per group (must be >= 2).
///
/// # Returns
///
/// `Vec<Vec<usize>>` of parent index groups. Returns empty vec if population
/// has fewer than 2 individuals or case scores are empty.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::selection::epsilon_lexicase_selection;
/// use genetic_algorithms::chromosomes::Binary;
/// let population: Vec<Binary> = vec![Binary::new(); 10];
/// let pairs = epsilon_lexicase_selection(&population, 5, Some(0.01), 2);
/// ```
pub fn epsilon_lexicase_selection<U>(
    chromosomes: &[U],
    number_of_couples: usize,
    epsilon: Option<f64>,
    num_parents: usize,
) -> Vec<Vec<usize>>
where
    U: ChromosomeT + VectorFitness,
{
    let num_parents = num_parents.max(2);
    debug!(target = "selection_events", method = "epsilon_lexicase"; "Starting epsilon-lexicase selection with number_of_couples={} epsilon={:?}", number_of_couples, epsilon);

    if chromosomes.len() < 2 || chromosomes[0].fitness_values().is_empty() {
        return Vec::new();
    }

    let num_cases = chromosomes[0].fitness_values().len();
    if chromosomes.iter().any(|c| c.fitness_values().len() != num_cases) {
        log::warn!(target: "selection_events", "epsilon_lexicase: fitness_values length mismatch — returning empty selection");
        return Vec::new();
    }
    let per_case_eps = match epsilon {
        Some(e) => vec![e; num_cases],
        None => compute_mad_epsilons(chromosomes, num_cases),
    };
    let mut rng = crate::rng::make_rng();
    let mut mating = Vec::with_capacity(number_of_couples);

    while mating.len() < number_of_couples {
        let mut group = Vec::with_capacity(num_parents);
        for _ in 0..num_parents {
            group.push(select_one_winner(chromosomes, num_cases, &per_case_eps, &mut rng));
        }
        trace!(target = "selection_events", method = "epsilon_lexicase"; "Group: {:?}", group);
        mating.push(group);
    }

    debug!(target = "selection_events", method = "epsilon_lexicase"; "Epsilon-lexicase selection finished: {} groups", mating.len());
    mating
}
