use crate::{Token, TokenTree};

impl TokenTree {
    /// Create a new `Literal` token tree.
    pub fn literal(token: Token) -> TokenTree {
        return TokenTree::Literal { token };
    }

    /// Create a new `Sequence` token tree.
    pub fn sequence(rule: String, items: Vec<TokenTree>) -> TokenTree {
        if items.is_empty() {
            panic!("Empty token sequences not allowed. Use empty token literal instead.");
        }
        return TokenTree::Sequence {
            meta: vec![rule],
            items,
        };
    }

    /// Create a new `Group` token tree.
    pub fn group(rule: String, open: TokenTree, content: TokenTree, close: TokenTree) -> TokenTree {
        return TokenTree::Group {
            meta: vec![rule],
            open: Box::new(open),
            close: Box::new(close),
            content: Box::new(content),
        };
    }

    /// Create a new `Branch` token tree.
    pub fn branch(
        rule: String,
        left: TokenTree,
        delimiter: TokenTree,
        right: TokenTree,
    ) -> TokenTree {
        return TokenTree::Branch {
            meta: vec![rule],
            left: Box::new(left),
            right: Box::new(right),
            delimiter: Box::new(delimiter),
        };
    }

    /// Add extra string data `value` to this token tree.
    pub fn push_extra(&mut self, value: String) {
        match self {
            TokenTree::Literal { token } => token.push_extra(value),
            TokenTree::Sequence { meta, .. } => meta.push(value),
            TokenTree::Group { meta, .. } => meta.push(value),
            TokenTree::Branch { meta, .. } => meta.push(value),
        }
    }

    /// Index of the first character of this token tree in the input string.
    /// (This needs to traverse to the first available token in the tree)
    pub fn starts_at(&self) -> usize {
        match self {
            TokenTree::Literal { token } => token.starts_at,
            TokenTree::Sequence { items, .. } => items[0].starts_at(),
            TokenTree::Group { open, .. } => open.starts_at(),
            TokenTree::Branch { left, .. } => left.starts_at(),
        }
    }

    /// Name of the rule that created this token tree.
    pub fn rule(&self) -> &String {
        match self {
            TokenTree::Literal { token } => token.rule(),
            TokenTree::Sequence { meta, .. } => &meta[0],
            TokenTree::Group { meta, .. } => &meta[0],
            TokenTree::Branch { meta, .. } => &meta[0],
        }
    }

    /// A slice of string extras attached to this token tree.
    pub fn extras(&self) -> &[String] {
        return match self {
            TokenTree::Literal { token } => token.extras(),
            TokenTree::Sequence { meta, .. } => &meta[1..],
            TokenTree::Group { meta, .. } => &meta[1..],
            TokenTree::Branch { meta, .. } => &meta[1..],
        };
    }

    /// Return a vector of references to direct child trees of this `TokenTree`.
    pub fn children(&self) -> Vec<&TokenTree> {
        return match self {
            TokenTree::Literal { .. } => Vec::new(),
            TokenTree::Sequence { items, .. } => items.iter().collect(),
            TokenTree::Group {
                open,
                content,
                close,
                ..
            } => vec![open, content, close],
            TokenTree::Branch {
                left,
                delimiter,
                right,
                ..
            } => vec![left, delimiter, right],
        };
    }

    /// True if this is an error fragment.
    pub fn is_error(&self) -> bool {
        return self.rule().starts_with("error");
    }

    /// If this token is an error token, return its error message, otherwise return `None`.
    pub fn error_message(&self) -> Option<&String> {
        if !self.is_error() {
            return None;
        }
        match self {
            TokenTree::Literal { token } => token.error_message(),
            TokenTree::Sequence { meta, .. } => meta.get(1),
            TokenTree::Group { meta, .. } => meta.get(1),
            TokenTree::Branch { meta, .. } => meta.get(1),
        }
    }

    /// If this `TokenTree` is a `Literal`, return its inner token.
    pub fn as_token(&self) -> Option<&Token> {
        match self {
            TokenTree::Literal { token } => Some(token),
            _ => None,
        }
    }

    /// If this `TokenTree` is a `Sequence`, return its child trees.
    pub fn as_sequence(&self) -> Option<&[TokenTree]> {
        match self {
            TokenTree::Sequence { items, .. } => Some(items),
            _ => None,
        }
    }

    /// If this `TokenTree` is a `Group`, return its `open`, `content` and `close` subtrees.
    pub fn as_group(&self) -> Option<(&TokenTree, &TokenTree, &TokenTree)> {
        match self {
            TokenTree::Group {
                open,
                content,
                close,
                ..
            } => Some((open, content, close)),
            _ => None,
        }
    }

    /// If this `TokenTree` is a `Branch`, return its `left`, `delimiter` and `right` subtrees.
    pub fn as_branch(&self) -> Option<(&TokenTree, &TokenTree, &TokenTree)> {
        match self {
            TokenTree::Branch {
                left,
                delimiter,
                right,
                ..
            } => Some((left, delimiter, right)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Token, TokenTree};

    #[test]
    fn test_literal() {
        let token = Token::new(4, "test".into(), "test2".into());
        let error_token = Token::new_error(
            8,
            "error-test".into(),
            "tessssst".into(),
            "You did something wrong".to_string(),
        );

        {
            let mut literal = TokenTree::literal(token.clone());
            assert_eq!(4, literal.starts_at());
            assert_eq!(&"test".to_string(), literal.rule());
            assert_eq!(false, literal.is_error());
            assert_eq!(Some(&token), literal.as_token());
            assert_eq!(None, literal.as_sequence());
            assert_eq!(None, literal.as_branch());
            assert_eq!(None, literal.as_group());
            assert_eq!(None, literal.error_message());
            assert_eq!(0, literal.extras().len());
            literal.push_extra("Test extra".into());
            assert_eq!(1, literal.extras().len());
            assert_eq!("Test extra".to_string(), literal.extras()[0]);
        }

        {
            let error_literal = TokenTree::literal(error_token.clone());
            assert_eq!(true, error_literal.is_error());
            assert!(error_literal.error_message().is_some());
        }
    }
}
