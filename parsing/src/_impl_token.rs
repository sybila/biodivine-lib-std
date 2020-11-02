use crate::{Extras, Token};
use std::ops::Index;

impl Token {
    /// Create a new standard token with no extra information.
    pub fn new(starts_at: usize, rule: &str, value: &str) -> Token {
        let meta = vec![rule.into(), value.into()];
        return Token::mk_token(starts_at, false, meta);
    }

    /// Take a `Token` and return the same token but with an error.
    pub fn with_error(mut self, message: &str) -> Token {
        if self.has_error {
            panic!("Token already has an error: {}.", self.error_message());
        }
        self.meta.insert(self.header_len(), message.into());
        self.has_error = true;
        return self;
    }

    /// **(internal)** Constructor with input validation.
    fn mk_token(starts_at: usize, has_error: bool, meta: Vec<String>) -> Token {
        assert!(has_error || (meta.len() == 2));
        assert!(!has_error || (meta.len() == 3));
        return Token {
            starts_at,
            has_error,
            meta,
        };
    }

    /// Add extra string data `value` to this `Token`.
    pub fn push_extra(&mut self, value: &str) {
        self.meta.push(value.into());
    }

    /// Index of the first character of this `Token` in the input string.
    pub fn starts_at(&self) -> usize {
        return self.starts_at;
    }

    /// Name of the rule that generated this `Token`.
    pub fn rule(&self) -> &str {
        return &self.meta[0];
    }

    /// Actual value of this `Token` as seen in the input string.
    pub fn value(&self) -> &str {
        return &self.meta[1];
    }

    /// An iterator over string extras attached to this `Token`.
    pub fn extras(&self) -> Extras {
        /* Error message is at [2], value at [1]. */
        return Extras::new(&self.meta, self.header_len());
    }

    /// Safely obtain token tree extra at given position.
    pub fn get_extra(&self, index: usize) -> Option<&str> {
        return self.meta.get(index + self.header_len()).map(|i| i.as_str());
    }

    /// True if this is an error `Token`.
    pub fn has_error(&self) -> bool {
        return self.has_error;
    }

    /// If this `Token` is an error token, return its error message, otherwise return `None`.
    pub fn get_error_message(&self) -> Option<&str> {
        return match self.has_error {
            true => self.meta.get(2).map(|s| s.as_str()),
            false => None,
        };
    }

    /// Return error message if this `Token` has error, otherwise panic.
    pub fn error_message(&self) -> &str {
        return match self.has_error {
            true => &self.meta[2],
            false => panic!("Token {:?} has no error message.", self),
        };
    }

    /// **(internal)** Number of semantic elements in the `meta` vector before the extras start.
    fn header_len(&self) -> usize {
        return 2 + if self.has_error { 1 } else { 0 };
    }
}

/// Indexing into the `extras` array of a `Token`.
impl Index<usize> for Token {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.meta[index + self.header_len()];
    }
}

#[cfg(test)]
mod tests {
    use crate::Token;

    #[test]
    fn test_normal_token() {
        let mut token = Token::new(10, "test-rule", "test-value");
        token.push_extra("extra_1");
        token.push_extra("extra_2");
        assert_eq!(token.rule(), "test-rule");
        assert_eq!(token.value(), "test-value");
        assert_eq!(token.extras().count(), 2);
        let mut extras = token.extras();
        assert_eq!(extras.next(), Some("extra_1"));
        assert_eq!(extras.next(), Some("extra_2"));
        assert_eq!(extras.next(), None);
        assert_eq!(&token[0], "extra_1");
        assert_eq!(&token[1], "extra_2");
        assert_eq!(token.get_error_message(), None);
        assert!(!token.has_error());
    }

    #[test]
    fn test_error_token() {
        let mut token = Token::new(10, "error-test-rule", "test-value").with_error("test-error");
        token.push_extra("extra_1");
        token.push_extra("extra_2");
        assert_eq!(token.rule(), "error-test-rule");
        assert_eq!(token.value(), "test-value");
        assert_eq!(token.extras().count(), 2);
        assert_eq!(token.error_message(), "test-error");
        assert!(token.has_error());
    }
}
