//! This modules implements AST nodes and their evaluation mechanism.
//!
//! An Abstract Syntax Tree consists of [`Node`]s, which are built by a
//! [`Parser`](crate::Parser). AST can be evaluated or used to generate code.
use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Function;

/// Internal representation of numbers.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Number {
    Int(i128),
    Flt(f64),
}

/// [`Node`] provides a blanket trait for both [`BinaryNode`] and [`UnaryNode`].
#[cfg_attr(feature = "serde", typetag::serde(tag = "type"))]
pub trait Node: Debug + Display {
    /// Finds the value of this [`Node`].
    fn evaluate(&self) -> Number;

    fn to_tree(&self) -> Vec<String>;
}

/// Convenience type alias for a [`Node`] stored on the heap.
pub type NodeBox = Box<dyn Node>;

/// [`BinaryAction`] is an action done by a [`Node`] using two operands.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BinaryAction {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// [`BinaryNode`] is a [`Node`] that performs an action on two operands.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BinaryNode {
    /// Left-hand side operand (if any) of this [`BinaryNode`].
    left: NodeBox,

    /// Action to be performed by this [`BinaryNode`].
    actor: BinaryAction,

    /// Right-hand side operand of this [`BinaryNode`].
    right: NodeBox,
}

/// [`BinaryAction`] is an action done by a [`Node`] using one operand.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnaryAction {
    Neg,
    Iden,
    Func(Function),
}

/// [`BinaryNode`] is a [`Node`] that performs an action on one operand.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnaryNode {
    /// Action to be performed by this [`UnaryNode`].
    actor: UnaryAction,

    /// The sole operand of this [`UnaryNode`].
    operand: NodeBox,
}

/// [`PlainNode`] simply stores the numbers without any action.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PlainNode(Number);

// -----------------------------------------------------------------------------
// All impls onwards.
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// 1. impls for Number.
// -----------------------------------------------------------------------------
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        // Integer comparison.
        if let Self::Int(a) = self {
            if let Self::Int(b) = other {
                return a == b;
            }
        }

        // Floating point comparison.
        let epsilon = f64::EPSILON * 1e3;
        let a = match self {
            Self::Int(n) => *n as f64,
            Self::Flt(n) => *n,
        };
        let b = match other {
            Self::Int(n) => *n as f64,
            Self::Flt(n) => *n,
        };

        (a - b).abs() < epsilon
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Integer comparison.
        if let Self::Int(a) = self {
            if let Self::Int(b) = other {
                return a.partial_cmp(b);
            }
        }

        // Floating point comparison.
        let epsilon = f64::EPSILON * 1e3;
        let a = match self {
            Self::Int(n) => *n as f64,
            Self::Flt(n) => *n,
        };
        let b = match other {
            Self::Int(n) => *n as f64,
            Self::Flt(n) => *n,
        };

        a.partial_cmp(&b).map(|o| {
            if o == std::cmp::Ordering::Equal || (a - b).abs() < epsilon {
                std::cmp::Ordering::Equal
            } else {
                o
            }
        })
    }
}

impl Add for Number {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(a) => match rhs {
                Self::Int(b) => Self::Int(a + b),
                Self::Flt(b) => Self::Flt((a as f64) + b),
            },
            Self::Flt(a) => match rhs {
                Self::Int(b) => Self::Flt(a + (b as f64)),
                Self::Flt(b) => Self::Flt(a + b),
            },
        }
    }
}

impl Sub for Number {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(a) => match rhs {
                Self::Int(b) => Self::Int(a - b),
                Self::Flt(b) => Self::Flt((a as f64) - b),
            },
            Self::Flt(a) => match rhs {
                Self::Int(b) => Self::Flt(a - (b as f64)),
                Self::Flt(b) => Self::Flt(a - b),
            },
        }
    }
}

impl Mul for Number {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Self::Int(a) => match rhs {
                Self::Int(b) => Self::Int(a * b),
                Self::Flt(b) => Self::Flt((a as f64) * b),
            },
            Self::Flt(a) => match rhs {
                Self::Int(b) => Self::Flt(a * (b as f64)),
                Self::Flt(b) => Self::Flt(a * b),
            },
        }
    }
}

impl Div for Number {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if rhs == Self::Int(0) || rhs == Self::Flt(0.0) {
            return Self::Flt(f64::NAN);
        }

