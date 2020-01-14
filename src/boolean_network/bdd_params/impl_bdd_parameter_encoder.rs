use crate::boolean_network::bdd_params::BddParameterEncoder;
use crate::boolean_network::{BooleanNetwork, ParameterId, VariableId};
use crate::graph::StateId;
use biodivine_lib_bdd::{BddValuationIterator, BddVariable, BddVariableSetBuilder};

impl BddParameterEncoder {
    pub fn new(network: &BooleanNetwork) -> BddParameterEncoder {
        let mut builder = BddVariableSetBuilder::new();
        let mut parameter_bdd_variables: Vec<Vec<BddVariable>> = Vec::new();
        let mut anonymous_bdd_variables: Vec<Vec<BddVariable>> = Vec::new();

        // Create bdd variables for named parameters
        for i_p in 0..network.parameters.len() {
            let p = &network.parameters[i_p];

            // Create a Bdd variable for each possible valuation of the parameter function
            let bdd_variables_for_p: Vec<BddVariable> =
                BddValuationIterator::new(p.cardinality as u16)
                    .map(|valuation| {
                        let bdd_name = format!("{}{}", p.name, valuation);
                        builder.make_variable(&bdd_name)
                    })
                    .collect();

            parameter_bdd_variables.push(bdd_variables_for_p);
        }

        // Create bdd variables for anonymous parameters
        let mut regulators = Vec::with_capacity(network.num_vars());
        for variable in network.variable_ids() {
            regulators.push(network.regulatory_graph.get_regulators(variable));
            if network.update_functions[variable.0].is_none() {
                // Create a Bdd variable for each possible valuation of the parameter function
                let cardinality = network.regulatory_graph.num_regulators(variable);
                let bdd_variables_for_p: Vec<BddVariable> =
                    BddValuationIterator::new(cardinality as u16)
                        .map(|valuation| {
                            let bdd_name = format!(
                                "\\{{{}}}{}",
                                network.get_variable(variable).name,
                                valuation
                            );
                            builder.make_variable(&bdd_name)
                        })
                        .collect();

                anonymous_bdd_variables.push(bdd_variables_for_p);
            } else {
                // add empty vector if there are no anonymous parameters
                anonymous_bdd_variables.push(Vec::new());
            }
        }

        return BddParameterEncoder {
            parameter_bdd_variables,
            anonymous_bdd_variables,
            regulators,
            bdd_variable_set: builder.build(),
        };
    }

    /// Inspect given `parameter` function for specified variable `args` and decide
    /// which `BddVariable` governs the behaviour of the function in a specific `state`.
    pub fn evaluate_parameter(
        &self,
        state: StateId,
        parameter: ParameterId,
        args: &Vec<VariableId>,
    ) -> BddVariable {
        let table_index = Self::compute_table_index(state, args);
        return self.parameter_bdd_variables[parameter.0][table_index];
    }

    /// Inspect the anonymous parameter for the given `variable` and determine which
    /// `BddVariable` governs the behaviour of the parameter function in a specific `state`.
    pub fn evaluate_anonymous_parameter(
        &self,
        state: StateId,
        variable: VariableId,
    ) -> BddVariable {
        let table_index = Self::compute_table_index(state, &self.regulators[variable.0]);
        return self.anonymous_bdd_variables[variable.0][table_index];
    }

