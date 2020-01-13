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
pub mod bdd_params;
pub mod builder;
mod impl_boolean_network;
mod impl_boolean_network_string_serialisation;

/// An index of a variable in the `variables` vector of a `BooleanNetwork`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VariableId(pub(super) usize);

/// An index of a parameter in the `parameters` vector of a `BooleanNetwork`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParameterId(pub(super) usize);

/// Possible monotonous effect of a `Regulation` in a `BooleanNetwork`.
///
/// Activation means that the update function is increasing in the annotated argument.
/// Inhibition signifies decreasing function.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Effect {
    ACTIVATION,
    INHIBITION,
}

/// A variable of a `BooleanNetwork`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Variable {
    name: String,
}

/// A parameter of a `BooleanNetwork`. Parameter is an uninterpreted function with
/// a fixed cardinality.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Parameter {
    name: String,
    cardinality: usize,
}

/// Describes an interaction relationship between two `Variable`s in a `BooleanNetwork`.
///
/// Every regulation can have a monotonous `Effect` and can be marked as `observable`.
/// Observability means that the regulation must manifest itself somewhere in the
/// corresponding update function (but not necessarily always).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Regulation {
    source: VariableId,
    target: VariableId,
    observable: bool,
    effect: Option<Effect>,
}

/// Represents one parametrised boolean network with uninterpreted boolean
/// functions as parameters.
///
/// Note that if the `UpdateFunction` for a variable is unspecified, it means the
/// behaviour of the whole function is undefined (only guided by the regulations).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BooleanNetwork {
    regulatory_graph: RegulatoryGraph,
    parameters: Vec<Parameter>,
    parameter_to_index: HashMap<String, ParameterId>,
    update_functions: Vec<Option<UpdateFunction>>,
}

/// Update function is a boolean formula that can contain *parameters*, i.e. uninterpreted
/// boolean functions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UpdateFunction {
    Parameter {
        id: ParameterId,
        inputs: Vec<VariableId>,
    },
    Variable {
        id: VariableId,
    },
    Not(Box<UpdateFunction>),
    And(Box<UpdateFunction>, Box<UpdateFunction>),
    Or(Box<UpdateFunction>, Box<UpdateFunction>),
    Xor(Box<UpdateFunction>, Box<UpdateFunction>),
    Iff(Box<UpdateFunction>, Box<UpdateFunction>),
    Imp(Box<UpdateFunction>, Box<UpdateFunction>),
}
