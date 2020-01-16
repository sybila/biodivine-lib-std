use crate::boolean_network::bdd_params::{BddParameterEncoder, BddParams};
use crate::boolean_network::builder::VariableIdIterator;
use crate::boolean_network::BooleanNetwork;
use crate::graph::{Graph, StateId};
use crate::RangeStateIterator;

mod impl_boolean_async_graph;
mod impl_evolution_operators;

pub struct BooleanAsyncGraph {
    network: BooleanNetwork,
    parameter_encoder: BddParasmeterEncoder,
    unit_set: BddParams,
}

pub struct FwdBooleanAsyncGraph<'a> {
    graph: &'a BooleanAsyncGraph,
}

pub struct FwdBooleanEdgeIterator<'a> {
    graph: &'a BooleanAsyncGraph,
    source: StateId,
    variables: VariableIdIterator,
}

pub struct BwdBooleanAsyncGraph<'a> {
    graph: &'a BooleanAsyncGraph,
}

pub struct BwdBooleanEdgeIterator<'a> {
    graph: &'a BooleanAsyncGraph,
    target: StateId,
    variables: VariableIdIterator,
}

impl<'a> Graph<BddParams> for &'a BooleanAsyncGraph {
    type ForwardEvolution = FwdBooleanAsyncGraph<'a>;
    type BackwardEvolution = BwdBooleanAsyncGraph<'a>;
    type StatesIterator = RangeStateIterator;

    fn states(&self) -> Self::StatesIterator {
        return RangeStateIterator::new(self.num_states());
    }

    fn forward_evolution(&self) -> Self::ForwardEvolution {
        return FwdBooleanAsyncGraph { graph: self };
    }

    fn backward_evolution(&self) -> Self::BackwardEvolution {
        return BwdBooleanAsyncGraph { graph: self };
    }
}
