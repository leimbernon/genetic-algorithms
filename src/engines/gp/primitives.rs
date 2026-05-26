//! Built-in GP primitive node types.
//!
//! This module provides two ready-to-use [`GpNode`] implementations:
//!
//! - [`MathNode`] — arithmetic functions and numeric terminals (Add, Sub, Mul,
//!   ProtectedDiv, Const, Var). Suitable for symbolic regression.
//! - [`BoolNode`] — Boolean logic functions (And, Or, Not, Gt, Lt). Suitable
//!   for classification tree learning.
//!
//! Both can be used as the type parameter `N` in `GpChromosome<N>`.

use super::node::{GpNode, Node};
use rand::Rng;
use std::fmt;

// ---------------------------------------------------------------------------
// MathNode
// ---------------------------------------------------------------------------

/// Arithmetic primitive set for symbolic regression.
///
/// | Variant | Arity | Description |
/// |---------|-------|-------------|
/// | `Add` | 2 | `args[0] + args[1]` |
/// | `Sub` | 2 | `args[0] - args[1]` |
/// | `Mul` | 2 | `args[0] * args[1]` |
/// | `ProtectedDiv` | 2 | `args[0] / args[1]`; returns `1.0` when `args[1].abs() < 1e-10` |
/// | `Const(f64)` | 0 | constant value (ERC) |
/// | `Var(usize)` | 0 | variable placeholder; actual value injected by fitness fn |
#[derive(Clone, Debug)]
pub enum MathNode {
    /// Binary addition.
    Add,
    /// Binary subtraction.
    Sub,
    /// Binary multiplication.
    Mul,
    /// Protected division — returns 1.0 when the denominator is near zero.
    ProtectedDiv,
    /// Ephemeral random constant (ERC).
    Const(f64),
    /// Variable placeholder. Index selects which input variable to use.
    ///
    /// **Variable injection:** Call [`Node::<MathNode>::eval_with_vars`] instead of
    /// the standard recursive `evaluate` path. `eval_with_vars(&vars)` substitutes
    /// `vars[i]` for every `Var(i)` node automatically. Calling the plain `evaluate`
    /// on a `Var` node returns `0.0` (fallback — no args are passed to a terminal).
    Var(usize),
}

impl fmt::Display for MathNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathNode::Add => write!(f, "add"),
            MathNode::Sub => write!(f, "sub"),
            MathNode::Mul => write!(f, "mul"),
            MathNode::ProtectedDiv => write!(f, "pdiv"),
            MathNode::Const(v) => write!(f, "{:.4}", v),
            MathNode::Var(i) => write!(f, "x{}", i),
        }
    }
}

impl GpNode for MathNode {
    fn arity(&self) -> usize {
        match self {
            MathNode::Add | MathNode::Sub | MathNode::Mul | MathNode::ProtectedDiv => 2,
            MathNode::Const(_) | MathNode::Var(_) => 0,
        }
    }

    fn evaluate(&self, args: &[f64]) -> f64 {
        match self {
            MathNode::Add => args[0] + args[1],
            MathNode::Sub => args[0] - args[1],
            MathNode::Mul => args[0] * args[1],
            MathNode::ProtectedDiv => {
                if args[1].abs() < 1e-10 {
                    1.0
                } else {
                    args[0] / args[1]
                }
            }
            MathNode::Const(v) => *v,
            // Var(i) is a terminal (arity 0) — args is always empty when called via
            // the standard recursive evaluate path. Use Node::<MathNode>::eval_with_vars
            // to evaluate trees that contain Var nodes with runtime variable values.
            MathNode::Var(_) => 0.0,
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, MathNode::Const(_) | MathNode::Var(_))
    }

    /// Returns a fresh ephemeral random constant sampled from [-1.0, 1.0].
    fn sample_random_terminal(rng: &mut impl Rng) -> Self {
        MathNode::Const(rng.random_range(-1.0_f64..=1.0_f64))
    }

    fn all_functions() -> Vec<Self> {
        vec![
            MathNode::Add,
            MathNode::Sub,
            MathNode::Mul,
            MathNode::ProtectedDiv,
        ]
    }
}

/// Default for `MathNode` is `Const(0.0)` — a zero constant.
///
/// This placeholder satisfies the `N: Default` bound on `GpGa<N>` and
/// `GpChromosome<N>`. The engine always overwrites nodes before use; the
/// default value is never meaningful at runtime.
impl Default for MathNode {
    fn default() -> Self {
        MathNode::Const(0.0)
    }
}

