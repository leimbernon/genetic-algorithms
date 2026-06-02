//! GP subsystem tests — Waves 0–3.
//!
//! Non-ignored tests validate the core API contracts (GpNode, Node<N>,
//! GpChromosome, TreeChromosome, MathNode, BoolNode) and GP operators
//! (SubtreeCrossover, PointMutation, HoistMutation, bloat limits).

use genetic_algorithms::error::GaError;
use genetic_algorithms::gp::{
    ramped_half_and_half, BoolNode, GpChromosome, GpConfiguration, GpCrossover, GpGa, GpMutation,
    GpNode, MathNode, Node, TreeChromosome,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fmt;

// ---------------------------------------------------------------------------
// TestNode — minimal inline GP primitive for core API tests
// ---------------------------------------------------------------------------

/// A minimal 4-variant GpNode used for type-level tests in this file.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum TestNode {
    Add,
    Mul,
    #[default]
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
    let chr: GpChromosome<TestNode> =
        GpChromosome::with_root(Box::new(Node::Terminal(TestNode::X)));
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
    assert!(
        s.starts_with('('),
        "expected S-expr to start with '(', got: {}",
        s
    );
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
    assert_eq!(MathNode::Const(7.0).arity(), 0);
    assert!(MathNode::Const(7.0).is_terminal());
    assert_eq!(MathNode::Var(0).arity(), 0);
    assert!(MathNode::Var(0).is_terminal());

    // all_functions returns exactly 4 variants
    let fns = MathNode::all_functions();
    assert_eq!(fns.len(), 4);

    // sample_random_terminal produces a terminal
    let t = MathNode::sample_random_terminal(&mut rng);
    assert!(
        t.is_terminal(),
        "sample_random_terminal must return a terminal"
    );

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
    assert!(
        result.is_ok(),
        "Expected Ok from crossover, got: {:?}",
        result
    );
    let (c1, c2): (GpChromosome<TestNode>, GpChromosome<TestNode>) = result.unwrap();

    // Both children must respect the limits
    assert!(
        c1.depth() <= 10,
        "child1 depth {} exceeds limit",
        c1.depth()
    );
    assert!(
        c2.depth() <= 10,
        "child2 depth {} exceeds limit",
        c2.depth()
    );
    assert!(
        c1.node_count() <= 100,
        "child1 node_count {} exceeds limit",
        c1.node_count()
    );
    assert!(
        c2.node_count() <= 100,
        "child2 node_count {} exceeds limit",
        c2.node_count()
    );
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
    assert!(
        found_depth_error,
        "Expected at least one TreeDepthExceeded across seeds 0-19"
    );

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
    assert!(
        found_size_error,
        "Expected at least one TreeSizeExceeded across seeds 0-19"
    );

    // A crossover with permissive limits should always succeed
    let p1 = GpChromosome::with_root(build_tree(2));
    let p2 = GpChromosome::with_root(build_tree(2));
    assert!(GpCrossover::SubtreeCrossover
        .apply(&p1, &p2, 100, 1000, &mut rng)
        .is_ok());
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
    assert_eq!(
        chr.node_count(),
        before_count,
        "PointMutation changed node_count"
    );
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
    let mut terminal_chr =
        GpChromosome::<TestNode>::with_root(Box::new(Node::Terminal(TestNode::X)));
    let result2 = GpMutation::HoistMutation.apply(&mut terminal_chr, 100, 1000, &mut rng);
    assert!(result2.is_ok());
    assert_eq!(terminal_chr.node_count(), 1);
}

#[test]
fn test_bloat_limit_mutation() {
    let rng = SmallRng::seed_from_u64(99);
    // SubtreeMutation with mutation_max_depth=5 on a chromosome with max_depth=1
    // — the generated subtree (depth up to 5) will violate the max_depth=1 constraint.
    // We try multiple seeds until we hit the error.
    let found_error = (0u64..50).any(|seed| {
        let mut r = SmallRng::seed_from_u64(seed);
        let mut chr = GpChromosome::with_root(build_tree(2));
        matches!(
            GpMutation::SubtreeMutation {
                mutation_max_depth: 5
            }
            .apply(&mut chr, 1, 1000, &mut r),
            Err(GaError::TreeDepthExceeded(_))
        )
    });
    assert!(
        found_error,
        "Expected SubtreeMutation to return TreeDepthExceeded for max_depth=1 across seeds 0-49"
    );
    let _ = rng;
}

// ---------------------------------------------------------------------------
// Wave 2 engine tests
// ---------------------------------------------------------------------------

#[test]
fn test_gpga_ramp_half_and_half() {
    let mut rng = SmallRng::seed_from_u64(77);
    let pop_size = 20;
    let init_max_depth = 4;

    let pop: Vec<GpChromosome<TestNode>> =
        ramped_half_and_half::<TestNode>(pop_size, init_max_depth, &mut rng);

    // Population must have exactly pop_size individuals.
    assert_eq!(pop.len(), pop_size, "population size should equal pop_size");

    // Every chromosome must have depth >= 1 and depth <= init_max_depth + 1
    // (grow can exceed by 1 via a 50% chance at the deepest level).
    for (i, chr) in pop.iter().enumerate() {
        let d = chr.depth();
        assert!(d >= 1, "chromosome {} has depth {} < 1", i, d);
        // full_tree(d) produces exactly depth d; grow_tree(d) can produce up
        // to d. Neither method produces trees deeper than init_max_depth.
        assert!(
            d <= init_max_depth,
            "chromosome {} has depth {} > init_max_depth {}",
            i,
            d,
            init_max_depth
        );
        let n = chr.node_count();
        assert!(n >= 1, "chromosome {} has 0 nodes", i);
    }
}

