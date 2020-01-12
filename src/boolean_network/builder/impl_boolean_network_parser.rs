use crate::boolean_network::builder::{
    BooleanNetworkBuilder, RegulationTemplate, RegulatoryGraph, UpdateFunctionTemplate,
};
use crate::boolean_network::BooleanNetwork;
use regex::Regex;
use std::convert::TryFrom;

impl TryFrom<&str> for BooleanNetwork {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let function_re =
            Regex::new(r"^\$\s*(?P<name>[a-zA-Z0-9_]+)\s*:\s*(?P<function>.+)$").unwrap();

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

        let mut bn_builder = BooleanNetworkBuilder::new_from_regulatory_graph(regulatory_graph);

        for (variable, function) in update_functions {
            bn_builder.add_update_function(&variable, function)?;
        }

        return Ok(bn_builder.build());
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::{
        BooleanNetwork, Effect, Parameter, ParameterId, Regulation, UpdateFunction, Variable,
        VariableId,
    };
    use std::convert::TryFrom;

    #[test]
    fn test_boolean_network_parser() {
        let bn_string = "
            a -> b
            a -?? a
            b -|? c
            c -? a
            c -| d
            $a: a & (p(c) => c)
            $b: p(a) <=> q(a, a)
            $c: q(b, b) => (b ^ k)
        ";

        let f1 = UpdateFunction::And(
            Box::new(UpdateFunction::Variable { id: VariableId(0) }),
            Box::new(UpdateFunction::Imp(
                Box::new(UpdateFunction::Parameter {
                    id: ParameterId(0),
                    inputs: vec![VariableId(2)],
                }),
                Box::new(UpdateFunction::Variable { id: VariableId(2) }),
            )),
        );

        let f2 = UpdateFunction::Iff(
            Box::new(UpdateFunction::Parameter {
                id: ParameterId(0),
                inputs: vec![VariableId(0)],
            }),
            Box::new(UpdateFunction::Parameter {
                id: ParameterId(1),
                inputs: vec![VariableId(0), VariableId(0)],
            }),
        );

        let f3 = UpdateFunction::Imp(
            Box::new(UpdateFunction::Parameter {
                id: ParameterId(1),
                inputs: vec![VariableId(1), VariableId(1)],
            }),
            Box::new(UpdateFunction::Xor(
                Box::new(UpdateFunction::Variable { id: VariableId(1) }),
                Box::new(UpdateFunction::Parameter {
                    id: ParameterId(2),
                    inputs: Vec::new(),
                }),
            )),
        );

        let bn = BooleanNetwork {
            variables: vec![
                Variable {
                    name: "a".to_string(),
                },
                Variable {
                    name: "b".to_string(),
                },
                Variable {
                    name: "c".to_string(),
                },
                Variable {
                    name: "d".to_string(),
                },
            ],
            parameters: vec![
                Parameter {
                    name: "p".to_string(),
                    cardinality: 1,
                },
                Parameter {
                    name: "q".to_string(),
                    cardinality: 2,
                },
                Parameter {
                    name: "k".to_string(),
                    cardinality: 0,
                },
            ],
            regulations: vec![
                Regulation {
                    source: VariableId(0),
                    target: VariableId(1),
                    observable: true,
                    effect: Some(Effect::ACTIVATION),
                },
                Regulation {
                    source: VariableId(0),
                    target: VariableId(0),
                    observable: false,
                    effect: None,
                },
                Regulation {
                    source: VariableId(1),
                    target: VariableId(2),
                    observable: false,
                    effect: Some(Effect::INHIBITION),
                },
                Regulation {
                    source: VariableId(2),
                    target: VariableId(0),
                    observable: true,
                    effect: None,
                },
                Regulation {
                    source: VariableId(2),
                    target: VariableId(3),
                    observable: true,
                    effect: Some(Effect::INHIBITION),
                },
            ],
            update_functions: vec![Some(f1), Some(f2), Some(f3), None],
        };

        assert_eq!(bn, BooleanNetwork::try_from(bn_string).unwrap());
    }

}
