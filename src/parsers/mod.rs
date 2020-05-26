//! Utility methods for writing parsers, mostly focused on boolean formulas right now.
//!
//! Note that we are definitely not trying to be super fast (no single-copy or whatever parsers
//! here). Just creating an architecture that will work well with a lot of different formats.
//!
//!
//!

pub mod tokens;
