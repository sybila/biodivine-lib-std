use crate::parsers::groups::*;
use crate::parsers::tokens::Token;

/// **(internal)** This little abomination is used to hold information about unfinished groups.
///
/// It is accessed as a stack, because we are emulating recursion.
///
/// - 'a: TokenTreeBuilder which owns the group templates.
/// - 'b: Token vector.
/// - 'c: Original string data.
///
/// - First element is a reference to the template which triggered the group.
/// - Second element is a reference to the opening token which triggered the group.
/// - Third element is the token forest which the group generates (this will become data
///    once the group is closed)
type GroupStack<'a, 'b, 'c, Payload> = Vec<(
    &'a GroupRule<Payload>,
    &'b Token<'c, Payload>,
    TokenForest<'c, Payload>,
)>;

/// Builds a tree structure from tokens based on group rules.
impl<Payload: Clone> TokenTreeBuilder<Payload> {
    /// Create a new builder from a list of group rules.
    pub fn new(templates: Vec<GroupRule<Payload>>) -> TokenTreeBuilder<Payload> {
        return TokenTreeBuilder {
            group_templates: templates,
        };
    }

    /// Transform a sequence of `Token`s into a `TokenForest` using the group rules in this builder.
    pub fn group_tokens<'a, 'b>(
        &'a self,
        tokens: &[Token<'b, Payload>],
    ) -> Result<TokenForest<'b, Payload>, GroupError> {
        let mut root_forest: TokenForest<Payload> = Vec::new();
        let mut group_stack: GroupStack<Payload> = Vec::new();
        for token in tokens {
            if let Some(template) = self.opens(token) {
                // We are starting a new group - push it to the stack and continue
                group_stack.push((template, token, Vec::new()));
            } else {
                // Try to close currently processed group with the current token
                if Self::try_to_close_group(token, &mut root_forest, &mut group_stack) {
                    continue;
                }
                // If this wasn't a closing token for the current group, check if it can close something else.
                if let Some(rule) = self.can_close(token) {
                    // In which case, this is an error.
                    return Err(GroupError::unexpected_group_end(rule, token));
                }
                // If it does not open or close current or any other group, we can push it to current forest:
                let value = TokenTree::Value(token.clone());
                Self::push_result(value, &mut root_forest, &mut group_stack);
            }
        }
        return if let Some((rule, token, _)) = group_stack.last() {
            Err(GroupError::unclosed_group(rule, token, None))
        } else {
            Ok(root_forest)
        };
    }

