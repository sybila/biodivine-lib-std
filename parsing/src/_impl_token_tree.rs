use crate::{Extras, Token, TokenTree, TokenTreeType};
use std::ops::Index;

/// Constructors for `TokenTrees`.
impl TokenTree {
    /// Create a literal `TokenTree` from a `Token`.
    pub fn new_literal(token: Token) -> TokenTree {
        return TokenTree {
            tree_type: TokenTreeType::LITERAL,
            starts_at: token.starts_at,
            has_error: false,
            meta: token.meta,
            children: vec![],
        };
    }

    /// Create a sequence of child `TokenTrees`.
    pub fn new_sequence(rule: &str, children: Vec<TokenTree>) -> TokenTree {
        if children.is_empty() {
            panic!("Cannot create an empty sequence for {}.", rule);
        }
        return TokenTree {
            tree_type: TokenTreeType::SEQUENCE,
            starts_at: children[0].starts_at(),
            has_error: false,
            meta: vec![rule.into()],
            children,
        };
    }

    /// Create a group `TokenTree`.
    pub fn new_group(
        rule: &str,
        open: TokenTree,
        content: TokenTree,
        close: TokenTree,
    ) -> TokenTree {
        return TokenTree {
            tree_type: TokenTreeType::GROUP,
            starts_at: open.starts_at(),
            has_error: false,
            meta: vec![rule.into()],
            children: vec![open, content, close],
        };
    }

    /// Create a branch `TokenTree`.
    pub fn new_branch(
        rule: &str,
        left: TokenTree,
        delimiter: TokenTree,
        right: TokenTree,
    ) -> TokenTree {
        return TokenTree {
            tree_type: TokenTreeType::BRANCH,
            starts_at: left.starts_at(),
            has_error: false,
            meta: vec![rule.into()],
            children: vec![left, delimiter, right],
        };
    }

    /// Take a `TokenTree` and return the same tree with an error.
    pub fn with_error(mut self, message: &str) -> TokenTree {
        if self.has_error {
            panic!("Tree already has an error: {}.", self.error_message());
        }
        self.meta.insert(self.header_len(), message.into());
        self.has_error = true;
        return self;
    }
}

/// Basic properties common for all `TokenTrees`.
impl TokenTree {
    /// Type of this `TokenTree`.
    pub fn tree_type(&self) -> TokenTreeType {
        return self.tree_type;
    }

    /// Index of the first character of this `TokenTree` in the input string.
    pub fn starts_at(&self) -> usize {
        return self.starts_at;
    }

    /// Name of the rule that generated this `TokenTree`.
    pub fn rule(&self) -> &str {
        return &self.meta[0];
    }

    /// An iterator over string extras attached to this `TokenTree`.
    pub fn extras(&self) -> Extras {
        /* In literals, error message is at third, everywhere else it is second. */
        return Extras::new(&self.meta, self.header_len());
    }

    /// Safely obtain token tree extra at the given position.
    pub fn get_extra(&self, index: usize) -> Option<&str> {
        return self.meta.get(index + self.header_len()).map(|i| i.as_str());
    }

    /// Add extra string data `value` to this `TokenTree`.
    pub fn push_extra(&mut self, value: &str) {
        self.meta.push(value.into());
    }

    /// True if this is an error `TokenTree` (does not consider child trees).
    pub fn has_error(&self) -> bool {
        return self.has_error;
    }

    /// If this `TokenTree` has an error, return its error message, otherwise return `None`.
    pub fn get_error_message(&self) -> Option<&str> {
        return match self.has_error {
            /* Error message is always the last item in the header. */
            true => self.meta.get(self.header_len() - 1).map(|s| s.as_str()),
            false => None,
        };
    }

    /// Return error message if this `TokenTree` has error, otherwise panic.
    pub fn error_message(&self) -> &str {
        return self
            .get_error_message()
            .unwrap_or_else(|| panic!("Token {:?} has no error message.", self));
    }

    /// Return a slice of all children of this `TokenTree` (useful for tree traversal).
    pub fn children(&self) -> &[TokenTree] {
        return &self.children;
    }

    /// **(internal)** Number of semantic elements in the `meta` vector before the extras start.
    fn header_len(&self) -> usize {
        return 1 + if self.is_literal() { 1 } else { 0 } + if self.has_error { 1 } else { 0 };
    }
}

/// Special methods for extracting data from `TokenTree` based on its `TokenTreeType`.
impl TokenTree {
    pub fn is_literal(&self) -> bool {
        return self.tree_type == TokenTreeType::LITERAL;
    }

    pub fn is_sequence(&self) -> bool {
        return self.tree_type == TokenTreeType::SEQUENCE;
    }

    pub fn is_group(&self) -> bool {
        return self.tree_type == TokenTreeType::GROUP;
    }

    pub fn is_branch(&self) -> bool {
        return self.tree_type == TokenTreeType::BRANCH;
    }

    pub fn get_literal_value(&self) -> Option<&str> {
        return match self.is_literal() {
            true => self.meta.get(1).map(|i| i.as_str()),
            false => None,
        };
    }

    pub fn literal_value(&self) -> &str {
        return match self.is_literal() {
            true => &self.meta[1],
            false => panic!("Reading `value` of {:?}.", self.tree_type),
        };
    }

    pub fn get_sequence(&self) -> Option<&[TokenTree]> {
        return match self.is_sequence() {
            true => Some(&self.children),
            false => None,
        };
    }

    pub fn sequence(&self) -> &[TokenTree] {
        return self
            .get_sequence()
            .unwrap_or_else(|| panic!("Reading `sequence` of {:?}.", self.tree_type));
    }

    pub fn get_group_open(&self) -> Option<&TokenTree> {
        return match self.is_group() {
            true => Some(&self.children[0]),
            false => None,
        };
    }

