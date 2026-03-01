pub mod selection;
pub mod crossover;
pub mod mutation;
pub mod survivor;

#[derive(Copy, Clone)]
pub enum Selection {
    Random,
    RouletteWheel,
    StochasticUniversalSampling,
    Tournament
}
#[derive(Copy, Clone, PartialEq)]
pub enum Crossover {
    Cycle,
    MultiPoint,
    Uniform,
}
#[derive(Copy, Clone)]
pub enum Mutation {
    Swap,
    Inversion,
    Scramble,
    Value,
}
#[derive(Copy, Clone)]
pub enum Survivor {
    Fitness,
    Age,
}