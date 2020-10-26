use crate::parsers::tokens2::{SwitchTokenMatcher, TokenMatcherBox, TokenMatcher};

impl<L, R> SwitchTokenMatcher<L, R> {

    pub fn new(left: TokenMatcherBox<Option<L>>, right: TokenMatcherBox<Option<R>>) -> SwitchTokenMatcher<L, R> {
        return SwitchTokenMatcher { left, right };
    }

}

impl<L, R> TokenMatcher<(Option<L>, Option<R>)> for SwitchTokenMatcher<L, R> {
    fn clean_state(&self) -> (Option<L>, Option<R>) {
        return (self.left.clean_state(), self.right.clean_state());
    }

    fn scan_token(&self, state: &mut (Option<L>, Option<R>), data: &str) -> Option<Vec<String>> {
        if state.0.is_some() {
            return self.left.scan_token(&mut state.0, data);
        } else if state.1.is_some() {
            return self.right.scan_token(&mut state.1, data);
        } else {
            let left = self.left.scan_token(&mut state.0, data);
            if left.is_some() {
                return left;
            }
            return self.right.scan_token(&mut state.1, data);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::tokens2::{ConstTokenMatcher, SequenceTokenMatcher, WeakUntilTokenMatcher, GroupTokenMatcher, RegexTokenMatcher, SwitchTokenMatcher, TokenMatcher};

    #[test]
    pub fn test_switch_token_matcher() {
        // Matches a string with escape chars
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
        // Some other things
        let whitespace = RegexTokenMatcher::new("whitespace", r"\s+");
        let identifier = RegexTokenMatcher::new("identifier", r"[a-z]+");
        let plus = ConstTokenMatcher::new("plus", "+");
        let not_string = SequenceTokenMatcher::new(vec![
            Box::new(whitespace), Box::new(identifier), Box::new(plus),
        ]);

        // Technically, right has no state, so only left can be "locked", but we can pretend it has.
        let switch: SwitchTokenMatcher<(), ()> = SwitchTokenMatcher::new(Box::new(string_matcher), Box::new(not_string));
        let mut state = switch.clean_state();
        assert!(state.0.is_none() && state.1.is_none());
        let m = switch.scan_token(&mut state, "hello +\"str\\\"\"\n ++").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "identifier");
        assert_eq!(&m[1], "hello");
        let m = switch.scan_token(&mut state, " +\"str\\\"\"\n ++").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "whitespace");
        assert_eq!(&m[1], " ");
        let m = switch.scan_token(&mut state, "+\"str\\\"\"\n ++").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "plus");
        assert_eq!(&m[1], "+");
        let m = switch.scan_token(&mut state, "\"str\\\"\"\n ++").unwrap();
        assert!(state.0.is_some() && state.1.is_none());
        assert_eq!(&m[0], "quote");
        assert_eq!(&m[1], "\"");
        let m = switch.scan_token(&mut state, "str\\\"\"\n ++").unwrap();
        assert!(state.0.is_some() && state.1.is_none());
        assert_eq!(&m[0], "string-content");
        assert_eq!(&m[1], "str");
        let m = switch.scan_token(&mut state, "\\\"\"\n ++").unwrap();
        assert!(state.0.is_some() && state.1.is_none());
        assert_eq!(&m[0], "quote-esc");
        assert_eq!(&m[1], "\\\"");
        let m = switch.scan_token(&mut state, "\"\n ++").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "quote");
        assert_eq!(&m[1], "\"");
        let m = switch.scan_token(&mut state, "\n ++").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "whitespace");
        assert_eq!(&m[1], "\n ");
        let m = switch.scan_token(&mut state, "++").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "plus");
        let m = switch.scan_token(&mut state, "+").unwrap();
        assert!(state.0.is_none() && state.1.is_none());
        assert_eq!(&m[0], "plus");
    }

}