use crate::{Token, TokenExtras};
use std::ops::Index;

impl<'a> Iterator for TokenExtras<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        return self.token.meta.get(self.index).map(|i| i.as_str());
    }
}

/// Indexing into the `extras` array of a token.
impl Index<usize> for Token {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.meta[index + if self.has_error { 3 } else { 2 }];
    }
}
