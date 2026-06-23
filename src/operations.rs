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
//! | [`Mutation`] | Mutation operator enum (Swap, Inversion, Scramble, Value, BitFlip, Creep, Gaussian, Polynomial, NonUniform, PermutationInsert, Insertion, Deletion, Cauchy, LevyFlight, Uniform, ListValue) |
//! | [`Survivor`] | Survivor operator enum (Fitness, Age, MuPlusLambda, MuCommaLambda, DeterministicCrowding) |
//! | [`Extension`] | Extension strategy enum (Noop, MassExtinction, MassGenesis, MassDegeneration, MassDeduplication) |
//!
//! # When to use
//! Configure operators via the builder methods on your engine of choice. Custom
//! operators can be implemented by implementing the corresponding operator trait
//! `SelectionOperator`, `CrossoverOperator`, etc.) in the [`traits`](crate::traits) module.

pub mod crossover;
pub mod extension;
pub mod local_search;
pub mod mutation;
pub mod selection;
pub mod survivor;

pub use local_search::{
    factory, factory_with_config, HillClimbingConfig, LocalSearch, LocalSearchApplicationStrategy,
    LocalSearchMode,
};

/// Parent-selection strategies.
///
/// Determines how individuals are chosen from the current population to
/// become parents for the next generation's offspring.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Selection;
/// use genetic_algorithms::traits::{ConfigurationT, SelectionConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_selection_method(Selection::Tournament);
/// ```
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
    /// Configure `niche_radius` via the selection configuration.
    Clearing,
    /// Lexicase selection. Requires chromosomes implementing
    /// [`MultiCaseFitness`](crate::traits::MultiCaseFitness).
    /// Scalar fitness is synced to the mean of case scores after each selection event.
    Lexicase,
    /// Epsilon-lexicase selection. Like `Selection::Lexicase` but with relaxed
    /// per-case candidate retention. Configure tolerance via
    /// [`SelectionConfiguration::epsilon`](crate::configuration::SelectionConfiguration::epsilon);
    /// `0.0` = dynamic MAD per case.
    EpsilonLexicase,
}

/// Alignment strategy for variable-length crossover.
///
/// When the two parents have different DNA lengths, `AlignmentStrategy` determines
/// how the lengths are reconciled before the single-point crossover is applied.
///
/// - `Trim` — both parents are truncated to `min(len_a, len_b)` before recombination.
///   Offspring length equals `min(len_a, len_b)`.
/// - `Pad` — the shorter parent is padded with genes sampled from its own alleles
///   until both parents reach `max(len_a, len_b)`. Offspring length equals
///   `max(len_a, len_b)`.
///
/// # Examples
///
/// ```rust
/// use genetic_algorithms::operations::{AlignmentStrategy, Crossover};
///
/// let strategy = AlignmentStrategy::Trim;
/// assert_eq!(strategy, AlignmentStrategy::Trim);
///
/// // Use with VariableLength crossover:
/// let crossover = Crossover::VariableLength(AlignmentStrategy::Pad);
/// assert_eq!(crossover, Crossover::VariableLength(AlignmentStrategy::Pad));
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlignmentStrategy {
    /// Trim both parents to the shorter length before crossover.
    /// Offspring have length `min(len_a, len_b)`.
    Trim,
    /// Pad the shorter parent (from its alleles) to the longer length before crossover.
    /// Offspring have length `max(len_a, len_b)`.
    Pad,
}

