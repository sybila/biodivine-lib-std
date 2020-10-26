use crate::parsers::tokens2::{
    ConstTokenMatcher, RegexTokenMatcher, SequenceTokenMatcher, StaticTokenMatcher,
    StaticTokenMatcherBox, WeakUntilTokenMatcher,
};
use regex::Regex;

impl ConstTokenMatcher {
    pub fn new(name: &str, value: &str) -> ConstTokenMatcher {
        return ConstTokenMatcher {
            name: name.to_string(),
            value: value.to_string(),
        };
    }
}

impl RegexTokenMatcher {
    pub fn new(name: &str, regex: &str) -> RegexTokenMatcher {
        return RegexTokenMatcher {
            name: name.to_string(),
            regex: Regex::new(&format!("^{}", regex)).unwrap(),
        };
    }
}

impl SequenceTokenMatcher {
    pub fn new(matchers: Vec<StaticTokenMatcherBox>) -> SequenceTokenMatcher {
        return SequenceTokenMatcher(matchers);
    }
}

impl WeakUntilTokenMatcher {
    pub fn new(name: &str, until: StaticTokenMatcherBox) -> WeakUntilTokenMatcher {
        return WeakUntilTokenMatcher {
            name: name.to_string(),
            until,
        };
    }
}

impl StaticTokenMatcher for ConstTokenMatcher {
    fn scan_token_static(&self, data: &str) -> Option<Vec<String>> {
        return match data.starts_with(&self.value) {
            true => Some(vec![self.name.clone(), self.value.clone()]),
            false => None,
        };
    }
}

impl StaticTokenMatcher for RegexTokenMatcher {
    fn scan_token_static(&self, data: &str) -> Option<Vec<String>> {
        return match self.regex.captures(data) {
            None => None,
            Some(c) => {
                let mut result = vec![self.name.clone()];
                for m in c.iter() {
                    result.push(m.map(|m| m.as_str()).unwrap_or("").to_string());
                }
                Some(result)
            }
        };
    }
}

impl StaticTokenMatcher for SequenceTokenMatcher {
    fn scan_token_static(&self, data: &str) -> Option<Vec<String>> {
        for m in &self.0 {
            let matched = m.scan_token_static(data);
            if matched.is_some() {
                return matched;
            }
        }
        return None;
    }
}

impl StaticTokenMatcher for WeakUntilTokenMatcher {
    fn scan_token_static(&self, data: &str) -> Option<Vec<String>> {
        let mut i = 0;
        while i < data.len() {
            if let Some(blocker) = self.until.scan_token_static(&data[i..]) {
                return if i == 0 {
                    None
                } else {
                    let mut token = vec![self.name.clone(), data[..i].to_string()];
                    for s in blocker {
                        token.push(s);
                    }
                    Some(token)
                };
            }
            i += 1;
        }
        // Read the whole string, haven't found until
        return Some(vec![self.name.clone(), data.to_string()]);
    }
}

#[cfg(test)]
mod tests {
    use crate::parsers::tokens2::{
        ConstTokenMatcher, RegexTokenMatcher, SequenceTokenMatcher, StaticTokenMatcher,
        WeakUntilTokenMatcher,
    };

    #[test]
    pub fn test_const_matcher() {
        let m = ConstTokenMatcher::new("test-matcher", "<=>");
        let match1 = m.scan_token_static("<=> hello");
        let match2 = m.scan_token_static("hello <=>");
        assert!(match1.is_some());
        assert!(match2.is_none());
        let match1 = match1.unwrap();
        assert_eq!(match1.len(), 2);
        assert_eq!(&match1[0], "test-matcher");
        assert_eq!(&match1[1], "<=>");
    }

    #[test]
    pub fn test_regex_matcher() {
        let m = RegexTokenMatcher::new("test-matcher", r"([a-z][a-zA-Z]*):([0-9]+)");
        let match1 = m.scan_token_static("hello:42 and tokens");
        let match2 = m.scan_token_static("tokens and hello:42");
        assert!(match1.is_some());
        assert!(match2.is_none());
        let match1 = match1.unwrap();
        assert_eq!(match1.len(), 4);
        assert_eq!(&match1[0], "test-matcher");
        assert_eq!(&match1[1], "hello:42");
        assert_eq!(&match1[2], "hello");
        assert_eq!(&match1[3], "42");
    }

    #[test]
    pub fn test_sequence_matcher() {
        let m = SequenceTokenMatcher::new(vec![
            Box::new(ConstTokenMatcher::new("plus", "+")),
            Box::new(RegexTokenMatcher::new("identifier", "[a-z]+")),
        ]);
        let match1 = m.scan_token_static("hello+bye").unwrap();
        let match2 = m.scan_token_static("+bye").unwrap();
        let match3 = m.scan_token_static("12456+bye");
        assert!(match3.is_none());
        assert_eq!(&match1[0], "identifier");
        assert_eq!(&match1[1], "hello");
        assert_eq!(&match2[0], "plus");
        assert_eq!(&match2[1], "+");
    }

    #[test]
    pub fn test_weak_until_matcher() {
        let new_line = ConstTokenMatcher::new("new-line", "\n");
        let match_line = WeakUntilTokenMatcher::new("line", Box::new(new_line));
        let match1 = match_line
            .scan_token_static("hello world\nmultiline")
            .unwrap();
        let match2 = match_line.scan_token_static("\nmultiline");
        let match3 = match_line.scan_token_static("multiline").unwrap();
        assert!(match2.is_none());
        assert_eq!(&match1[0], "line");
        assert_eq!(&match1[1], "hello world");
        assert_eq!(&match1[2], "new-line");
        assert_eq!(&match1[3], "\n");
        assert_eq!(&match3[0], "line");
        assert_eq!(&match3[1], "multiline");
    }
}
