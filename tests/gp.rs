//! Wave 0 GP subsystem tests.
//!
//! Non-ignored tests validate the core API contracts (GpNode, Node<N>,
//! GpChromosome, TreeChromosome, MathNode, BoolNode). Ignored stubs are
//! placeholders for operators and engine tests added in Waves 1–3.

use genetic_algorithms::gp::{BoolNode, GpChromosome, GpNode, MathNode, Node, TreeChromosome};
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
// Wave 1–3 stub tests (ignored)
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn test_subtree_crossover() {
    todo!("implemented in Wave 1")
}

#[test]
#[ignore]
fn test_point_mutation() {
    todo!("implemented in Wave 1")
}

#[test]
#[ignore]
fn test_hoist_mutation() {
    todo!("implemented in Wave 1")
}

#[test]
#[ignore]
fn test_bloat_limit_crossover() {
    todo!("implemented in Wave 2")
}

#[test]
#[ignore]
fn test_bloat_limit_mutation() {
    todo!("implemented in Wave 2")
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
