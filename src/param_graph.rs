use crate::State;

/// `Params` represents a set of parameter valuations and thus typically behaves like a
/// normal set.
///
/// However, notice that there is no complement method available. This is because the
/// `unit` set of parameters can be different every time or completely unknown. To
/// implement complement, use `minus` with an appropriate `unit` set.
///
/// Also notice that we do not assume anything about the members of the set, we can't
/// iterate them or even retrieve them. This is because the sets might be uncountable
/// or the elements might not be well representable.
pub trait Params: Clone {
    fn union(&self, other: &Self) -> Self;
    fn intersect(&self, other: &Self) -> Self;
    fn minus(&self, other: &Self) -> Self;

    fn is_empty(&self) -> bool;
    fn is_subset(&self, other: &Self) -> bool;
}

/// A parametrised variant of an `EvolutionOperator`. For each state, a `Params` set is
/// returned as well which gives the set of parameter valuations for which the transition
/// is allowed.
pub trait EvolutionOperator {
    type State: State;
    type Params: Params;
    type Iterator: Iterator<Item = (Self::State, Self::Params)>;
    fn step(&self, current: Self::State) -> Self::Iterator;
}

/// A parametrised variant of a `Graph`.
pub trait Graph {
    type State: State;
    type Params: Params;
    type States: Iterator<Item = Self::State>;
    type FwdEdges: EvolutionOperator;
    type BwdEdges: EvolutionOperator;

    fn states(&self) -> Self::States;
    fn fwd(&self) -> Self::FwdEdges;
    fn bwd(&self) -> Self::BwdEdges;
}