    pub fn group_open(&self) -> &TokenTree {
        return self
            .get_group_open()
            .unwrap_or_else(|| panic!("Reading `open` of {:?}.", self.tree_type));
    }

    pub fn get_group_content(&self) -> Option<&TokenTree> {
        return match self.is_group() {
            true => Some(&self.children[1]),
            false => None,
        };
    }

    pub fn group_content(&self) -> &TokenTree {
        return self
            .get_group_content()
            .unwrap_or_else(|| panic!("Reading `content` of {:?}.", self.tree_type));
    }

    pub fn get_group_close(&self) -> Option<&TokenTree> {
        return match self.is_group() {
            true => Some(&self.children[2]),
            false => None,
        };
    }

    pub fn group_close(&self) -> &TokenTree {
        return self
            .get_group_close()
            .unwrap_or_else(|| panic!("Reading `close` of {:?}.", self.tree_type));
    }

    pub fn get_group(&self) -> Option<(&TokenTree, &TokenTree, &TokenTree)> {
        return match self.is_group() {
            true => Some((&self.children[0], &self.children[1], &self.children[2])),
            false => None,
        };
    }

    pub fn group(&self) -> (&TokenTree, &TokenTree, &TokenTree) {
        return self
            .get_group()
            .unwrap_or_else(|| panic!("Reading `group` of {:?}.", self.tree_type));
    }

    pub fn get_branch_left(&self) -> Option<&TokenTree> {
        return match self.is_branch() {
            true => Some(&self.children[0]),
            false => None,
        };
    }

    pub fn branch_left(&self) -> &TokenTree {
        return self
            .get_branch_left()
            .unwrap_or_else(|| panic!("Reading `left` of {:?}.", self.tree_type));
    }

    pub fn get_branch_delimiter(&self) -> Option<&TokenTree> {
        return match self.is_branch() {
            true => Some(&self.children[1]),
            false => None,
        };
    }

    pub fn branch_delimiter(&self) -> &TokenTree {
        return self
            .get_branch_delimiter()
            .unwrap_or_else(|| panic!("Reading `delimiter` of {:?}.", self.tree_type));
    }

    pub fn get_branch_right(&self) -> Option<&TokenTree> {
        return match self.is_branch() {
            true => Some(&self.children[2]),
            false => None,
        };
    }

    pub fn branch_right(&self) -> &TokenTree {
        return self
            .get_branch_right()
            .unwrap_or_else(|| panic!("Reading `right` of {:?}.", self.tree_type));
    }

    pub fn get_branch(&self) -> Option<(&TokenTree, &TokenTree, &TokenTree)> {
        return match self.is_branch() {
            true => Some((&self.children[0], &self.children[1], &self.children[2])),
            false => None,
        };
    }

    pub fn branch(&self) -> (&TokenTree, &TokenTree, &TokenTree) {
        return self
            .get_branch()
            .unwrap_or_else(|| panic!("Reading `branch` of {:?}.", self.tree_type));
    }
}

/// Indexing into the `extras` array of a `TokenTree`.
impl Index<usize> for TokenTree {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.meta[index + if self.has_error { 3 } else { 2 }];
    }
}

#[cfg(test)]
mod tests {
    use crate::{Token, TokenTree};

    #[test]
    fn token_tree_literal() {
        let mut literal = TokenTree::new_literal(Token::new(10, "my-rule", "my-value"));
        assert!(!literal.has_error);
        assert!(literal.is_literal());
        assert!(literal.children().is_empty());
        assert_eq!(literal.starts_at(), 10);
        assert_eq!(literal.rule(), "my-rule");
        assert_eq!(literal.literal_value(), "my-value");

        assert_eq!(literal.extras().count(), 0);
        assert_eq!(literal.get_extra(0), None);
        literal.push_extra("extra-value");
        assert_eq!(literal.extras().count(), 1);
        assert_eq!(literal.get_extra(0), Some("extra-value"));
        assert_eq!(&literal[0], "extra-value");

        let literal = literal.with_error("My Error!");
        assert!(literal.has_error);
        assert_eq!(literal.error_message(), "My Error!");
    }

    #[test]
    fn token_tree_sequence() {
        let t = Token::new(10, "my-rule", "my-value");
        let l = TokenTree::new_literal(t);
        let sequence =
            TokenTree::new_sequence("sequence-rule", vec![l.clone(), l.clone(), l.clone()]);
        assert!(sequence.is_sequence());
        assert_eq!(sequence.children().len(), 3);
        assert_eq!(sequence.sequence().len(), 3);
        assert_eq!(sequence.starts_at, 10);
        assert_eq!(sequence.rule(), "sequence-rule");
    }

    #[test]
    fn token_tree_group() {
        let t = Token::new(10, "my-rule", "my-value");
        let l = TokenTree::new_literal(t);
        let group = TokenTree::new_group("group-rule", l.clone(), l.clone(), l.clone());
        assert!(group.is_group());
        assert_eq!(group.children().len(), 3);
        assert_eq!(group.group(), (&l.clone(), &l.clone(), &l.clone()));
        assert_eq!(group.starts_at, 10);
        assert_eq!(group.rule(), "group-rule");
    }

    #[test]
    fn token_tree_branch() {
        let t = Token::new(10, "my-rule", "my-value");
        let l = TokenTree::new_literal(t);
        let group = TokenTree::new_branch("branch-rule", l.clone(), l.clone(), l.clone());
        assert!(group.is_branch());
        assert_eq!(group.children().len(), 3);
        assert_eq!(group.branch(), (&l.clone(), &l.clone(), &l.clone()));
        assert_eq!(group.starts_at, 10);
        assert_eq!(group.rule(), "branch-rule");
    }
}
