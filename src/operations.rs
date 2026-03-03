pub mod crossover;
pub mod mutation;
pub mod selection;
pub mod survivor;

#[derive(Copy, Clone)]
pub enum Selection {
    Random,
    RouletteWheel,
    StochasticUniversalSampling,
    Tournament,
    /// Rank-based selection: individuals are ranked by fitness and selection
    /// probability is proportional to rank, avoiding dominance by very fit individuals.
    Rank,
}
#[derive(Copy, Clone, PartialEq)]
pub enum Crossover {
    Cycle,
    MultiPoint,
    Uniform,
    SinglePoint,
    Order,
    /// Simulated Binary Crossover for Range<T> chromosomes.
    /// Uses a distribution index (eta) configured via `CrossoverConfiguration`.
    Sbx,
    /// Blend Crossover (BLX-α) for Range<T> chromosomes.
    /// Uses an alpha parameter configured via `CrossoverConfiguration`.
    BlendAlpha,
}
#[derive(Copy, Clone)]
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
}
#[derive(Copy, Clone)]
pub enum Survivor {
    Fitness,
    Age,
}
