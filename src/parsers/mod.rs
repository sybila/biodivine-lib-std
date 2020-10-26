//! Utility methods for writing simple parsers.
//!
//! Note that we are definitely not trying to be super fast (no zero-allocation or whatever parsers
//! here). Just creating an architecture that will work well with a lot of different formats.
//! So if you need to parse gigabytes of data, don't use this! Also, if you need to parse XML,
//! JSON or some standard format, just use some standard library please. This is mostly for
//! small ad-hoc formats (logical formulas, graph descriptions, etc.).
//!
//! Why not use existing rust libraries for custom parsing?
//!
//! For grammar-based parsers, the reason
//! is mostly that they are hard to re-use efficiently. We currently have a lot of logic-like
//! and math-like langauges that need to be maintained and they all share common features which
//! would have to be re-implemented in typical grammar-based parsers.
//!
//! For parser combinators, the reasons are mostly in error handling and support for tooling.
//! By separating the process into multiple tiers, we provide a way to better utilize intermediate
//! results for static analysis, error recovery or when building interactive editors.
//!
//! Our parsing architecture consists of three "tiers" where each extracts a specific type of
//! structural properties from the input. Each tier also supports parsing with error recovery,
//! making for example live editors with highlighting possible.
//!
//! ### Tier 0 - `Tokenizer`
//!
//! Tokenizers split the input string into well defined character sequences called tokens,
//! possibly skipping some non-semantic characters (whitespace, comments, etc.) or recovering from
//! basic errors.
//!
//! Each token is defined using a regular expression template and can contain some *payload*
//! data created from the matched regex by a pure function factory:
//!
//! ```rust
//! use biodivine_lib_std::const_token;
//! use biodivine_lib_std::parsers::tokens::{Tokenizer, TokenRule, Token};
//!
//! // A simple payload data type for our tokens:
//! #[derive(Clone, Eq, PartialEq)]
//! enum Payload {
//!     ParOpen, ParClose, BracketOpen, BracketClose,
//!     Plus, Times, Comma, Literal(i32), Identifier(String)
//! }
//!
//! // A tokenizer which will automatically ignore whitespace and line comments:
//! let tokenizer = Tokenizer::new(r"(\s+|//.*\n)", vec![
//!     const_token!(r",", Payload::Comma),
//!     const_token!(r"\+", Payload::Plus), const_token!(r"\*", Payload::Times),
//!     const_token!(r"\[", Payload::BracketOpen), const_token!(r"\]", Payload::BracketClose),
//!     const_token!(r"\(", Payload::ParOpen), const_token!(r"\)", Payload::ParClose),
//!     TokenRule::new(r"-?\d+", |c| {  // Integers
//!         Payload::Literal(c.get(0).and_then(|m| m.as_str().parse().ok()).unwrap())
//!     }),
//!     TokenRule::new(r"[a-zA-Z][a-zA-Z0-9]+", |c| {   // Identifiers
//!         Payload::Identifier(c.get(0).unwrap().as_str().to_string())
//!     }),
//! ]);
//! let tokens: Vec<Token<Payload>> = tokenizer.read(
//!     "[1,22] + // comment\n (4 * (3)) + (hello + -12)"
//! ).unwrap();
//! // '[' '1' ',' '22' ']' '+' '(' '4' '*' '(' '3' ')' ')' '+' '(' 'hello' '+' '-12' ')'
//! assert_eq!(tokens.len(), 19);
//! ```
//!
//! ### Tier 1 - `TokenTreeBuilder`
//!
//! Once the string is tokenized, we turn it into a token tree, splitting tokens into recursive
//! *groups*. Each group is defined by a rule which matches the opening and closing token:
//!
//! ```rust
//! # use biodivine_lib_std::{const_token, const_group};
//! # use biodivine_lib_std::parsers::tokens::{Tokenizer, TokenRule, Token};
//! # use biodivine_lib_std::parsers::groups::{TokenTreeBuilder, GroupRule, TokenForest};
//! # // A simple payload data type for our tokens:
//! # #[derive(Clone, Eq, PartialEq)]
//! # enum Payload {
//! #     ParOpen, ParClose, BracketOpen, BracketClose,
//! #     Plus, Times, Comma, Literal(i32), Identifier(String)
//! # }
//! # // A tokenizer which will automatically ignore whitespace and line comments:
//! # let tokenizer = Tokenizer::new(r"(\s+|//.*\n)", vec![
//! #    const_token!(r",", Payload::Comma),
//! #    const_token!(r"\+", Payload::Plus), const_token!(r"\*", Payload::Times),
//! #    const_token!(r"\[", Payload::BracketOpen), const_token!(r"\]", Payload::BracketClose),
//! #    const_token!(r"\(", Payload::ParOpen), const_token!(r"\)", Payload::ParClose),
//! #    TokenRule::new(r"-?\d+", |c| {  // Integers
//! #        Payload::Literal(c.get(0).and_then(|m| m.as_str().parse().ok()).unwrap())
//! #    }),
//! #    TokenRule::new(r"[a-zA-Z][a-zA-Z0-9]+", |c| {   // Identifiers
//! #        Payload::Identifier(c.get(0).unwrap().as_str().to_string())
//! #    }),
//! # ]);
//! let tree_builder = TokenTreeBuilder::new(vec![
//!     const_group!("parenthesis", Payload::ParOpen, Payload::ParClose),
//!     const_group!("brackets", Payload::BracketOpen, Payload::BracketClose)
//! ]);
//!
//! let tokens: Vec<Token<Payload>> = tokenizer.read(
//!     "[1,22] + // comment\n (4 * (3)) + (hello + -12)"
//! ).unwrap();
//!
//! let groups: TokenForest<Payload> = tree_builder.group_tokens(&tokens).unwrap();
//! assert_eq!(groups[0].name().unwrap(), "brackets");
//! assert_eq!(groups[0].children().unwrap().len(), 3); // '1' ',' '22'
//! assert_eq!(groups[2].name().unwrap(), "parenthesis");
//! assert_eq!(groups[2].children().unwrap().len(), 3); // '4' '*' '(3)'
//! assert_eq!(groups[4].name().unwrap(), "parenthesis");
//! assert_eq!(groups[4].children().unwrap().len(), 3); // 'hello' '+' '-12'
//! ```
//!
//! Aside from basic parenthesis-like groups, `TokenTreeBuilders` support many advanced use
//! cases, like matching tags or labeled groups (see module docs for examples).
//!
//! ### Tier 2 - Parser combinators
//!
//! Last tier is the most versitile one, but also the most loosely defined. Here, we provide
//! a set of utility methods based on the idea of parser combinators. This is the least developed
//! part of the module and can evolve more once it is clearer what functionality if truly
//! useful here.
//!

pub mod groups;
pub mod parsers;
pub mod tokens;

pub mod tokens2;

mod v2;

mod _impl_parse_error;

/// Represents an error during a grouping process.
///
/// If has reference to the positions of opening/closing tokens of the problematic
/// group, if such tokens were present (for example, for unclosed group that leaks past the
/// end of file, no ending position is given). At least one position should be specified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub starts_at: Option<usize>,
    pub ends_at: Option<usize>,
    pub message: String,
}
