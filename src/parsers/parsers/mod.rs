//! Defines utility methods for building `Parsers`. `Parsers` transform `TokenTrees` into
//! some useful structures.
//!
//! The module mostly contains utility macros for combining parsers using some logical rules
//! and does not have one central mechanism as other modules.

use crate::parsers::groups::TokenTree;
use crate::parsers::tokens::{Token, TokenRule};
use crate::parsers::ParseError;
use regex::internal::Input;

/// Parser is a function that takes a `TokenTree` and transforms it into the `Output` structure
/// if possible. If not possible, a vector of `ParseErrors` is returned. The vector can contain
/// the first encountered error or a wider range of errors if the parser is trying for partial
/// recovery (however, there is never a possibility to return a valid result if error
/// is encountered).
pub type Parser<Payload, Output> = fn(&[TokenTree<Payload>]) -> Result<Output, Vec<ParseError>>;

pub struct DynParser<Payload: Clone, Output>(
    Box<
        dyn Fn(
            &DynParser<Payload, Output>,
            usize,
            &[TokenTree<Payload>],
            &mut Vec<ParseError>,
        ) -> Option<Output>,
    >,
);
pub struct TokenTest<Payload: Clone>(Box<dyn Fn(&Token<Payload>) -> bool>);

impl<Payload: Clone, Output> DynParser<Payload, Output> {
    pub fn parse(
        &self,
        starts_at: usize,
        data: &[TokenTree<Payload>],
        errors: &mut Vec<ParseError>,
    ) -> Option<Output> {
        return (self.0)(self, starts_at, data, errors);
    }
}

impl<Payload: Clone> TokenTest<Payload> {
    pub fn const_data(data: &str) -> TokenTest<Payload> {
        let data = data.to_string(); // make a local copy
        return TokenTest(Box::new(move |t| t.data == data));
    }

    pub fn new<F>(closure: F) -> TokenTest<Payload>
    where
        F: Fn(&Token<Payload>) -> bool + 'static,
    {
        return TokenTest(Box::new(closure));
    }

    pub fn test(&self, token: &Token<Payload>) -> bool {
        return (self.0)(token);
    }
}

impl<Payload: Clone + Eq + 'static> TokenTest<Payload> {
    pub fn const_payload(payload: Payload) -> TokenTest<Payload> {
        return TokenTest(Box::new(move |t| t.payload == payload));
    }
}

impl<Payload: Clone + 'static, Output: 'static> DynParser<Payload, Output> {
    pub fn make_repeating<F>(
        item_parser: DynParser<Payload, Output>,
        split_by: TokenTest<Payload>,
        fold: F,
    ) -> DynParser<Payload, Output>
    where
        F: Fn(Output, Output) -> Output + 'static,
    {
        return DynParser(Box::new(move |self_parser, starts_at, forest, errors| {
            let split_position = forest
                .iter()
                .position(|i| i.value().map(|t| split_by.test(t)).unwrap_or(false));
            if let Some(split_position) = split_position {
                let item_forest = &forest[..split_position];
                let remaining_forest = &forest[(split_position + 1)..];
                let item = item_parser.parse(starts_at, item_forest, errors);
                let remaining_starts_at: usize = if remaining_forest.is_empty() {
                    forest[split_position].ends_at()
                } else {
                    remaining_forest[0].starts_at()
                };
                let remaining = self_parser.parse(remaining_starts_at, remaining_forest, errors);
                match (item, remaining) {
                    (Some(a), Some(b)) => Some(fold(a, b)),
                    _ => None,
                }
            } else {
                item_parser.parse(starts_at, forest, errors)
            }
        }));
    }
}

macro_rules! parser {
    ( $payload:ty, $output:ty, $initializer: block ) => {{
        type Payload = $payload;
        type Output = $output;

        $initializer
    }};
}

macro_rules! test_const_token {
    ( $name:ident, $payload:expr ) => {
        fn $name(token: &Token<Payload>) -> bool {
            return token.payload == $payload;
        }
    };
}

macro_rules! test_const_data_token {
    ( $name:ident, $payload:expr ) => {
        fn $name(token: &Token<Payload>) -> bool {
            return token.data == $payload;
        }
    };
}

