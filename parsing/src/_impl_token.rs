use crate::{Token, TokenExtras};

impl Token {
    /// Create a new standard token with no extra information.
    pub fn new(starts_at: usize, rule: &str, value: &str) -> Token {
        let meta = vec![rule.into(), value.into()];
        return Token::mk_token(starts_at, false, meta);
    }

    /// Create a new error token.
    pub fn new_error(starts_at: usize, rule: &str, value: &str, message: &str) -> Token {
        let meta = vec![rule.into(), value.into(), message.into()];
        return Token::mk_token(starts_at, true, meta);
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

    /// Add extra string data `value` to this token.
    pub fn push_extra(&mut self, value: &str) {
        self.meta.push(value.into());
    }

    /// Index of the first character of this token in the input string.
    pub fn starts_at(&self) -> usize {
        return self.starts_at;
    }

    /// Name of the rule that generated this token.
    pub fn rule(&self) -> &str {
        return &self.meta[0];
    }

    /// Actual value of this token as seen in the input string.
    pub fn value(&self) -> &str {
        return &self.meta[1];
    }

    /// An iterator over string extras attached to this token.
    pub fn extras(&self) -> TokenExtras {
        /* Error message is at [2], value at [1]. Iterator will move to next index first. */
        return TokenExtras {
            token: self,
            index: if self.has_error { 2 } else { 1 },
        };
    }

    /// True if this is an error fragment.
    pub fn has_error(&self) -> bool {
        return self.has_error;
    }

    /// If this token is an error token, return its error message, otherwise return `None`.
    pub fn error_message(&self) -> Option<&str> {
        return match self.has_error {
            true => self.meta.get(2).map(|s| s.as_str()),
            false => None,
        };
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
        assert_eq!(token.error_message(), None);
        assert!(!token.has_error());
    }

    #[test]
    fn test_error_token() {
        let mut token = Token::new_error(10, "error-test-rule", "test-value", "test-error");
        token.push_extra("extra_1");
        token.push_extra("extra_2");
        assert_eq!(token.rule(), "error-test-rule");
        assert_eq!(token.value(), "test-value");
        assert_eq!(token.extras().count(), 2);
        assert_eq!(token.error_message(), Some("test-error"));
        assert!(token.has_error());
    }
}
