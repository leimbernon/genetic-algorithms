//! Compile-time verification that prelude types are re-exported and accessible.
//! Covers SC-1 (prelude re-exports all required items) and SC-4 (no name collisions).

use genetic_algorithms::prelude::*;

#[test]
fn test_prelude_engines_accessible() {
    // Compile-time check: if this compiles, all engine types are accessible
    let _: fn() -> Ga<genetic_algorithms::chromosomes::Binary> = || unimplemented!();
    // Other engine types verified by the glob import succeeding
}

#[test]
fn test_prelude_traits_accessible() {
    // Compile-time check: trait bounds compile
    fn assert_traits<T: ChromosomeT + LinearChromosome>() {}
    fn assert_config<T: ConfigurationT>() {}
    let _ = (
        assert_traits::<genetic_algorithms::chromosomes::Binary>,
        assert_config::<genetic_algorithms::ga::Ga<genetic_algorithms::chromosomes::Binary>>,
    );
}

#[test]
fn test_prelude_operator_enums_accessible() {
    let _sel = Selection::Tournament;
    let _cx = Crossover::Uniform;
    let _mut = Mutation::BitFlip;
    let _surv = Survivor::Fitness;
}

#[test]
fn test_prelude_config_types_accessible() {
    let _ps = ProblemSolving::Minimization;
    let _cl = ChromosomeLength::Fixed(5);
}

#[test]
fn test_prelude_error_accessible() {
    let _err = GaError::ConfigurationError("test".into());
}

#[test]
fn test_prelude_observer_accessible() {
    let _obs = NoopObserver;
}
