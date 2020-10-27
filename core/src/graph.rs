use crate::State;

/// `EvolutionOperator`s represent an evolution of non-deterministic dynamical system with
/// discrete time, i.e. given a current state, they provide possible states in the next time step.
pub trait EvolutionOperator {
    type State: State;
    type Iterator: Iterator<Item = Self::State>;
    fn step(&self, current: Self::State) -> Self::Iterator;
}

/// `Graph` is a dynamical system with finite state space which can be explored forward
/// and backward in time.
pub trait Graph {
    type State: State;
    type States: Iterator<Item = Self::State>;
    type FwdEdges: EvolutionOperator;
    type BwdEdges: EvolutionOperator;

    fn states(&self) -> Self::States;
    fn fwd(&self) -> Self::FwdEdges;
    fn bwd(&self) -> Self::BwdEdges;
}
