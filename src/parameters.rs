pub trait ParamSet: Clone {
    fn union(&self, other: &Self) -> Self;
    fn intersect(&self, other: &Self) -> Self;
    fn minus(&self, other: &Self) -> Self;

    fn is_subset_of(&self, other: &Self) -> bool;
    fn is_empty(&self) -> bool;
}
