use std::hash::Hash;

pub mod graph;
pub mod param_graph;
pub mod util; // not stabilised yet

mod impl_id_state;
mod impl_id_state_range;

/// A marker trait for anything that can be a state of a graph.
///
/// Currently, we require each state to be a `Copy` struct, i.e. it has to be
/// "small enough" so that it can be copied whenever needed. In the future, we might
/// lift this restriction if the need for more complex states arises. Meanwhile, one
/// can use dynamically indexed states.
pub trait State: Hash + Eq + Clone + Copy {}

/// A very basic implementation of a `State` which simply stores a single `usize` index.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IdState(usize);

/// A simple `IdState` iterator used for graphs where the states are consecutive integers.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IdStateRange {
    next: usize,
    remaining: usize,
}
