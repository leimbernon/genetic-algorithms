//! Operations — operator enums, factory dispatchers, and runtime selection.
//!
//! Provides runtime-selectable genetic operators across five categories:
//! Selection, Crossover, Mutation, Survivor, and Extension. Each operator
//! follows the enum + factory function pattern for runtime dispatch, defined
//! by the enums in this module and implemented in the sub-modules.
//!
//! All operators are dispatched via the configuration system: you select an
//! operator variant in the builder (e.g., `with_selection_method(Selection::Tournament)`)
//! and the factory function constructs the appropriate implementation.
//!
//! # Key items
//!
//! | Item | Description |
//! |------|-------------|
//! | [`Selection`] | Selection operator enum (Tournament, RouletteWheel, SUS, Rank, Boltzmann, Truncation, Clearing, Random) |
//! | [`Crossover`] | Crossover operator enum (Cycle, MultiPoint, Uniform, SinglePoint, Order, PMX, SBX, BlendAlpha, Arithmetic, Edge, Clone, Rejuvenate) |
//! | [`Mutation`] | Mutation operator enum (Swap, Inversion, Scramble, Value, BitFlip, Creep, Gaussian, Polynomial, NonUniform, Insertion, Cauchy, LevyFlight, Uniform, ListValue) |
//! | [`Survivor`] | Survivor operator enum (Fitness, Age, MuPlusLambda, MuCommaLambda, DeterministicCrowding) |
//! | [`Extension`] | Extension strategy enum (Noop, MassExtinction, MassGenesis, MassDegeneration, MassDeduplication) |
//!
//! # When to use
//! Configure operators via the builder methods on your engine of choice. Custom
//! operators can be implemented by implementing the corresponding operator trait
//! ([`SelectionOperator`], [`CrossoverOperator`], etc.) in the [`traits`] module.

pub mod crossover;
pub mod extension;
pub mod local_search;
pub mod mutation;
pub mod selection;
pub mod survivor;

pub use local_search::{
    factory, factory_with_config, HillClimbingConfig, LocalSearch,
    LocalSearchApplicationStrategy, LocalSearchMode,
};

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
    /// Edge Recombination Crossover for permutation chromosomes (TSP, scheduling).
    /// Builds a union adjacency list from both parents and constructs offspring that
    /// preserve adjacency relationships found in either parent. Requires unique gene IDs.
    EdgeRecombination,
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
    /// DE-style differential mutation for `Range<T>` chromosomes.
    /// Computes mutant vector as `x_r1 + F * (x_r2 - x_r3)` from three distinct
    /// random population members (all distinct from the target), clamped to gene
    /// ranges. Configure F via `MutationConfiguration::differential_f` (default 0.5).
    /// Requires `population_size >= 4`. Applied automatically by the standard GA
    /// engine — do not call `factory_with_params` for this variant.
    Differential,
    /// Cauchy (Lorentzian) perturbation for `Range<T>` chromosomes.
    /// Uses the inverse-CDF method: `noise = scale * tan(π * (u - 0.5))`, where `u ~ Uniform(0, 1)`.
    /// Configure scale via [`crate::configuration::MutationConfiguration::cauchy_scale`]
    /// or the [`crate::traits::MutationConfig::with_cauchy_scale`] builder. Default scale: `1.0`.
    /// Returns `GaError::MutationError` for non-`Range<T>` chromosomes (Binary, List).
    Cauchy,
    /// Lévy Flight mutation for `Range<T>` chromosomes (Mantegna's algorithm).
    /// Generates heavy-tailed steps via `step = σ_u * u / |v|^(1/α)`.
    /// Configure the stability index (α) via [`crate::configuration::MutationConfiguration::levy_alpha`]
    /// or [`crate::traits::MutationConfig::with_levy_alpha`]. Valid range: (0.0, 2.0). Default α: `1.5`.
    /// Returns `GaError::MutationError` for non-`Range<T>` chromosomes.
    LevyFlight,
    /// Uniform reset mutation for `Range<T>` chromosomes.
    /// Resets a single randomly chosen gene to a uniform sample within its declared range.
    /// Equivalent to gene re-initialization. No configuration parameters required.
    /// Returns `GaError::MutationError` for non-`Range<T>` chromosomes.
    Uniform,
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
