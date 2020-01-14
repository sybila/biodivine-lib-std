use crate::boolean_network::async_graph::BooleanAsyncGraph;
use crate::boolean_network::bdd_params::{BddParameterEncoder, BddParams};
use crate::boolean_network::UpdateFunction::*;
use crate::boolean_network::{BooleanNetwork, Effect, UpdateFunction, VariableId};
use crate::graph::StateId;
use crate::parameters::ParamSet;
use biodivine_lib_bdd::{Bdd, BddVariableSet};

impl BooleanAsyncGraph {
    pub fn new(network: BooleanNetwork) -> Result<BooleanAsyncGraph, String> {
        return if network.num_vars() > 32 {
            Err("Can't create state space graph. At most 32 variables supported.".to_string())
        } else {
            let encoder = BddParameterEncoder::new(&network);
            let fake_graph = BooleanAsyncGraph {
                unit_set: BddParams {
                    bdd: encoder.bdd_variable_set.mk_true(),
                },
                parameter_encoder: encoder,
                network: network.clone(),
            };
            let mut condition = fake_graph.unit_set.clone();
            for regulation in network.regulatory_graph.regulations() {
                if let Some(effect) = regulation.effect {
                    let monotonicity =
                        fake_graph.ensure_monotonous(regulation.source, regulation.target, effect);
                    condition = condition.intersect(&monotonicity);
                }
                if regulation.observable {
                    let observability =
                        fake_graph.ensure_observable(regulation.source, regulation.target);
                    condition = condition.intersect(&observability);
                }
            }
            if condition.is_empty() {
                Err(
                    "There are no update functions satisfying given regulation constraints"
                        .to_string(),
                )
            } else {
                Ok(BooleanAsyncGraph {
                    parameter_encoder: BddParameterEncoder::new(&network),
                    network,
                    unit_set: condition,
                })
            }
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
    pub fn unit_params(&self) -> &BddParams {
        return &self.unit_set;
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
                    self.unit_params().clone()
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

    fn ensure_observable(&self, regulator: VariableId, target: VariableId) -> BddParams {
        let all_regulators = self.network.regulatory_graph.get_regulators(target);
        // index of the regulator in the context of other regulators.
        let regulator_index = all_regulators.iter().position(|v| *v == regulator).unwrap();
        let regulator_mask = 1 << regulator_index;
        // Number of entries in the function table of this target.
        let function_table_size = 1 << all_regulators.len();
        // Indices of table entries that have the regulator set to zero (inactive)
        let inactive_table_indices = (0..function_table_size).filter(|i| i & regulator_mask == 0);

        let mut condition = self.bdd_variable_set().mk_false();
        if let Some(update_function) = &self.network.update_functions[target.0] {
            for inactive_index in inactive_table_indices {
                let inactive_state =
                    Self::pack_table_index_into_state_id(inactive_index, &all_regulators);
                let active_state = inactive_state.flip_bit(regulator);

                let inactive_true = self
                    .eval_update_function(inactive_state, update_function)
                    .bdd;
                let active_true = self.eval_update_function(active_state, update_function).bdd;

                condition = condition.or(&active_true.iff(&inactive_true).not());
            }
        } else {
            for inactive_index in inactive_table_indices {
                let inactive_state =
                    Self::pack_table_index_into_state_id(inactive_index, &all_regulators);
                let active_state = inactive_state.flip_bit(regulator);

                let inactive_var = self
                    .parameter_encoder
                    .evaluate_anonymous_parameter(inactive_state, target);
                let active_var = self
                    .parameter_encoder
                    .evaluate_anonymous_parameter(active_state, target);

                let inactive_true = self.parameter_encoder.bdd_variable_set.mk_var(inactive_var);
                let active_true = self.parameter_encoder.bdd_variable_set.mk_var(active_var);

                condition = condition.or(&active_true.iff(&inactive_true).not());
            }
        }

        return BddParams { bdd: condition };
    }

    fn ensure_monotonous(
        &self,
        regulator: VariableId,
        target: VariableId,
        effect: Effect,
    ) -> BddParams {
        let all_regulators = self.network.regulatory_graph.get_regulators(target);
        // index of the regulator in the context of other regulators.
        let regulator_index = all_regulators.iter().position(|v| *v == regulator).unwrap();
        let regulator_mask = 1 << regulator_index;
        // Number of entries in the function table of this target.
        let function_table_size = 1 << all_regulators.len();
        // Indices of table entries that have the regulator set to zero (inactive)
        let inactive_table_indices = (0..function_table_size).filter(|i| i & regulator_mask == 0);

        let mut condition = self.bdd_variable_set().mk_true();
        if let Some(update_function) = &self.network.update_functions[target.0] {
            for inactive_index in inactive_table_indices {
                let inactive_state =
                    Self::pack_table_index_into_state_id(inactive_index, &all_regulators);
                let active_state = inactive_state.flip_bit(regulator);

                let inactive_true = self
                    .eval_update_function(inactive_state, update_function)
                    .bdd;
                let active_true = self.eval_update_function(active_state, update_function).bdd;

                Self::ensure_monotonicity(&mut condition, &active_true, &inactive_true, effect);
            }
        } else {
            for inactive_index in inactive_table_indices {
                let inactive_state =
                    Self::pack_table_index_into_state_id(inactive_index, &all_regulators);
                let active_state = inactive_state.flip_bit(regulator);

                let inactive_var = self
                    .parameter_encoder
                    .evaluate_anonymous_parameter(inactive_state, target);
                let active_var = self
                    .parameter_encoder
                    .evaluate_anonymous_parameter(active_state, target);

                let inactive_true = self.parameter_encoder.bdd_variable_set.mk_var(inactive_var);
                let active_true = self.parameter_encoder.bdd_variable_set.mk_var(active_var);

                Self::ensure_monotonicity(&mut condition, &active_true, &inactive_true, effect);
            }
        }

        return BddParams { bdd: condition };
    }

    /// Helper method which ensures that the given `condition` describes the desired
    /// monotonicity `effect` given two parameters sets, one that is `true` for the active
    /// and one for the inactive states.
    fn ensure_monotonicity(condition: &mut Bdd, active: &Bdd, inactive: &Bdd, effect: Effect) {
        let monotonicity = if effect == Effect::ACTIVATION {
            // increasing: [f(0) = 1] => [f(1) = 1]
            inactive.imp(&active)
        } else {
            // decreasing: [f(0) = 0] => [f(1) = 0] which is equivalent to [f(0) = 1] => [f(1) = 1]
            active.imp(&inactive)
        };
        *condition = condition.and(&monotonicity);
    }

    /// Take an index into the function table and the variables that are inputs of the function
    /// and transform this index into a state id that has the values of regulators set according
    /// to the table_index and rest of the variables are set to zero.
    fn pack_table_index_into_state_id(table_index: usize, regulators: &Vec<VariableId>) -> StateId {
        let mut state: usize = 0;
        for i in 0..regulators.len() {
            let regulator = regulators[i];
            if (table_index >> i) & 1 == 1 {
                // if we have one in the table index
                // then we also put one in teh state
                state |= 1 << regulator.0;
            }
        }
        return StateId(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::async_graph::BooleanAsyncGraph;
    use crate::boolean_network::BooleanNetwork;
    use std::convert::TryFrom;

    #[test]
    fn test_graph_unit_set_anonymous_params() {
        let network = BooleanNetwork::try_from(
            "
            a ->? b
            a -> a
            b -| b
            b -|? a
        ",
        )
        .unwrap();
        let graph = BooleanAsyncGraph::new(network).unwrap();
        // both functions can have 3 different valuations, so 9 in total
        assert_eq!(9.0, graph.unit_set.cardinality());
    }

    #[test]
    fn test_graph_unit_set_names_params() {
        let network = BooleanNetwork::try_from(
            "
            a ->? b
            a -> a
            b -| b
            b -|? a
            $a: a | p(b)
            $b: q(a, b) & a
        ",
        )
        .unwrap();
        let graph = BooleanAsyncGraph::new(network).unwrap();
        // p can have 2 valuations, q can have 4, 8 in total
        // actually, for b, there is only one possible function but it is achieved
        // regardless of two values of q.
        assert_eq!(8.0, graph.unit_set.cardinality());
    }
}
