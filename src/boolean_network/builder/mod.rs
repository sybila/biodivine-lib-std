//! In this module, we provide a safe method for building parametrised `BooleanNetwork`s
//! either directly, or from a predefined string format (the same format the network
//! can be serialized into).
//!
//! The process has two important parts:
//!
//! - First, based on a set of given regulations, a `RegulatoryGraph` is constructed.
//! `RegulatoryGraph` is a partial `BooleanNetwork` without the specified update functions.
//! - A `BooleanNetworkBuilder` initialized with a `RegulatoryGraph` can be constructed
//! and used to include actual parametrised update functions in order to create a `BooleanNetwork`.

use crate::boolean_network::{Effect, Regulation, Variable, VariableId};
use std::collections::HashMap;

mod display_update_function_template;
mod impl_boolean_network_builder;
mod impl_boolean_network_parser;
mod impl_regulatory_graph;
mod impl_update_function_template;
mod try_from_regulation_template;
mod try_from_update_function_template;

/// **(internal)** Update function template is an abstract syntax tree of an `UpdateFunction`.
///
/// It can be transformed into a proper `UpdateFunction` by combining it with an
/// existing `RegulatoryGraph` or `BooleanNetwork`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum UpdateFunctionTemplate {
    Parameter { name: String, inputs: Vec<String> },
    Variable { name: String },
    Not(Box<UpdateFunctionTemplate>),
    And(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Or(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Xor(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Iff(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Imp(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
}

/// **(internal)** A template for a regulation object that can be later transformed into a
/// real `Regulation` once variable indices are known.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RegulationTemplate {
    source: String,
    target: String,
    observable: bool,
    effect: Option<Effect>,
}

/// A partial representation of a `BooleanNetwork` without the exact update functions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryGraph {
    variables: Vec<Variable>,
    regulations: Vec<Regulation>,
    variable_to_index: HashMap<String, VariableId>,
}
