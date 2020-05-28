use crate::parsers::groups::{GroupError, GroupRule};
use crate::parsers::tokens::Token;

impl GroupError {
    pub fn unexpected_group_end<P: Clone>(rule: &GroupRule<P>, token: &Token<P>) -> GroupError {
        return GroupError {
            starts_at: None,
            ends_at: Some(token.starts_at),
            message: format!("Unexpected group closing {}({:?}).", rule.name, token.data),
        };
    }

    pub fn unclosed_group<P: Clone>(
        rule: &GroupRule<P>,
        start: &Token<P>,
        end: Option<&Token<P>>,
    ) -> GroupError {
        return GroupError {
            starts_at: Some(start.starts_at),
            ends_at: end.map(|i| i.starts_at),
            message: format!("Unclosed group {}({:?})", rule.name, start.data),
        };
    }
}
