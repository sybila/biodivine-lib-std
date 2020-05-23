//! A struct that implements `Set` is assumed to hold a collection of values, however the
//! collection itself does not have to be explicit. Items of a `Set` can be even uncountable.
//!
//! Because Rust currently does not have a `Set` trait (and even if it had, its use case would
//! probably differ from ours), we introduce our own `Set` trait. As an example implementation,
//! see `ExplicitSet` which simply delegates to rust `HashSet`.
//!
//! Basic set-like operations are provided:
//!
//! ```rust
//! use biodivine_lib_std::collections::sets::{ExplicitSet, Set};
//! let a = ExplicitSet::new_with_items(vec!["a", "b", "hello"]);
//! let b = ExplicitSet::new_with_items(vec!["hello", "my", "darling"]);
//! let i = ExplicitSet::new_with_items(vec!["hello"]);
//! assert!(!a.is_empty());
//! assert!(!a.is_subset(&b));
//! assert!(!a.is_subset(&i));
//! assert!(a.is_subset(&a.union(&b)));
//! assert_eq!(i, a.intersect(&b));
//! ```
//!
//! ### Set elements and iteration
//!
//! Our sets don't have to be countable (or contain instantiable elements
//! for that matter). However, some sets are (and some algorithms require this). We therefore
//! also provide `ElementSet` trait which defines what type of elements appear in the sets and
//! allows *testing for their presence* or *picking a single representing element*:
//!
//! ```rust
//! use biodivine_lib_std::collections::sets::{ExplicitSet, ElementSet, Set};
//! let a = ExplicitSet::new_with_items(vec!["a", "b"]);
//! assert!(a.contains(&"b"));
//! assert!(a.contains(&a.pick().unwrap()));
//! let x = a.pick().unwrap();
//! assert!(x == "a" || x == "b");
//! assert!(ExplicitSet::<i32>::empty().pick().is_none());
//! ```
//!
//! Furthermore, an `ElementSet` can implement `IterableSet` where one can also iterate over
//! all elements in the sets:
//!
//! ```rust
//! use biodivine_lib_std::collections::sets::{ExplicitSet, IterableSet};
//! let a = ExplicitSet::new_with_items(vec!["a", "b"]);
//! for x in a.iter() {
//!     assert!(x == "a" || x == "b");
//! }
//! assert_eq!(2, a.iter().count());
//! ```

use std::collections::HashSet;
use std::hash::Hash;

mod _impl_explicit_set;
mod _impl_set_for_option_set;

/// `Set` is a collection of elements. The elements do not have to be instantiable and the
/// collections.sets can be infinite or even uncountable. However, we generally assume that collections.sets can be
/// cloned, tested for inclusion/equality and tested for emptiness.
///
/// In general, collections.sets are not `Copy` and therefore we pass them by reference where appropriate.
pub trait Set: Clone + Eq {
    /// Construct an empty collections.sets of this type.
    ///
    /// *Note:* For collections.sets that do not have a universe-independent empty-collections.sets representation,
    /// we recommend representing the actual collections.sets as `Option<SetType>`. A blanket `Option<SetType>`
    /// implementation is provided for all `Set` implementations. This blanket implementation
    /// does not use the empty constructor (returning `None`) — the original implementation
    /// can therefore panic.
    fn empty() -> Self;

    /// Compute the union collections.sets $A \cup B = \\{ x \mid x \in A \lor x \in B \\}$.
    fn union(&self, other: &Self) -> Self;

    /// Compute the intersection collections.sets $A \cap B = \\{ x \mid x \in A \land x \in B \\}$.
    fn intersect(&self, other: &Self) -> Self;

    /// Compute the difference collections.sets $A \setminus B = \\{ x \mid x \in A \land \neg (x \in B) \\}$.
    fn minus(&self, other: &Self) -> Self;

    /// True if this collections.sets is an empty collections.sets.
    fn is_empty(&self) -> bool;

    /// True if this collections.sets is a subset of the given collections.sets ($A \subseteq B$).
    fn is_subset(&self, other: &Self) -> bool;
}

/// `ElementSet` is a `Set` that contains instantiable elements. It can still be
/// infinite or uncountable, but has to contain elements which are representable in rust.
///
/// Because of these restrictions, `ElementSet` does not allow modifying the collections.sets using
/// individual elements (i.e. `add`, `remove`, etc.) - only for testing the presence
/// of elements and for picking *some* representative element of the collections.sets.
pub trait ElementSet: Set {
    /// A type of elements stored in this collections.sets.
    type Element: Clone + Eq;

    /// Returns true if the given element is present in the collections.sets: $e \in A$.
    fn contains(&self, e: &Self::Element) -> bool;

    /// Return *some* element from the collections.sets. Note that the choice does not have to be
    /// deterministic (for example, it may depend on internal state of the collections.sets).
    ///
    /// Also, we assume that typically the elements are not stored explicitly and
    /// have to be created specifically for the pick operation, we therefore immediately
    /// return an owned value, not a reference.
    fn pick(&self) -> Option<Self::Element>;
}

/// If the elements of a `Set` are countable and can be iterated, one can implement
/// an `IterableSet` which allows to explore individual elements of the collections.sets.
pub trait IterableSet: ElementSet {
    type ElementIterator: Iterator<Item = Self::Element>;

    /// Returns an iterator over the elements of the collections.sets. Note that the iterator is over
    /// owned elements and not references. This is slightly less efficient but usually
    /// not drastically since the collections.sets will typically not store all elements explicitly
    /// anyway, meaning they will be created during the iteration anyway.
    fn iter(&self) -> Self::ElementIterator;
}

/// A basic example implementation of a `Set`, based on the standard rust `HashSet`. For usage
/// examples, see module description.
#[derive(Clone, Debug)]
pub struct ExplicitSet<T: Hash + Clone + Eq>(HashSet<T>);
