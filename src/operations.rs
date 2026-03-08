pub mod crossover;
pub mod mutation;
pub mod selection;
pub mod survivor;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Selection {
    Random,
    RouletteWheel,
    StochasticUniversalSampling,
    Tournament,
    /// Rank-based selection: individuals are ranked by fitness and selection
    /// probability is proportional to rank, avoiding dominance by very fit individuals.
    Rank,
    /// Boltzmann selection: uses a temperature parameter to control selective pressure.
    /// High temperature → uniform selection (exploration), low temperature → strong
    /// selective pressure (exploitation).
    Boltzmann,
    /// Truncation selection: only the top portion of the population is eligible
    /// for reproduction, providing very high selective pressure.
    Truncation,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Crossover {
    Cycle,
    MultiPoint,
    Uniform,
    SinglePoint,
    Order,
    /// Partially Mapped Crossover for permutation-based chromosomes.
    /// Preserves absolute positions within a segment and relative order outside it.
    Pmx,
    /// Simulated Binary Crossover for Range<T> chromosomes.
    /// Uses a distribution index (eta) configured via `CrossoverConfiguration`.
    Sbx,
    /// Blend Crossover (BLX-α) for Range<T> chromosomes.
    /// Uses an alpha parameter configured via `CrossoverConfiguration`.
    BlendAlpha,
    /// Arithmetic (whole) crossover for Range<T> chromosomes.
    /// Child = α·parent1 + (1-α)·parent2. Uses `arithmetic_alpha` from configuration.
    Arithmetic,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mutation {
    Swap,
    Inversion,
    Scramble,
    Value,
    BitFlip,
    /// Small uniform perturbation mutation for Range<T> chromosomes.
    /// Requires a step size configured via `MutationConfiguration`.
    Creep,
    /// Gaussian (normal distribution) perturbation mutation for Range<T> chromosomes.
    /// Requires a sigma configured via `MutationConfiguration`.
    Gaussian,
    /// Polynomial mutation for Range<T> chromosomes (NSGA-II style).
    /// Uses a distribution index (eta_m) from `MutationConfiguration`.
    Polynomial,
    /// Non-uniform mutation for Range<T> chromosomes.
    /// Mutation magnitude decreases over generations.
    NonUniform,
    /// Insertion mutation for permutation-based chromosomes.
    /// Removes a gene and reinserts it at a different position.
    Insertion,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Survivor {
    Fitness,
    Age,
    /// (μ+λ) strategy: parents and offspring compete together for survival.
    MuPlusLambda,
    /// (μ,λ) strategy: only offspring (age == 0) are eligible for survival.
    MuCommaLambda,
}
