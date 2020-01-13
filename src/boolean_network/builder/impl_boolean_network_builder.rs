use crate::boolean_network::builder::{RegulatoryGraph, UpdateFunctionTemplate};
use crate::boolean_network::{BooleanNetwork, ParameterId};
use std::collections::HashMap;
use std::convert::TryFrom;

impl BooleanNetwork {
    /// Create a new `BooleanNetwork` without any `UpdateFunction`s or parameters.
    pub fn new(graph: RegulatoryGraph) -> BooleanNetwork {
        let num_vars = graph.num_vars();
        return BooleanNetwork {
            regulatory_graph: graph,
            parameters: Vec::new(),
            parameter_to_index: HashMap::new(),
            update_functions: vec![None; num_vars],
        };
    }

    /// Add a new update function to an existing `BooleanNetwork`.
    ///
    /// Parameters present in `update_function` are added to the network when needed.
    ///
    /// Returns an error if the network has no such variable or there already is an update function
    /// for the given variable. Also, `update_function` must be a valid `UpdateFunction` and
    /// cannot clash with other parameter definitions (in name or cardinality)
    pub fn add_update_function(
        &mut self,
        variable: &str,
        update_function: &str,
    ) -> Result<(), String> {
        let variable_index = self
            .regulatory_graph
            .get_variable_id(variable)
            .ok_or(format!(
                "Can't add update function. Unknown variable {}.",
                variable
            ))?;

        if let Some(_) = self.update_functions[variable_index.0] {
            return Err(format!(
                "Can't add update function. Function for {} already set.",
                variable
            ));
        }

        let update_function = UpdateFunctionTemplate::try_from(update_function)?;
        let update_function = *update_function.swap_unary_parameters(&self.regulatory_graph);

        let parameters = update_function.extract_parameters();

        // Check if parameters are used correctly
        for p_in_f in &parameters {
            if self.regulatory_graph.get_variable_id(p_in_f.name.as_str()) != None {
                return Err(format!("Can't add update function for {}. {} can't be both a parameter and a variable.", variable, p_in_f.name));
            }
            if let Some(id) = self.get_parameter_id(p_in_f.name.as_str()) {
                // This is an existing parameter - check consistency.
                let p_in_bn = self.get_parameter(id);
                if p_in_f.cardinality != p_in_bn.cardinality {
                    return Err(format!(
                        "Can't add update function for {}. {} appears with cardinality {} and {}.",
                        variable, p_in_f.name, p_in_f.cardinality, p_in_bn.cardinality
                    ));
                }
            }
        }

        // Check if regulation constraints are satisfied.
        let variables = update_function.extract_variables();
        for var in &variables {
            let regulator_id = self
                .regulatory_graph
                .get_variable_id(var.name.as_str())
                .ok_or(format!(
                    "Can't add update function for {}. Function contains unknown variable {}.",
                    variable, var.name
                ))?;

            if self
                .regulatory_graph
                .get_regulation(regulator_id, variable_index)
                == None
            {
                return Err(format!(
                    "Can't add update function for {}. Variable {} does not regulate {}.",
                    variable, var.name, variable
                ));
            }
        }

        // Actually add new parameters now that everything is verified.
        for p_in_f in &parameters {
            if self.get_parameter_id(p_in_f.name.as_str()) == None {
                self.parameter_to_index
                    .insert(p_in_f.name.clone(), ParameterId(self.parameters.len()));
                self.parameters.push(p_in_f.clone());
            }
        }

        // Now we can build the update function.
        let update_function = *update_function.build(
            &self.regulatory_graph.variable_to_index,
            &self.parameter_to_index,
        )?;

        self.update_functions[variable_index.0] = Some(update_function);

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::builder::RegulatoryGraph;
    use crate::boolean_network::BooleanNetwork;

    #[test]
    fn test_invalid_update_function() {
        let mut rg = RegulatoryGraph::new(&vec!["a".to_string(), "b".to_string()]);
        rg.add_regulation_string("a -> b").unwrap();
        rg.add_regulation_string("b -| a").unwrap();

        let mut bn = BooleanNetwork::new(rg);

        // unknown variable
        assert!(bn.add_update_function("c", "!a").is_err());
        bn.add_update_function("a", "p => b").unwrap();
        // duplicate function
        assert!(bn.add_update_function("a", "!a").is_err());
        // name clash
        assert!(bn.add_update_function("b", "a & a()").is_err());
        // cardinality clash
        assert!(bn.add_update_function("b", "p(a) => a").is_err());
        // missing regulation
        assert!(bn.add_update_function("b", "p(b) => a").is_err());
    }

}
