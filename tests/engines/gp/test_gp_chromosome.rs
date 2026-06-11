use genetic_algorithms::gp::{GpChromosome, GpGene, GpNode, Node, MathNode, TreeChromosome};
use genetic_algorithms::traits::{ChromosomeT, GeneT, VectorFitness};

fn add_tree() -> Box<Node<MathNode>> {
    Box::new(Node::Function {
        value: MathNode::Add,
        children: vec![
            Box::new(Node::Terminal(MathNode::Const(3.0))),
            Box::new(Node::Terminal(MathNode::Const(4.0))),
        ],
    })
}

#[test]
fn gp_gene_set_id_returns_self() {
    let mut g = GpGene;
    g.set_id(42);
    // GpGene is a zero-sized marker — set_id is a no-op, just verify it compiles
}

#[test]
fn gp_chromosome_with_root_and_tree_access() {
    let c = GpChromosome::with_root(add_tree());
    let tree = c.tree();
    assert!(matches!(tree, Node::Function { .. }));
}

#[test]
fn gp_chromosome_depth_and_node_count() {
    let c = GpChromosome::with_root(add_tree());
    assert_eq!(c.depth(), 2); // root at level 1, children at level 2
    assert_eq!(c.node_count(), 3); // root + 2 children
}

#[test]
fn gp_chromosome_tree_mut() {
    let mut c = GpChromosome::with_root(add_tree());
    let _tree_mut = c.tree_mut(); // just verify it returns a mutable ref
}

#[test]
fn gp_chromosome_fitness_and_set_fitness() {
    let mut c = GpChromosome::with_root(add_tree());
    assert!((c.fitness() - 0.0).abs() < 1e-10);
    c.set_fitness(42.0);
    assert!((c.fitness() - 42.0).abs() < 1e-10);
}

#[test]
fn gp_chromosome_age_and_set_age() {
    let mut c = GpChromosome::with_root(add_tree());
    assert_eq!(c.age(), 0);
    c.set_age(5);
    assert_eq!(c.age(), 5);
}

#[test]
fn gp_chromosome_calculate_fitness_with_fn() {
    let mut c = GpChromosome::with_root(add_tree());
    c.set_tree_fitness_fn(|node| {
        // evaluate add(3.0, 4.0) = 7.0
        if let Node::Function { value, children } = node {
            let args: Vec<f64> = children
                .iter()
                .map(|ch| if let Node::Terminal(MathNode::Const(v)) = &**ch { *v } else { 0.0 })
                .collect();
            value.evaluate(&args)
        } else {
            0.0
        }
    });
    c.calculate_fitness();
    assert!((c.fitness() - 7.0).abs() < 1e-10, "expected 7.0, got {}", c.fitness());
}

#[test]
fn gp_chromosome_calculate_fitness_no_fn_is_noop() {
    let mut c = GpChromosome::with_root(add_tree());
    c.calculate_fitness();
    assert!((c.fitness() - 0.0).abs() < 1e-10);
}

#[test]
fn gp_chromosome_vector_fitness() {
    let mut c = GpChromosome::with_root(add_tree());
    assert!(c.fitness_values().is_empty());
    c.set_fitness_values(vec![1.0, 2.0]);
    assert_eq!(c.fitness_values(), &[1.0, 2.0]);
}

#[test]
fn gp_chromosome_clone() {
    let c1 = GpChromosome::with_root(add_tree());
    let c2 = c1.clone();
    assert!((c1.fitness() - c2.fitness()).abs() < 1e-10);
    assert_eq!(c1.depth(), c2.depth());
}

#[test]
fn gp_chromosome_default() {
    let c: GpChromosome<MathNode> = GpChromosome::default();
    assert_eq!(c.node_count(), 1);
    assert_eq!(c.depth(), 1); // single terminal root is at depth 1
}

#[test]
fn gp_chromosome_display() {
    let c = GpChromosome::with_root(add_tree());
    let s = format!("{}", c);
    assert!(s.contains("add") || s.contains("3") || s.contains("4"),
        "Display output '{}' should contain tree content", s);
}

#[test]
#[should_panic(expected = "GpChromosome::dna()")]
fn gp_chromosome_dna_panics() {
    use genetic_algorithms::traits::LinearChromosome;
    let c = GpChromosome::with_root(add_tree());
    let _ = c.dna();
}

#[test]
#[should_panic(expected = "GpChromosome::dna_mut()")]
fn gp_chromosome_dna_mut_panics() {
    use genetic_algorithms::traits::LinearChromosome;
    let mut c = GpChromosome::with_root(add_tree());
    let _ = c.dna_mut();
}

#[test]
#[should_panic(expected = "GpChromosome::set_dna()")]
fn gp_chromosome_set_dna_panics() {
    use genetic_algorithms::traits::LinearChromosome;
    use std::borrow::Cow;
    let mut c = GpChromosome::with_root(add_tree());
    c.set_dna(Cow::Owned(vec![]));
}
