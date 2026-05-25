//! GP subsystem tests — Waves 0–3.
//!
//! Non-ignored tests validate the core API contracts (GpNode, Node<N>,
//! GpChromosome, TreeChromosome, MathNode, BoolNode) and GP operators
//! (SubtreeCrossover, PointMutation, HoistMutation, bloat limits).

use genetic_algorithms::error::GaError;
use genetic_algorithms::gp::{
    BoolNode, GpChromosome, GpCrossover, GpMutation, GpNode, MathNode, Node, TreeChromosome,
};
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::fmt;

// ---------------------------------------------------------------------------
// TestNode — minimal inline GP primitive for core API tests
// ---------------------------------------------------------------------------

/// A minimal 4-variant GpNode used for type-level tests in this file.
#[derive(Clone, Debug)]
enum TestNode {
    Add,
    Mul,
    X,
    Const(i32),
}

impl fmt::Display for TestNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestNode::Add => write!(f, "add"),
            TestNode::Mul => write!(f, "mul"),
            TestNode::X => write!(f, "x"),
            TestNode::Const(n) => write!(f, "{}", n),
        }
    }
}

impl Default for TestNode {
    fn default() -> Self {
        TestNode::X
    }
}

impl GpNode for TestNode {
    fn arity(&self) -> usize {
        match self {
            TestNode::Add | TestNode::Mul => 2,
            TestNode::X | TestNode::Const(_) => 0,
        }
    }

    fn evaluate(&self, args: &[f64]) -> f64 {
        match self {
            TestNode::Add => args[0] + args[1],
            TestNode::Mul => args[0] * args[1],
            TestNode::X => 0.0,
            TestNode::Const(n) => *n as f64,
        }
    }

    fn sample_random_terminal(_rng: &mut impl rand::Rng) -> Self {
        TestNode::X
    }

    fn all_functions() -> Vec<Self> {
        vec![TestNode::Add, TestNode::Mul]
    }
}

// ---------------------------------------------------------------------------
// Non-ignored Wave 0 tests
// ---------------------------------------------------------------------------

#[test]
fn test_gp_node_trait() {
    // arity and is_terminal
    assert_eq!(TestNode::Add.arity(), 2);
    assert_eq!(TestNode::Mul.arity(), 2);
    assert_eq!(TestNode::X.arity(), 0);
    assert_eq!(TestNode::Const(42).arity(), 0);

    assert!(!TestNode::Add.is_terminal());
    assert!(!TestNode::Mul.is_terminal());
    assert!(TestNode::X.is_terminal());
    assert!(TestNode::Const(7).is_terminal());

    // all_functions returns both non-terminals
    let fns = TestNode::all_functions();
    assert_eq!(fns.len(), 2);
}

#[test]
fn test_tree_chromosome_not_linear() {
    let chr: GpChromosome<TestNode> = GpChromosome::with_root(Box::new(Node::Terminal(TestNode::X)));
    assert_eq!(chr.depth(), 1);
    assert_eq!(chr.node_count(), 1);
}

#[test]
fn test_node_drop_iterative() {
    // Build a right-spine tree of depth 10 — each level is a Function with one
    // Terminal child and one recursive child.
    let mut root: Box<Node<TestNode>> = Box::new(Node::Terminal(TestNode::Const(0)));
    for _ in 0..10 {
        root = Box::new(Node::Function {
            value: TestNode::Add,
            children: vec![Box::new(Node::Terminal(TestNode::X)), root],
        });
    }
    // Dropping this tree should not overflow the stack.
    drop(root);
    // If we reach here the iterative Drop worked correctly.
}

#[test]
fn test_display_prefix_sexpr() {
    // Build: (add x x)
    let root = Box::new(Node::Function {
        value: TestNode::Add,
        children: vec![
            Box::new(Node::Terminal(TestNode::X)),
            Box::new(Node::Terminal(TestNode::X)),
        ],
    });
    let chr = GpChromosome::with_root(root);
    let s = chr.to_string();
    assert!(s.starts_with('('), "expected S-expr to start with '(', got: {}", s);
}

#[test]
fn test_display_nested() {
    // Build the tree: (add (mul x 3))
    // Structure: Function(Add, [Function(Mul, [Terminal(X), Terminal(Const(3))]) ])
    let inner = Box::new(Node::Function {
        value: TestNode::Mul,
        children: vec![
            Box::new(Node::Terminal(TestNode::X)),
            Box::new(Node::Terminal(TestNode::Const(3))),
        ],
    });
    let root = Box::new(Node::Function {
        value: TestNode::Add,
        children: vec![inner],
    });
    let chr = GpChromosome::with_root(root);
    assert_eq!(chr.to_string(), "(add (mul x 3))");
}

// ---------------------------------------------------------------------------
// MathNode tests (non-ignored)
// ---------------------------------------------------------------------------

