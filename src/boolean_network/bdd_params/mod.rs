use crate::boolean_network::{BooleanNetwork, ParameterId, VariableId};
use crate::graph::StateId;
use crate::parameters::ParamSet;
use biodivine_lib_bdd::{
    Bdd, BddValuationIterator, BddVariable, BddVariableSet, BddVariableSetBuilder,
};

#[derive(Clone, Debug, Hash)]
pub struct BddParams {
    pub(super) bdd: Bdd,
}

pub struct BddParameterEncoder {
    // Number of regulators for each variable - used for anonymous parameters.
    regulators: Vec<Vec<VariableId>>,
    // For each parameter function, give a table of BDDVariables which correspond to values of the function for different inputs.
    parameter_bdd_variables: Vec<Vec<BddVariable>>,
    // The same as `parameter_bdd_variables`, but for anonymous (unspecified) parameters, i.e. missing update functions.
    anonymous_bdd_variables: Vec<Vec<BddVariable>>,
    // A BDDVariable set that can be used for other things...
    pub(super) bdd_variable_set: BddVariableSet,
}

impl ParamSet for BddParams {
    fn union(&self, other: &Self) -> Self {
        return BddParams {
            bdd: self.bdd.or(&other.bdd),
        };
    }

    fn intersect(&self, other: &Self) -> Self {
        return BddParams {
            bdd: self.bdd.and(&other.bdd),
        };
    }

    fn minus(&self, other: &Self) -> Self {
        return BddParams {
            bdd: self.bdd.and_not(&other.bdd),
        };
    }

    fn is_subset_of(&self, other: &Self) -> bool {
        // TODO: Introduce special function for this in bdd-lib to avoid allocation
        return self.minus(other).is_empty();
    }

    fn is_empty(&self) -> bool {
        return !self.bdd.is_false();
    }
}

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
                            let bdd_name =
                                format!("\\({}){}", network.get_variable(variable).name, valuation);
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
