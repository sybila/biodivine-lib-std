use crate::boolean_network::builder::RegulationTemplate;
use crate::boolean_network::Effect;
use regex::Regex;
use std::convert::TryFrom;

impl TryFrom<&str> for RegulationTemplate {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let re = Regex::new(r"^\s*(?P<source>[a-zA-Z0-9_]+)\s*-(?P<effect>[|>?])(?P<observable>\??)\s*(?P<target>[a-zA-Z0-9_]+)\s*$").unwrap();
        let captures = re.captures(value);
        return if let Some(captures) = captures {
            Ok(RegulationTemplate {
                source: captures["source"].to_string(),
                target: captures["target"].to_string(),
                observable: captures["observable"].is_empty(),
                effect: match &captures["effect"] {
                    "?" => None,
                    "|" => Some(Effect::INHIBITION),
                    ">" => Some(Effect::ACTIVATION),
                    _ => unreachable!("Nothing else matches this group."),
                },
            })
        } else {
            Err(format!("Line {} does not describe a regulation", value))
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::builder::RegulationTemplate;
    use crate::boolean_network::Effect;
    use std::convert::TryFrom;

    #[test]
    fn parse_regulation_valid() {
        assert_eq!(
            RegulationTemplate {
                source: "abc".to_string(),
                target: "123".to_string(),
                observable: true,
                effect: Some(Effect::ACTIVATION)
            },
            RegulationTemplate::try_from("  abc -> 123 ").unwrap()
        );

        assert_eq!(
            RegulationTemplate {
                source: "abc".to_string(),
                target: "123".to_string(),
                observable: false,
                effect: Some(Effect::ACTIVATION)
            },
            RegulationTemplate::try_from("  abc ->? 123 ").unwrap()
        );

        assert_eq!(
            RegulationTemplate {
                source: "hello_world".to_string(),
                target: "world_hello_123".to_string(),
                observable: true,
                effect: Some(Effect::INHIBITION)
            },
            RegulationTemplate::try_from("hello_world -| world_hello_123").unwrap()
        );

        assert_eq!(
            RegulationTemplate {
                source: "hello_world".to_string(),
                target: "world_hello_123".to_string(),
                observable: false,
                effect: Some(Effect::INHIBITION)
            },
            RegulationTemplate::try_from("hello_world -|? world_hello_123").unwrap()
        );

        assert_eq!(
            RegulationTemplate {
                source: "abc".to_string(),
                target: "abc".to_string(),
                observable: true,
                effect: None
            },
            RegulationTemplate::try_from("abc -? abc").unwrap()
        );

        assert_eq!(
            RegulationTemplate {
                source: "abc".to_string(),
                target: "abc".to_string(),
                observable: false,
                effect: None
            },
            RegulationTemplate::try_from("abc -?? abc").unwrap()
        );
    }

    #[test]
    fn parse_regulation_invalid() {
        assert!(RegulationTemplate::try_from("").is_err());
        assert!(RegulationTemplate::try_from("var1 var2 -> var3").is_err());
        assert!(RegulationTemplate::try_from("var -| v?r").is_err());
        assert!(RegulationTemplate::try_from(" -? foo").is_err());
        assert!(RegulationTemplate::try_from("hello -?> there").is_err());
        assert!(RegulationTemplate::try_from("world -??? is").is_err());
        assert!(RegulationTemplate::try_from("   te - ? st").is_err());
    }
}
