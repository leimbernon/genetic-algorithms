//! Parsimony pressure for variable-length chromosomes.
//!
//! Provides [`apply_parsimony_pressure`], which wraps any survivor-selection call
//! with a length-based fitness adjustment. The stored `fitness()` value on each
//! chromosome is **never** permanently mutated — the adjustment is applied
//! temporarily before sorting and reversed immediately afterward.
//!
//! # Formula
//!
//! ```text
//! effective_fitness = fitness ∓ (length_penalty × dna_length)
//! ```
//!
//! Sign convention (auto-adjusted per `ProblemSolving` mode):
//! - **Maximization** — subtract: `fitness - (penalty × length)` (longer ≡ worse)
//! - **Minimization** — add: `fitness + (penalty × length)` (longer ≡ worse)

use crate::configuration::{LimitConfiguration, ProblemSolving};
use crate::error::GaError;
use crate::operations::Survivor;
use crate::traits::LinearChromosome;
use log::debug;

/// Computes the parsimony-adjusted fitness for a single chromosome.
///
/// Does **not** mutate the chromosome — purely a computation helper.
///
/// # Arguments
///
/// * `raw_fitness` — The stored fitness value.
/// * `dna_length` — The chromosome's current DNA length.
/// * `length_penalty` — Parsimony coefficient (positive value).
/// * `problem_solving` — Optimization direction.
fn adjusted_fitness(
    raw_fitness: f64,
    dna_length: usize,
    length_penalty: f64,
    problem_solving: ProblemSolving,
) -> f64 {
    let adjustment = length_penalty * dna_length as f64;
    match problem_solving {
        ProblemSolving::Maximization => raw_fitness - adjustment,
        ProblemSolving::Minimization | ProblemSolving::FixedFitness => raw_fitness + adjustment,
    }
}

/// Runs survivor selection with parsimony pressure applied.
///
/// Temporarily adjusts each chromosome's fitness by `±(length_penalty × dna_length)`,
/// invokes the standard survivor selection, then restores the original fitness values.
///
/// # Arguments
///
/// * `survivor` — The survivor-selection strategy to use.
/// * `chromosomes` — The combined parent + offspring population (modified in place).
/// * `population_size` — Desired post-selection size.
/// * `limit_configuration` — Controls sorting direction and optional target.
/// * `length_penalty` — Parsimony coefficient. Sign is auto-adjusted per mode.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::operations::survivor::apply_parsimony_pressure;
/// use genetic_algorithms::operations::Survivor;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::configuration::{LimitConfiguration, ProblemSolving};
/// let mut population: Vec<Binary> = vec![Binary::new(); 20];
/// let limits = LimitConfiguration::default().with_problem_solving(ProblemSolving::Maximization);
/// let _ = apply_parsimony_pressure(Survivor::Fitness, &mut population, 10, limits, 0.001);
/// ```
///
/// # Errors
///
/// Propagates any `GaError` from the underlying survivor-selection call.
pub fn apply_parsimony_pressure<U: LinearChromosome>(
    survivor: Survivor,
    chromosomes: &mut Vec<U>,
    population_size: usize,
    limit_configuration: LimitConfiguration,
    length_penalty: f64,
) -> Result<(), GaError> {
    debug!(
        target = "survivor_events",
        method = "parsimony";
        "Applying parsimony pressure (penalty={}, mode={:?}, pop={})",
        length_penalty, limit_configuration.problem_solving, chromosomes.len()
    );

    let problem_solving = limit_configuration.problem_solving;

    // Step 1: apply temporary fitness adjustment
    for c in chromosomes.iter_mut() {
        let raw = c.fitness();
        let len = c.dna().len();
        let adj = adjusted_fitness(raw, len, length_penalty, problem_solving);
        c.set_fitness(adj);
    }

    // Step 2: run standard survivor selection (operates on adjusted fitness)
    let result = super::factory(survivor, chromosomes, population_size, limit_configuration);

    // Step 3: restore original fitness for all survivors
    // We can re-derive original from adjusted: raw = adjusted ∓ (penalty × len)
    for c in chromosomes.iter_mut() {
        let adj = c.fitness();
        let len = c.dna().len();
        let raw = match problem_solving {
            ProblemSolving::Maximization => adj + length_penalty * len as f64,
            ProblemSolving::Minimization | ProblemSolving::FixedFitness => {
                adj - length_penalty * len as f64
            }
        };
        c.set_fitness(raw);
    }

    result
}
