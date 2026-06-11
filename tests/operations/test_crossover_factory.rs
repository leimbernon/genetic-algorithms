//! Coverage tests for `src/operations/crossover.rs` factory functions and
//! `CrossoverOperator` trait dispatch.
//!
//! The module was at ~50% coverage because the factory dispatch paths for
//! SBX, BLX-α, Arithmetic, multi-parent UNDX/SPX/PCX, and the error paths
//! for unsupported types / wrong parent counts were never exercised.

use genetic_algorithms::chromosomes::Range as RangeChromosome;
use genetic_algorithms::configuration::CrossoverConfiguration;
use genetic_algorithms::error::GaError;
use genetic_algorithms::genotypes::Range as RangeGenotype;
use genetic_algorithms::operations::crossover::{factory, factory_multi_parent_dispatch};
use genetic_algorithms::operations::Crossover;
use genetic_algorithms::traits::{CrossoverOperator, LinearChromosome};
use std::borrow::Cow;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_f64_parent(values: &[f64]) -> RangeChromosome<f64> {
    let mut c = RangeChromosome::<f64>::new();
    let dna: Vec<RangeGenotype<f64>> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| RangeGenotype::new(i as i32, vec![(0.0, 100.0)], v))
        .collect();
    c.set_dna(Cow::Owned(dna));
    c
}

fn make_f64_parents() -> (RangeChromosome<f64>, RangeChromosome<f64>) {
    (
        make_f64_parent(&[10.0, 40.0, 70.0]),
        make_f64_parent(&[90.0, 60.0, 30.0]),
    )
}

// ─── CrossoverOperator impl on Crossover enum ────────────────────────────────

#[test]
fn crossover_operator_cycle_via_enum() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = Chromosome {
        dna: vec![Gene { id: 3 }, Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = Crossover::Cycle.crossover(&p1, &p2);
    assert!(result.is_ok(), "Cycle via enum should succeed");
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_operator_uniform_via_enum() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = Crossover::Uniform.crossover(&p1, &p2);
    assert!(result.is_ok(), "Uniform via enum should succeed");
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_operator_single_point_via_enum() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = Chromosome {
        dna: vec![Gene { id: 5 }, Gene { id: 6 }, Gene { id: 7 }, Gene { id: 8 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let result = Crossover::SinglePoint.crossover(&p1, &p2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_operator_multipoint_returns_error_via_enum() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = p1.clone();
    // MultiPoint via enum (no number_of_points) must return CrossoverError
    let result = Crossover::MultiPoint.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "MultiPoint without number_of_points should error, got: {:?}",
        result
    );
}

#[test]
fn crossover_operator_multipoint_via_configuration() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }, Gene { id: 4 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = Chromosome {
        dna: vec![Gene { id: 5 }, Gene { id: 6 }, Gene { id: 7 }, Gene { id: 8 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let config = CrossoverConfiguration {
        method: Crossover::MultiPoint,
        number_of_points: Some(2),
        ..CrossoverConfiguration::default()
    };
    let result = config.crossover(&p1, &p2);
    assert!(result.is_ok(), "MultiPoint via config should succeed, got: {:?}", result);
    assert_eq!(result.unwrap().len(), 2);
}

// ─── Sbx, BlendAlpha, Arithmetic via Crossover enum ─────────────────────────

#[test]
fn crossover_operator_sbx_via_enum() {
    let (p1, p2) = make_f64_parents();
    let result = Crossover::Sbx.crossover(&p1, &p2);
    assert!(result.is_ok(), "SBX via enum should succeed: {:?}", result);
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_operator_blend_alpha_via_enum() {
    let (p1, p2) = make_f64_parents();
    let result = Crossover::BlendAlpha.crossover(&p1, &p2);
    assert!(result.is_ok(), "BlendAlpha via enum should succeed: {:?}", result);
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_operator_arithmetic_via_enum() {
    let (p1, p2) = make_f64_parents();
    let result = Crossover::Arithmetic.crossover(&p1, &p2);
    assert!(result.is_ok(), "Arithmetic via enum should succeed: {:?}", result);
    assert_eq!(result.unwrap().len(), 2);
}

// ─── Sbx, BlendAlpha, Arithmetic error when non-Range chromosome ─────────────

#[test]
fn crossover_operator_sbx_error_non_range() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = p1.clone();
    let result = Crossover::Sbx.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "SBX with non-Range should error, got: {:?}",
        result
    );
}

#[test]
fn crossover_operator_blend_alpha_error_non_range() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = p1.clone();
    let result = Crossover::BlendAlpha.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "BlendAlpha with non-Range should error, got: {:?}",
        result
    );
}

#[test]
fn crossover_operator_arithmetic_error_non_range() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = p1.clone();
    let result = Crossover::Arithmetic.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Arithmetic with non-Range should error, got: {:?}",
        result
    );
}

// ─── Multi-parent variants via enum return error ──────────────────────────────

#[test]
fn crossover_operator_undx_via_enum_returns_error() {
    let (p1, p2) = make_f64_parents();
    let result = Crossover::Undx { num_parents: 3 }.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "UNDX via 2-parent path should error, got: {:?}",
        result
    );
}

#[test]
fn crossover_operator_spx_via_enum_returns_error() {
    let (p1, p2) = make_f64_parents();
    let result = Crossover::Spx { num_parents: 3 }.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "SPX via 2-parent path should error, got: {:?}",
        result
    );
}

