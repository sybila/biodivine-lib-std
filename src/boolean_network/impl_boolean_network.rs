use crate::boolean_network::{
    BooleanNetwork, Parameter, ParameterId, UpdateFunction, Variable, VariableId,
};
use std::iter::Map;
use std::ops::Range;

impl BooleanNetwork {
    pub fn get_update_function(&self, id: VariableId) -> &Option<UpdateFunction> {
        return &self.update_functions[id.0];
    }

    pub fn get_variable(&self, id: VariableId) -> &Variable {
        return self.regulatory_graph.get_variable(id);
    }

    pub fn get_parameter(&self, id: ParameterId) -> &Parameter {
        return &self.parameters[id.0];
    }

    pub fn get_parameter_id(&self, param: &str) -> Option<ParameterId> {
        return self.parameter_to_index.get(param).map(|p| *p);
    }

    pub fn variable_ids(&self) -> Map<Range<usize>, fn(usize) -> VariableId> {
        return self.regulatory_graph.variable_ids();
    }

    pub fn parameter_ids(&self) -> Map<Range<usize>, fn(usize) -> ParameterId> {
        return (0..self.parameters.len()).map(|i| ParameterId(i));
    }
}
