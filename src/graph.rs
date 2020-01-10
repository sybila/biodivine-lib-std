use crate::parameters::ParamSet;

pub trait EvolutionOperator<P>
where
    P: ParamSet,
{
    type EdgeIterator: Iterator<Item = (usize, P)>;

    fn step(&self, source: usize) -> Self::EdgeIterator;
}
