//! Genetic operators: selection, crossover, mutation, and survivor selection.
//!
//! Each sub-module contains the operator implementations and a `factory`
//! function that dispatches to the correct variant at runtime based on the
//! configuration enums defined here ([`Selection`], [`Crossover`],
//! [`Mutation`], [`Survivor`]).

pub mod crossover;
pub mod extension;
pub mod mutation;
pub mod selection;
pub mod survivor;

/// Parent-selection strategies.
///
/// Determines how individuals are chosen from the current population to
/// become parents for the next generation's offspring.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Selection {
    /// Pure random selection — every individual has equal probability.
    Random,
    /// Fitness-proportionate selection — probability is proportional to fitness.
    RouletteWheel,
    /// Like roulette wheel but with evenly spaced pointers for lower variance.
    StochasticUniversalSampling,
    /// Pairwise tournament — two (or more) individuals compete and the fitter wins.
    Tournament,
    /// Rank-based selection: individuals are ranked by fitness and selection
    /// probability is proportional to rank, avoiding dominance by very fit individuals.
    Rank,
    /// Boltzmann selection: uses a temperature parameter to control selective pressure.
    /// High temperature -> uniform selection (exploration), low temperature -> strong
    /// selective pressure (exploitation).
    Boltzmann,
    /// Truncation selection: only the top portion of the population is eligible
    /// for reproduction, providing very high selective pressure.
    Truncation,
    /// Clearing selection: identifies niche winners (the best individual within
    /// `niche_radius` in fitness space) and removes all other individuals in each
    /// niche from the selection pool. Eligible individuals are then paired randomly.
    /// Promotes population diversity by preventing niche domination.
    /// Configure `niche_radius` via [`SelectionConfiguration::niche_radius`].
    Clearing,
}

/// Crossover (recombination) strategies.
///
/// Determines how two parent chromosomes are combined to produce offspring.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Crossover {
    /// Cycle crossover — preserves the position of each gene from one parent.
    Cycle,
    /// Multi-point crossover — alternates segments between parents at N random cut points.
    MultiPoint,
    /// Uniform crossover — each gene is independently chosen from either parent.
    Uniform,
    /// Single-point crossover — one cut point splits both parents into two halves that are swapped.
    SinglePoint,
    /// Order crossover (OX) — preserves relative ordering, suited for permutation chromosomes.
    Order,
    /// Partially Mapped Crossover for permutation-based chromosomes.
    /// Preserves absolute positions within a segment and relative order outside it.
    Pmx,
    /// Simulated Binary Crossover for `Range<T>` chromosomes.
    /// Uses a distribution index (eta) configured via `CrossoverConfiguration`.
    Sbx,
    /// Blend Crossover (BLX-alpha) for `Range<T>` chromosomes.
    /// Uses an alpha parameter configured via `CrossoverConfiguration`.
    BlendAlpha,
    /// Arithmetic (whole) crossover for `Range<T>` chromosomes.
    /// Child = alpha * parent1 + (1 - alpha) * parent2. Uses `arithmetic_alpha` from configuration.
    Arithmetic,
    /// Clone crossover — copies parents directly as offspring without any genetic exchange.
    /// Useful for mutation-only strategies and baseline experiments.
    Clone,
    /// Rejuvenate crossover — clones parents as offspring and resets their ages to zero.
    /// Useful for combating population aging: top performers are preserved but treated as new
    /// individuals, preventing age-based survivor selection from eliminating them.
    Rejuvenate,
}

/// Mutation strategies.
///
/// Determines how offspring chromosomes are randomly altered to maintain
/// genetic diversity.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mutation {
    /// Swap mutation — two random genes exchange positions.
    Swap,
    /// Inversion mutation — a random sub-sequence of the chromosome is reversed.
    Inversion,
    /// Scramble mutation — a random sub-sequence is shuffled in place.
    Scramble,
    /// Value mutation — a single gene is replaced with a random allele.
    Value,
    /// Bit-flip mutation — each bit (gene) is flipped with a given probability (binary chromosomes).
    BitFlip,
    /// Small uniform perturbation mutation for `Range<T>` chromosomes.
    /// Requires a step size configured via `MutationConfiguration`.
    Creep,
    /// Gaussian (normal distribution) perturbation mutation for `Range<T>` chromosomes.
    /// Requires a sigma configured via `MutationConfiguration`.
    Gaussian,
    /// Polynomial mutation for `Range<T>` chromosomes (NSGA-II style).
    /// Uses a distribution index (eta_m) from `MutationConfiguration`.
    Polynomial,
    /// Non-uniform mutation for `Range<T>` chromosomes.
    /// Mutation magnitude decreases over generations.
    NonUniform,
    /// Insertion mutation for permutation-based chromosomes.
    /// Removes a gene and reinserts it at a different position.
    Insertion,
    /// List-value mutation — replaces a single gene's value with a different allele
    /// from that gene's allele set. Requires a `ListChromosome<T>`.
    ListValue,
}

/// Survivor-selection strategies.
///
/// Determines which individuals from the combined parent+offspring pool
/// survive into the next generation.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Survivor {
    /// Keep the fittest individuals regardless of age.
    Fitness,
    /// Keep the youngest individuals (most recently created).
    Age,
    /// (mu+lambda) strategy: parents and offspring compete together for survival.
    MuPlusLambda,
    /// (mu,lambda) strategy: only offspring (age == 0) are eligible for survival.
    MuCommaLambda,
    /// Deterministic Crowding: each offspring (identified by `age() == 0`) is
    /// paired with its most similar parent (lowest Hamming distance on gene IDs),
    /// and the fitter of the two survives. Unpaired offspring survive unconditionally.
    /// Promotes population diversity by replacing similar individuals.
    DeterministicCrowding,
}

/// Extension strategies for population diversity control.
///
/// Extensions are optional diversity-rescue mechanisms that trigger when
/// population diversity drops below a configurable threshold.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Extension {
    /// No extension — diversity drops are ignored.
    Noop,
    /// Random cull to a survival rate, protecting elite individuals.
    MassExtinction,
    /// Trim to the 2 best chromosomes, regrow population from scratch.
    MassGenesis,
    /// Apply N mutation rounds to the whole population, protecting elite.
    MassDegeneration,
    /// Remove duplicate chromosomes (by gene comparison), regrow population.
    MassDeduplication,
}

impl Extension {
    /// Returns the extension variant name as a static string.
    ///
    /// Used by [`ExtensionEvent`](crate::observer::ExtensionEvent) to avoid heap allocation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Extension::Noop => "Noop",
            Extension::MassExtinction => "MassExtinction",
            Extension::MassGenesis => "MassGenesis",
            Extension::MassDegeneration => "MassDegeneration",
            Extension::MassDeduplication => "MassDeduplication",
        }
    }
}
