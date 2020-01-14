use crate::boolean_network::async_graph::{
    BwdBooleanAsyncGraph, BwdBooleanEdgeIterator, FwdBooleanAsyncGraph, FwdBooleanEdgeIterator,
};
use crate::boolean_network::bdd_params::BddParams;
use crate::graph::{EvolutionOperator, StateId};
use crate::parameters::ParamSet;

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
            Some((target, edge_params.intersect(&self.graph.unit_set)))
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
            Some((source, edge_params.intersect(&self.graph.unit_set)))
        } else {
            None
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::async_graph::BooleanAsyncGraph;
    use crate::boolean_network::bdd_params::BddParams;
    use crate::boolean_network::BooleanNetwork;
    use crate::graph::StateId;
    use crate::graph::{EvolutionOperator, Graph};
    use crate::parameters::ParamSet;
    use std::collections::HashSet;
    use std::convert::TryFrom;

    #[test]
    fn test_no_param_network() {
        let network = BooleanNetwork::try_from(
            "
            a -> b
            a -> a
            b -| a
            b -| b
            $a: a & !b
            $b: a | !b
        ",
        )
        .unwrap();
        let graph = &BooleanAsyncGraph::new(network).unwrap();

        let edges: HashSet<(StateId, StateId)> = vec![
            (StateId(0b00), StateId(0b10)),
            (StateId(0b10), StateId(0b00)),
            (StateId(0b00), StateId(0b10)),
            (StateId(0b01), StateId(0b11)),
            (StateId(0b11), StateId(0b10)),
        ]
        .into_iter()
        .collect();

        let fwd = graph.forward_evolution();
        let bwd = graph.backward_evolution();

        for s in graph.states() {
            for (t, p) in fwd.step(s) {
                assert_eq!(
                    !p.is_empty(),
                    edges.contains(&(s, t)),
                    "Edge ({:?},{:?})",
                    s,
                    t
                );
            }
            for (t, p) in bwd.step(s) {
                assert_eq!(
                    !p.is_empty(),
                    edges.contains(&(t, s)),
                    "Edge ({:?},{:?})",
                    t,
                    s
                );
            }
        }
    }

    #[test]
    fn test_anonymous_param_network() {
        let network = BooleanNetwork::try_from(
            "
            a ->? b
            a -> a
            b -|? a
            b -| b
        ",
        )
        .unwrap();
        let graph = &BooleanAsyncGraph::new(network).unwrap();
        let fwd = graph.forward_evolution();
        let bwd = graph.backward_evolution();

        let edges: HashSet<(StateId, StateId, i32)> = vec![
            (StateId(0b00), StateId(0b10), 2 * 3),
            (StateId(0b10), StateId(0b00), 3 * 3),
            (StateId(0b00), StateId(0b01), 1 * 3),
            (StateId(0b11), StateId(0b10), 1 * 3),
            (StateId(0b01), StateId(0b11), 3 * 3),
            (StateId(0b11), StateId(0b01), 2 * 3),
        ]
        .into_iter()
        .collect();

        assert_eq!(9.0, graph.unit_set.cardinality());

        let mut fwd_edges: HashSet<(StateId, StateId, BddParams)> = HashSet::new();
        let mut bwd_edges: HashSet<(StateId, StateId, BddParams)> = HashSet::new();

        for s in graph.states() {
            let successors = fwd.step(s);
            for (t, p) in successors {
                if p.cardinality() > 0.0 {
                    assert!(edges.contains(&(s, t, p.cardinality() as i32)));
                }
                fwd_edges.insert((s, t, p));
            }
            for (t, p) in bwd.step(s) {
                if p.cardinality() > 0.0 {
                    assert!(edges.contains(&(t, s, p.cardinality() as i32)));
                }
                bwd_edges.insert((t, s, p));
            }
        }

        assert_eq!(fwd_edges, bwd_edges)
    }
}
