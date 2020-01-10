use crate::boolean_network::builder::{UpdateFunctionTemplate, RegulatoryGraph};
use crate::boolean_network::builder::UpdateFunctionTemplate::*;

impl UpdateFunctionTemplate {

    /// Swap variables in this function that don't occur in the given `rg` for
    /// unary parameters.
    pub fn swap_unary_parameters(self, rg: &RegulatoryGraph) -> Box<UpdateFunctionTemplate> {
        return Box::new(match self {
            Variable { name} => {
                if rg.has_variable(&name) {
                    Variable { name }
                } else {
                    Parameter { name, inputs: Vec::new() }
                }
            },
            Parameter { .. } => self,
            Not(inner) => Not(inner.swap_unary_parameters(rg)),
            And(a, b) => And(
                a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)
            ),
            Or(a, b) => Or(
                a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)
            ),
            Imp(a, b) => Imp(
                a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)
            ),
            Iff(a, b) => Iff(
                a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)
            ),
            Xor(a, b) => Xor(
                a.swap_unary_parameters(rg), b.swap_unary_parameters(rg)
            )
        })
    }

   /* pub fn extract_parameters(&self) -> Vec<crate::boolean_network::Parameter> {
        return match self {

        }
    }*/

}