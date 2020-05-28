//! Utility methods for writing simple parsers.
//!
//! Note that we are definitely not trying to be super fast (no zero-allocation or whatever parsers
//! here). Just creating an architecture that will work well with a lot of different formats.
//! So if you need to parse gigabytes of data, don't use this! Also, if you need to parse XML,
//! JSON or some standard format, just use some standard library please. This is mostly for
//! small ad-hoc formats (logical formulas, graph descriptions, etc.).
//!
//! Our parsing architecture consists of three "tiers":
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

pub mod groups;
pub mod tokens;
//pub mod parsers;
