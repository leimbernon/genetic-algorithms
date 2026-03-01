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
}
#[derive(Copy, Clone, PartialEq)]
pub enum Crossover {
    Cycle,
    MultiPoint,
    Uniform,
    SinglePoint,
    Order,
}
#[derive(Copy, Clone)]
pub enum Mutation {
    Swap,
    Inversion,
    Scramble,
    Value,
    BitFlip,
}
#[derive(Copy, Clone)]
pub enum Survivor {
    Fitness,
    Age,
}
