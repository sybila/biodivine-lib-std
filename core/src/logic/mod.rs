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


trait TokenRule {
    type State : Default + Clone + Eq;
    fn name(&self) -> &str;
    fn try_read_token(&self, state: &mut Self::State, data: &str) -> Option<(String, Vec<String>)>;
}

trait StaticTokenRule : TokenRule<State=()> {
    fn try_read_token_static(&self, data: &str) -> Option<(String, Vec<String>)> {
        return TokenRule::try_read_token(self, &mut(), data);
    }
}

struct SwitchRule<L: TokenRule, R: TokenRule> {
    left: L, right: R
}

struct StaticRule {}

impl<L: TokenRule, R: TokenRule> TokenRule for SwitchRule<L, R> {
    type State = (L::State, R::State);

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn try_read_token(&self, state: &mut Self::State, data: &str) -> Option<(String, Vec<String>)> {
        if state.0 != L::State::default() {
            return self.left.try_read_token(&mut state.0, data);
        } else if state.1 != R::State::default() {
            return self.right.try_read_token(&mut state.1, data);
        } else {
            return self.left.try_read_token(&mut state.0, data)
                .or_else(|| self.right.try_read_token(&mut state.1, data));
        }
    }
}

impl TokenRule for StaticRule {
    type State = ();

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn try_read_token(&self, state: &mut Self::State, data: &str) -> Option<(String, Vec<String>)> {
        unimplemented!()
    }
}

struct RuleSequence {
    items: Vec<Box<dyn StaticTokenRule>>
}

impl TokenRule for RuleSequence {
    type State = ();

    fn name(&self) -> &str {
        unimplemented!()
    }

    fn try_read_token(&self, state: &mut Self::State, data: &str) -> Option<(String, Vec<String>)> {
        for i in &self.items {
            return i.try_read_token_static(data);
        }
        return None;
    }
}