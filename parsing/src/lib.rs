mod _impl_token;
mod _impl_token_iterator_and_indexing;
mod _impl_token_tree;

/// Annotated subsequence of the input string.
///
/// Each token has a `value`, an associated `rule` name and a starting
/// position in the input string. It can possibly contain other `extra`
/// metadata provided by the rule which created it.
///
/// The token can be declared to be an *error token*. Then it must also have
/// an associated human readable error message.
///
/// Note that token `value` can be empty. This typically does not happen
/// directly in the tokenizer but is introduced as error handling
/// measure in further processing stages to propagate errors about missing
/// content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    starts_at: usize,
    has_error: bool,
    meta: Vec<String>, // [rule, value, message (if error), extras, ...]
}

/// A collection of tokens with some given structural hierarchy.
///
/// Basic building blocks of a `TokenTree` are the `Literal`, which simply
/// wraps an existing `Token`, and a `Sequence` which wraps together
/// several other `TokenTrees`.
///
/// `Group` and `Branch` allow more granular characterisation. `Group` declares
/// three child `TokenTrees`: `open`, `content` and `close`. `Branch` also declares
/// three children, but their semantics is different: `left`, `delimiter` and `right`.
///
/// While `Group` specifies that the `content` is enclosed in the `open` and `close`
/// subtrees (i.e. `open` and `close` are matched, `content` is inferred), in `Branch`,
/// the `delimiter` (matched) separates the `left` and `right` subtrees (inferred).
///
/// Similar to `Token`, each `TokenTree` has a `rule` string which describes the rule
/// from which it was created. Also, the same conditions about `error` rules apply
/// (rule starting with an `error` must have a human readable error message).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenTree {
    Literal {
        token: Token,
    },
    Sequence {
        meta: Vec<String>,
        items: Vec<TokenTree>,
    },
    Group {
        meta: Vec<String>,
        open: Box<TokenTree>,
        close: Box<TokenTree>,
        content: Box<TokenTree>,
    },
    Branch {
        meta: Vec<String>,
        delimiter: Box<TokenTree>,
        left: Box<TokenTree>,
        right: Box<TokenTree>,
    },
}

/// Iterator over token metadata.
pub struct TokenExtras<'a> {
    token: &'a Token,
    index: usize,
}
