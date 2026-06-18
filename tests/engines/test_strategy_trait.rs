//! Integration tests for the Strategy<U> trait — STR-01.

use std::borrow::Cow;

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::ProblemSolving;
use genetic_algorithms::ga::Ga;
use genetic_algorithms::genotypes::Range as RangeGene;
use genetic_algorithms::operations::{Crossover, GaussianParams, Mutation, Selection, Survivor};
use genetic_algorithms::population::Population;
use genetic_algorithms::traits::{
    ChromosomeT, ConfigurationT, CrossoverConfig, LinearChromosome, MutationConfig,
    SelectionConfig, StoppingConfig,
};
use genetic_algorithms::Strategy;
use genetic_algorithms::{HillClimbConfiguration, HillClimbEngine, HillClimbMode};
use genetic_algorithms::{PermutateConfiguration, PermutateEngine};

fn sphere(dna: &[RangeGene<f64>]) -> f64 {
    dna.iter().map(|g| g.value() * g.value()).sum()
}

fn make_candidate(value: f64) -> RangeChromosome<f64> {
    let gene = RangeGene::new(0, vec![(-10.0, 10.0)], value);
    let mut c = <RangeChromosome<f64> as Default>::default();
    c.set_dna(Cow::Owned(vec![gene]));
    c.set_fitness(value * value);
    c
}

fn make_ga_population(n: usize) -> Vec<RangeChromosome<f64>> {
    (0..n)
        .map(|i| {
            let val = (i as f64) - (n as f64 / 2.0);
            let gene = RangeGene::new(0, vec![(-10.0, 10.0)], val);
            let mut c = <RangeChromosome<f64> as Default>::default();
            c.set_dna(Cow::Owned(vec![gene.clone(), gene.clone()]));
            c
        })
        .collect()
}

#[test]
fn test_strategy_box_dyn_compiles() {
    let chromosomes = make_ga_population(5);
    let population = Population::new(chromosomes);

    let mut ga: Box<dyn Strategy<RangeChromosome<f64>>> = Box::new(
        Ga::new()
            .with_problem_solving(ProblemSolving::Minimization)
            .with_selection_method(Selection::Random)
            .with_crossover_method(Crossover::SinglePoint)
            .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
            .with_survivor_method(Survivor::Fitness)
            .with_max_generations(10)
            .with_population(population)
            .with_fitness_fn(sphere),
    );

    let result = ga.run();
    assert!(
        result.is_ok(),
        "Ga via Box<dyn Strategy> must succeed: {:?}",
        result.err()
    );
    assert!(ga.best().is_some(), "best() must return Some after run");
}

#[test]
fn test_box_dyn_strategy_hill_climb_compiles() {
    let initial = make_candidate(5.0);
    let config = HillClimbConfiguration::default();

    let mut engine: Box<dyn Strategy<RangeChromosome<f64>>> =
        Box::new(HillClimbEngine::new(config, initial, |c| {
            let val = c.dna()[0].value();
            let lo = -10.0;
            let hi = 10.0;
            let mut n1 = <RangeChromosome<f64>>::default();
            n1.set_dna(Cow::Owned(vec![RangeGene::new(
                0,
                vec![(lo, hi)],
                val - 0.1,
            )]));
            n1.set_fitness((val - 0.1) * (val - 0.1));
            let mut n2 = <RangeChromosome<f64>>::default();
            n2.set_dna(Cow::Owned(vec![RangeGene::new(
                0,
                vec![(lo, hi)],
                val + 0.1,
            )]));
            n2.set_fitness((val + 0.1) * (val + 0.1));
            vec![n1, n2]
        }));

    let result = engine.run();
    assert!(
        result.is_ok(),
        "HillClimbEngine via Box<dyn Strategy> must succeed: {:?}",
        result.err()
    );
    assert!(engine.best().is_some(), "best() must return Some after run");
}

#[test]
fn test_box_dyn_strategy_permutate_compiles() {
    let candidates = vec![
        make_candidate(3.0),
        make_candidate(1.0),
        make_candidate(2.0),
    ];
    let config = PermutateConfiguration::default();

    let mut engine: Box<dyn Strategy<RangeChromosome<f64>>> =
        Box::new(PermutateEngine::new(config, candidates));

    let result = engine.run();
    assert!(
        result.is_ok(),
        "PermutateEngine via Box<dyn Strategy> must succeed: {:?}",
        result.err()
    );
    let best = engine.best();
    assert!(best.is_some(), "best() must return Some after run");
    assert_eq!(
        best.unwrap().fitness(),
        1.0,
        "minimization should pick the candidate with fitness 1.0"
    );
}

#[test]
fn test_runtime_strategy_swap() {
    // Build a GA
    let chromosomes = make_ga_population(5);
    let population = Population::new(chromosomes);
    let ga: Box<dyn Strategy<RangeChromosome<f64>>> = Box::new(
        Ga::new()
            .with_problem_solving(ProblemSolving::Minimization)
            .with_selection_method(Selection::Random)
            .with_crossover_method(Crossover::SinglePoint)
            .with_mutation_method(Mutation::Gaussian(GaussianParams { sigma: None }))
            .with_survivor_method(Survivor::Fitness)
            .with_max_generations(5)
            .with_population(population)
            .with_fitness_fn(sphere),
    );

    // Build a HillClimbEngine
    let initial = make_candidate(3.0);
    let hill_climb: Box<dyn Strategy<RangeChromosome<f64>>> = Box::new(HillClimbEngine::new(
        HillClimbConfiguration::default()
            .with_mode(HillClimbMode::Stochastic)
            .with_no_improvement_limit(5),
        initial,
        |c| {
            let val = c.dna()[0].value();
            let lo = -10.0;
            let hi = 10.0;
            let mut n1 = <RangeChromosome<f64>>::default();
            n1.set_dna(Cow::Owned(vec![RangeGene::new(
                0,
                vec![(lo, hi)],
                val - 0.1,
            )]));
            n1.set_fitness((val - 0.1) * (val - 0.1));
            vec![n1]
        },
    ));

    let mut strategies: Vec<Box<dyn Strategy<RangeChromosome<f64>>>> = vec![ga, hill_climb];

    for strategy in strategies.iter_mut() {
        let result = strategy.run();
        assert!(
            result.is_ok(),
            "strategy.run() must succeed: {:?}",
            result.err()
        );
        assert!(
            strategy.best().is_some(),
            "best() must return Some after run"
        );
    }
}