// ---------------------------------------------------------------------------
// BoolNode
// ---------------------------------------------------------------------------

/// Boolean logic primitive set.
///
/// | Variant | Arity | Evaluate |
/// |---------|-------|----------|
/// | `And` | 2 | `1.0` iff both args != `0.0` |
/// | `Or` | 2 | `1.0` iff either arg != `0.0` |
/// | `Not` | 1 | `1.0` iff `arg[0] == 0.0` |
/// | `Gt` | 2 | `1.0` iff `args[0] > args[1]` |
/// | `Lt` | 2 | `1.0` iff `args[0] < args[1]` |
///
/// **Note:** `BoolNode` has no terminal variants. When building mixed trees
/// combine `BoolNode` functions with `MathNode::Const` or `MathNode::Var`
/// terminals, or add your own terminal variants.
#[derive(Clone, Debug, Default)]
pub enum BoolNode {
    /// Logical AND.
    /// `BoolNode::default()` returns `And` (arity 2).
    #[default]
    And,
    /// Logical OR.
    Or,
    /// Logical NOT (arity 1).
    Not,
    /// Greater-than comparison.
    Gt,
    /// Less-than comparison.
    Lt,
}

impl fmt::Display for BoolNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoolNode::And => write!(f, "and"),
            BoolNode::Or => write!(f, "or"),
            BoolNode::Not => write!(f, "not"),
            BoolNode::Gt => write!(f, "gt"),
            BoolNode::Lt => write!(f, "lt"),
        }
    }
}

impl GpNode for BoolNode {
    fn arity(&self) -> usize {
        match self {
            BoolNode::And | BoolNode::Or | BoolNode::Gt | BoolNode::Lt => 2,
            BoolNode::Not => 1,
        }
    }

    fn evaluate(&self, args: &[f64]) -> f64 {
        match self {
            BoolNode::And => {
                if args[0] != 0.0 && args[1] != 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            BoolNode::Or => {
                if args[0] != 0.0 || args[1] != 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            BoolNode::Not => {
                if args[0] == 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            BoolNode::Gt => {
                if args[0] > args[1] {
                    1.0
                } else {
                    0.0
                }
            }
            BoolNode::Lt => {
                if args[0] < args[1] {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    fn is_terminal(&self) -> bool {
        // All BoolNode variants are functions (arity > 0).
        false
    }

    /// # Panics
    ///
    /// Always panics. `BoolNode` has no terminal variants — calling this is a
    /// programming error. Combine `BoolNode` with `MathNode::Const` or
    /// `MathNode::Var` for leaf nodes in mixed primitive sets.
    fn sample_random_terminal(_rng: &mut impl Rng) -> Self {
        unreachable!(
            "BoolNode has no terminals; do not use BoolNode as a standalone GpNode — \
             combine with MathNode or add terminal variants"
        )
    }

    fn all_functions() -> Vec<Self> {
        vec![
            BoolNode::And,
            BoolNode::Or,
            BoolNode::Not,
            BoolNode::Gt,
            BoolNode::Lt,
        ]
    }
}

// ---------------------------------------------------------------------------
// Variable-injection evaluation for MathNode trees (CR-02)
// ---------------------------------------------------------------------------

impl Node<MathNode> {
    /// Evaluates an expression tree, injecting variable values from `vars`.
    ///
    /// `Var(i)` nodes return `vars[i]`, falling back to `0.0` when `i` is out of
    /// bounds. All other nodes are evaluated using the standard recursive
    /// [`GpNode::evaluate`] contract.
    ///
    /// Use this method (rather than a manual tree walk) whenever trees may contain
    /// `Var` terminals.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Tree represents "x0 + x1"
    /// let result = tree.eval_with_vars(&[3.0, 5.0]);
    /// assert!((result - 8.0).abs() < 1e-10);
    /// ```
    pub fn eval_with_vars(&self, vars: &[f64]) -> f64 {
        match self {
            Node::Terminal(MathNode::Var(i)) => vars.get(*i).copied().unwrap_or(0.0),
            Node::Terminal(other) => other.evaluate(&[]),
            Node::Function { value, children } => {
                let child_vals: Vec<f64> =
                    children.iter().map(|c| c.eval_with_vars(vars)).collect();
                value.evaluate(&child_vals)
            }
        }
    }
}