#[test]
fn test_math_node_gp_node_impl() {
    let mut rng = SmallRng::seed_from_u64(42);

    // Function nodes
    assert_eq!(MathNode::Add.arity(), 2);
    assert!(!MathNode::Add.is_terminal());

    // Terminal nodes
    assert_eq!(MathNode::Const(3.14).arity(), 0);
    assert!(MathNode::Const(3.14).is_terminal());
    assert_eq!(MathNode::Var(0).arity(), 0);
    assert!(MathNode::Var(0).is_terminal());

    // all_functions returns exactly 4 variants
    let fns = MathNode::all_functions();
    assert_eq!(fns.len(), 4);

    // sample_random_terminal produces a terminal
    let t = MathNode::sample_random_terminal(&mut rng);
    assert!(t.is_terminal(), "sample_random_terminal must return a terminal");

    // ProtectedDiv returns 1.0 on zero denominator
    assert_eq!(MathNode::ProtectedDiv.evaluate(&[5.0, 0.0]), 1.0);
    assert!((MathNode::ProtectedDiv.evaluate(&[6.0, 2.0]) - 3.0).abs() < 1e-10);

    // evaluate
    assert_eq!(MathNode::Add.evaluate(&[1.0, 2.0]), 3.0);
    assert_eq!(MathNode::Sub.evaluate(&[5.0, 3.0]), 2.0);
    assert_eq!(MathNode::Mul.evaluate(&[3.0, 4.0]), 12.0);
    assert_eq!(MathNode::Const(7.0).evaluate(&[]), 7.0);
}

#[test]
fn test_bool_node_gp_node_impl() {
    // Arity
    assert_eq!(BoolNode::And.arity(), 2);
    assert_eq!(BoolNode::Or.arity(), 2);
    assert_eq!(BoolNode::Not.arity(), 1);
    assert_eq!(BoolNode::Gt.arity(), 2);
    assert_eq!(BoolNode::Lt.arity(), 2);

    // all_functions returns all 5 variants
    let fns = BoolNode::all_functions();
    assert_eq!(fns.len(), 5);

    // evaluate
    assert_eq!(BoolNode::And.evaluate(&[1.0, 1.0]), 1.0);
    assert_eq!(BoolNode::And.evaluate(&[1.0, 0.0]), 0.0);
    assert_eq!(BoolNode::Or.evaluate(&[0.0, 0.0]), 0.0);
    assert_eq!(BoolNode::Or.evaluate(&[1.0, 0.0]), 1.0);
    assert_eq!(BoolNode::Not.evaluate(&[0.0]), 1.0);
    assert_eq!(BoolNode::Not.evaluate(&[1.0]), 0.0);
    assert_eq!(BoolNode::Gt.evaluate(&[2.0, 1.0]), 1.0);
    assert_eq!(BoolNode::Gt.evaluate(&[1.0, 2.0]), 0.0);
    assert_eq!(BoolNode::Lt.evaluate(&[1.0, 2.0]), 1.0);
    assert_eq!(BoolNode::Lt.evaluate(&[2.0, 1.0]), 0.0);
}

// ---------------------------------------------------------------------------
// Helper: build a balanced tree of given depth using TestNode
// ---------------------------------------------------------------------------

/// Builds a balanced tree of the given depth using TestNode::Add as the
/// function node and TestNode::X as terminals.
fn build_tree(depth: usize) -> Box<Node<TestNode>> {
    if depth <= 1 {
        Box::new(Node::Terminal(TestNode::X))
    } else {
        Box::new(Node::Function {
            value: TestNode::Add,
            children: vec![build_tree(depth - 1), build_tree(depth - 1)],
        })
    }
}

// ---------------------------------------------------------------------------
// Wave 1 operator tests
// ---------------------------------------------------------------------------

#[test]
fn test_subtree_crossover() {
    let mut rng = SmallRng::seed_from_u64(42);
    // Build two depth-2 trees (3 nodes each)
    let p1 = GpChromosome::with_root(build_tree(2));
    let p2 = GpChromosome::with_root(build_tree(2));

    let result = GpCrossover::SubtreeCrossover.apply(&p1, &p2, 10, 100, &mut rng);
    assert!(result.is_ok(), "Expected Ok from crossover, got: {:?}", result);
    let (c1, c2): (GpChromosome<TestNode>, GpChromosome<TestNode>) = result.unwrap();

    // Both children must respect the limits
    assert!(c1.depth() <= 10, "child1 depth {} exceeds limit", c1.depth());
    assert!(c2.depth() <= 10, "child2 depth {} exceeds limit", c2.depth());
    assert!(c1.node_count() <= 100, "child1 node_count {} exceeds limit", c1.node_count());
    assert!(c2.node_count() <= 100, "child2 node_count {} exceeds limit", c2.node_count());
}

