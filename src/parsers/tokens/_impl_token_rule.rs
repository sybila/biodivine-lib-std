use crate::parsers::tokens::TokenRule;
use regex::{Captures, Regex};
use std::fmt::{Debug, Formatter};

impl<Payload> TokenRule<Payload> {
    /// Create a new token rule using given regex and a factory function.
    ///
    /// Regex will be prefixed with `^` anchor.
    pub fn new(regex: &str, factory: fn(&Captures) -> Payload) -> TokenRule<Payload> {
        return TokenRule {
            regex: Regex::new(format!("^{}", regex).as_str()).unwrap(),
            factory,
        };
    }

    /// Try to match this token at the start of a given string slice.
    pub fn try_match<'a>(&self, data: &'a str) -> Option<(Captures<'a>, Payload)> {
        return if let Some(m) = self.regex.captures(data) {
            let payload = (self.factory)(&m);
            Some((m, payload))
        } else {
            None
        };
    }
}

/// Since payload factories cannot be printed, we at least display the template regex.
impl<Payload> Debug for TokenRule<Payload> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "TokenTemplate({:?})", self.regex);
    }
}

#[cfg(test)]
mod tests {
    use crate::const_token;
    use crate::parsers::tokens::TokenRule;

    #[test]
    pub fn test_match_literal_template() {
        let eq_op_template = const_token!("<=>", 10);
        let matched = eq_op_template.try_match("<==>");
        assert!(matched.is_none());
        let (captures, payload) = eq_op_template.try_match("<=> x").unwrap();
        assert_eq!(captures.get(0).unwrap().as_str(), "<=>");
        assert_eq!(payload, 10);
        println!("{:?}", eq_op_template); // should print TokenTemplate("^<=>")
    }

    #[test]
    pub fn test_match_identifier_template() {
        // Token which matches some identifier starting with '$'
        let id_template = TokenRule::<String>::new(r"\$([a-z]+)", |c| {
            return c.get(1).unwrap().as_str().to_string();
        });
        let (captures, payload) = id_template.try_match("$hello there").unwrap();
        assert_eq!(captures.get(0).unwrap().as_str(), "$hello");
        assert_eq!(payload, "hello".to_string());
    }
}