macro_rules! repeating_parser {
    ( $name:ident, $parse_item:ident, $split_by:ident, $fold_items:path ) => {
        fn $name(forest: &[TokenTree<Payload>]) -> Result<Output, Vec<ParseError>> {
            return if let Some(position) = forest
                .iter()
                .position(|i| i.value().map($split_by).unwrap_or(false))
            {
                if (position == 0) {
                    Err(vec![ParseError::invalid(
                        "Expected expression, found nothing",
                        forest,
                    )])
                } else {
                    let item = $parse_item(&forest[..position]);
                    let remaining = $name(&forest[(position + 1)..]);
                    match (item, remaining) {
                        (Ok(a), Ok(b)) => Ok($fold_items(a, b)),
                        (Err(mut e1), Err(mut e2)) => {
                            e1.append(&mut e2);
                            Err(e1)
                        }
                        (Err(e), _) | (_, Err(e)) => Err(e),
                    }
                }
            } else {
                if forest.len() == 0 {
                    Err(vec![ParseError::invalid(
                        "Expected expression, found nothing",
                        forest,
                    )])
                } else {
                    $parse_item(forest)
                }
            };
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::parsers::groups::{GroupRule, TokenForest, TokenTree, TokenTreeBuilder};
    use crate::parsers::parsers::{DynParser, Parser, TokenTest};
    use crate::parsers::tokens::{Token, TokenRule, Tokenizer};
    use crate::parsers::ParseError;
    use crate::{const_data_group, const_token};

    #[derive(Clone, Debug)]
    enum Arithmetic {
        Value(String),
        Plus(Box<Arithmetic>, Box<Arithmetic>),
        Times(Box<Arithmetic>, Box<Arithmetic>),
    }

    impl Arithmetic {
        fn mk_plus(a: Arithmetic, b: Arithmetic) -> Arithmetic {
            return Arithmetic::Plus(Box::new(a), Box::new(b));
        }
        fn mk_times(a: Arithmetic, b: Arithmetic) -> Arithmetic {
            return Arithmetic::Times(Box::new(a), Box::new(b));
        }
    }

    struct ParseTree<'a, 'b, 'c, Payload: Clone> {
        starts_at: usize,
        tokens: &'a [TokenTree<'b, Payload>],
        errors: &'c Vec<ParseError>,
    }

    #[test]
    pub fn test_read_repeating_simple() {
        let tokenizer = Tokenizer::ignoring_whitespace(vec![
            const_token!(r"\+", None),
            const_token!(r"\*", None),
            const_token!(r"\(", None),
            const_token!(r"\)", None),
            TokenRule::new(r"\d+", |c| {
                Some(Arithmetic::Value(c.get(0).unwrap().as_str().to_string()))
            }),
        ]);
        let tree_builder = TokenTreeBuilder::new(vec![const_data_group!("parenthesis", "(", ")")]);

        let forest = tokenizer.read("1 + 2 * 1 + (3 + 2 * 9) * 5 + 4").unwrap();
        let forest = tree_builder.group_tokens(&forest).unwrap();

        let parser: Parser<Option<Arithmetic>, Arithmetic> =
            parser!(Option<Arithmetic>, Arithmetic, {
                fn read_atom(
                    forest: &[TokenTree<Option<Arithmetic>>],
                ) -> Result<Arithmetic, Vec<ParseError>> {
                    // assume forest is not empty
                    if forest.len() != 1 {
                        Err(vec![ParseError::invalid(
                            "Expected value or parenthesis block.",
                            forest,
                        )])
                    } else {
                        match &forest[0] {
                            TokenTree::Value(token) => {
                                // Value
                                if let Some(a) = &token.payload {
                                    Ok(a.clone())
                                } else {
                                    Err(vec![ParseError::invalid("Expected value.", forest)])
                                }
                            }
                            TokenTree::Group { data, .. } => {
                                // parenthesis
                                if data.is_empty() {
                                    Err(vec![ParseError::invalid("Expected expression.", forest)])
                                } else {
                                    Ok(read_plus(&data)?)
                                }
                            }
                        }
                    }
                };

                test_const_data_token!(is_plus, "+");
                test_const_data_token!(is_times, "*");
                repeating_parser!(read_times, read_atom, is_times, Arithmetic::mk_times);
                repeating_parser!(read_plus, read_times, is_plus, Arithmetic::mk_plus);
                read_plus
            });

        println!("Parsed: {:?}", parser(&forest));
    }
}