#[test]
fn crossover_operator_pcx_via_enum_returns_error() {
    let (p1, p2) = make_f64_parents();
    let result = Crossover::Pcx { num_parents: 3 }.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "PCX via 2-parent path should error, got: {:?}",
        result
    );
}

// ─── CrossoverConfiguration operator dispatch ────────────────────────────────

#[test]
fn crossover_config_sbx_with_custom_eta() {
    let (p1, p2) = make_f64_parents();
    let config = CrossoverConfiguration {
        method: Crossover::Sbx,
        sbx_eta: Some(5.0),
        ..CrossoverConfiguration::default()
    };
    let result = config.crossover(&p1, &p2);
    assert!(result.is_ok(), "SBX config with eta=5.0 should succeed: {:?}", result);
    let children = result.unwrap();
    assert_eq!(children.len(), 2);
    // Children must stay within declared range [0.0, 100.0]
    for child in &children {
        for gene in child.dna() {
            let (lo, hi) = gene.ranges[0];
            assert!(
                gene.value >= lo && gene.value <= hi,
                "Gene value {} out of range [{}, {}]",
                gene.value,
                lo,
                hi
            );
        }
    }
}

#[test]
fn crossover_config_blend_alpha_with_custom_alpha() {
    let (p1, p2) = make_f64_parents();
    let config = CrossoverConfiguration {
        method: Crossover::BlendAlpha,
        blend_alpha: Some(0.3),
        ..CrossoverConfiguration::default()
    };
    let result = config.crossover(&p1, &p2);
    assert!(result.is_ok(), "BLX-alpha config should succeed: {:?}", result);
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_config_arithmetic_with_custom_alpha() {
    let (p1, p2) = make_f64_parents();
    let config = CrossoverConfiguration {
        method: Crossover::Arithmetic,
        arithmetic_alpha: Some(0.75),
        ..CrossoverConfiguration::default()
    };
    let result = config.crossover(&p1, &p2);
    assert!(result.is_ok(), "Arithmetic config should succeed: {:?}", result);
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn crossover_config_multipoint_missing_number_returns_error() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = p1.clone();
    let config = CrossoverConfiguration {
        method: Crossover::MultiPoint,
        number_of_points: None, // missing
        ..CrossoverConfiguration::default()
    };
    let result = config.crossover(&p1, &p2);
    assert!(
        result.is_err(),
        "MultiPoint config without number_of_points should error"
    );
}

// ─── factory() function ──────────────────────────────────────────────────────

#[test]
fn factory_single_point_via_free_fn() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let p1 = Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }, Gene { id: 3 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p2 = Chromosome {
        dna: vec![Gene { id: 4 }, Gene { id: 5 }, Gene { id: 6 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let config = CrossoverConfiguration {
        method: Crossover::SinglePoint,
        ..CrossoverConfiguration::default()
    };
    let result = factory(&p1, &p2, config);
    assert!(result.is_ok(), "factory() SinglePoint should succeed");
    assert_eq!(result.unwrap().len(), 2);
}

// ─── factory_multi_parent_dispatch() ─────────────────────────────────────────

#[test]
fn factory_multi_parent_dispatch_undx_f64() {
    let p1 = make_f64_parent(&[10.0, 20.0, 30.0]);
    let p2 = make_f64_parent(&[40.0, 50.0, 60.0]);
    let p3 = make_f64_parent(&[70.0, 80.0, 90.0]);
    let parents: Vec<&RangeChromosome<f64>> = vec![&p1, &p2, &p3];
    let config = CrossoverConfiguration {
        method: Crossover::Undx { num_parents: 3 },
        ..CrossoverConfiguration::default()
    };
    let result = factory_multi_parent_dispatch(&parents, config);
    assert!(
        result.is_ok(),
        "factory_multi_parent_dispatch UNDX should succeed: {:?}",
        result
    );
}

#[test]
fn factory_multi_parent_dispatch_spx_f64() {
    let p1 = make_f64_parent(&[10.0, 20.0]);
    let p2 = make_f64_parent(&[50.0, 60.0]);
    let p3 = make_f64_parent(&[80.0, 90.0]);
    let parents: Vec<&RangeChromosome<f64>> = vec![&p1, &p2, &p3];
    let config = CrossoverConfiguration {
        method: Crossover::Spx { num_parents: 3 },
        ..CrossoverConfiguration::default()
    };
    let result = factory_multi_parent_dispatch(&parents, config);
    assert!(
        result.is_ok(),
        "factory_multi_parent_dispatch SPX should succeed: {:?}",
        result
    );
}

#[test]
fn factory_multi_parent_dispatch_pcx_f64() {
    let p1 = make_f64_parent(&[10.0, 20.0, 30.0]);
    let p2 = make_f64_parent(&[40.0, 50.0, 60.0]);
    let p3 = make_f64_parent(&[70.0, 80.0, 90.0]);
    let parents: Vec<&RangeChromosome<f64>> = vec![&p1, &p2, &p3];
    let config = CrossoverConfiguration {
        method: Crossover::Pcx { num_parents: 3 },
        ..CrossoverConfiguration::default()
    };
    let result = factory_multi_parent_dispatch(&parents, config);
    assert!(
        result.is_ok(),
        "factory_multi_parent_dispatch PCX should succeed: {:?}",
        result
    );
}

#[test]
fn factory_multi_parent_dispatch_too_few_parents_errors() {
    let p1 = make_f64_parent(&[10.0, 20.0]);
    let p2 = make_f64_parent(&[50.0, 60.0]);
    let parents: Vec<&RangeChromosome<f64>> = vec![&p1, &p2]; // only 2 — need >= 3
    let config = CrossoverConfiguration {
        method: Crossover::Undx { num_parents: 3 },
        ..CrossoverConfiguration::default()
    };
    let result = factory_multi_parent_dispatch(&parents, config);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "< 3 parents should error, got: {:?}",
        result
    );
}

#[test]
fn factory_multi_parent_dispatch_non_multi_parent_method_errors() {
    let p1 = make_f64_parent(&[10.0]);
    let p2 = make_f64_parent(&[50.0]);
    let p3 = make_f64_parent(&[80.0]);
    let parents: Vec<&RangeChromosome<f64>> = vec![&p1, &p2, &p3];
    let config = CrossoverConfiguration {
        method: Crossover::SinglePoint,
        ..CrossoverConfiguration::default()
    };
    let result = factory_multi_parent_dispatch(&parents, config);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Non-multi-parent method should error, got: {:?}",
        result
    );
}

#[test]
fn factory_multi_parent_dispatch_non_range_errors() {
    use crate::structures::{Chromosome, Gene};
    use genetic_algorithms::fitness::FitnessFnWrapper;
    let make_chr = || Chromosome {
        dna: vec![Gene { id: 1 }, Gene { id: 2 }],
        fitness: 0.0,
        age: 0,
        fitness_values: vec![],
        fitness_fn: FitnessFnWrapper::default(),
    };
    let p1 = make_chr();
    let p2 = make_chr();
    let p3 = make_chr();
    let parents: Vec<&Chromosome> = vec![&p1, &p2, &p3];
    let config = CrossoverConfiguration {
        method: Crossover::Undx { num_parents: 3 },
        ..CrossoverConfiguration::default()
    };
    let result = factory_multi_parent_dispatch(&parents, config);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "Non-Range with UNDX should error, got: {:?}",
        result
    );
}

// ─── CrossoverConfiguration multi-parent undx/spx/pcx via impl ───────────────

#[test]
fn crossover_config_undx_via_enum_returns_error() {
    let (p1, p2) = make_f64_parents();
    let config = CrossoverConfiguration {
        method: Crossover::Undx { num_parents: 3 },
        ..CrossoverConfiguration::default()
    };
    let result = config.crossover(&p1, &p2);
    assert!(
        matches!(result, Err(GaError::CrossoverError(_))),
        "CrossoverConfiguration Undx via 2-parent crossover should error"
    );
}
