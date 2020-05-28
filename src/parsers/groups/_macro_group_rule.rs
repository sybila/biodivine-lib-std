/// Create a new `GroupRule` with a given name and two string literals that exactly match the
/// opening and closing token data for that new rule.
///
/// ```rust
/// use biodivine_lib_std::const_data_group;
/// use biodivine_lib_std::parsers::groups::GroupRule;
/// use biodivine_lib_std::parsers::tokens::Token;
///
/// let rule: GroupRule<()> = const_data_group!("my group", "(", ")");
/// assert!(rule.is_group(
///     &Token { starts_at: 0, data: "(", payload: () },
///     &Token { starts_at: 10, data: ")", payload: () }
/// ));
/// ```
#[macro_export]
macro_rules! const_data_group {
    ( $n:expr, $o:expr, $c:expr ) => {{
        GroupRule::new(
            $n.to_string().as_str(),
            |t| t.data == $o,
            |t| t.data == $c,
            |o, c| {
                return o.data == $o && c.data == $c;
            },
        )
    }};
}

/// Create a new `GroupRule` with a given name and two payload **constants** that exactly match the
/// opening and closing token payloads. To use this macro, `Payload` must implement `PartialEq`.
///
/// ```rust
/// use biodivine_lib_std::const_group;
/// use biodivine_lib_std::parsers::groups::GroupRule;
/// use biodivine_lib_std::parsers::tokens::Token;
///
/// #[derive(Clone, PartialEq)]
/// enum P { Open, Close }
///
/// let rule: GroupRule<P> = const_group!("my group", P::Open, P::Close);
/// assert!(rule.is_group(
///     &Token { starts_at: 0, data: "(", payload: P::Open },
///     &Token { starts_at: 10, data: ")", payload: P::Close }
/// ));
/// ```
#[macro_export]
macro_rules! const_group {
    ( $n:expr, $o:expr, $c:expr ) => {{
        GroupRule::new(
            $n.to_string().as_str(),
            |t| t.payload == $o,
            |t| t.payload == $c,
            |o, c| {
                return o.payload == $o && c.payload == $c;
            },
        )
    }};
}

/// Create a new `GroupRule` with a given name and two **patterns** that match the
/// opening and closing token payloads.
///
/// ```rust
/// use biodivine_lib_std::pattern_group;
/// use biodivine_lib_std::parsers::groups::GroupRule;
/// use biodivine_lib_std::parsers::tokens::Token;
///
/// #[derive(Clone, PartialEq)]
/// enum P { Open(u32), Close(u32) }
///
/// let rule: GroupRule<P> = pattern_group!("my group", P::Open(_), P::Close(_));
/// assert!(rule.is_group(
///     &Token { starts_at: 0, data: "(", payload: P::Open(1) },
///     &Token { starts_at: 10, data: ")", payload: P::Close(2) }
/// ));
/// ```
#[macro_export]
macro_rules! pattern_group {
    ( $n:expr, $o:pat, $c:pat ) => {{
        GroupRule::new(
            $n.to_string().as_str(),
            |t| match t.payload {
                $o => true,
                _ => false,
            },
            |t| match t.payload {
                $c => true,
                _ => false,
            },
            |o, c| {
                let opens = match o.payload {
                    $o => true,
                    _ => false,
                };
                let closes = match c.payload {
                    $c => true,
                    _ => false,
                };
                return opens && closes;
            },
        )
    }};
}

#[cfg(test)]
mod tests {
    use crate::parsers::groups::GroupRule;
    use crate::parsers::tokens::Token;

    #[test]
    pub fn test_const_data_group_macro() {
        let group: GroupRule<()> = const_data_group!("parenthesis", "(", ")");
        let ref t1 = Token::new(0, "(", ());
        let ref t2 = Token::new(0, ")", ());
        let ref t3 = Token::new(0, "a", ());

        assert!(group.opens(t1));
        assert!(group.closes(t2));
        assert!(group.is_group(t1, t2));

        assert!(!group.opens(t3));
        assert!(!group.closes(t3));
        assert!(!group.is_group(t1, t3));
        assert!(!group.is_group(t3, t2));
    }

    #[test]
    pub fn test_const_payload_group_macro() {
        let group: GroupRule<&str> = const_group!("parenthesis", "open", "close");
        let ref t1 = Token::new(0, "(", "open");
        let ref t2 = Token::new(0, ")", "close");
        let ref t3 = Token::new(0, "a", "variable");

        assert!(group.opens(t1));
        assert!(group.closes(t2));
        assert!(group.is_group(t1, t2));

        assert!(!group.opens(t3));
        assert!(!group.closes(t3));
        assert!(!group.is_group(t1, t3));
        assert!(!group.is_group(t3, t2));
    }

    #[test]
    pub fn test_pattern_group_macro() {
        let group: GroupRule<Option<usize>> = pattern_group!("parenthesis", Some(1), Some(2));
        let ref t1 = Token::new(0, "(", Some(1));
        let ref t2 = Token::new(0, ")", Some(2));
        let ref t3 = Token::new(0, "a", None);

        assert!(group.opens(t1));
        assert!(group.closes(t2));
        assert!(group.is_group(t1, t2));

        assert!(!group.opens(t3));
        assert!(!group.closes(t3));
        assert!(!group.is_group(t1, t3));
        assert!(!group.is_group(t3, t2));
    }
}
