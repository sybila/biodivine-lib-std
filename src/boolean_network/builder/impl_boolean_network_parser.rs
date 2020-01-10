use std::convert::TryFrom;
use crate::boolean_network::BooleanNetwork;
use crate::boolean_network::builder::{RegulationTemplate, RegulatoryGraph, UpdateFunctionTemplate};
use regex::Regex;

impl TryFrom<&str> for BooleanNetwork {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let function_re = Regex::new(r"^\$\s*(?P<name>[a-zA-Z0-9_]+)\s*:\s*(?P<function>.+)$").unwrap();

        // Every line that is not empty and does not match an
        // update function pattern must be parsed into a regulation.
        let mut regulations: Vec<RegulationTemplate> = Vec::new();
        for line in value.lines() {
            let line = line.trim();
            if !line.is_empty() && !function_re.is_match(line) {
                regulations.push(RegulationTemplate::try_from(line)?);
            }
        }

        // Every line that matches the update function pattern must be parsed as update function.
        let mut update_functions: Vec<(String, UpdateFunctionTemplate)> = Vec::new();
        for line in value.lines() {
            let line = line.trim();
            if let Some(captures) = function_re.captures(line) {
                let name = captures["name"].to_string();
                let function = UpdateFunctionTemplate::try_from(&captures["function"])?;
                update_functions.push((name, function));
            }
        }

        let regulatory_graph = RegulatoryGraph::from_regulations(regulations);

        return Err("".to_string())
    }
}