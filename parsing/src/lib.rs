mod _impl_iterator_for_extras;
mod _impl_token;
mod _impl_token_tree;

/// Annotated subsequence of the input string.
///
/// Each token has a `value`, an associated `rule` name and a starting
/// position in the input string. It can possibly contain other `extra`
/// metadata provided by the rule which created it.
///
/// The token can be declared to be an *error token*. Then it must also have
/// an associated human readable `error_message`.
///
/// Note that token `value` can be empty. This can be used to declare errors
/// for missing values, but is more common later on for `TokenTrees`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    starts_at: usize,
    has_error: bool,
    meta: Vec<String>, // [rule, value, message (if error), extras, ...]
}

/// A collection of tokens with some given structural hierarchy.
///
/// `TokenTree` inherits most properties of `Token`, but adds a reference to its child trees
/// (`children`) and its `TokenTreeType` (`type`). Child trees allow referencing other token
/// trees deeper in the hierarchy. The number and structure of child trees is dictated by
/// the tree type.
///
/// Properties `starts_at`, `rule`, `has_error`, `error_message` and `extras` work the same way
/// as in `Token`. `TokenTree` has no `value`, but a `TokenTree` corresponding to a `Token`
/// (`TokenTreeType::LITERAL`) can be converted back to a `Token` to obtain the value.
///
/// Currently, there are four basic `TokenTreeTypes`:
///  - `LITERAL`: Corresponds to a `Token`. Has no child trees.
///  - `SEQUENCE`: General sequence with an arbitrary number of child trees.
///  - `GROUP`: Three child trees — `open`, `close`, and `content`. The rules for
/// creating groups match the `open` and `close` child trees and the `content` is inferred.
///  - `BRANCH`: Also three child trees — `left`, `right` and `delimiter`. Contrary to `GROUP`,
/// here `delimiter` is matched and `left/right` are inferred.
///
/// Note that both `LITERAL` and `SEQUENCE` types can contain empty `value` and `children`. This
/// can happen as a result of error propagation (some rule creates empty token/sequence to
/// declare some missing value), but it can be also a result of filtering, e.g. when removing
/// comments or unused tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenTree {
    tree_type: TokenTreeType,
    starts_at: usize,
    has_error: bool,
    meta: Vec<String>, // [rule, value (if literal), message (if error), extras, ...]
    // Group: [open, content, close]
    // Branch: [left, delimiter, right]
    // In general, should be sorted by order of appearance in the source string.
    children: Vec<TokenTree>,
}

/// Possible types of a `TokenTree`. See `TokenTree` for explanation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenTreeType {
    LITERAL,
    SEQUENCE,
    GROUP,
    BRANCH,
}

/// Iterator over metadata.
pub struct Extras<'a> {
    meta: &'a Vec<String>,
    skip: usize,
    index: usize,
}
