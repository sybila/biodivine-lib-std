use crate::parsers::tokens::TokenTemplate;
use regex::{Captures, Regex};
use std::fmt::{Debug, Formatter};

impl<Payload> TokenTemplate<Payload> {
    /// Create a new token template using given regex and a factory function.
    ///
    /// Regex will be prefixed with `^` anchor.
    pub fn new(regex: &str, factory: fn(&Captures) -> Payload) -> TokenTemplate<Payload> {
        return TokenTemplate {
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
impl<Payload> Debug for TokenTemplate<Payload> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "TokenTemplate({:?})", self.regex);
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::tokens::TokenTemplate;

    #[test]
    pub fn test_match_literal_template() {
        let eq_op_template = TokenTemplate::<Option<String>>::new("<=>", |_| None);
        let matched = eq_op_template.try_match("<==>");
        assert!(matched.is_none());
        let (captures, payload) = eq_op_template.try_match("<=> x").unwrap();
        assert_eq!(captures.get(0).unwrap().as_str(), "<=>");
        assert!(payload.is_none());
        println!("{:?}", eq_op_template); // sholud print TokenTemplate("^<=>")
    }

    #[test]
    pub fn test_match_identifier_template() {
        // Token which matches some identifier starting with '$'
        let id_template = TokenTemplate::<Option<String>>::new(r"\$([a-z]+)", |c| {
            return Some(c.get(1).unwrap().as_str().to_string());
        });
        let (captures, payload) = id_template.try_match("$hello there").unwrap();
        assert_eq!(captures.get(0).unwrap().as_str(), "$hello");
        assert_eq!(payload, Some("hello".to_string()));
    }
}
