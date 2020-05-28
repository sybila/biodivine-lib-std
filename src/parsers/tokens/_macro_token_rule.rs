/// Create a `TokenRule` which always produces the same constant payload.
#[macro_export]
macro_rules! const_token {
    ( $r:expr, $c:expr ) => {{
        TokenRule::new($r, |_| $c)
    }};
}

#[cfg(test)]
mod tests {
    use crate::parsers::tokens::TokenRule;

    #[test]
    pub fn test_const_token() {
        let token: TokenRule<u32> = const_token!("<=>", 10);
        assert_eq!(token.try_match("<=> a").unwrap().1, 10);
    }
}
