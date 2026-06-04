//! Tests for `MultiRangeGenotype<T>` — GeneT impl, per-gene bounds/rate fields,
//! constructor, accessors, Default, Clone.
//!
//! Covers GEN-03: MultiRangeGenotype flat-fielded gene struct.

use genetic_algorithms::genotypes::MultiRangeGenotype;
use genetic_algorithms::traits::GeneT;

// ─── Constructor and accessors ───────────────────────────────────────────────

#[test]
fn multi_range_genotype_new_id_accessor() {
    let gene = MultiRangeGenotype::new(3, -5.0_f64, 5.0, 0.0, 0.1);
    assert_eq!(gene.id(), 3);
}

#[test]
fn multi_range_genotype_new_lo_hi_fields() {
    let gene = MultiRangeGenotype::new(0, -5.0_f64, 5.0, 1.0, 0.1);
    assert_eq!(gene.lo, -5.0);
    assert_eq!(gene.hi, 5.0);
}

#[test]
fn multi_range_genotype_new_value_accessor() {
    let gene = MultiRangeGenotype::new(0, 0.0_f64, 1.0, 0.75, 0.1);
    assert_eq!(gene.value(), 0.75);
}

#[test]
fn multi_range_genotype_new_mutation_rate_field() {
    let gene = MultiRangeGenotype::new(0, 0.0_f64, 1.0, 0.5, 0.42);
    assert_eq!(gene.mutation_rate, 0.42);
}

// ─── GeneT impl ──────────────────────────────────────────────────────────────

#[test]
fn multi_range_genotype_set_id_mutates_and_returns_self() {
    let mut gene = MultiRangeGenotype::new(0_i32, 0_i32, 10_i32, 5_i32, 0.1);
    let result = gene.set_id(7);
    assert_eq!(result.id(), 7);
    assert_eq!(gene.id(), 7);
}

#[test]
fn multi_range_genotype_set_value_mutates() {
    let mut gene = MultiRangeGenotype::new(0, 0.0_f64, 1.0, 0.0, 0.1);
    gene.set_value(0.9);
    assert_eq!(gene.value(), 0.9);
}

// ─── Default impl ────────────────────────────────────────────────────────────

#[test]
fn multi_range_genotype_default_all_zero() {
    let gene = <MultiRangeGenotype<f64> as Default>::default();
    assert_eq!(gene.id, 0);
    assert_eq!(gene.lo, 0.0);
    assert_eq!(gene.hi, 0.0);
    assert_eq!(gene.value, 0.0);
    assert_eq!(gene.mutation_rate, 0.0);
}

#[test]
fn multi_range_genotype_default_i32() {
    let gene = <MultiRangeGenotype<i32> as Default>::default();
    assert_eq!(gene.id, 0);
    assert_eq!(gene.lo, 0);
    assert_eq!(gene.hi, 0);
    assert_eq!(gene.value, 0);
    assert_eq!(gene.mutation_rate, 0.0);
}

// ─── Clone ───────────────────────────────────────────────────────────────────

#[test]
fn multi_range_genotype_clone_preserves_all_five_fields() {
    let gene = MultiRangeGenotype::new(7, -1.0_f64, 1.0, 0.5, 0.25);
    let cloned = gene.clone();
    assert_eq!(cloned.id, gene.id);
    assert_eq!(cloned.lo, gene.lo);
    assert_eq!(cloned.hi, gene.hi);
    assert_eq!(cloned.value, gene.value);
    assert_eq!(cloned.mutation_rate, gene.mutation_rate);
}

// ─── No Arc field — struct literal compiles with only flat fields ─────────────

#[test]
fn multi_range_genotype_struct_literal_flat_fields_only() {
    // This test ensures the struct has exactly the five expected flat fields.
    // If `Arc` were present this would need to be different.
    let _gene = MultiRangeGenotype {
        id: 1_i32,
        lo: 0.0_f64,
        hi: 10.0_f64,
        value: 5.0_f64,
        mutation_rate: 0.05_f64,
    };
}
