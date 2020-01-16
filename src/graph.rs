use crate::parameters::ParamSet;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StateId(pub(super) usize);

pub trait EvolutionOperator<P: ParamSet> {
    type EdgeIterator: Iterator<Item = (StateId, P)>;

    fn step(&self, source: StateId) -> Self::EdgeIterator;
}

pub trait Graph<P: ParamSet> {
    type ForwardEvolution: EvolutionOperator<P>;
    type BackwardEvolution: EvolutionOperator<P>;
    type StatesIterator: Iterator<Item = StateId>;

    fn states(&self) -> Self::StatesIterator;
    fn forward_evolution(&self) -> Self::ForwardEvolution;
    fn backward_evolution(&self) -> Self::BackwardEvolution;
}

impl StateId {
    pub fn is_set(&self, var: usize) -> bool {
        return (self.0 >> var) & 1 == 1;
    }

    pub fn flip_bit(&self, var: usize) -> StateId {
        let mask = 1 << var;
        return StateId(self.0 ^ mask);
    }
}
