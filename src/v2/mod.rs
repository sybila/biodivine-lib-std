use std::hash::Hash;

trait State: Hash + Eq {}

trait EvolutionOperator {
    type State: State;
    type Iterator: Iterator<Item = Self::State>;
    fn step(&self, source: Self::State) -> Self::Iterator;
}

trait Graph<S: State> {
    type State: State;
    type States: Iterator<Item = Self::State>;
    type FwdEdges: EvolutionOperator;
    type BwdEdges: EvolutionOperator;

    fn states(&self) -> Self::States;
    fn fwd(&self) -> Self::FwdEdges;
    fn bwd(&self) -> Self::BwdEdges;
}

trait StateSet {
    type State: State;
    type Iterator: Iterator<Item = Self::State>;

    fn iter(&self) -> Self::Iterator;
    fn contains(&self, state: &Self::State) -> bool;
}
