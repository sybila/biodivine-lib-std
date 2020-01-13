use crate::boolean_network::async_graph::BooleanAsyncGraph;
use crate::boolean_network::bdd_params::{BddParameterEncoder, BddParams};
use crate::boolean_network::UpdateFunction::*;
use crate::boolean_network::{BooleanNetwork, UpdateFunction, VariableId};
use crate::graph::StateId;
use crate::parameters::ParamSet;
use biodivine_lib_bdd::BddVariableSet;

impl BooleanAsyncGraph {
    pub fn new(network: BooleanNetwork) -> Result<BooleanAsyncGraph, String> {
        return if network.num_vars() > 32 {
            Err("Can't create state space graph. At most 32 variables supported.".to_string())
        } else {
            Ok(BooleanAsyncGraph {
                parameter_encoder: BddParameterEncoder::new(&network),
                network,
            })
        };
    }

    pub fn num_states(&self) -> usize {
        return 1 << self.network.num_vars();
    }

    /// Compute the parameter set which enables the value of `variable` to be flipped
    /// in the given `state`.
    pub fn edge_params(&self, state: StateId, variable: VariableId) -> BddParams {
        // First, compute the parameter set that sends value of variable to true in this state
        let update_function = &self.network.update_functions[variable.0];
        let edge_params = if let Some(update_function) = update_function {
            self.eval_update_function(state, update_function)
        } else {
            let var = self
                .parameter_encoder
                .evaluate_anonymous_parameter(state, variable);
            BddParams {
                bdd: self.parameter_encoder.bdd_variable_set.mk_var(var),
            }
        };

        // Now if we actually want to go to false, invert the set:
        let edge_params = if state.is_set(variable) {
            self.unit_params().minus(&edge_params)
        } else {
            edge_params
        };

        return edge_params;
    }

    /// Return the parameter set that for which this graph is admissible
    pub fn unit_params(&self) -> BddParams {
        unimplemented!()
    }

    pub fn empty_params(&self) -> BddParams {
        return BddParams {
            bdd: self.bdd_variable_set().mk_false(),
        };
    }

    pub fn bdd_variable_set(&self) -> &BddVariableSet {
        return &self.parameter_encoder.bdd_variable_set;
    }

    /// Compute the parameter set for which the given update function evaluates to one
    /// in the given state. The function evaluates to false exactly in the opposite parameters.
    fn eval_update_function(&self, state: StateId, update_function: &UpdateFunction) -> BddParams {
        return match update_function {
            Variable { id } => {
                if state.is_set(*id) {
                    self.unit_params()
                } else {
                    self.empty_params()
                }
            }
            Parameter { id, inputs } => {
                let bdd_var = self
                    .parameter_encoder
                    .evaluate_parameter(state, *id, inputs);
                BddParams {
                    bdd: self.parameter_encoder.bdd_variable_set.mk_var(bdd_var),
                }
            }
            Not(inner) => {
                let inner = self.eval_update_function(state, inner);
                let all = self.unit_params();
                BddParams {
                    bdd: all.bdd.and_not(&inner.bdd),
                }
            }
            And(a, b) => {
                let bdd_a = self.eval_update_function(state, a);
                let bdd_b = self.eval_update_function(state, b);
                BddParams {
                    bdd: bdd_a.bdd.and(&bdd_b.bdd),
                }
            }
            Or(a, b) => {
                let bdd_a = self.eval_update_function(state, a);
                let bdd_b = self.eval_update_function(state, b);
                BddParams {
                    bdd: bdd_a.bdd.or(&bdd_b.bdd),
                }
            }
            Imp(a, b) => {
                let bdd_a = self.eval_update_function(state, a);
                let bdd_b = self.eval_update_function(state, b);
                BddParams {
                    bdd: bdd_a.bdd.imp(&bdd_b.bdd),
                }
            }
            Iff(a, b) => {
                let bdd_a = self.eval_update_function(state, a);
                let bdd_b = self.eval_update_function(state, b);
                BddParams {
                    bdd: bdd_a.bdd.iff(&bdd_b.bdd),
                }
            }
            Xor(a, b) => {
                let bdd_a = self.eval_update_function(state, a);
                let bdd_b = self.eval_update_function(state, b);
                BddParams {
                    bdd: bdd_a.bdd.xor(&bdd_b.bdd),
                }
            }
        };
    }
}
