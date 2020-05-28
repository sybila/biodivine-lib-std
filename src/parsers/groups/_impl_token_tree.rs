use crate::parsers::groups::{TokenForest, TokenTree};
use crate::parsers::tokens::Token;

impl<Payload: Clone> TokenTree<'_, Payload> {
    pub fn value(&self) -> Option<&Token<Payload>> {
        return if let TokenTree::Value(token) = self {
            Some(token)
        } else {
            None
        };
    }

    pub fn name(&self) -> Option<&String> {
        return if let TokenTree::Group { name, .. } = self {
            Some(name)
        } else {
            None
        };
    }

    pub fn children(&self) -> Option<&TokenForest<Payload>> {
        return if let TokenTree::Group { data, .. } = self {
            Some(data)
        } else {
            None
        };
    }
}
