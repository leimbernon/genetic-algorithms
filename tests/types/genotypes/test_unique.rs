//! Tests for `UniqueGenotype<T>` — GEN-01: GeneT impl for the unique permutation gene type.
//!
//! Covers: id/set_id behavior, value access, Default, Display.

use genetic_algorithms::genotypes::UniqueGenotype;
use genetic_algorithms::traits::GeneT;

/// UniqueGenotype::new sets id and value correctly.
#[test]
fn unique_genotype_new_id() {
    let gene = UniqueGenotype::new(7, 42i32);
    assert_eq!(gene.id(), 7, "id() should return the id passed to new()");
}

/// GeneT::set_id mutates in place and returns &mut Self.
#[test]
fn unique_genotype_set_id_mutates() {
    let mut gene = UniqueGenotype::new(0, 10i32);
    let ret = gene.set_id(5);
    assert_eq!(ret.id(), 5, "set_id should mutate id and return &mut Self");
    assert_eq!(gene.id(), 5, "id should be updated after set_id");
}

/// Default returns id: 0, value: T::default().
#[test]
fn unique_genotype_default() {
    let gene = <UniqueGenotype<i32> as Default>::default();
    assert_eq!(gene.id, 0, "Default id should be 0");
    assert_eq!(gene.value, 0i32, "Default value should be T::default()");
}

/// Display formats as "{id}:{value}".
#[test]
fn unique_genotype_display() {
    let gene = UniqueGenotype::new(3, 99i32);
    let s = format!("{}", gene);
    assert_eq!(s, "3:99", "Display should format as 'id:value'");
}

/// value() clones and returns the current value.
#[test]
fn unique_genotype_value_clone() {
    let gene = UniqueGenotype::new(0, 42i32);
    assert_eq!(gene.value(), 42);
}

/// set_value updates the value and returns &mut Self.
#[test]
fn unique_genotype_set_value_mutates() {
    let mut gene = UniqueGenotype::new(0, 1i32);
    gene.set_value(99);
    assert_eq!(gene.value, 99, "value should be updated after set_value");
}

/// UniqueGenotype does NOT have an Arc<[T]> field (alphabet lives on chromosome).
/// This is a compile-time check — if the struct had an alphabet field, the code
/// accessing gene.value would also need to reference gene.alphabet. The test
/// exercises the struct without any such field.
#[test]
fn unique_genotype_has_no_alphabet_field() {
    let gene: UniqueGenotype<i32> = UniqueGenotype { id: 1, value: 42 };
    // If alphabet were a field, this struct literal would not compile.
    assert_eq!(gene.id, 1);
    assert_eq!(gene.value, 42);
}
