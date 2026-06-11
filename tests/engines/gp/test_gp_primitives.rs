use genetic_algorithms::gp::{BoolNode, GpNode, MathNode, Node};

// ── MathNode ─────────────────────────────────────────────────────────────────

#[test]
fn math_node_arity() {
    assert_eq!(MathNode::Add.arity(), 2);
    assert_eq!(MathNode::Sub.arity(), 2);
    assert_eq!(MathNode::Mul.arity(), 2);
    assert_eq!(MathNode::ProtectedDiv.arity(), 2);
    assert_eq!(MathNode::Const(1.0).arity(), 0);
    assert_eq!(MathNode::Var(0).arity(), 0);
}

#[test]
fn math_node_is_terminal() {
    assert!(!MathNode::Add.is_terminal());
    assert!(!MathNode::Sub.is_terminal());
    assert!(!MathNode::Mul.is_terminal());
    assert!(!MathNode::ProtectedDiv.is_terminal());
    assert!(MathNode::Const(1.0).is_terminal());
    assert!(MathNode::Var(0).is_terminal());
}

#[test]
fn math_node_evaluate_add() {
    assert!((MathNode::Add.evaluate(&[3.0, 4.0]) - 7.0).abs() < 1e-10);
}

#[test]
fn math_node_evaluate_sub() {
    assert!((MathNode::Sub.evaluate(&[10.0, 3.0]) - 7.0).abs() < 1e-10);
}

#[test]
fn math_node_evaluate_mul() {
    assert!((MathNode::Mul.evaluate(&[3.0, 4.0]) - 12.0).abs() < 1e-10);
}

#[test]
fn math_node_evaluate_protected_div_normal() {
    assert!((MathNode::ProtectedDiv.evaluate(&[10.0, 2.0]) - 5.0).abs() < 1e-10);
}

#[test]
fn math_node_evaluate_protected_div_zero_denominator() {
    assert!((MathNode::ProtectedDiv.evaluate(&[10.0, 0.0]) - 1.0).abs() < 1e-10);
    assert!((MathNode::ProtectedDiv.evaluate(&[5.0, 1e-11]) - 1.0).abs() < 1e-10);
}

#[test]
fn math_node_evaluate_const() {
    assert!((MathNode::Const(42.0).evaluate(&[]) - 42.0).abs() < 1e-10);
}

#[test]
fn math_node_evaluate_var_fallback() {
    // Var returns 0.0 via standard evaluate (no vars injected)
    assert!((MathNode::Var(0).evaluate(&[]) - 0.0).abs() < 1e-10);
}

#[test]
fn math_node_display() {
    assert_eq!(format!("{}", MathNode::Add), "add");
    assert_eq!(format!("{}", MathNode::Sub), "sub");
    assert_eq!(format!("{}", MathNode::Mul), "mul");
    assert_eq!(format!("{}", MathNode::ProtectedDiv), "pdiv");
    assert_eq!(format!("{}", MathNode::Const(1.5)), "1.5000");
    assert_eq!(format!("{}", MathNode::Var(2)), "x2");
}

#[test]
fn math_node_all_functions() {
    let fns = MathNode::all_functions();
    assert_eq!(fns.len(), 4);
}

#[test]
fn math_node_default_is_const_zero() {
    matches!(MathNode::default(), MathNode::Const(v) if v == 0.0);
}

// ── BoolNode ─────────────────────────────────────────────────────────────────

#[test]
fn bool_node_arity() {
    assert_eq!(BoolNode::And.arity(), 2);
    assert_eq!(BoolNode::Or.arity(), 2);
    assert_eq!(BoolNode::Not.arity(), 1);
    assert_eq!(BoolNode::Gt.arity(), 2);
    assert_eq!(BoolNode::Lt.arity(), 2);
}

#[test]
fn bool_node_is_terminal() {
    assert!(!BoolNode::And.is_terminal());
    assert!(!BoolNode::Not.is_terminal());
}

