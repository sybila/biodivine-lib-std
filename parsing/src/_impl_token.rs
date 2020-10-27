use crate::Token;
use std::ops::Index;

impl Token {
    /// Create a new standard token with no extra information.
    pub fn new(position: usize, rule: String, value: String) -> Token {
        if cfg!(feature = "shields_up") && rule.starts_with("error") {
            panic!("Missing error message for token '{}'[{}].", value, rule);
        }
        return Token {
            starts_at: position,
            meta: vec![rule, value],
        };
    }

    /// Create a new error token.
    pub fn new_error(position: usize, rule: String, value: String, message: String) -> Token {
        if cfg!(feature = "shields_up") && !rule.starts_with("error") {
            panic!("Unexpected error message for token '{}'[{}].", value, rule);
        }
        return Token {
            starts_at: position,
            meta: vec![rule, value, message],
        };
    }

    /// Add extra string data `value` to this token.
    pub fn push_extra(&mut self, value: String) {
        self.meta.push(value);
    }

    /// Index of the first character of this token in the input string.
    pub fn starts_at(&self) -> usize {
        return self.starts_at;
    }

    /// Name of the rule that generated this token.
    pub fn rule(&self) -> &String {
        return &self.meta[0];
    }

    /// Actual value of this token as seen in the input string.
    pub fn value(&self) -> &String {
        return &self.meta[1];
    }

    /// A slice of string extras attached to this token.
    pub fn extras(&self) -> &[String] {
        return &self.meta[2..];
    }

    /// True if this is an error fragment.
    pub fn is_error(&self) -> bool {
        return self.meta[0].starts_with("error");
    }

    /// If this token is an error token, return its error message, otherwise return `None`.
    pub fn error_message(&self) -> Option<&String> {
        if !self.is_error() {
            return None;
        }
        return self.meta.get(2);
    }
}

/// Indexing into the `extras` array of a token.
impl Index<usize> for Token {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.meta[index + 2];
    }
}

#[cfg(test)]
mod tests {
    use crate::Token;

    #[test]
    fn test_normal_token() {
        let mut token = Token::new(10, "test-rule".into(), "test-value".into());
        token.push_extra("extra_1".into());
        token.push_extra("extra_2".into());
        assert_eq!(token.rule(), &("test-rule".to_string()));
        assert_eq!(token.value(), &("test-value".to_string()));
        assert_eq!(token.extras().len(), 2);
        assert_eq!(token.extras()[0], ("extra_1".to_string()));
        assert_eq!(token.extras()[1], ("extra_2".to_string()));
        assert_eq!(token[0], "extra_1".to_string());
        assert_eq!(token[1], "extra_2".to_string());
        assert_eq!(token.error_message(), None);
        assert!(!token.is_error());
    }

    #[test]
    #[should_panic]
    fn test_error_as_normal() {
        Token::new(3, "error-test".into(), "value".into());
    }

    #[test]
    fn test_error_token() {
        let mut token = Token::new_error(10, "error-test-rule".into(), "test-value".into(), "test-error".into());
        token.push_extra("extra_1".to_string());
        token.push_extra("extra_2".to_string());
        assert_eq!(token.rule(), &("error-test-rule".to_string()));
        assert_eq!(token.value(), &("test-value".to_string()));
        assert_eq!(token.extras().len(), 3);
        assert_eq!(token.extras()[0], ("test-error".to_string()));
        assert_eq!(token.extras()[1], ("extra_1".to_string()));
        assert_eq!(token.extras()[2], ("extra_2".to_string()));
        assert_eq!(token.extras()[0], ("test-error".to_string()));
        assert_eq!(token[1], "extra_1".to_string());
        assert_eq!(token[2], "extra_2".to_string());
        assert_eq!(token.error_message(), Some(&("test-error".to_string())));
        assert!(token.is_error());
    }

    #[test]
    #[should_panic]
    fn test_normal_as_error() {
        Token::new_error(3, "my-rule".into(), "value".into(), "???".into());
    }

}
