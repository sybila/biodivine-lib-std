//! Tokenizers are the "Tier 0" of our parsing architecture. They transform raw strings into
//! sequences of well defined tokens, however, without any other meaningful structure.
//!
//! Every `Token` carries information about its starting position in the original string (mostly
//! for error handling purposes), a reference to the original string (in case you want to
//! reconstruct more data from it) and a custom `Payload`.
//!
//! ### Token Rules
//!
//! To construct a `Tokenizer`, you have to provide a list of `TokenRule`s which will be
//! evaluated on the input string. A `TokenRule` consist of a regular expression which
//! will be matched against the string and a callback function that is used to construct the
//! `Payload` from the matched result.
//!
//! ```rust
//! use biodivine_lib_std::parsers::tokens::TokenRule;
//!
//! // Token that matches a constant string '<=>'
//! let eq_op_template = TokenRule::<Option<String>>::new("<=>", |_| None);
//! let matched = eq_op_template.try_match("<==>");
//! assert!(matched.is_none());
//! let (captures, payload) = eq_op_template.try_match("<=> x").unwrap();
//! assert_eq!(captures.get(0).unwrap().as_str(), "<=>");
//! assert!(payload.is_none());
//!
//! // Token which matches some identifier starting with '$'
//! let id_template = TokenRule::<Option<String>>::new(r"\$([a-z]+)", |c| {
//!     return Some(c.get(1).unwrap().as_str().to_string());
//! });
//! let (captures, payload) = id_template.try_match("$hello there").unwrap();
//! assert_eq!(captures.get(0).unwrap().as_str(), "$hello");
//! assert_eq!(payload, Some("hello".to_string()));
//! ```
//!
//! Note that `TokenRule::new` will automatically add the `^` anchor at the beginning
//! of the provided regex to ensure that the token starts at the inspected position.
//!
//! If the token payload does not depend on the captured string data, you can use `const_token!`
//! macro to generate token rules more concisely (see below).
//!
//! ### Tokenizers
//!
//! From several token rules, we can construct a `Tokenizer` - a tokenizer will try to
//! repeatedly match the rules on a provided string, separating it into individual tokens.
//! Note that rules are matched in the order in which they are provided, so you have
//! to always give the most specific tokens first (currently we do not perform any ambiguity
//! checks).
//!
//! `Tokenizer` also allows you to skip some contents of a string described by a `Regex`.
//! You can view this `Regex` as a special type of token that will be matched (as the first
//! rule) but does not emit any output. This defaults to whitespace, but can be also
//! easily extended to ignore basic comments:
//!
//! ``` rust
//! use biodivine_lib_std::const_token;
//! use biodivine_lib_std::parsers::tokens::{Tokenizer, TokenRule};
//!
//! // Create a tokenizer which will skip line comments starting with '#'
//! let tokenizer = Tokenizer::<Option<i32>>::new(r"(\s+|#.*\n)", vec![
//!     const_token!(r"\+", None),    // Plus operator
//!     const_token!(r"\*", None),   // Times operator
//!     TokenRule::new(r"-?\d+", |c| {
//!         Some(c.get(0).and_then(|m| m.as_str().parse::<i32>().ok()).unwrap())
//!     }) // Integer
//! ]);
//! let tokens = tokenizer.read("3 + 4 # line comment\n\t\t * 5").unwrap();
//! assert_eq!(tokens.len(), 5);
//! assert_eq!(tokens[0].data, "3");
//! assert_eq!(tokens[1].data, "+");
//! assert_eq!(tokens[2].data, "4");
//! assert_eq!(tokens[3].data, "*");
//! assert_eq!(tokens[4].data, "5");
//! ```
//!
//! Notice the `()` around the ignore regex - this is necessary because tokenizer will insert
//! the `^` anchor into the regex, which would otherwise only apply to the first alternation (`\s+`).
//! Also notice that since all of this are regular expressions, we have to escape symbols like `+`
//! or `*`.
//!
//! Finally, `Tokenizer` allows another neat feature: error recovery. You can ask the tokenizer
//! to recover from errors, which means that instead of failing, the tokenizer will emit an error
//! and then continue looking for a new token (for each consecutive sequence of unmatched tokens,
//! there is only one error):
//!
//! ```rust
//! # use biodivine_lib_std::const_token;
//! # use biodivine_lib_std::parsers::tokens::{Tokenizer, TokenRule};
//! # let tokenizer = Tokenizer::<Option<i32>>::new(r"(\s+|#.*\n)", vec![
//! #     const_token!(r"\+", None),    // Plus operator
//! #     const_token!(r"\*", None),   // Times operator
//! #     TokenRule::new(r"-?\d+", |c| Some(c.get(0).and_then(|m| m.as_str().parse().ok()).unwrap())) // Integer
//! # ]);
//! let error = tokenizer.read("3 - 4 * -13 hello + 5").err().unwrap();
//! assert_eq!(error.position, 2);
//! let (tokens, errors) = tokenizer.read_with_recovery("3 - 4 * -13 hello + 5");
//! assert_eq!(tokens.len(), 6);
//! assert_eq!(tokens[0].data, "3");
//! assert_eq!(tokens[1].data, "4");
//! assert_eq!(tokens[2].data, "*");
//! assert_eq!(tokens[3].data, "-13");
//! assert_eq!(tokens[4].data, "+");
//! assert_eq!(tokens[5].data, "5");
//! assert_eq!(errors.len(), 2);
//! assert_eq!(errors[0].position, 2);
//! assert_eq!(errors[1].position, 12);
//! ```
//!
//! This is especially useful when you are creating tools with syntax highlighting or other
//! interactive input elements, where you want to notify the user about all current errors.
//!

use regex::{Captures, Regex};
use std::fmt::Debug;

mod _impl_token_rule;
mod _impl_tokenizer;
mod _macro_token_rule;

/// A fragment of input text labeled with optional `Payload` data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a, Payload: Clone> {
    pub starts_at: usize,
    pub data: &'a str,
    pub payload: Payload,
}

/// Result of tokenization for an invalid string. Carries the error position and a human readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizerError {
    pub position: usize,
    pub message: String,
}

/// Executable `Regex`-based template used to isolate individual tokens and construct payloads for them.
///
/// Note that you can't use closures as factory functions - only pure functions allowed.
#[derive(Clone)]
pub struct TokenRule<Payload> {
    regex: Regex,
    factory: fn(&Captures) -> Payload,
}

/// Transforms a string using a set of provided `TokenRules`s into a vector of `Token`s.
pub struct Tokenizer<Payload> {
    pub ignore: Option<Regex>,
    templates: Vec<TokenRule<Payload>>,
}

impl<Payload: Clone> Token<'_, Payload> {
    /// A utility constructor.
    pub fn new(starts_at: usize, data: &str, payload: Payload) -> Token<Payload> {
        return Token {
            starts_at,
            data,
            payload,
        };
    }
}

impl TokenizerError {
    /// A utility constructor.
    pub fn new(data: &str, position: usize) -> TokenizerError {
        return TokenizerError {
            position,
            message: format!("Unexpected character '{}'.", &data[position..position + 1]),
        };
    }
}