        match self {
            Self::Int(a) => match rhs {
                Self::Int(b) => Self::Int(a / b),
                Self::Flt(b) => Self::Flt((a as f64) / b),
            },
            Self::Flt(a) => match rhs {
                Self::Int(b) => Self::Flt(a / (b as f64)),
                Self::Flt(b) => Self::Flt(a / b),
            },
        }
    }
}

impl Neg for Number {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Int(a) => Self::Int(-a),
            Self::Flt(a) => Self::Flt(-a),
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Flt(n) => write!(f, "{}", n),
            Self::Int(n) => write!(f, "{}", n),
        }
    }
}

impl From<u8> for Number {
    fn from(n: u8) -> Self {
        Self::Int(n as i128)
    }
}

impl From<u16> for Number {
    fn from(n: u16) -> Self {
        Self::Int(n as i128)
    }
}

impl From<u32> for Number {
    fn from(n: u32) -> Self {
        Self::Int(n as i128)
    }
}

impl From<u64> for Number {
    fn from(n: u64) -> Self {
        Self::Int(n as i128)
    }
}

impl From<i8> for Number {
    fn from(n: i8) -> Self {
        Self::Int(n as i128)
    }
}

impl From<i16> for Number {
    fn from(n: i16) -> Self {
        Self::Int(n as i128)
    }
}

impl From<i32> for Number {
    fn from(n: i32) -> Self {
        Self::Int(n as i128)
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Self::Int(n as i128)
    }
}

impl From<i128> for Number {
    fn from(n: i128) -> Self {
        Self::Int(n)
    }
}

impl From<f32> for Number {
    fn from(n: f32) -> Self {
        Self::Flt(n as f64)
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Self::Flt(n)
    }
}

// -----------------------------------------------------------------------------
// 2. impls for BinaryAction.
// -----------------------------------------------------------------------------

impl BinaryAction {
    pub fn evaluate(&self, left: Number, right: Number) -> Number {
        match self {
            Self::Add => left + right,
            Self::Sub => left - right,
            Self::Mul => left * right,
            Self::Div => left / right,
            Self::Pow => {
                // Integer base and exponent are kept as integer.
                if let Number::Int(n) = left {
                    if let Number::Int(m) = right {
                        if m >= 0 {
                            return Number::Int(n.pow(m as u32));
                        } else {
                            return Number::Flt((n as f64).powf(m as f64));
                        }
                    }
                }

                // Otherwise, both are converted to float.
                let left = match left {
                    Number::Int(n) => n as f64,
                    Number::Flt(n) => n,
                };
                let right = match right {
                    Number::Int(n) => n as f64,
                    Number::Flt(n) => n,
                };
                Number::Flt(left.powf(right))
            }
        }
    }
}

impl Display for BinaryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Op(")?;
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Pow => write!(f, "^"),
        }?;
        write!(f, ")")
    }
}

// -----------------------------------------------------------------------------
// 3. impls for BinaryNode.
// -----------------------------------------------------------------------------

#[cfg_attr(feature = "serde", typetag::serde)]
impl Node for BinaryNode {
    fn evaluate(&self) -> Number {
        // Evaluate both sub-nodes.
        let left = self.left.evaluate();
        let right = self.right.evaluate();

        // Then evalute this node.
        self.actor.evaluate(left, right)
    }

    fn to_tree(&self) -> Vec<String> {
        // Get actor.
        let actor = self.actor.to_string();

        // Process left side.
        let mut left_tree = self.left.to_tree();
        left_tree[0].insert_str(0, "`-- ");
        for line in left_tree.iter_mut().skip(1) {
            line.insert_str(0, "|   ");
        }

        // Process right side.
        let mut right_tree = self.right.to_tree();
        right_tree[0].insert_str(0, "`-- ");
        for line in right_tree.iter_mut().skip(1) {
            line.insert_str(0, "    ");
        }

        // Combine all three.
        let mut tree = vec![actor];
        tree.extend(left_tree);
        tree.extend(right_tree);

        tree
    }
}

impl Display for BinaryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tree().join("\n"))
    }
}

impl BinaryNode {
    /// Creates a new [`BinaryNode`].
    #[rustfmt::skip]
    pub fn new(
        left: NodeBox,
        actor: BinaryAction,
        right: NodeBox
    ) -> BinaryNode {
        Self { left, actor, right }
    }
}

