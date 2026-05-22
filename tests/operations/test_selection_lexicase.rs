//! Tests for the Lexicase and Epsilon-Lexicase selection operators.

#[allow(unused_imports)]
use crate::structures::{Gene, MultiCaseChromosome};
#[allow(unused_imports)]
use genetic_algorithms::{
    configuration::SelectionConfiguration, fitness::FitnessFnWrapper, operations::Selection,
    traits::MultiCaseFitness,
};

#[allow(dead_code)]
fn make_multi_case_chromosome(case_scores: Vec<f64>, dna: Vec<Gene>) -> MultiCaseChromosome {
    let mean = if case_scores.is_empty() {
        0.0
    } else {
        case_scores.iter().sum::<f64>() / case_scores.len() as f64
    };
    let mut c = MultiCaseChromosome {
        dna,
        fitness: mean,
        age: 0,
        case_scores: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    c.set_case_fitness(case_scores);
    c
}

#[allow(dead_code)]
fn pop_with_cases(case_score_matrix: &[Vec<f64>]) -> Vec<MultiCaseChromosome> {
    case_score_matrix
        .iter()
        .map(|scores| make_multi_case_chromosome(scores.clone(), vec![Gene { id: 1 }]))
        .collect()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_lexicase_returns_correct_couple_count() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_lexicase_case_order_is_shuffled() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_lexicase_syncs_scalar_fitness_to_mean() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_factory_rejects_lexicase() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_factory_rejects_epsilon_lexicase() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_epsilon_lexicase_fixed_tolerance() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_epsilon_lexicase_dynamic_mad() {
    unimplemented!()
}

#[test]
#[ignore = "Wave 0 stub — implemented in Plan 02"]
fn test_multi_case_fitness_trait_roundtrip() {
    unimplemented!()
}