    /// Transform a sequence of `Token`s into a `TokenForest`, recovering from encountered errors if possible.
    ///
    /// There are two types of errors:
    ///  - Dangling unclosed group at the end of the sequence: this simply emits an error,
    /// since there is no "recovery" option there.
    ///  - Unexpected closing tag, which can be caused by two problems. (A) User might
    /// have forgotten to close the current group and this in fact closes a previous group
    /// (e.g. `{[}`). (B) this is a token for a group which was never opened (e.g.`{]}`).
    ///
    /// To recover from the second type of error, we consider all groups waiting to be closed
    /// and search for the nearest one that might be closed by the encountered token. If no such
    /// group is found, handle this as (B). If such a group is found (A), mark all unprocessed
    /// groups encountered after it as unclosed, close the found group and continue from this state.
    pub fn group_tokens_with_recovery<'a, 'b>(
        &'a self,
        tokens: &[Token<'a, Payload>],
    ) -> (TokenForest<'a, Payload>, Vec<GroupError>) {
        let mut root_forest: TokenForest<Payload> = Vec::new();
        let mut group_stack: GroupStack<Payload> = Vec::new();
        let mut errors: Vec<GroupError> = Vec::new();
        for token in tokens {
            if let Some(template) = self.opens(token) {
                group_stack.push((template, token, Vec::new()));
            } else {
                if Self::try_to_close_group(token, &mut root_forest, &mut group_stack) {
                    continue;
                }
                if let Some(closes) = self.can_close(token) {
                    // We found a closing token that is not supposed to be here - there are two options
                    // 1. It is really extra and we should ignore it
                    // 2. Current group is unclosed and this token closes some other group on the stack
                    // Recovery strategy: Try to find a group on the stack that this token closes.
                    // If found, pop every unclosed group with an error and then close the right group.
                    // If there is no group to close, emit this as unexpected closing token and continue.
                    let can_close_from_stack =
                        group_stack.iter().rev().fold(false, |a, (rule, start, _)| {
                            a || (rule.is_group)(start, token)
                        });
                    if can_close_from_stack {
                        // Pop unfinished groups until the one that matches is found.
                        while let Some((rule, start, forest)) = group_stack.pop() {
                            let closes = rule.is_group(start, token);
                            let group: TokenTree<Payload> = TokenTree::Group {
                                name: rule.name.clone(),
                                open: start.clone(),
                                close: if closes { Some(token.clone()) } else { None },
                                data: forest,
                            };
                            if closes {
                                // This rule closes the found token, so emit this as a properly closed group.
                                Self::push_result(group, &mut root_forest, &mut group_stack);
                                break;
                            } else {
                                // This rule is forcibly closed - emit it into the tree, but emit also a group error.
                                errors.push(GroupError::unclosed_group(rule, start, Some(token)));
                                Self::push_result(group, &mut root_forest, &mut group_stack);
                            }
                        }
                    } else {
                        // There is no way this token finishes anything on the stack
                        errors.push(GroupError::unexpected_group_end(closes, token));
                    }
                } else {
                    // If it does not open or close current or any other group, we can push it to current forest:
                    let value = TokenTree::Value(token.clone());
                    Self::push_result(value, &mut root_forest, &mut group_stack);
                }
            }
        }
        let mut tail_errors = Vec::new();
        while let Some((rule, start, forest)) = group_stack.pop() {
            // We are done and we still have unclosed groups! Push the unfinished groups into result, but emit errors for them.
            let group: TokenTree<Payload> = TokenTree::Group {
                name: rule.name.clone(),
                open: start.clone(),
                close: None,
                data: forest,
            };
            Self::push_result(group, &mut root_forest, &mut group_stack);
            tail_errors.push(GroupError::unclosed_group(rule, start, None));
        }
        // We want to emit errors from first to last, so we have to emit error in revers order
        // (because we have to push results in stack order, but this is not error order)
        for err in tail_errors.into_iter().rev() {
            errors.push(err);
        }
        return (root_forest, errors);
    }

    /// **(internal)** Try to close the last group on the `stack` with the given token.
    ///
    /// If the token indeed closes the group, remove it from the `stack` and push the corresponding
    /// `TokenTree` into the result (see `push_result`), returning `true`. Otherwise just
    /// return `false`.
    ///
    ///  - 'a: TokenTreeBuilder
    ///  - 'b: token vector
    ///  - 'c: original string data
    ///  - 'd: caller
    fn try_to_close_group<'a, 'b, 'c, 'd>(
        token: &'b Token<'c, Payload>,
        root: &'d mut TokenForest<'c, Payload>,
        stack: &'d mut GroupStack<'a, 'b, 'c, Payload>,
    ) -> bool {
        if let Some((rule, start, _)) = stack.last() {
            if rule.is_group(start, token) {
                // Found a closing token for the currently processed group!
                // Remove it from the stack and turn it into a `TokenTree`.
                let (rule, start, forest) = stack.pop().unwrap();
                let group: TokenTree<Payload> = TokenTree::Group {
                    name: rule.name.clone(),
                    open: start.clone(),
                    close: Some(token.clone()),
                    data: forest,
                };
                Self::push_result(group, root, stack);
                return true;
            };
        };
        return false;
    }

    /// **(internal)** Push a new `TokenTree` into the result data structures.
    ///
    /// If there is some unprocessed group, add it as a child of this group, otherwise add it
    /// as a child of the root token forest.
    ///
    ///  - 'a: Token tree builder which owns the group templates
    ///  - 'b: token vector
    ///  - 'c: original string data
    ///  - 'd: calling method
    fn push_result<'a, 'b, 'c, 'd>(
        token: TokenTree<'c, Payload>,
        root: &'d mut TokenForest<'c, Payload>,
        stack: &'d mut GroupStack<'a, 'b, 'c, Payload>,
    ) {
        let forest = stack.last_mut().map(|t| &mut t.2).unwrap_or(root);
        forest.push(token);
    }

    /// **(internal)** Returns a reference to a group template which uses given token as an opening
    /// token.
    fn opens(&self, token: &Token<Payload>) -> Option<&GroupRule<Payload>> {
        return (&self.group_templates).iter().find(|r| r.opens(token));
    }

    /// **(internal)** Returns a reference to the first group template which uses given token as
    /// a closing token.
    fn can_close(&self, token: &Token<Payload>) -> Option<&GroupRule<Payload>> {
        return (&self.group_templates).iter().find(|r| r.closes(token));
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::groups::{GroupRule, TokenForest, TokenTree, TokenTreeBuilder};
    use crate::parsers::tokens::{Token, TokenRule, Tokenizer};
    use crate::{const_data_group, const_token};

    fn tokenize(value: &str) -> Vec<Token<()>> {
        let tokenizer = Tokenizer::ignoring_whitespace(vec![
            const_token!(r"\(", ()),
            const_token!(r"\)", ()),
            const_token!(r"\[", ()),
            const_token!(r"\]", ()),
            const_token!(r"\{", ()),
            const_token!(r"\}", ()),
            const_token!(r"[a-z]+", ()),
        ]);
        return tokenizer.read(value).unwrap();
    }

    fn builder() -> TokenTreeBuilder<()> {
        return TokenTreeBuilder::new(vec![
            const_data_group!("parenthesis", "(", ")"),
            const_data_group!("brackets", "[", "]"),
            const_data_group!("block", "{", "}"),
        ]);
    }

    // Utility methods for easy testing
    impl<P: Clone> TokenTree<'_, P> {
        pub fn assert_children(&self) -> &TokenForest<P> {
            return self.children().unwrap();
        }
        pub fn assert_name(&self) -> &String {
            return self.name().unwrap();
        }
        pub fn assert_value(&self) -> &Token<P> {
            return self.value().unwrap();
        }
    }

    #[test]
    pub fn test_groups_with_recovery() {
        let tokens = tokenize("({}})([)[]{{");
        let builder = builder();
        let (forest, errors) = builder.group_tokens_with_recovery(&tokens);

        assert_eq!(errors.len(), 4);
        assert_eq!(errors[0].starts_at, None);
        assert_eq!(errors[0].ends_at, Some(3));
        assert_eq!(errors[1].starts_at, Some(6));
        assert_eq!(errors[1].ends_at, Some(7));
        assert_eq!(errors[2].starts_at, Some(10));
        assert_eq!(errors[2].ends_at, None);
        assert_eq!(errors[3].starts_at, Some(11));
        assert_eq!(errors[3].ends_at, None);

        assert_eq!(forest.len(), 4);
        assert_eq!(forest[0].assert_name(), "parenthesis");
        assert_eq!(forest[0].assert_children().len(), 1);
        assert_eq!(forest[0].assert_children()[0].assert_name(), "block");
        assert_eq!(forest[0].assert_children()[0].assert_children().len(), 0);
        assert_eq!(forest[1].assert_name(), "parenthesis");
        assert_eq!(forest[1].assert_children().len(), 1);
        assert_eq!(forest[1].assert_children()[0].assert_name(), "brackets");
        assert_eq!(forest[1].assert_children()[0].assert_children().len(), 0);
        assert_eq!(forest[2].assert_name(), "brackets");
        assert_eq!(forest[2].assert_children().len(), 0);
        assert_eq!(forest[3].assert_name(), "block");
        assert_eq!(forest[3].assert_children().len(), 1);
        assert_eq!(forest[3].assert_children()[0].name().unwrap(), "block");
        assert_eq!(forest[3].assert_children()[0].assert_children().len(), 0);
    }

    #[test]
    pub fn test_groups_unclosed() {
        let tokens = tokenize("{}(()()");
        let result = builder().group_tokens(&tokens);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.starts_at, Some(2));
        assert_eq!(error.ends_at, None)
    }