/// Crossover (recombination) strategies.
///
/// Determines how two parent chromosomes are combined to produce offspring.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Crossover;
/// use genetic_algorithms::traits::{ConfigurationT, CrossoverConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_crossover_method(Crossover::Uniform);
/// ```
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
    /// Variable-length crossover for chromosomes with different DNA lengths.
    ///
    /// Applies single-point crossover after aligning the two parents according to
    /// the specified [`AlignmentStrategy`]:
    /// - [`AlignmentStrategy::Trim`] — both parents are truncated to `min(len_a, len_b)`.
    /// - [`AlignmentStrategy::Pad`] — the shorter parent is padded to `max(len_a, len_b)`.
    ///
    /// Fixed-length crossover operators (`SinglePoint`, `Uniform`, etc.) return
    /// `GaError::CrossoverError` when parents have unequal lengths. Use this variant
    /// when variable-length chromosomes are in the population.
    VariableLength(AlignmentStrategy),
    /// Multi-group PMX crossover for `MultiUniqueChromosome<T>`.
    /// Applies Partially Mapped Crossover (PMX) independently within each permutation
    /// group defined by `MultiUniqueChromosome::group_ranges()`. Each group is treated
    /// as a separate permutation; genes are never exchanged across group boundaries.
    MultiGroupPmx,
    /// Multi-group OX crossover for `MultiUniqueChromosome<T>`.
    /// Applies Order Crossover (OX) independently within each permutation group defined
    /// by `MultiUniqueChromosome::group_ranges()`. Each group is treated as a separate
    /// permutation; relative order is preserved within groups, not across them.
    MultiGroupOx,
    /// Unimodal Normal Distribution Crossover (UNDX) for real-valued chromosomes.
    ///
    /// Requires at least 3 parents and chromosomes implementing
    /// [`RealValued`](crate::traits::RealValued). Dispatched via
    /// `factory_multi_parent` (Plan 02). Each call produces **1 offspring**
    /// centered at the centroid of all parents, perturbed normally along
    /// inter-parent directions.
    ///
    /// `num_parents` must be `>= 3`.
    Undx { num_parents: usize },
    /// Simplex Crossover (SPX) for real-valued chromosomes.
    ///
    /// Requires at least 3 parents and chromosomes implementing
    /// [`RealValued`](crate::traits::RealValued). Dispatched via
    /// `factory_multi_parent` (Plan 02). Samples offspring uniformly from
    /// the expanded simplex defined by the parent vertices.
    ///
    /// `num_parents` must be `>= 3`.
    Spx { num_parents: usize },
    /// Parent-Centric Crossover (PCX) for real-valued chromosomes.
    ///
    /// Requires at least 3 parents and chromosomes implementing
    /// [`RealValued`](crate::traits::RealValued). Dispatched via
    /// `factory_multi_parent` (Plan 02). Produces offspring biased toward
    /// the primary parent (index 0) with perturbation along directions from
    /// the other parents.
    ///
    /// `num_parents` must be `>= 3`.
    Pcx { num_parents: usize },
}

/// Mutation strategies.
///
/// Determines how offspring chromosomes are randomly altered to maintain
/// genetic diversity.
///
/// # Per-variant parameters
///
/// Several variants carry inline `Option<f64>` parameters. When `None`, a sensible
/// default is applied (see each variant's documentation). This replaces the old
/// `MutationConfiguration` operator-specific fields (`step`, `sigma`, etc.) which
/// have been removed in v3.0.0.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::{Mutation, GaussianParams};
/// use genetic_algorithms::traits::{ConfigurationT, MutationConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: Some(0.1) }));
/// ```
/// Parameters for [`Mutation::Creep`] — small uniform perturbation mutation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreepParams {
    /// Step size for the perturbation. Default: `0.01`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub step: Option<f64>,
}

/// Parameters for [`Mutation::Gaussian`] — Gaussian (normal distribution) perturbation mutation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GaussianParams {
    /// Standard deviation of the Gaussian noise. Default: `0.1`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma: Option<f64>,
}

/// Parameters for [`Mutation::Polynomial`] — polynomial mutation (NSGA-II style).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PolynomialParams {
    /// Distribution index η_m. Default: `20.0`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub eta: Option<f64>,
}

/// Parameters for [`Mutation::NonUniform`] — non-uniform mutation with generation-based decay.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NonUniformParams {
    /// Decay parameter b. Default: `2.0`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub b: Option<f64>,
}

/// Parameters for [`Mutation::Differential`] — DE-style differential mutation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DifferentialParams {
    /// F scale factor for the perturbation. Default: `0.5`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub f: Option<f64>,
}

/// Parameters for [`Mutation::Cauchy`] — Cauchy (Lorentzian) perturbation mutation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CauchyParams {
    /// Scale parameter γ. Default: `1.0`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub scale: Option<f64>,
}

/// Parameters for [`Mutation::LevyFlight`] — Lévy Flight mutation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LevyFlightParams {
    /// Stability index α. Default: `1.5`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub alpha: Option<f64>,
}