    // Compute which function table entry does the arguments correspond to in the given `state`.
    fn compute_table_index(state: StateId, args: &Vec<VariableId>) -> usize {
        let mut table_index: usize = 0;
        for i in 0..args.len() {
            if state.is_set(args[i]) {
                table_index += 1;
            }
            if i < args.len() - 1 {
                table_index = table_index << 1;
            }
        }
        return table_index;
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::bdd_params::BddParameterEncoder;
    use crate::boolean_network::{BooleanNetwork, ParameterId, VariableId};
    use crate::graph::StateId;
    use std::convert::TryFrom;

    #[test]
    fn explicit_parameter_encoder_test() {
        let network = BooleanNetwork::try_from(
            "
            a -> b
            a -| a
            b -> a
            $a: p(a,b) => q(b)
            $b: q(a)
        ",
        )
        .unwrap();
        let encoder = BddParameterEncoder::new(&network);

        let variables = encoder.bdd_variable_set.variables();
        assert_eq!(6, variables.len());

        let p = network.get_parameter_id("p").unwrap();
        let q = network.get_parameter_id("q").unwrap();

        // The order of parameters is not fixed, so we can't explicitly test exact BddVariables.
        // We can however test that all should appear.

        let mut actual_vars = Vec::new();
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b00),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b01),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b10),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b11),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(StateId(0b00), q, &vec![VariableId(0)]));
        actual_vars.push(encoder.evaluate_parameter(StateId(0b01), q, &vec![VariableId(0)]));
        actual_vars.sort();

        assert_eq!(variables, actual_vars);

        // Also, some basic identities should hold:

        let a = encoder.evaluate_parameter(StateId(0b00), p, &vec![VariableId(1), VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b00), p, &vec![VariableId(0), VariableId(1)]);
        assert_eq!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b10), p, &vec![VariableId(1), VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b01), p, &vec![VariableId(0), VariableId(1)]);
        assert_eq!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b01), p, &vec![VariableId(1), VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b10), p, &vec![VariableId(0), VariableId(1)]);
        assert_eq!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b11), p, &vec![VariableId(1), VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b11), p, &vec![VariableId(0), VariableId(1)]);
        assert_eq!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b01), p, &vec![VariableId(1), VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b01), p, &vec![VariableId(0), VariableId(1)]);
        assert_ne!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b00), q, &vec![VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b10), q, &vec![VariableId(0)]);
        assert_eq!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b10), q, &vec![VariableId(1)]);
        let b = encoder.evaluate_parameter(StateId(0b11), q, &vec![VariableId(1)]);
        assert_eq!(a, b);

        let a = encoder.evaluate_parameter(StateId(0b00), q, &vec![VariableId(0)]);
        let b = encoder.evaluate_parameter(StateId(0b01), q, &vec![VariableId(0)]);
        assert_ne!(a, b);
    }

    #[test]
    fn anonymous_parameter_encoder_test() {
        let network = BooleanNetwork::try_from(
            "
            a -> b
            a -| a
            b -> a
        ",
        )
        .unwrap();
        let encoder = BddParameterEncoder::new(&network);

        let variables = encoder.bdd_variable_set.variables();
        assert_eq!(6, variables.len());

        let a = network.get_variable_id("a").unwrap();
        let b = network.get_variable_id("b").unwrap();

        let mut actual_vars = Vec::new();
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b00), a));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b01), a));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b10), a));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b11), a));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b00), b));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b01), b));
        actual_vars.sort();

        assert_eq!(variables, actual_vars);

        let v1 = encoder.evaluate_anonymous_parameter(StateId(0b10), b);
        let v2 = encoder.evaluate_anonymous_parameter(StateId(0b00), b);
        assert_eq!(v1, v2);

        let v1 = encoder.evaluate_anonymous_parameter(StateId(0b11), b);
        let v2 = encoder.evaluate_anonymous_parameter(StateId(0b01), b);
        assert_eq!(v1, v2);

        let v1 = encoder.evaluate_anonymous_parameter(StateId(0b01), b);
        let v2 = encoder.evaluate_anonymous_parameter(StateId(0b00), b);
        assert_ne!(v1, v2);
    }

    #[test]
    fn mixed_parameter_encoder_test() {
        let network = BooleanNetwork::try_from(
            "
            a -> b
            a -| a
            b -> a
            $a: b & p(a, b)
        ",
        )
        .unwrap();
        let encoder = BddParameterEncoder::new(&network);

        let variables = encoder.bdd_variable_set.variables();
        assert_eq!(6, variables.len());

        let p = network.get_parameter_id("p").unwrap();
        let b = network.get_variable_id("b").unwrap();

        let mut actual_vars = Vec::new();
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b00),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b01),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b10),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_parameter(
            StateId(0b11),
            p,
            &vec![VariableId(0), VariableId(1)],
        ));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b01), b));
        actual_vars.push(encoder.evaluate_anonymous_parameter(StateId(0b00), b));
        actual_vars.sort();

        assert_eq!(variables, actual_vars);
    }
}
