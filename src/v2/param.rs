use super::State;

trait Params: Clone {}

trait EvolutionOperator {
    type State: State;
    type Params: Params;
    type Iterator: Iterator<Item = (Self::State, Self::Params)>;
    fn step(&self, source: Self::State) -> Self::Iterator;
}

trait Graph {
    type State: State;
    type Params: Params;
    type States: Iterator<Item = Self::State>;
    type FwdEdges: EvolutionOperator;
    type BwdEdges: EvolutionOperator;

    fn states(&self) -> Self::States;
    fn fwd(&self) -> Self::FwdEdges;
    fn bwd(&self) -> Self::BwdEdges;
}

trait StateSet {
    type State: State;
    type Params: Params;
    type Iterator: Iterator<Item = (Self::State, Self::Params)>;

    fn iter(&self) -> Self::Iterator;
    fn contains(&self, state: &Self::State) -> &Self::Params;
}
