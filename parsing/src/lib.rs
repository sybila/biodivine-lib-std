mod _impl_token;

/// Annotated subsequence of the input string.
///
/// Each token has a `value`, an associated `rule` name and a starting
/// position in the input string. It can possibly contain other `extra`
/// metadata provided by the rule which created it.
///
/// If the rule name starts with `error`, then the token also must have
/// an associated human readable error message.
///
/// Note that token `value` can be empty. This typically does not happen
/// directly in the tokenizer but is introduced as error handling
/// measure in further processing stages.
pub struct Token {
    starts_at: usize,
    meta: Vec<String>, // [rule, value, message (is error), extras, ...]
}

pub enum TokenTree {
    Literal {
        token: Token,
    },
    Sequence {
        meta: Vec<String>,
        items: Vec<TokenTree>,
    },
    Group {
        open: Box<TokenTree>,
        close: Box<TokenTree>,
        inner: Box<TokenTree>,
    },
    Branch {
        delimiter: Box<TokenTree>,
        left: Box<TokenTree>,
        right: Box<TokenTree>,
    },
}
