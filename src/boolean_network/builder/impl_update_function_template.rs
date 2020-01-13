use crate::boolean_network::builder::UpdateFunctionTemplate::*;
use crate::boolean_network::builder::{RegulatoryGraph, UpdateFunctionTemplate};
use crate::boolean_network::Parameter as BNParameter;
use crate::boolean_network::Variable as BNVariable;
use crate::boolean_network::{ParameterId, UpdateFunction, VariableId};
use std::collections::{HashMap, HashSet};

impl UpdateFunctionTemplate {
    /// Swap variables in this function that don't occur in the given `rg` for unary parameters.
    pub fn swap_unary_parameters(self, rg: &RegulatoryGraph) -> Box<UpdateFunctionTemplate> {
        return Box::new(match self {
            Variable { name } => {
                if rg.get_variable_id(&name) != None {
                    Variable { name }
                } else {
                    Parameter {
                        name,
                        inputs: Vec::new(),
                    }
                }
            }
            Parameter { .. } => self,
            Not(inner) => Not(inner.swap_unary_parameters(rg)),
            And(a, b) => And(a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)),
            Or(a, b) => Or(a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)),
            Imp(a, b) => Imp(a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)),
            Iff(a, b) => Iff(a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)),
            Xor(a, b) => Xor(a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)),
        });
    }

    /// Find all parameters in this update function and put them in a separate hash set.
    pub fn extract_parameters(&self) -> HashSet<BNParameter> {
        return match self {
            Parameter { name, inputs } => {
                let mut set = HashSet::new();
                set.insert(BNParameter {
                    name: name.clone(),
                    cardinality: inputs.len(),
                });
                set
            }
            Variable { .. } => HashSet::new(),
            Not(inner) => inner.extract_parameters(),
            And(a, b) => extract_parameters_util(a, b),
            Or(a, b) => extract_parameters_util(a, b),
            Imp(a, b) => extract_parameters_util(a, b),
            Iff(a, b) => extract_parameters_util(a, b),
            Xor(a, b) => extract_parameters_util(a, b),
        };
    }

    /// Find all variables in this update function and put them in a separate hash set.
    pub fn extract_variables(&self) -> HashSet<BNVariable> {
        return match self {
            Parameter { .. } => HashSet::new(),
            Variable { name } => {
                let mut set = HashSet::new();
                set.insert(BNVariable { name: name.clone() });
                return set;
            }
            Not(inner) => inner.extract_variables(),
            And(a, b) => extract_variable_util(a, b),
            Or(a, b) => extract_variable_util(a, b),
            Imp(a, b) => extract_variable_util(a, b),
            Iff(a, b) => extract_variable_util(a, b),
            Xor(a, b) => extract_variable_util(a, b),
        };
    }

    /// Transform this template into a full-on `UpdateFunction`.
    pub fn build(
        &self,
        variable_to_index: &HashMap<String, VariableId>,
        parameter_to_index: &HashMap<String, ParameterId>,
    ) -> Result<Box<UpdateFunction>, String> {
        return Ok(Box::new(match self {
            Variable { name } => {
                let index = variable_to_index.get(name).ok_or(format!(
                    "Can't build update function. Unknown variable {}.",
                    name
                ))?;
                UpdateFunction::Variable { id: *index }
            }
            Parameter { name, inputs } => {
                let index = parameter_to_index.get(name).ok_or(format!(
                    "Can't build update function. Unknown parameter {}.",
                    name
                ))?;
                let mut args = Vec::with_capacity(inputs.len());
                for input in inputs {
                    let index = variable_to_index.get(input).ok_or(format!(
                        "Can't build update function. Unknown variable {} in {}",
                        input, self
                    ))?;
                    args.push(*index);
                }
                UpdateFunction::Parameter {
                    id: *index,
                    inputs: args,
                }
            }
            Not(inner) => UpdateFunction::Not(inner.build(variable_to_index, parameter_to_index)?),
            And(a, b) => UpdateFunction::And(
                a.build(variable_to_index, parameter_to_index)?,
                b.build(variable_to_index, parameter_to_index)?,
            ),
            Or(a, b) => UpdateFunction::Or(
                a.build(variable_to_index, parameter_to_index)?,
                b.build(variable_to_index, parameter_to_index)?,
            ),
            Imp(a, b) => UpdateFunction::Imp(
                a.build(variable_to_index, parameter_to_index)?,
                b.build(variable_to_index, parameter_to_index)?,
            ),
            Iff(a, b) => UpdateFunction::Iff(
                a.build(variable_to_index, parameter_to_index)?,
                b.build(variable_to_index, parameter_to_index)?,
            ),
            Xor(a, b) => UpdateFunction::Xor(
                a.build(variable_to_index, parameter_to_index)?,
                b.build(variable_to_index, parameter_to_index)?,
            ),
        }));
    }
}

fn extract_parameters_util(
    a: &UpdateFunctionTemplate,
    b: &UpdateFunctionTemplate,
) -> HashSet<BNParameter> {
    let mut a = a.extract_parameters();
    a.extend(b.extract_parameters());
    return a;
}

fn extract_variable_util(
    a: &UpdateFunctionTemplate,
    b: &UpdateFunctionTemplate,
) -> HashSet<BNVariable> {
    let mut a = a.extract_variables();
    a.extend(b.extract_variables());
    return a;
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::builder::RegulatoryGraph;
    use crate::boolean_network::builder::UpdateFunctionTemplate;
    use crate::boolean_network::Parameter as BNParameter;
    use crate::boolean_network::Variable as BNVariable;
    use std::collections::HashSet;
    use std::convert::TryFrom;

    #[test]
    fn test_swap_unary_parameters() {
        let rg = RegulatoryGraph::new(&vec![
            "abc".to_string(),
            "as123".to_string(),
            "hello".to_string(),
        ]);
        let function =
            UpdateFunctionTemplate::try_from("f & (!abc | as123_param => p(abc, hello))").unwrap();
        let expected =
            UpdateFunctionTemplate::try_from("f() & (!abc | as123_param() => p(abc, hello))")
                .unwrap();

        assert_eq!(expected, *function.swap_unary_parameters(&rg));
    }

    #[test]
    fn test_extract_parameters() {
        let function =
            UpdateFunctionTemplate::try_from("f() & !var1 => ((par(a, b, c) | g) <=> q(a))").unwrap();
        let params = function.extract_parameters();
        let mut expected = HashSet::new();
        expected.insert(BNParameter {
            name: "f".to_string(),
            cardinality: 0,
        });
        expected.insert(BNParameter {
            name: "par".to_string(),
            cardinality: 3,
        });
        expected.insert(BNParameter {
            name: "q".to_string(),
            cardinality: 1,
        });

        assert_eq!(expected, params);
    }

    #[test]
    fn test_extract_variables() {
        let function =
            UpdateFunctionTemplate::try_from("f() & !var1 => ((par(a, b, c) | g) <=> q(a))").unwrap();
        let params = function.extract_variables();
        let mut expected = HashSet::new();
        expected.insert(BNVariable { name: "var1".to_string() });
        expected.insert(BNVariable { name: "g".to_string() });

        assert_eq!(expected, params);
    }

}
