use crate::parsers::tokens::{Token, TokenTemplate, Tokenizer, TokenizerError};
use regex::Regex;
use std::fmt::{Debug, Formatter};

impl<Payload> Tokenizer<Payload> {
    /// Construct a new tokenizer with an `ignore` regex that will be skipped if encountered.
    ///
    /// The regex is prefixed with the `^` anchor.
    pub fn new(ignore: &str, templates: Vec<TokenTemplate<Payload>>) -> Tokenizer<Payload> {
        return Tokenizer {
            templates,
            ignore: Some(Regex::new(format!("^{}", ignore).as_str()).unwrap()),
        };
    }

    /// Create a new tokenizer which will always maych every character in the string (ignoring nothing).
    pub fn ignore_nothing(templates: Vec<TokenTemplate<Payload>>) -> Tokenizer<Payload> {
        return Tokenizer {
            templates,
            ignore: None,
        };
    }

    /// Create a new tokenizer which will automatically skip whitespace characters
    /// in the string.
    pub fn ignoring_whitespace(templates: Vec<TokenTemplate<Payload>>) -> Tokenizer<Payload> {
        return Tokenizer {
            templates,
            ignore: Some(Regex::new(r"^\s+").unwrap()),
        };
    }

    /// Read a string into a vector of tokens, or produce an error if unexpected characters
    /// are encountered.
    pub fn read<'a>(&self, data: &'a str) -> Result<Vec<Token<'a, Payload>>, TokenizerError> {
        let mut tokens = Vec::new();
        let mut position: usize = self.unwrap_whitespace(data, 0);
        while position < data.len() {
            if let Some(next_position) = self.match_token(data, position, &mut tokens) {
                position = next_position;
            } else {
                return Err(TokenizerError {
                    position,
                    message: format!("Unexpected '{}'.", &data[position..position + 1]),
                });
            }
        }
        return Ok(tokens);
    }

    /// Try to tokenize a given string, recovering after errors.
    ///
    /// If a character cannot be tokenized, emit and error and seek to the first position
    /// where a valid token can be constructed. For each consecutive sequence of invalid characters,
    /// only one error is emitted.
    pub fn read_with_recovery<'a>(
        &self,
        data: &'a str,
    ) -> (Vec<Token<'a, Payload>>, Vec<TokenizerError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        let mut position: usize = self.unwrap_whitespace(data, 0);
        let mut looking_for_recovery = false; // true when error was emitted and we are looking for next valid token
        while position < data.len() {
            let next_position = self.match_token(data, position, &mut tokens);
            if let Some(next_position) = next_position {
                // Found token - end recovery and continue at new position
                looking_for_recovery = false;
                position = next_position;
            } else {
                // No token found - start/continue recovery
                if !looking_for_recovery {
                    // If this is the problematic position, emit error
                    errors.push(TokenizerError {
                        position,
                        message: format!("Unexpected '{}'.", &data[position..position + 1]),
                    });
                }
                looking_for_recovery = true;
                position = self.unwrap_whitespace(data, position + 1);
            }
        }
        return (tokens, errors);
    }

    /// **(internal)** Utility method which will try to match all templates and write the
    /// matched token into the `tokens` vector (returns new position on which to continue
    /// matching - including ignored characters).
    fn match_token<'a>(
        &self,
        data: &'a str,
        position: usize,
        tokens: &mut Vec<Token<'a, Payload>>,
    ) -> Option<usize> {
        for template in self.templates.iter() {
            if let Some((matched, payload)) = template.try_match(&data[position..]) {
                let matched = matched.get(0).unwrap();
                tokens.push(Token {
                    starts_at: position,
                    payload,
                    data: matched.as_str(),
                });
                return Some(self.unwrap_whitespace(data, position + matched.end()));
            }
        }
        return None;
    }

    /// **(internal)** Utility method which will move position to the first non-ignore character
    fn unwrap_whitespace(&self, data: &str, position: usize) -> usize {
        if let Some(ignore) = &self.ignore {
            if let Some(matched) = ignore.find(&data[position..]) {
                // We have to repeat this, because ignore regex can be matched repeatedly
                // (imagine multiple consecutive line comments)
                // Fingers crossed this tail recursion can be eliminated byt the compiler!
                return self.unwrap_whitespace(data, position + matched.end());
            }
        }
        return position;
    }
}

impl<Payload> Debug for Tokenizer<Payload> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(
            f,
            "Tokenizer(ignore: {:?}, tokens: {:?})",
            self.ignore,
            self.templates
                .iter()
                .map(|t| &t.regex)
                .collect::<Vec<&Regex>>()
        );
    }
}

#[cfg(test)]
mod tests {
    use self::TestPayload::*;
    use crate::parsers::tokens::{TokenTemplate, Tokenizer};

    #[derive(Debug, Eq, PartialEq, Clone)]
    enum TestPayload {
        Whitespace,
        ParOpen,
        ParClose,
        Neg,
        And,
        Or,
        KeyValue(String, String),
        Identifier(String),
    }

