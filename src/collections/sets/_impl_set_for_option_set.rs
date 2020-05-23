use super::Set;

/// A blanket implementation of `Set` for all `Option<SetType>`. This is useful
/// in two cases:
///  - You cannot initialize the empty collections.sets statically. You can then implement `Set`,
///  panicking when empty collections.sets is requested and use `Option<SetType>` as your *actual* collections.sets
///  implementation.
///  - As part of your control flow, you use `Option` to denote invalid state which
///  are now implicitly considered as empty collections.sets using this implementation.
impl<T> Set for Option<T>
where
    T: Set,
{
    fn empty() -> Self {
        return None;
    }

    fn union(&self, other: &Self) -> Self {
        return match self {
            None => other.clone(),
            Some(a) => match other {
                None => Some(a.clone()),
                Some(b) => Some(a.union(b)),
            },
        };
    }

    fn intersect(&self, other: &Self) -> Self {
        return match self {
            None => None,
            Some(a) => match other {
                None => None,
                Some(b) => Some(a.intersect(b)),
            },
        };
    }

    fn minus(&self, other: &Self) -> Self {
        return match self {
            None => None,
            Some(a) => match other {
                None => Some(a.clone()),
                Some(b) => Some(a.minus(b)),
            },
        };
    }

    fn is_empty(&self) -> bool {
        return if let Some(a) = self {
            a.is_empty()
        } else {
            true
        };
    }

    fn is_subset(&self, other: &Self) -> bool {
        return match self {
            None => true, // empty is subset of anything
            Some(a) => match other {
                None => false, // non empty is never a subset of empty
                Some(b) => a.is_subset(b),
            },
        };
    }
}
