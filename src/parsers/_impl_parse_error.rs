use crate::parsers::groups::{GroupRule, TokenTree};
use crate::parsers::tokens::Token;
use crate::parsers::ParseError;

impl ParseError {
    pub fn unexpected_group_end<P: Clone>(rule: &GroupRule<P>, token: &Token<P>) -> ParseError {
        return ParseError {
            starts_at: None,
            ends_at: Some(token.starts_at),
            message: format!("Unexpected group closing {}({:?}).", rule.name, token.data),
        };
    }

    pub fn unclosed_group<P: Clone>(
        rule: &GroupRule<P>,
        start: &Token<P>,
        end: Option<&Token<P>>,
    ) -> ParseError {
        return ParseError {
            starts_at: Some(start.starts_at),
            ends_at: end.map(|i| i.starts_at),
            message: format!("Unclosed group {}({:?})", rule.name, start.data),
        };
    }

    pub fn invalid<P: Clone>(message: &str, forest: &[TokenTree<P>]) -> ParseError {
        return ParseError {
            message: message.to_string(),
            starts_at: forest.first().map(|it| it.starts_at()),
            ends_at: forest.last().map(|it| it.ends_at()),
        };
    }
}
