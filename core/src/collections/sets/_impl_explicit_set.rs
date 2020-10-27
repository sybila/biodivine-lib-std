use super::{ElementSet, ExplicitSet, IterableSet, Set};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::IntoIter;
use std::collections::HashSet;
use std::hash::Hash;

impl<T: Clone + Hash + Eq> ExplicitSet<T> {
    /// Create a new `ExplicitSet` with a vector of items (preferred to `From<HashSet>`
    /// when using macro to create items).
    pub fn new_with_items(items: Vec<T>) -> ExplicitSet<T> {
        // This is mostly because vectors have a macro but collections.sets do not...
        return ExplicitSet(items.into_iter().collect());
    }
}

impl<T: Clone + Hash + Eq> From<HashSet<T>> for ExplicitSet<T> {
    fn from(set: HashSet<T, RandomState>) -> Self {
        return ExplicitSet(set);
    }
}

impl<T: Clone + Hash + Eq> Into<HashSet<T>> for ExplicitSet<T> {
    fn into(self) -> HashSet<T, RandomState> {
        return self.0;
    }
}

impl<T: Clone + Hash + Eq> Set for ExplicitSet<T> {
    fn empty() -> Self {
        return ExplicitSet(HashSet::new());
    }

    fn union(&self, other: &Self) -> Self {
        return ExplicitSet(self.0.union(&other.0).cloned().collect());
    }

    fn intersect(&self, other: &Self) -> Self {
        return ExplicitSet(self.0.intersection(&other.0).cloned().collect());
    }

    fn minus(&self, other: &Self) -> Self {
        return ExplicitSet(self.0.difference(&other.0).cloned().collect());
    }

    fn is_empty(&self) -> bool {
        return self.0.is_empty();
    }

    fn is_subset(&self, other: &Self) -> bool {
        return self.0.difference(&other.0).next().is_none();
    }
}

impl<T: Clone + Hash + Eq> ElementSet for ExplicitSet<T> {
    type Element = T;

    fn contains(&self, e: &Self::Element) -> bool {
        return self.0.contains(e);
    }

    fn pick(&self) -> Option<Self::Element> {
        return self.0.iter().next().map(|i| i.clone());
    }
}

impl<'a, T: Clone + Hash + Eq> IterableSet for ExplicitSet<T> {
    type ElementIterator = IntoIter<Self::Element>;

    fn iter(&self) -> Self::ElementIterator {
        // Not the most efficient, but it'll do for now.
        return self.0.clone().into_iter();
    }
}

impl<T: Clone + Hash + Eq> Eq for ExplicitSet<T> {}
impl<T: Clone + Hash + Eq> PartialEq for ExplicitSet<T> {
    fn eq(&self, other: &Self) -> bool {
        return self
            .0
            .union(&other.0)
            .all(|x| self.0.contains(x) && other.0.contains(x));
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ElementSet, ExplicitSet, IterableSet, Set};
    use std::collections::HashSet;

    #[test]
    pub fn simple_explicit_set_test() {
        let a = ExplicitSet::new_with_items(vec![1, 2, 3]);
        let b = ExplicitSet::new_with_items(vec![3, 4, 5]);
        assert_eq!(
            ExplicitSet::new_with_items(vec![1, 2, 3, 4, 5]),
            a.union(&b)
        );
        assert_eq!(ExplicitSet::new_with_items(vec![3]), a.intersect(&b));
        assert_eq!(ExplicitSet::new_with_items(vec![1, 2]), a.minus(&b));
        assert!(!a.is_empty());
        assert!(!b.is_empty());
        assert!(!a.is_subset(&b));
        assert!(!b.is_subset(&a));
        let union = a.union(&b);
        assert!(a.is_subset(&union));
        assert!(b.is_subset(&union));
        assert!(ExplicitSet::<i32>::empty().is_empty());
        assert!(ExplicitSet::<i32>::empty().is_subset(&a));
    }

    #[test]
    pub fn element_explicit_set_test() {
        let set = ExplicitSet::new_with_items(vec![1, 2, 3]);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(!set.contains(&0));
        assert!(!set.contains(&4));
        assert!(set.contains(&set.pick().unwrap()));
        assert_eq!(None, ExplicitSet::<i32>::empty().pick());
    }

    #[test]
    pub fn iterator_explicit_set_test() {
        let mut set = HashSet::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);
        let elements = ExplicitSet::from(set);
        let from_iterator = elements.iter().collect::<HashSet<i32>>();
        assert_eq!(elements, ExplicitSet::from(from_iterator.clone()));
        assert_eq!(
            None,
            from_iterator.symmetric_difference(&elements.into()).next()
        );
    }
}
