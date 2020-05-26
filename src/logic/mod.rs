//! General constructs used when working with logical formulas. Right now mostly
//! focused on boolean formulas.
//!
//!

use std::fmt::Debug;

/// Enumeration of supported binary boolean operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoolOp {
    And,
    Or,
    Xor,
    Iff,
    Imp,
}

/// A representation of a boolean formula with generic atomic propositions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoolExpr<A: Eq + Clone + Debug> {
    Atom(A),
    Not(Box<BoolExpr<A>>),
    Op {
        op: BoolOp,
        left: Box<BoolExpr<A>>,
        right: Box<BoolExpr<A>>,
    },
}