// -----------------------------------------------------------------------------
// 4. impls for UnaryAction.
// -----------------------------------------------------------------------------

impl UnaryAction {
    pub fn evaluate(&self, operand: Number) -> Number {
        match self {
            Self::Neg => -operand,
            Self::Iden => operand,
            Self::Func(f) => UnaryAction::evaluate_function(f, operand),
        }
    }

    fn evaluate_function(func: &Function, operand: Number) -> Number {
        match func {
            Function::Sin => match operand {
                Number::Int(n) => Number::Flt((n as f64).sin()),
                Number::Flt(n) => Number::Flt(n.sin()),
            },
            Function::Cos => match operand {
                Number::Int(n) => Number::Flt((n as f64).cos()),
                Number::Flt(n) => Number::Flt(n.cos()),
            },
            Function::Tan => match operand {
                Number::Int(n) => Number::Flt((n as f64).tan()),
                Number::Flt(n) => Number::Flt(n.tan()),
            },
            Function::Sec => match operand {
                Number::Int(n) => Number::Flt((n as f64).cos().recip()),
                Number::Flt(n) => Number::Flt(n.cos().recip()),
            },
            Function::Csc => match operand {
                Number::Int(n) => Number::Flt((n as f64).sin().recip()),
                Number::Flt(n) => Number::Flt(n.sin().recip()),
            },

            Function::Cot => match operand {
                Number::Int(n) => Number::Flt((n as f64).tan().recip()),
                Number::Flt(n) => Number::Flt(n.tan().recip()),
            },

            Function::Asin => match operand {
                Number::Int(n) => Number::Flt((n as f64).asin()),
                Number::Flt(n) => Number::Flt(n.asin()),
            },

            Function::Acos => match operand {
                Number::Int(n) => Number::Flt((n as f64).acos()),
                Number::Flt(n) => Number::Flt(n.acos()),
            },

            Function::Atan => match operand {
                Number::Int(n) => Number::Flt((n as f64).atan()),
                Number::Flt(n) => Number::Flt(n.atan()),
            },

            Function::Asec => match operand {
                Number::Int(n) => Number::Flt((n as f64).recip().acos()),
                Number::Flt(n) => Number::Flt(n.recip().acos()),
            },

            Function::Acsc => match operand {
                Number::Int(n) => Number::Flt((n as f64).recip().asin()),
                Number::Flt(n) => Number::Flt(n.recip().asin()),
            },

            Function::Acot => match operand {
                Number::Int(n) => Number::Flt((n as f64).recip().atan()),
                Number::Flt(n) => Number::Flt(n.recip().atan()),
            },
        }
    }
}

impl Display for UnaryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => write!(f, "Op(-)"),
            Self::Iden => write!(f, "Op(+)"),
            Self::Func(func) => write!(f, "Func({})", func),
        }
    }
}

// -----------------------------------------------------------------------------
// 5. impls for UnaryNode.
// -----------------------------------------------------------------------------

#[cfg_attr(feature = "serde", typetag::serde)]
impl Node for UnaryNode {
    fn evaluate(&self) -> Number {
        // Evaluate the operand.
        let operand = self.operand.evaluate();

        // Then evaluate this node.
        self.actor.evaluate(operand)
    }

    fn to_tree(&self) -> Vec<String> {
        // Get actor.
        let actor = self.actor.to_string();

        // Process left side.
        let mut left_tree = self.operand.to_tree();
        left_tree[0].insert_str(0, "`-- ");
        for line in left_tree.iter_mut().skip(1) {
            line.insert_str(0, "|   ");
        }

        // Combine.
        left_tree.insert(0, actor);

        left_tree
    }
}

impl Display for UnaryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tree().join("\n"))
    }
}

impl UnaryNode {
    /// Creates a new [`UnaryNode`].
    pub fn new(actor: UnaryAction, operand: NodeBox) -> UnaryNode {
        Self { actor, operand }
    }
}

// -----------------------------------------------------------------------------
// 6. impls for PlainNode.
// -----------------------------------------------------------------------------

#[cfg_attr(feature = "serde", typetag::serde)]
impl Node for PlainNode {
    fn evaluate(&self) -> Number {
        self.0
    }

    fn to_tree(&self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

impl Display for PlainNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_tree().join("\n"))
    }
}

impl PlainNode {
    pub fn new(value: Number) -> PlainNode {
        Self(value)
    }
}
