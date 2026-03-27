use genetic_algorithms::genotypes::List;
use genetic_algorithms::traits::GeneT;
use genetic_algorithms::error::GaError;

// ── List::new ────────────────────────────────────────────────────────────

#[test]
fn list_gene_new_valid_id_zero() {
    let gene = List::new(0, vec!['a', 'b', 'c'], 'z').unwrap();
    assert_eq!(gene.id(), 0);
    assert_eq!(gene.value(), 'a'); // value derived from alleles[0], not passed 'z'
    assert_eq!(gene.alleles, vec!['a', 'b', 'c']);
}

#[test]
fn list_gene_new_valid_id_nonzero() {
    let gene = List::new(2, vec!['a', 'b', 'c'], 'a').unwrap();
    assert_eq!(gene.id(), 2);
    assert_eq!(gene.value(), 'c'); // alleles[2]
}

#[test]
fn list_gene_new_id_out_of_bounds() {
    let result = List::new(3, vec!['a', 'b', 'c'], 'a');
    assert!(matches!(result, Err(GaError::ValidationError(_))));
}

#[test]
fn list_gene_new_negative_id() {
    let result = List::new(-1, vec!['a', 'b', 'c'], 'a');
    assert!(matches!(result, Err(GaError::ValidationError(_))));
}

#[test]
fn list_gene_new_empty_alleles() {
    let result = List::new(0, vec![], 'a');
    match result {
        Err(GaError::ValidationError(msg)) => {
            assert!(msg.contains("empty"), "message was: {}", msg)
        }
        _ => panic!("expected ValidationError for empty alleles"),
    }
}

// ── GeneT::id ────────────────────────────────────────────────────────────

#[test]
fn list_gene_id_returns_stored_id() {
    let gene = List::new(1, vec!['x', 'y', 'z'], 'x').unwrap();
    assert_eq!(gene.id(), 1);
}

// ── GeneT::set_id ────────────────────────────────────────────────────────

#[test]
fn list_gene_set_id_updates_value() {
    let mut gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
    gene.set_id(1);
    assert_eq!(gene.id(), 1);
    assert_eq!(gene.value(), 'b');
}

#[test]
fn list_gene_set_id_out_of_bounds_ignored() {
    let mut gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
    gene.set_id(99); // out of bounds — should be silently ignored
    assert_eq!(gene.id(), 0);
    assert_eq!(gene.value(), 'a');
}

// ── Default ──────────────────────────────────────────────────────────────

#[test]
fn list_gene_default() {
    let gene: List<char> = Default::default();
    assert_eq!(gene.id, 0);
    assert!(gene.alleles.is_empty());
    assert_eq!(gene.value, char::default());
}

// ── Clone ────────────────────────────────────────────────────────────────

#[test]
fn list_gene_clone_is_independent() {
    let gene = List::new(0, vec!['a', 'b', 'c'], 'a').unwrap();
    let mut clone = gene.clone();
    clone.alleles.push('d');
    assert_eq!(gene.alleles.len(), 3); // original unchanged
}

// ── serde (feature-gated) ────────────────────────────────────────────────

#[cfg(feature = "serde")]
#[test]
fn list_gene_serde_roundtrip() {
    let gene = List::new(1, vec!['a', 'b', 'c'], 'a').unwrap();
    let json = serde_json::to_string(&gene).expect("serialize");
    let restored: List<char> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.id, 1);
    assert_eq!(restored.alleles, vec!['a', 'b', 'c']);
    assert_eq!(restored.value, 'b');
}