#[test]
fn test_bloat_limit_crossover() {
    let mut rng = SmallRng::seed_from_u64(0);
    // Build two depth-3 trees (depth=3). Crossing them with max_depth=2 should
    // frequently produce depth > 2 and return TreeDepthExceeded.
    // We run a few seeds to ensure we hit the error (probabilistic).
    let found_depth_error = (0u64..20).any(|seed| {
        let mut r = SmallRng::seed_from_u64(seed);
        let p1 = GpChromosome::with_root(build_tree(3));
        let p2 = GpChromosome::with_root(build_tree(3));
        matches!(
            GpCrossover::SubtreeCrossover.apply(&p1, &p2, 2, 1000, &mut r),
            Err(GaError::TreeDepthExceeded(_))
        )
    });
    assert!(found_depth_error, "Expected at least one TreeDepthExceeded across seeds 0-19");

    // Size limit: use a moderately-sized tree and a tiny node limit
    let found_size_error = (0u64..20).any(|seed| {
        let mut r = SmallRng::seed_from_u64(seed);
        let p1 = GpChromosome::with_root(build_tree(4));
        let p2 = GpChromosome::with_root(build_tree(4));
        matches!(
            GpCrossover::SubtreeCrossover.apply(&p1, &p2, 1000, 5, &mut r),
            Err(GaError::TreeSizeExceeded(_))
        )
    });
    assert!(found_size_error, "Expected at least one TreeSizeExceeded across seeds 0-19");

    // A crossover with permissive limits should always succeed
    let p1 = GpChromosome::with_root(build_tree(2));
    let p2 = GpChromosome::with_root(build_tree(2));
    assert!(GpCrossover::SubtreeCrossover.apply(&p1, &p2, 100, 1000, &mut rng).is_ok());
}

#[test]
fn test_point_mutation() {
    let mut rng = SmallRng::seed_from_u64(7);
    // A depth-2 tree: (Add X X) — 3 nodes
    let chr = GpChromosome::with_root(build_tree(2));
    let before_count = chr.node_count();
    let before_depth = chr.depth();

    let mut chr = chr;
    let result = GpMutation::PointMutation { p_per_node: 1.0 }.apply(&mut chr, 100, 1000, &mut rng);
    assert!(result.is_ok(), "PointMutation returned error: {:?}", result);

    // Tree shape must be preserved
    assert_eq!(chr.node_count(), before_count, "PointMutation changed node_count");
    assert_eq!(chr.depth(), before_depth, "PointMutation changed depth");
}

#[test]
fn test_hoist_mutation() {
    let mut rng = SmallRng::seed_from_u64(13);

    // A depth-3 tree has 7 nodes — hoist should shrink it
    let chr = GpChromosome::with_root(build_tree(3));
    let before_count = chr.node_count();
    assert!(before_count > 1, "need a multi-node tree for hoist");

    let mut chr = chr;
    let result = GpMutation::HoistMutation.apply(&mut chr, 100, 1000, &mut rng);
    assert!(result.is_ok(), "HoistMutation returned error: {:?}", result);

    // Tree must shrink or stay the same (never grow)
    assert!(
        chr.node_count() <= before_count,
        "HoistMutation grew tree from {} to {} nodes",
        before_count,
        chr.node_count()
    );

    // Edge case: terminal root — hoist is a no-op, returns Ok(())
    let mut terminal_chr = GpChromosome::<TestNode>::with_root(Box::new(Node::Terminal(TestNode::X)));
    let result2 = GpMutation::HoistMutation.apply(&mut terminal_chr, 100, 1000, &mut rng);
    assert!(result2.is_ok());
    assert_eq!(terminal_chr.node_count(), 1);
}

#[test]
fn test_bloat_limit_mutation() {
    let mut rng = SmallRng::seed_from_u64(99);
    // SubtreeMutation with mutation_max_depth=5 on a chromosome with max_depth=1
    // — the generated subtree (depth up to 5) will violate the max_depth=1 constraint.
    // We try multiple seeds until we hit the error.
    let found_error = (0u64..50).any(|seed| {
        let mut r = SmallRng::seed_from_u64(seed);
        let mut chr = GpChromosome::with_root(build_tree(2));
        matches!(
            GpMutation::SubtreeMutation { mutation_max_depth: 5 }.apply(&mut chr, 1, 1000, &mut r),
            Err(GaError::TreeDepthExceeded(_))
        )
    });
    assert!(found_error, "Expected SubtreeMutation to return TreeDepthExceeded for max_depth=1 across seeds 0-49");
    let _ = rng;
}

#[test]
#[ignore]
fn test_gpga_ramp_half_and_half() {
    todo!("implemented in Wave 2")
}

#[test]
#[ignore]
fn test_gpga_run_symbolic_regression() {
    todo!("implemented in Wave 3")
}

#[test]
#[ignore]
fn test_generation_stats_avg_node_count() {
    todo!("implemented in Wave 2")
}

#[cfg(feature = "serde")]
#[test]
#[ignore]
fn test_serde_deep_tree() {
    todo!("implemented in Wave 3")
}