#[test]
fn bool_node_evaluate_and() {
    assert!((BoolNode::And.evaluate(&[1.0, 1.0]) - 1.0).abs() < 1e-10);
    assert!((BoolNode::And.evaluate(&[1.0, 0.0]) - 0.0).abs() < 1e-10);
    assert!((BoolNode::And.evaluate(&[0.0, 0.0]) - 0.0).abs() < 1e-10);
}

#[test]
fn bool_node_evaluate_or() {
    assert!((BoolNode::Or.evaluate(&[0.0, 1.0]) - 1.0).abs() < 1e-10);
    assert!((BoolNode::Or.evaluate(&[0.0, 0.0]) - 0.0).abs() < 1e-10);
}

#[test]
fn bool_node_evaluate_not() {
    assert!((BoolNode::Not.evaluate(&[0.0]) - 1.0).abs() < 1e-10);
    assert!((BoolNode::Not.evaluate(&[1.0]) - 0.0).abs() < 1e-10);
}

#[test]
fn bool_node_evaluate_gt_lt() {
    assert!((BoolNode::Gt.evaluate(&[5.0, 3.0]) - 1.0).abs() < 1e-10);
    assert!((BoolNode::Gt.evaluate(&[3.0, 5.0]) - 0.0).abs() < 1e-10);
    assert!((BoolNode::Lt.evaluate(&[2.0, 4.0]) - 1.0).abs() < 1e-10);
    assert!((BoolNode::Lt.evaluate(&[4.0, 2.0]) - 0.0).abs() < 1e-10);
}

#[test]
fn bool_node_display() {
    assert_eq!(format!("{}", BoolNode::And), "and");
    assert_eq!(format!("{}", BoolNode::Or), "or");
    assert_eq!(format!("{}", BoolNode::Not), "not");
    assert_eq!(format!("{}", BoolNode::Gt), "gt");
    assert_eq!(format!("{}", BoolNode::Lt), "lt");
}

#[test]
fn bool_node_all_functions() {
    let fns = BoolNode::all_functions();
    assert_eq!(fns.len(), 5);
}

// ── eval_with_vars ────────────────────────────────────────────────────────────

#[test]
fn eval_with_vars_var_substitution() {
    // Tree: x0 + x1
    let tree = Node::Function {
        value: MathNode::Add,
        children: vec![
            Box::new(Node::Terminal(MathNode::Var(0))),
            Box::new(Node::Terminal(MathNode::Var(1))),
        ],
    };
    let result = tree.eval_with_vars(&[3.0, 5.0]);
    assert!((result - 8.0).abs() < 1e-10, "x0+x1 with [3,5] = {}", result);
}

#[test]
fn eval_with_vars_out_of_bounds_fallback() {
    // Var(5) with only 2 vars → falls back to 0.0
    let tree = Node::Terminal(MathNode::Var(5));
    let result = tree.eval_with_vars(&[1.0, 2.0]);
    assert!((result - 0.0).abs() < 1e-10);
}

#[test]
fn eval_with_vars_const_terminal() {
    let tree = Node::Terminal(MathNode::Const(7.0));
    let result = tree.eval_with_vars(&[1.0, 2.0]);
    assert!((result - 7.0).abs() < 1e-10);
}

#[test]
fn eval_with_vars_nested() {
    // Tree: mul(x0, add(x1, const(2.0))) = x0 * (x1 + 2)
    let tree = Node::Function {
        value: MathNode::Mul,
        children: vec![
            Box::new(Node::Terminal(MathNode::Var(0))),
            Box::new(Node::Function {
                value: MathNode::Add,
                children: vec![
                    Box::new(Node::Terminal(MathNode::Var(1))),
                    Box::new(Node::Terminal(MathNode::Const(2.0))),
                ],
            }),
        ],
    };
    // x0=3, x1=4 → 3 * (4 + 2) = 18
    let result = tree.eval_with_vars(&[3.0, 4.0]);
    assert!((result - 18.0).abs() < 1e-10, "expected 18, got {}", result);
}
