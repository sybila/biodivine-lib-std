use crate::parsers::groups::GroupRule;
use crate::parsers::tokens::Token;
use std::fmt::{Debug, Formatter};

impl<Payload> GroupRule<Payload>
where
    Payload: Clone,
{
    /// Creates a new `GroupRule` with specified name and test functions.
    ///
    /// If your rule does does not require complex logic, check out `pattern_group`,
    /// `const_group`, and `const_data_group` macros, which will generate rules using
    /// simpler conditions.
    pub fn new(
        name: &str,
        opens: fn(&Token<Payload>) -> bool,
        closes: fn(&Token<Payload>) -> bool,
        is_group: fn(&Token<Payload>, &Token<Payload>) -> bool,
    ) -> GroupRule<Payload> {
        return GroupRule {
            name: name.to_string(),
            opens,
            closes,
            is_group,
        };
    }

    /// Tests whether the given `token` opens a group defined by this rule.
    pub fn opens(&self, token: &Token<Payload>) -> bool {
        return (self.opens)(token);
    }

    /// Tests whether the given `token` closes a group defined by this rule.
    pub fn closes(&self, token: &Token<Payload>) -> bool {
        return (self.closes)(token);
    }

    /// Tests whether this specific combination of tokens represents a valid group.
    pub fn is_group(&self, open: &Token<Payload>, close: &Token<Payload>) -> bool {
        return (self.is_group)(open, close);
    }
}

impl<Payload: Clone> Debug for GroupRule<Payload> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "GroupRule({})", self.name);
    }
}
