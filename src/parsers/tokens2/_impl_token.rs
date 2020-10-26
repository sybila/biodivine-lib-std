use crate::parsers::tokens2::Token;

impl Token {
    /// Index of the first position of this token in the original string.
    pub fn starts_at(&self) -> usize {
        return self.starts_at;
    }

    /// Index after the last position of this token in the original string.
    pub fn ends_at(&self) -> usize {
        return self.starts_at + self.data[1].len();
    }

    /// String identifier of the rule that matched this token.
    pub fn rule(&self) -> &str {
        return &self.data[0];
    }

    /// Get the actual string value of this token.
    pub fn value(&self) -> &str {
        return &self.data[0];
    }

    /// Get additional string data from the token. These depend on the tokenizer which
    /// matched the token. For example, regex tokenizers will insert all matched groups
    /// from the regex into this data.
    pub fn get_extra(&self, i: usize) -> Option<&str> {
        return self.data.get(i + 1).map(|s| s.as_str());
    }
}