#[test]
fn test_gpga_run_symbolic_regression() {
    // A simple fitness: constant-zero tree has fitness 0.0, closer to target is better.
    // Use minimization; the engine should return Ok with a valid best chromosome.
    let config = GpConfiguration::new()
        .with_population_size(20)
        .with_max_generations(10)
        .with_init_max_depth(3)
        .with_max_depth(6)
        .with_max_node_count(50);

    let mut engine = GpGa::<TestNode>::with_ramped_half_and_half(config, |_tree| {
        // Simple fitness: just return 1.0 (minimization target)
        1.0
    });

    let result = engine.run();
    assert!(
        result.is_ok(),
        "GpGa::run() returned Err: {:?}",
        result.err()
    );

    let result = result.unwrap();
    assert!(
        !result.best_fitness.is_nan(),
        "best_fitness should not be NaN"
    );
    assert_eq!(result.generations, 10, "should complete all generations");
    assert_eq!(
        result.population.len(),
        20,
        "final population should have pop_size individuals"
    );
}

#[test]
fn test_generation_stats_avg_node_count() {
    // Verify that GpGa populates avg_node_count in every GenerationStats.
    use genetic_algorithms::observer::GaObserver;
    use genetic_algorithms::stats::GenerationStats;
    use std::sync::{Arc, Mutex};

    // Collect stats via observer.
    let collected: Arc<Mutex<Vec<GenerationStats>>> = Arc::new(Mutex::new(Vec::new()));
    let collected_clone = Arc::clone(&collected);

    struct StatsCollector {
        stats: Arc<Mutex<Vec<GenerationStats>>>,
    }

    impl GaObserver<GpChromosome<TestNode>> for StatsCollector {
        fn on_generation_end(&self, stats: &GenerationStats) {
            self.stats.lock().unwrap().push(stats.clone());
        }
    }

    let observer = Arc::new(StatsCollector {
        stats: collected_clone,
    });

    let config = GpConfiguration::new()
        .with_population_size(10)
        .with_max_generations(3)
        .with_init_max_depth(3)
        .with_max_depth(6)
        .with_max_node_count(50);

    let mut engine =
        GpGa::<TestNode>::with_ramped_half_and_half(config, |_tree| 1.0).with_observer(observer);

    engine.run().expect("run should succeed");

    let stats = collected.lock().unwrap();
    assert_eq!(stats.len(), 3, "should have 3 generation stats entries");
    for (i, s) in stats.iter().enumerate() {
        assert!(
            s.avg_node_count > 0.0,
            "generation {} avg_node_count should be > 0.0, got {}",
            i,
            s.avg_node_count
        );
    }
}

/// Verify that a depth-64 right-spine tree can be serialized and deserialized
/// without stack overflow, using the `serde_stacker` Serializer/Deserializer
/// wrappers.
///
/// # Design
///
/// A depth-64 right-spine tree is the engine's practical maximum (configurable
/// via `GpConfiguration::with_max_depth`). The `serde_stacker` crate is wired
/// into the `serde` feature so that users checkpointing GP runs have access to
/// stack-safe serialization for arbitrarily deep trees.
///
/// For depth 64 the JSON nesting is 64 × 2 levels (struct + array per node),
/// totalling 128 levels — exactly at serde_json's default recursion limit.
/// Using `serde_stacker` wrappers ensures correctness even at this boundary and
/// for any user-configured depth beyond the default.
#[cfg(feature = "serde")]
#[test]
fn test_serde_deep_tree() {
    // Build a right-spine Function chain of depth 64 using TestNode::Add (arity 2).
    // Structure: Add(Add(Add(...Add(X, X)..., X), X), X) — depth 64.
    // Build inside-out: start with a Terminal, wrap in Function Add 63 times.
    let mut root: Node<TestNode> = Node::Terminal(TestNode::X);
    for _ in 0..63 {
        root = Node::Function {
            value: TestNode::Add,
            children: vec![Box::new(root), Box::new(Node::Terminal(TestNode::X))],
        };
    }
    assert_eq!(
        root.depth(),
        64,
        "tree must be exactly depth 64 before serialization"
    );

    let chr = GpChromosome::<TestNode>::with_root(Box::new(root));

    // Serialize: wrap the serde_json Serializer with serde_stacker::Serializer
    // so the call stack grows dynamically for arbitrarily deep trees.
    let mut out = Vec::<u8>::new();
    {
        let mut json_ser = serde_json::Serializer::new(&mut out);
        let stacker_ser = serde_stacker::Serializer::new(&mut json_ser);
        use serde::Serialize as _;
        chr.serialize(stacker_ser)
            .expect("serialize must not overflow");
    }
    let json = String::from_utf8(out).expect("output must be valid UTF-8");

    // Deserialize: disable serde_json's built-in recursion limit (requires the
    // `unbounded_depth` feature on serde_json), then wrap with
    // serde_stacker::Deserializer so the Rust call stack grows dynamically for
    // deeply nested trees rather than overflowing.
    let restored: GpChromosome<TestNode> = {
        let mut json_de = serde_json::Deserializer::from_str(&json);
        json_de.disable_recursion_limit();
        let stacker_de = serde_stacker::Deserializer::new(&mut json_de);
        use serde::Deserialize as _;
        GpChromosome::<TestNode>::deserialize(stacker_de).expect("deserialize must not overflow")
    };

    assert_eq!(restored.depth(), 64, "round-trip must preserve tree depth");
}
