//! "Tier 1" of our parsing architecture are the `TokenTreeBuilders`. A token tree holds `Token` values,
//! but is structured hierarchically, ensuring proper opening and closing of various groups.
//!
//! An element of a `TokenTree` is either a `Value` (effectively a `Token`), or a `Group`.
//! A `Group` has a name (inherited from the rule which created it), starting and ending `Token`
//! and `TokenForest` (vector of `TokenTree`s) representing the inner values of the group.
//!
//! ### Group Rules
//!
//! To define a group, we specify `GroupRules`. Such rule has a name and three test functions:
//! `opens`, `closes` and `is_group`. `opens` and `closes` identify tokens which can potentially
//! define the group, and `is_group` is used to definitely verify that the two given tokens
//! indeed form a group together.
//!
//! ```rust
//! use biodivine_lib_std::const_data_group;
//! use biodivine_lib_std::parsers::groups::GroupRule;
//! use biodivine_lib_std::parsers::tokens::Token;
//!
//! let group_rule: GroupRule<()> = GroupRule::new("parenthesis",
//!     |t| t.data == "(",
//!     |t| t.data == ")",
//!     |o, c| o.data == "(" && o.data == ")"
//! );
//! assert!(group_rule.opens(&Token { starts_at: 0, data: "(", payload: () }));
//! assert!(group_rule.closes(&Token { starts_at: 0, data: ")", payload: () }));
//!
//! // The same rule can be created using a macro:
//! let group_rule: GroupRule<()> = const_data_group!("parenthesis", "(", ")");
//! ```
//!
//! By using functions instead of direct value comparison, we enable many advanced use cases,
//! for example matching tags:
//!
//! ```rust
//! use biodivine_lib_std::parsers::groups::GroupRule;
//! use biodivine_lib_std::parsers::tokens::Token;
//!
//! #[derive(Clone)]
//! enum P { TagOpen(String), TagClose(String), Other }
//!
//! let group_rule: GroupRule<P> = GroupRule::new("tags",
//!     |t| matches!(t.payload, P::TagOpen(_)),
//!     |t| matches!(t.payload, P::TagClose(_)),
//!     |o, c| {
//!         if let P::TagOpen(o) = &o.payload {
//!             if let P::TagClose(c) = &c.payload {
//!                 return o == c;
//!             }
//!         }
//!         return false;
//!     }
//! );
//!
//! let o1 = Token { starts_at: 0, data: "<abc>", payload: P::TagOpen("abc".to_string()) };
//! let o2 = Token { starts_at: 0, data: "<f>", payload: P::TagOpen("f".to_string()) };
//! let c1 = Token { starts_at: 0, data: "</abc>", payload: P::TagClose("abc".to_string()) };
//!
//! assert!(group_rule.opens(&o1));
//! assert!(group_rule.opens(&o2));
//! assert!(group_rule.closes(&c1));
//! assert!(group_rule.is_group(&o1, &c1));
//! assert!(!group_rule.is_group(&o2, &c1));
//!
//! ```
//!
//! ### `TokenTree` Builders
//!
//! Similar to `Tokenizers`, `TokenTreeBuilders` match a series of `GroupRules` on a stream
//! of tokens to create a `TokenForest` (a vector of `TokenTrees`) from them.
//!
//! ```rust
//! use biodivine_lib_std::{const_data_group, const_token};
//! use biodivine_lib_std::parsers::groups::{TokenTreeBuilder, GroupRule};
//! use biodivine_lib_std::parsers::tokens::{Tokenizer, TokenRule};
//!
//! let tokenizer = Tokenizer::ignore_nothing(vec![
//!     const_token!(r"\(", ()), const_token!(r"\)", ()),
//!     const_token!(r"\[", ()), const_token!(r"\]", ()),
//! ]);
//! let tree_builder = TokenTreeBuilder::new(vec![
//!     const_data_group!("parenthesis", "(", ")"),
//!     const_data_group!("bracket", "[", "]"),
//! ]);
//!
//! let tokens = tokenizer.read("([])").unwrap();
//! let forest = tree_builder.group_tokens(&tokens).unwrap();
//! assert_eq!(forest[0].name().unwrap(), "parenthesis");
//! assert_eq!(forest[0].children().unwrap()[0].name().unwrap(), "bracket");
//! ```
//!
//! `TokenTreeBuilders` also support error recovery:
//!
//! ```rust
//! # use biodivine_lib_std::{const_data_group, const_token};
//! # use biodivine_lib_std::parsers::groups::{TokenTreeBuilder, GroupRule};
//! # use biodivine_lib_std::parsers::tokens::{Tokenizer, TokenRule};
//! # let tokenizer = Tokenizer::ignore_nothing(vec![
//! #     const_token!(r"\(", ()), const_token!(r"\)", ()),
//! #     const_token!(r"\[", ()), const_token!(r"\]", ()),
//! # ]);
//! # let tree_builder = TokenTreeBuilder::new(vec![
//! #     const_data_group!("parenthesis", "(", ")"),
//! #     const_data_group!("bracket", "[", "]"),
//! # ]);
//! let tokens = tokenizer.read("([]])").unwrap();
//! let (forest, errors) = tree_builder.group_tokens_with_recovery(&tokens);
//! assert_eq!(errors[0].ends_at, Some(3));
//! assert_eq!(forest[0].name().unwrap(), "parenthesis");
//! assert_eq!(forest[0].children().unwrap().len(), 1);
//! assert_eq!(forest[0].children().unwrap()[0].name().unwrap(), "bracket")
//! ```
//!

use crate::parsers::tokens::Token;

mod _impl_group_rule;
mod _impl_token_tree;
mod _impl_token_tree_builder;
mod _macro_group_rule;

/// Group rule is a template for matching groups in the `TokenTreeBuilder`.
///
/// It has a human-readable name (mainly used to generate error messages) and three test functions:
///  - `opens` tests whether a token is an opening token for this group type.
///  - `closes` tests whether a token is a closing token for this group type.
///  - `is_group` tests whether the given pair of tokens actually forms a group.
///
/// Using this mechanism allows for more advanced group matching than just parenthesis.
///
/// First, these are all functions, so it does not matter what the token payload is - you can
/// match just a specific part of it. This can be used to recognize sequences like `[Name](` ... `)`,
/// where `Name` is actually a part of the token payload.
///
/// Second, we do not assume any two valid open and close tokens form a group. There is
/// one more extra check using `is_group` before a group is formed. This makes it possible
/// to create rules like `<tag>` ... `</tag>` where we correctly recognize that opening and closing
/// `tag` are the same.
pub struct GroupRule<Payload: Clone> {
    pub name: String,
    opens: fn(&Token<Payload>) -> bool,
    closes: fn(&Token<Payload>) -> bool,
    is_group: fn(&Token<Payload>, &Token<Payload>) -> bool,
}

/// A tree-like structure of tokens that represents a stream of tokens processed into groups.
///
/// Notice that `close` token in the `Group` variant is optional. This is because the group
/// can be created during error recovery, in which case the closing token might be missing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenTree<'a, Payload: Clone> {
    Value(Token<'a, Payload>),
    Group {
        name: String,
        open: Token<'a, Payload>,
        close: Option<Token<'a, Payload>>,
        data: TokenForest<'a, Payload>,
    },
}

/// Alias for a vector of `TokenTree`s.
pub type TokenForest<'a, Payload> = Vec<TokenTree<'a, Payload>>;

/// Transforms a stream of tokens into a tree-like structure based on the given group rules.
pub struct TokenTreeBuilder<Payload: Clone> {
    group_templates: Vec<GroupRule<Payload>>,
}
