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

    pub fn starts_at(&self) -> usize {
        return match self {
            TokenTree::Value(token) => token.starts_at,
            TokenTree::Group { open, .. } => open.starts_at,
        };
    }

    pub fn ends_at(&self) -> usize {
        let t = self.last_token();
        return t.starts_at + t.data.len();
    }

    pub fn last_token(&self) -> Token<Payload> {
        return match self {
            TokenTree::Value(t) => t.clone(),
            TokenTree::Group {
                open, close, data, ..
            } => close
                .as_ref()
                .cloned()
                .or(data.last().map(|f| f.last_token()))
                .unwrap_or(open.clone()),
        };
    }
}
