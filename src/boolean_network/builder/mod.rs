use crate::boolean_network::{Effect, Parameter, Regulation, Variable, VariableId, UpdateFunction};
use std::collections::HashMap;

mod impl_regulation_parser;
mod impl_regulatory_graph;
mod impl_update_function_parser;
mod impl_boolean_network_parser;
mod impl_boolean_network_builder;
mod impl_update_function_template;

/// Update function template is an abstract syntax tree of an `UpdateFunction`.
///
/// It can be transformed into a proper `UpdateFunction` by combining it with an
/// existing boolean network (or more specifically, a template of it).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UpdateFunctionTemplate {
    Parameter { name: String, inputs: Vec<String> },
    Variable { name: String },
    Not(Box<UpdateFunctionTemplate>),
    And(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Or(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Xor(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Iff(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
    Imp(Box<UpdateFunctionTemplate>, Box<UpdateFunctionTemplate>),
}

/// A template for a regulation object that can be later transformed into real `Regulation`
/// once all variables are known.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RegulationTemplate {
    source: String,
    target: String,
    observable: bool,
    effect: Option<Effect>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegulatoryGraph {
    variables: Vec<Variable>,
    regulations: Vec<Regulation>,
    variable_to_index: HashMap<String, VariableId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BooleanNetworkBuilder {
    regulatory_graph: RegulatoryGraph,
    parameters: Vec<Parameter>,
    update_functions: Vec<Option<UpdateFunction>>
}