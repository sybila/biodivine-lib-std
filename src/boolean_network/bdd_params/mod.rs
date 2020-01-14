use crate::boolean_network::{BooleanNetwork, ParameterId, VariableId};
use crate::graph::StateId;
use crate::parameters::ParamSet;
use biodivine_lib_bdd::{
    Bdd, BddValuationIterator, BddVariable, BddVariableSet, BddVariableSetBuilder,
};

mod impl_bdd_parameter_encoder;

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
