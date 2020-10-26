use crate::parsers::tokens2::{
    GroupTokenMatcher, RecursiveGroupTokenMatcher, StaticTokenMatcherBox, TokenMatcher,
    TokenMatcherBox,
};

impl<S> GroupTokenMatcher<S> {
    pub fn new(
        open: StaticTokenMatcherBox,
        close: StaticTokenMatcherBox,
        body: TokenMatcherBox<S>,
    ) -> GroupTokenMatcher<S> {
        return GroupTokenMatcher { open, close, body };
    }
}

impl<S> RecursiveGroupTokenMatcher<S> {
    pub fn new(
        open: StaticTokenMatcherBox,
        close: StaticTokenMatcherBox,
        body: TokenMatcherBox<S>,
    ) -> RecursiveGroupTokenMatcher<S> {
        return RecursiveGroupTokenMatcher { open, close, body };
    }
}

impl<S> TokenMatcher<Option<S>> for GroupTokenMatcher<S> {
    fn clean_state(&self) -> Option<S> {
        return None;
    }

    fn scan_token(&self, state: &mut Option<S>, data: &str) -> Option<Vec<String>> {
        return if let Some(inner_state) = state {
            // We are in a group - try to close it, if not possible, read body.
            let close = self.close.scan_token_static(data);
            if close.is_some() {
                *state = None;
                return close;
            }
            self.body.scan_token(inner_state, data)
        } else {
            // We are not reading the group - try to open it.
            let open = self.open.scan_token_static(data);
            if open.is_some() {
                *state = Some(self.body.clean_state());
            }
            open
        };
    }
}

// We have to use Option because we want to have all "conditional" matchers using Option
// so that they can be later connected using some either-like construct.
// Here, it holds that the vector is never empty if it exists.
impl<S> TokenMatcher<Option<Vec<S>>> for RecursiveGroupTokenMatcher<S> {
    fn clean_state(&self) -> Option<Vec<S>> {
        return None;
    }

    fn scan_token(&self, state: &mut Option<Vec<S>>, data: &str) -> Option<Vec<String>> {
        return if let Some(stack) = state {
            let open = self.open.scan_token_static(data);
            if open.is_some() {
                stack.push(self.body.clean_state());
                return open;
            }
            let close = self.close.scan_token_static(data);
            if close.is_some() {
                stack.pop();
                if stack.is_empty() {
                    *state = None;
                }
                return close;
            }
            self.body.scan_token(stack.last_mut().unwrap(), data)
        } else {
            let open = self.open.scan_token_static(data);
            if open.is_some() {
                *state = Some(vec![self.body.clean_state()]);
            }
            open
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::tokens2::{
        ConstTokenMatcher, GroupTokenMatcher, RecursiveGroupTokenMatcher, SequenceTokenMatcher,
        TokenMatcher, WeakUntilTokenMatcher,
    };

    #[test]
    pub fn test_group_token_matcher() {
        // A matcher for escaped strings (an escaped string can be handled also a little
        // more easily, but you then don't have each escaped character as separate token,
        // so good luck with syntax highlighting).
        let quote = ConstTokenMatcher::new("quote", "\"");
        let quote_esc = ConstTokenMatcher::new("quote-esc", "\\\"");
        let string_content_end =
            SequenceTokenMatcher::new(vec![Box::new(quote_esc.clone()), Box::new(quote.clone())]);
        let string_content =
            WeakUntilTokenMatcher::new("string-content", Box::new(string_content_end));
        let string_matcher: GroupTokenMatcher<()> = GroupTokenMatcher::new(
            Box::new(quote.clone()),
            Box::new(quote.clone()),
            Box::new(SequenceTokenMatcher(vec![
                Box::new(quote_esc.clone()),
                Box::new(string_content),
            ])),
        );
        let mut state = string_matcher.clean_state();
        assert!(state.is_none());
        let m = string_matcher.scan_token(&mut state, "not string \"string\\\"with escapes\"");
        assert!(state.is_none());
        assert!(m.is_none());
        let m = string_matcher
            .scan_token(&mut state, "\"string\\\"with escapes\"")
            .unwrap();
        assert!(state.is_some());
        assert_eq!(&m[0], "quote");
        let m = string_matcher
            .scan_token(&mut state, "string\\\"with escapes\"")
            .unwrap();
        assert!(state.is_some());
        assert_eq!(&m[0], "string-content");
        assert_eq!(&m[1], "string");
        let m = string_matcher
            .scan_token(&mut state, "\\\"with escapes\"")
            .unwrap();
        assert!(state.is_some());
        assert_eq!(&m[0], "quote-esc");
        let m = string_matcher
            .scan_token(&mut state, "with escapes\"")
            .unwrap();
        assert!(state.is_some());
        assert_eq!(&m[0], "string-content");
        assert_eq!(&m[1], "with escapes");
        let m = string_matcher.scan_token(&mut state, "\"").unwrap();
        assert!(state.is_none());
        assert_eq!(&m[0], "quote");
    }

    #[test]
    pub fn test_recursive_group_token_matcher() {
        // A matcher for nested block comments. As opposed to strings, this typically can't be
        // handled in a more clever way, unless you do it in some higher
        // level of the parsing process.
        let open = ConstTokenMatcher::new("block-open", "/*");
        let close = ConstTokenMatcher::new("block-close", "*/");
        let block_content_end =
            SequenceTokenMatcher::new(vec![Box::new(open.clone()), Box::new(close.clone())]);
        let block_content =
            WeakUntilTokenMatcher::new("block-content", Box::new(block_content_end));
        let block_matcher: RecursiveGroupTokenMatcher<()> = RecursiveGroupTokenMatcher::new(
            Box::new(open.clone()),
            Box::new(close.clone()),
            Box::new(block_content),
        );
        let mut state = block_matcher.clean_state();
        assert!(state.is_none());
        let m =
            block_matcher.scan_token(&mut state, "not comment /* block /* comment */ string */");
        assert!(state.is_none());
        assert!(m.is_none());
        let m = block_matcher
            .scan_token(&mut state, "/* block /* comment */ string */")
            .unwrap();
        assert_eq!(state.clone().unwrap().len(), 1);
        assert_eq!(&m[0], "block-open");
        let m = block_matcher
            .scan_token(&mut state, " block /* comment */ string */")
            .unwrap();
        assert_eq!(state.clone().unwrap().len(), 1);
        assert_eq!(&m[0], "block-content");
        assert_eq!(&m[1], " block ");
        let m = block_matcher
            .scan_token(&mut state, "/* comment */ string */")
            .unwrap();
        assert_eq!(state.clone().unwrap().len(), 2);
        assert_eq!(&m[0], "block-open");
        let m = block_matcher
            .scan_token(&mut state, " comment */ string */")
            .unwrap();
        assert_eq!(state.clone().unwrap().len(), 2);
        assert_eq!(&m[0], "block-content");
        assert_eq!(&m[1], " comment ");
        let m = block_matcher
            .scan_token(&mut state, "*/ string */")
            .unwrap();
        assert_eq!(state.clone().unwrap().len(), 1);
        assert_eq!(&m[0], "block-close");
        let m = block_matcher.scan_token(&mut state, " string */").unwrap();
        assert_eq!(state.clone().unwrap().len(), 1);
        assert_eq!(&m[0], "block-content");
        assert_eq!(&m[1], " string ");
        let m = block_matcher.scan_token(&mut state, "*/").unwrap();
        assert!(state.is_none());
        assert_eq!(&m[0], "block-close");
    }
}
