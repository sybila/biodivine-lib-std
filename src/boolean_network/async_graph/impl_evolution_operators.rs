use crate::boolean_network::async_graph::{
    BwdBooleanAsyncGraph, BwdBooleanEdgeIterator, FwdBooleanAsyncGraph, FwdBooleanEdgeIterator,
};
use crate::boolean_network::bdd_params::BddParams;
use crate::graph::{EvolutionOperator, StateId};

impl<'a> EvolutionOperator<BddParams> for FwdBooleanAsyncGraph<'a> {
    type EdgeIterator = FwdBooleanEdgeIterator<'a>;

    fn step(&self, source: StateId) -> Self::EdgeIterator {
        return FwdBooleanEdgeIterator {
            graph: self.graph,
            variables: self.graph.network.variable_ids(),
            source,
        };
    }
}

impl Iterator for FwdBooleanEdgeIterator<'_> {
    type Item = (StateId, BddParams);

    fn next(&mut self) -> Option<Self::Item> {
        return if let Some(var) = self.variables.next() {
            let target = self.source.flip_bit(var);
            let edge_params = self.graph.edge_params(self.source, var);
            Some((target, edge_params))
        } else {
            None
        };
    }
}

impl<'a> EvolutionOperator<BddParams> for BwdBooleanAsyncGraph<'a> {
    type EdgeIterator = BwdBooleanEdgeIterator<'a>;

    fn step(&self, source: StateId) -> Self::EdgeIterator {
        return BwdBooleanEdgeIterator {
            graph: self.graph,
            variables: self.graph.network.variable_ids(),
            target: source,
        };
    }
}

impl Iterator for BwdBooleanEdgeIterator<'_> {
    type Item = (StateId, BddParams);

    fn next(&mut self) -> Option<Self::Item> {
        return if let Some(var) = self.variables.next() {
            let source = self.target.flip_bit(var);
            let edge_params = self.graph.edge_params(source, var);
            Some((source, edge_params))
        } else {
            None
        };
    }
}
