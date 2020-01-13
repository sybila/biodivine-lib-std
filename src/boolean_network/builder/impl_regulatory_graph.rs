use crate::boolean_network::builder::{RegulationTemplate, RegulatoryGraph};
use crate::boolean_network::{Effect, Regulation, Variable, VariableId};
use crate::util::build_index_map;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::iter::Map;
use std::ops::Range;

impl RegulatoryGraph {
    /// Create a new empty `RegulatoryGraph` with given `variables`.
    ///
    /// The ordering of the variables will be kept as provided.
    pub fn new(variables: &Vec<String>) -> RegulatoryGraph {
        return RegulatoryGraph {
            variables: variables
                .iter()
                .map(|name| Variable { name: name.clone() })
                .collect(),
            regulations: Vec::new(),
            variable_to_index: build_index_map(variables, |_, i| VariableId(i)),
        };
    }

    /// Create a `RegulatoryGraph` from a vector of regulation strings.
    ///
    /// This is equivalent to calling `add_regulation_string` for each member of the vector.
    /// However, the set of graph variables is determined only from the regulations
    /// and the variables are ordered alphabetically.
    pub fn from_regulation_strings(regulations: Vec<String>) -> Result<RegulatoryGraph, String> {
        let mut parsed_regulations = Vec::with_capacity(regulations.len());
        for r in regulations {
            parsed_regulations.push(RegulationTemplate::try_from(r.as_str())?);
        }

        let mut variable_names = HashSet::new();
        for r in &parsed_regulations {
            variable_names.insert(r.source.clone());
            variable_names.insert(r.target.clone());
        }

        let mut variable_names: Vec<String> = variable_names.into_iter().collect();
        variable_names.sort();

        let mut rg = RegulatoryGraph::new(&variable_names);

        for r in parsed_regulations {
            rg.add_regulation(&r.source, &r.target, r.observable, r.effect)?;
        }

        return Ok(rg);
    }

    /// Add a new regulation to the graph. If `source` or `target` are not graph variables,
    /// returns error. Also returns error if such regulation already exists.
    pub fn add_regulation(
        &mut self,
        source: &str,
        target: &str,
        observable: bool,
        effect: Option<Effect>,
    ) -> Result<(), String> {
        let s = *self.variable_to_index.get(source).ok_or(format!(
            "Can't make a regulation. Unknown regulator species: {}",
            source
        ))?;
        let t = *self.variable_to_index.get(target).ok_or(format!(
            "Can't make a regulation. Unknown regulated species: {}",
            target
        ))?;
        if self.get_regulation(s, t) != None {
            return Err(format!(
                "Can't make a regulation. Regulation ({},{}) already defined.",
                source, target
            ));
        }
        self.regulations.push(Regulation {
            source: s,
            target: t,
            observable,
            effect,
        });
        return Ok(());
    }

    /// Add a new regulation by parsing it from a string. Source and target variables must
    /// be present in the graph.
    pub fn add_regulation_string(&mut self, regulation_string: &str) -> Result<(), String> {
        let r = RegulationTemplate::try_from(regulation_string)?;
        self.add_regulation(&r.source, &r.target, r.observable, r.effect)?;
        return Ok(());
    }

    /// Find a `VariableId` for the given variable name or `None` if such variable does not exist.
    pub fn get_variable_id(&self, var: &str) -> Option<VariableId> {
        return self.variable_to_index.get(var).map(|id| *id);
    }

    /// Obtain a `Variable` using a `VariableId`. Panics if the id is not valid in this graph.
    pub fn get_variable(&self, id: VariableId) -> &Variable {
        return &self.variables[id.0];
    }

    /// Find a `Regulation` between two given variables. If such regulation does not exist,
    /// return `None`
    pub fn get_regulation(&self, source: VariableId, target: VariableId) -> Option<&Regulation> {
        for r in &self.regulations {
            if r.source == source && r.target == target {
                return Some(r);
            }
        }
        return None;
    }

    pub fn num_vars(&self) -> usize {
        return self.variables.len();
    }

    pub fn variable_ids(&self) -> Map<Range<usize>, fn(usize) -> VariableId> {
        return (0..self.variables.len()).map(|i| VariableId(i));
    }

    pub fn regulations(&self) -> &Vec<Regulation> {
        return &self.regulations;
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
        assert_eq!(make_rg(), rg.unwrap());
    }

    #[test]
    fn test_regulatory_graph_from_individual_regulations() {
        let mut rg = RegulatoryGraph::new(&vec![
            "abc".to_string(),
            "hello".to_string(),
            "numbers_123".to_string(),
        ]);
        rg.add_regulation_string("abc -> hello").unwrap();
        rg.add_regulation_string("hello -|? abc").unwrap();
        rg.add_regulation_string("numbers_123 -?? abc").unwrap();
        rg.add_regulation_string("numbers_123 -? hello").unwrap();
        assert_eq!(make_rg(), rg);
    }
}