    fn make_token_templates() -> Vec<TokenTemplate<TestPayload>> {
        return vec![
            TokenTemplate::new(r"\(", |_| ParOpen),
            TokenTemplate::new(r"\)", |_| ParClose),
            TokenTemplate::new("!", |_| Neg),
            TokenTemplate::new("¬", |_| Neg),
            TokenTemplate::new("&", |_| And),
            TokenTemplate::new(r"\|", |_| Or),
            TokenTemplate::new(r"\s", |_| Whitespace),
            TokenTemplate::new(r"([a-z]+):([a-z]+)", |m| {
                let key = m.get(1).unwrap().as_str().to_string();
                let value = m.get(2).unwrap().as_str().to_string();
                KeyValue(key, value)
            }),
            TokenTemplate::new(r"[a-zA-Z_]+", |m| {
                Identifier(m.get(0).unwrap().as_str().to_string())
            }),
        ];
    }

    #[test]
    pub fn test_simple_tokenizer() {
        let tokenizer = Tokenizer::ignoring_whitespace(make_token_templates());
        println!("Start tokenizer...");
        let tokens: Vec<TestPayload> = tokenizer
            .read("(a & ¬b) & !hello:world")
            .unwrap()
            .into_iter()
            .map(|t| t.payload)
            .collect();
        assert_eq!(tokens.len(), 9);
        assert_eq!(tokens[0], ParOpen);
        assert_eq!(tokens[1], Identifier("a".to_string()));
        assert_eq!(tokens[2], And);
        assert_eq!(tokens[3], Neg);
        assert_eq!(tokens[4], Identifier("b".to_string()));
        assert_eq!(tokens[5], ParClose);
        assert_eq!(tokens[6], And);
        assert_eq!(tokens[7], Neg);
        assert_eq!(
            tokens[8],
            KeyValue("hello".to_string(), "world".to_string())
        );
        println!("{:?}", tokenizer);
    }

    #[test]
    pub fn test_tokenizer_with_ignoring() {
        // Create a tokenizer which will skip line comments starting with '#'
        let tokenizer = Tokenizer::<Option<i32>>::new(
            r"(\s+|#.*\n)",
            vec![
                TokenTemplate::new(r"\+", |_| None), // Plus operator
                TokenTemplate::new(r"\*", |_| None), // Times operator
                TokenTemplate::new(r"-?\d+", |c| {
                    Some(
                        c.get(0)
                            .and_then(|m| m.as_str().parse::<i32>().ok())
                            .unwrap(),
                    )
                }), // Integer
            ],
        );
        let tokens = tokenizer.read("3 + 4 # line comment\n\t\t * 5").unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].data, "3");
        assert_eq!(tokens[1].data, "+");
        assert_eq!(tokens[2].data, "4");
        assert_eq!(tokens[3].data, "*");
        assert_eq!(tokens[4].data, "5");
    }

    #[test]
    pub fn test_simple_tokenizer_with_whitespace() {
        let tokenizer = Tokenizer::ignore_nothing(make_token_templates());
        let tokens: Vec<TestPayload> = tokenizer
            .read("(a & ¬b) & !hello:world")
            .unwrap()
            .into_iter()
            .map(|t| t.payload)
            .collect();
        assert_eq!(tokens.len(), 13);
        assert_eq!(tokens[0], ParOpen);
        assert_eq!(tokens[1], Identifier("a".to_string()));
        assert_eq!(tokens[2], Whitespace);
        assert_eq!(tokens[3], And);
        assert_eq!(tokens[4], Whitespace);
        assert_eq!(tokens[5], Neg);
        assert_eq!(tokens[6], Identifier("b".to_string()));
        assert_eq!(tokens[7], ParClose);
        assert_eq!(tokens[8], Whitespace);
        assert_eq!(tokens[9], And);
        assert_eq!(tokens[10], Whitespace);
        assert_eq!(tokens[11], Neg);
        assert_eq!(
            tokens[12],
            KeyValue("hello".to_string(), "world".to_string())
        );
    }

    #[test]
    pub fn test_simple_tokenizer_error() {
        let tokenizer = Tokenizer::ignoring_whitespace(make_token_templates());
        let error = tokenizer.read("(a - b)").err().unwrap();
        assert_eq!(error.position, 3);
    }

    #[test]
    pub fn test_simple_tokenizer_with_recovery() {
        let tokenizer = Tokenizer::ignoring_whitespace(make_token_templates());
        let (tokens, errors) = tokenizer.read_with_recovery("!a + b:c ... |z)");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].payload, Neg);
        assert_eq!(tokens[1].payload, Identifier("a".to_string()));
        assert_eq!(
            tokens[2].payload,
            KeyValue("b".to_string(), "c".to_string())
        );
        assert_eq!(tokens[3].payload, Or);
        assert_eq!(tokens[4].payload, Identifier("z".to_string()));
        assert_eq!(tokens[5].payload, ParClose);

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].position, 3);
        assert_eq!(errors[1].position, 9);
    }
}
