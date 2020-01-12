use crate::boolean_network::builder::{
    BooleanNetworkBuilder, RegulationTemplate, RegulatoryGraph, UpdateFunctionTemplate,
};
use crate::boolean_network::{BooleanNetwork, ParameterId, VariableId};
use std::collections::HashMap;

impl BooleanNetworkBuilder {
    pub fn new_from_regulatory_graph(graph: RegulatoryGraph) -> BooleanNetworkBuilder {
        let num_variables = graph.variables.len();
        return BooleanNetworkBuilder {
            regulatory_graph: graph,
            parameters: Vec::new(),
            parameter_to_index: HashMap::new(),
            update_functions: vec![None; num_variables],
        };
    }

    pub fn add_update_function(
        &mut self,
        variable: &String,
        update_function: UpdateFunctionTemplate,
    ) -> Result<(), String> {
        let update_function = update_function.swap_unary_parameters(&self.regulatory_graph);

        let parameters = update_function.extract_parameters();

        // add new parameters and check for mismatch in parameter cardinality
        for p_in_f in &parameters {
            let mut found = false;
            for p_in_net in &self.parameters {
                if p_in_f.name == p_in_net.name {
                    found = true;
                    if p_in_f.cardinality != p_in_net.cardinality {
                        return Err(format!(
                            "Parameter {} occurs with cardinality {} and {}",
                            p_in_f.name, p_in_f.cardinality, p_in_net.cardinality
                        ));
                    }
                }
            }
            if self.regulatory_graph.has_variable(&p_in_f.name) {
                return Err(format!(
                    "{} can't be both a variable and a parameter",
                    p_in_f.name
                ));
            }
            if !found {
                self.parameter_to_index
                    .insert(p_in_f.name.clone(), ParameterId(self.parameters.len()));
                self.parameters.push(p_in_f.clone());
            }
        }

        let update_function = *update_function.build(
            &self.regulatory_graph.variable_to_index,
            &self.parameter_to_index,
        )?;

        let variable_index = *self
            .regulatory_graph
            .variable_to_index
            .get(variable)
            .ok_or(format!("(1) Unknown variable {}", variable))?;

        // check if update function only contains allowed regulations
        for regulator in update_function.variables() {
            if !self
                .regulatory_graph
                .has_regulation(regulator, variable_index)
            {
                return Err(format!(
                    "{} depends on {} but the regulation is not specified",
                    self.regulatory_graph.get_variable(variable_index),
                    self.regulatory_graph.get_variable(regulator)
                ));
            }
        }

        self.update_functions[variable_index.0] = Some(update_function);

        return Ok(());
    }

    pub fn build(self) -> BooleanNetwork {
        return BooleanNetwork {
            variables: self.regulatory_graph.variables,
            regulations: self.regulatory_graph.regulations,
            parameters: self.parameters,
            update_functions: self.update_functions,
        };
    }
}
