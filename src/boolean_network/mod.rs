//! This module contains data structures for storing and manipulating parametrised
//! boolean networks together with import and export procedures for various formats.
//!
//! Parametrised boolean network consists of *variables*, *regulations* and *update functions*.
//! Variables are basic components of the studied system, each can have a value of 0 or 1.
//! Regulations indicate which variables influence each other (and possibly in what way).
//!
//! Variables and regulations together form a *regulatory graph*.

use crate::boolean_network::builder::RegulatoryGraph;
use std::collections::HashMap;

pub mod async_graph;
