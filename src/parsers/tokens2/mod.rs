use regex::Regex;

mod _impl_group_token_matchers;
mod _impl_static_token_matchers;
mod _impl_switch_token_matchers;
mod _impl_token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    starts_at: usize,
    data: Vec<String>, // rule, value, extras, ...
}

type TokenMatcherBox<S> = Box<dyn TokenMatcher<S>>;
type StaticTokenMatcherBox = Box<dyn StaticTokenMatcher>;

/// Token matcher extracts a payload of a single token from a string. The matching can
/// depend on a mutable state that is updated by the matcher during the tokenization.
///
/// The payload is a vector of strings representing (in this order):
///  - name of the matched rule
///  - value of the matched token
///  - any additional string data
pub trait TokenMatcher<S> {
    fn clean_state(&self) -> S;
    fn scan_token(&self, state: &mut S, data: &str) -> Option<Vec<String>>;
}

/// A simplified version of the `TokenMatcher` - it assumes tokenization has no state.
/// It has a blanket implementation of `TokenMatcher<S>` (for any `S` that implements `Default`)
/// so it can be actually used in place of any normal `TokenMatcher`.
pub trait StaticTokenMatcher {
    fn scan_token_static(&self, data: &str) -> Option<Vec<String>>;
}

/// Blanket implementation of `TokenMatcher<S>` for any `StaticTokenMatcher` - allows
/// using static matcher where normal matcher is expected.
impl<T: StaticTokenMatcher, S: Default> TokenMatcher<S> for T {
    fn clean_state(&self) -> S {
        return S::default();
    }

    fn scan_token(&self, _: &mut S, data: &str) -> Option<Vec<String>> {
        return self.scan_token_static(data);
    }
}

/// A `StaticTokenMatcher` that always matches a fixed string value.
#[derive(Clone, Debug)]
pub struct ConstTokenMatcher {
    name: String,
    value: String,
}

/// A `StaticTokenMatcher` that matches a value based on a regular expression.
///
/// Extras: all extra groups specified in the regex.
#[derive(Clone, Debug)]
pub struct RegexTokenMatcher {
    name: String,
    regex: Regex,
}

/// A `StaticTokenMatcher` that contains a sequence of inner token matchers and applies them
/// in order as fallback if the previous matchers failed.
pub struct SequenceTokenMatcher(Vec<StaticTokenMatcherBox>);

/// A `StaticTokenMatcher` that will match everything on the input until a match for the
/// `until` token matcher is encountered.
///
/// This "stopping" token will not be included in the token value, but it will be pushed into
/// the extra values of the resulting token.
pub struct WeakUntilTokenMatcher {
    name: String,
    until: StaticTokenMatcherBox,
}

/// A token matcher that will try to match a group delimited using the given `open`/`close`
/// matchers, using a dedicated `body` matcher for tokens inside the group.
pub struct GroupTokenMatcher<S> {
    open: StaticTokenMatcherBox,
    close: StaticTokenMatcherBox,
    body: TokenMatcherBox<S>,
}

/// A token matcher that will match groups (like `GroupTokenMatcher`), but does so recursively,
/// keeping track about nesting using the shared state.
pub struct RecursiveGroupTokenMatcher<S> {
    open: StaticTokenMatcherBox,
    close: StaticTokenMatcherBox,
    body: TokenMatcherBox<S>,
}

/// A token matcher that behaves similar to the `SequenceTokenMatcher`, but as soon as the
/// state of one of the children becomes `Some`, it will only match this child until that
/// state is not `None` again.
pub struct SwitchTokenMatcher<L, R> {
    left: TokenMatcherBox<Option<L>>,
    right: TokenMatcherBox<Option<R>>
}
