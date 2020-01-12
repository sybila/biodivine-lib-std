use crate::boolean_network::builder::{RegulationTemplate, RegulatoryGraph};
use crate::boolean_network::{Effect, Regulation, Variable, VariableId};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

impl RegulatoryGraph {
    /// Create a regulatory graph same as in `from_regulations`, but first each string
    /// is parsed using the standard procedure for creating regulations.
    pub fn from_regulation_strings(regulations: Vec<String>) -> RegulatoryGraph {
        return RegulatoryGraph::from_regulations(
            regulations
                .into_iter()
                .map(|r| RegulationTemplate::try_from(r.as_str()).unwrap())
                .collect(),
        );
    }

    /// Create a regulatory graph from given list of regulation templates.
    ///
    /// The variables of the graph will be created as needed and sorted alphabetically.
    pub fn from_regulations(regulations: Vec<RegulationTemplate>) -> RegulatoryGraph {
        let mut variables = HashSet::new();
        for reg in &regulations {
            variables.insert(reg.source.clone());
            variables.insert(reg.target.clone());
        }
        let mut variables: Vec<Variable> = variables
            .into_iter()
            .map(|s| Variable { name: s })
            .collect();
        variables.sort();
        let mut variable_to_index = HashMap::new();
        for i in 0..variables.len() {
            variable_to_index.insert(variables[i].name.clone(), VariableId(i));
        }
        let regulations: Vec<Regulation> = regulations
            .into_iter()
            .map(|reg| Regulation {
                source: *variable_to_index.get(&reg.source).unwrap(),
                target: *variable_to_index.get(&reg.target).unwrap(),
                observable: reg.observable,
                effect: reg.effect,
            })
            .collect();
        return RegulatoryGraph {
            regulations,
            variables,
            variable_to_index,
        };
    }

    /// Create a new regulatory graph with given variables and no regulations.
    ///
    /// The ordering of the variables will be kept as provided.
    pub fn new(variables: Vec<String>) -> RegulatoryGraph {
        let mut variables_to_index = HashMap::new();
        for i in 0..variables.len() {
            variables_to_index.insert(variables[i].clone(), VariableId(i));
        }
        return RegulatoryGraph {
            variables: variables
                .into_iter()
                .map(|name| Variable { name })
                .collect(),
            regulations: Vec::new(),
            variable_to_index: variables_to_index,
        };
    }

    /// Add a new regulation. No new variables will be created, the function will panic instead.
    pub fn add_regulation(
        &mut self,
        source: &str,
        target: &str,
        observable: bool,
        effect: Option<Effect>,
    ) {
        self.regulations.push(Regulation {
            source: self.variable_to_index[source],
            target: self.variable_to_index[target],
            observable,
            effect,
        })
    }

    /// Add a new regulation by parsing it from a string. No new variables will be created.
    pub fn add_regulation_string(&mut self, regulation_string: &str) {
        let regulation = RegulationTemplate::try_from(regulation_string).unwrap();
        self.add_regulation(
            &regulation.source,
            &regulation.target,
            regulation.observable,
            regulation.effect,
        );
    }

    pub fn has_variable(&self, var: &str) -> bool {
        return self.variable_to_index.contains_key(var);
    }

    pub fn has_regulation(&self, source: VariableId, target: VariableId) -> bool {
        for r in &self.regulations {
            if r.source == source && r.target == target {
                return true;
            }
        }
        return false;
    }

    pub fn get_variable(&self, id: VariableId) -> &Variable {
        return &self.variables[id.0];
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::builder::RegulatoryGraph;
    use crate::boolean_network::{Effect, Regulation, Variable, VariableId};
    use std::collections::HashMap;

    fn make_rg() -> RegulatoryGraph {
        let mut map = HashMap::new();
        map.insert("abc".to_string(), VariableId(0));
        map.insert("hello".to_string(), VariableId(1));
        map.insert("numbers_123".to_string(), VariableId(2));
        return RegulatoryGraph {
            variables: vec![
                Variable {
                    name: "abc".to_string(),
                },
                Variable {
                    name: "hello".to_string(),
                },
                Variable {
                    name: "numbers_123".to_string(),
                },
            ],
            regulations: vec![
                Regulation {
                    // abc -> hello
                    source: VariableId(0),
                    target: VariableId(1),
                    observable: true,
                    effect: Some(Effect::ACTIVATION),
                },
                Regulation {
                    // hello -|? abc
                    source: VariableId(1),
                    target: VariableId(0),
                    observable: false,
                    effect: Some(Effect::INHIBITION),
                },
                Regulation {
                    // numbers_123 -?? abc
                    source: VariableId(2),
                    target: VariableId(0),
                    observable: false,
                    effect: None,
                },
                Regulation {
                    // numbers_123 -? hello
                    source: VariableId(2),
                    target: VariableId(1),
                    observable: true,
                    effect: None,
                },
            ],
            variable_to_index: map,
        };
    }

    #[test]
    fn test_regulatory_graph_from_regulation_list() {
        let rg = RegulatoryGraph::from_regulation_strings(vec![
            "abc -> hello".to_string(),
            "hello -|? abc".to_string(),
            "numbers_123 -?? abc".to_string(),
            "numbers_123 -? hello".to_string(),
        ]);
        assert_eq!(make_rg(), rg);
    }

    #[test]
    fn test_regulatory_graph_from_individual_regulations() {
        let mut rg = RegulatoryGraph::new(vec![
            "abc".to_string(),
            "hello".to_string(),
            "numbers_123".to_string(),
        ]);
        rg.add_regulation_string("abc -> hello");
        rg.add_regulation_string("hello -|? abc");
        rg.add_regulation_string("numbers_123 -?? abc");
        rg.add_regulation_string("numbers_123 -? hello");
        assert_eq!(make_rg(), rg);
    }
}
