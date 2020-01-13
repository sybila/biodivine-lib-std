use crate::boolean_network::builder::RegulatoryGraph;
use crate::boolean_network::BooleanNetwork;
use regex::Regex;
use std::convert::TryFrom;

impl TryFrom<&str> for BooleanNetwork {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Regex that matches lines which define an update function.
        let function_re =
            Regex::new(r"^\$\s*(?P<name>[a-zA-Z0-9_]+)\s*:\s*(?P<function>.+)$").unwrap();

        // Every non-empty line that is not an update function is considered to be a regulation:
        let mut regulations: Vec<String> = Vec::new();
        for line in value.lines() {
            let line = line.trim();
            if !line.is_empty() && !function_re.is_match(line) {
                regulations.push(line.to_string());
            }
        }

        let regulatory_graph = RegulatoryGraph::from_regulation_strings(regulations)?;
        let mut bn = BooleanNetwork::new(regulatory_graph);

        for line in value.lines() {
            if let Some(captures) = function_re.captures(line.trim()) {
                bn.add_update_function(&captures["name"], &captures["function"])?;
            }
        }

        return Ok(bn);
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::builder::RegulatoryGraph;
    use crate::boolean_network::{
        BooleanNetwork, Parameter, ParameterId, UpdateFunction, VariableId,
    };
    use crate::util::build_index_map;
    use std::convert::TryFrom;

    #[test]
    fn test_boolean_network_parser() {
        let bn_string = "
            a -> b
            a -?? a
            b -|? c
            c -? a
            c -| d
            $a: a & (p(c) => (c | c))
            $b: p(a) <=> q(a, a)
            $c: q(b, b) => !(b ^ k)
        ";

        let f1 = UpdateFunction::And(
            Box::new(UpdateFunction::Variable { id: VariableId(0) }),
            Box::new(UpdateFunction::Imp(
                Box::new(UpdateFunction::Parameter {
                    id: ParameterId(0),
                    inputs: vec![VariableId(2)],
                }),
                Box::new(UpdateFunction::Or(
                    Box::new(UpdateFunction::Variable { id: VariableId(2) }),
                    Box::new(UpdateFunction::Variable { id: VariableId(2) }),
                )),
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
            Box::new(UpdateFunction::Not(Box::new(UpdateFunction::Xor(
                Box::new(UpdateFunction::Variable { id: VariableId(1) }),
                Box::new(UpdateFunction::Parameter {
                    id: ParameterId(2),
                    inputs: Vec::new(),
                }),
            )))),
        );

        let mut rg = RegulatoryGraph::new(&vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ]);
        rg.add_regulation_string("a -> b").unwrap();
        rg.add_regulation_string("a -?? a").unwrap();
        rg.add_regulation_string("b -|? c").unwrap();
        rg.add_regulation_string("c -? a").unwrap();
        rg.add_regulation_string("c -| d").unwrap();

        let parameters = vec![
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
        ];

        let bn = BooleanNetwork {
            regulatory_graph: rg,
            parameter_to_index: build_index_map(
                &parameters.iter().map(|p| p.name.clone()).collect(),
                |_, i| ParameterId(i),
            ),
            parameters,
            update_functions: vec![Some(f1), Some(f2), Some(f3), None],
        };

        assert_eq!(bn, BooleanNetwork::try_from(bn_string).unwrap());
    }
}
