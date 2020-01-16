pub mod graph;
pub mod lock_free_array;
pub mod lock_free_array_queue;
pub mod parameters;
pub mod range_state_iterator;
pub mod reachability;

pub mod util;

/// A simple `StateId` iterator used for graphs where the states are consecutive integers.
pub struct RangeStateIterator {
    next: usize,
    remaining: usize,
}

pub mod v2;