    #[test]
    pub fn test_groups_unexpected_close() {
        let tokens = tokenize("{}())()");
        let result = builder().group_tokens(&tokens);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error.starts_at, None);
        assert_eq!(error.ends_at, Some(4))
    }

    #[test]
    pub fn test_groups_simple() {
        let tokens = tokenize("(){[test]{ and () text }[]}");
        let builder: TokenTreeBuilder<()> = builder();
        // (){[test]{ and () text }[]}
        let groups = builder.group_tokens(&tokens).unwrap();
        assert_eq!(groups, builder.group_tokens_with_recovery(&tokens).0);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].assert_name(), "parenthesis");
        assert_eq!(groups[0].assert_children().len(), 0);
        assert_eq!(groups[1].assert_name(), "block");
        assert_eq!(groups[1].assert_children().len(), 3);
        // [test]{ and () text }[]
        let groups = groups[1].assert_children();
        assert_eq!(groups[0].assert_name(), "brackets");
        assert_eq!(groups[0].assert_children().len(), 1);
        assert_eq!(groups[0].assert_children()[0].assert_value().data, "test");
        assert_eq!(groups[1].assert_name(), "block");
        assert_eq!(groups[1].assert_children().len(), 3);
        assert_eq!(groups[1].assert_children()[0].assert_value().data, "and");
        assert_eq!(groups[1].assert_children()[1].assert_name(), "parenthesis");
        assert_eq!(groups[1].assert_children()[1].assert_children().len(), 0);
        assert_eq!(groups[1].assert_children()[2].assert_value().data, "text");
        assert_eq!(groups[2].assert_name(), "brackets");
        assert_eq!(groups[2].assert_children().len(), 0);
    }
}