/// Parameters for [`Mutation::SelfAdaptiveGaussian`] — self-adaptive Gaussian mutation (ES-style).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfAdaptiveGaussianParams {
    /// Per-dimension learning rate τ. Default: `1.0 / sqrt(2.0 * n)`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tau: Option<f64>,
    /// Global learning rate τ'. Default: `1.0 / sqrt(2.0 * sqrt(n))`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub tau_prime: Option<f64>,
    /// Sigma lower bound. Default: `1e-5`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma_min: Option<f64>,
    /// Sigma upper bound. Default: no upper bound.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sigma_max: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    ///
    /// The `step` parameter controls the maximum perturbation magnitude.
    /// Default when `None`: `0.01`.
    Creep(CreepParams),
    /// Gaussian (normal distribution) perturbation mutation for `Range<T>` chromosomes.
    ///
    /// The `sigma` parameter is the standard deviation of the Gaussian noise.
    /// Default when `None`: `0.1`.
    Gaussian(GaussianParams),
    /// Polynomial mutation for `Range<T>` chromosomes (NSGA-II style).
    ///
    /// The `eta` parameter is the distribution index (higher values → smaller perturbations).
    /// Typical range: 20–100. Default when `None`: `20.0`.
    Polynomial(PolynomialParams),
    /// Non-uniform mutation for `Range<T>` chromosomes.
    /// Mutation magnitude decreases over generations.
    ///
    /// The `b` parameter controls decay rate. Default when `None`: `2.0`.
    NonUniform(NonUniformParams),
    /// Permutation-insert mutation for permutation-based chromosomes.
    /// Removes a gene and reinserts it at a different position, preserving all alleles
    /// and chromosome length. This is the permutation-preserving insertion move (formerly
    /// `Mutation::Insertion` in previous versions).
    PermutationInsert,
    /// Insertion mutation for variable-length chromosomes.
    /// Inserts a new gene at a random position, growing the chromosome length by 1
    /// (clamped to the configured maximum). Requires
    /// [`ChromosomeLength::Variable`](crate::chromosomes::ChromosomeLength) in
    /// the engine configuration. Returns `GaError::MutationError` for
    /// `ChromosomeLength::Fixed`.
    Insertion,
    /// Deletion mutation for variable-length chromosomes.
    /// Removes a gene at a random position, shrinking the chromosome length by 1
    /// (clamped to the configured minimum). Requires
    /// [`ChromosomeLength::Variable`](crate::chromosomes::ChromosomeLength) in
    /// the engine configuration. Returns `GaError::MutationError` for
    /// `ChromosomeLength::Fixed`.
    Deletion,
    /// List-value mutation — replaces a single gene's value with a different allele
    /// from that gene's allele set. Requires a `ListChromosome<T>`.
    ListValue,
    /// DE-style differential mutation for `Range<T>` chromosomes.
    /// Computes mutant vector as `x_r1 + F * (x_r2 - x_r3)` from three distinct
    /// random population members (all distinct from the target), clamped to gene
    /// ranges. Default `f` when `None`: `0.5`.
    /// Requires `population_size >= 4`. Applied automatically by the standard GA
    /// engine when configured.
    Differential(DifferentialParams),
    /// Cauchy (Lorentzian) perturbation for `Range<T>` chromosomes.
    /// Uses the inverse-CDF method: `noise = scale * tan(π * (u - 0.5))`, where `u ~ Uniform(0, 1)`.
    /// Default `scale` when `None`: `1.0`.
    /// Returns `GaError::MutationError` for non-`Range<T>` chromosomes (Binary, List).
    Cauchy(CauchyParams),
    /// Lévy Flight mutation for `Range<T>` chromosomes (Mantegna's algorithm).
    /// Generates heavy-tailed steps via `step = σ_u * u / |v|^(1/α)`.
    /// Valid `alpha` range: (0.0, 2.0). Default `alpha` when `None`: `1.5`.
    /// Returns `GaError::MutationError` for non-`Range<T>` chromosomes.
    LevyFlight(LevyFlightParams),
    /// Uniform reset mutation for `Range<T>` chromosomes.
    /// Resets a single randomly chosen gene to a uniform sample within its declared range.
    /// Equivalent to gene re-initialization. No configuration parameters required.
    /// Returns `GaError::MutationError` for non-`Range<T>` chromosomes.
    Uniform,
    /// Self-adaptive Gaussian mutation for chromosomes implementing
    /// [`SelfAdaptive`](crate::traits::SelfAdaptive).
    ///
    /// Co-evolves per-dimension step-size parameters (σ) alongside gene values
    /// using the standard Evolution Strategy log-normal update:
    ///
    /// ```text
    /// σ'_i = σ_i × exp(τ' × N_global(0,1) + τ × N_i_local(0,1))
    /// ```
    ///
    /// After updating σ, one randomly-selected gene is mutated by `N(0, σ'_i)`.
    ///
    /// All parameters default to ES-standard values when `None`:
    /// - `tau`: `1.0 / sqrt(2.0 * n)`
    /// - `tau_prime`: `1.0 / sqrt(2.0 * sqrt(n))`
    /// - `sigma_min`: `1e-5`
    /// - `sigma_max`: no upper bound
    ///
    /// Returns `GaError::MutationError` for chromosomes not implementing `SelfAdaptive`.
    SelfAdaptiveGaussian(SelfAdaptiveGaussianParams),
}

/// Survivor-selection strategies.
///
/// Determines which individuals from the combined parent+offspring pool
/// survive into the next generation.
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Survivor;
/// use genetic_algorithms::traits::ConfigurationT;
///
/// let _ga = Ga::<Binary>::new()
///     .with_survivor_method(Survivor::Fitness);
/// ```
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
///
/// # Examples
///
/// ```rust,no_run
/// use genetic_algorithms::ga::Ga;
/// use genetic_algorithms::chromosomes::Binary;
/// use genetic_algorithms::operations::Extension;
/// use genetic_algorithms::traits::{ConfigurationT, ExtensionConfig};
///
/// let _ga = Ga::<Binary>::new()
///     .with_extension_method(Extension::MassExtinction)
///     .with_extension_diversity_threshold(0.01);
/// ```
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
