use crate::boolean_network::UpdateFunction::*;
use crate::boolean_network::{UpdateFunction, VariableId};
use std::collections::HashSet;

impl UpdateFunction {
    pub fn variables(&self) -> HashSet<VariableId> {
        return match self {
            Parameter { id, inputs } => {
                let mut set = HashSet::new();
                for arg in inputs {
                    set.insert(arg.clone());
                }
                set
            }
            Variable { id } => {
                let mut set = HashSet::new();
                set.insert(*id);
                set
            }
            Not(inner) => inner.variables(),
            And(a, b) => extract_variable_util(a, b),
            Or(a, b) => extract_variable_util(a, b),
            Imp(a, b) => extract_variable_util(a, b),
            Iff(a, b) => extract_variable_util(a, b),
            Xor(a, b) => extract_variable_util(a, b),
        };
    }
}

fn extract_variable_util(a: &UpdateFunction, b: &UpdateFunction) -> HashSet<VariableId> {
    let mut a = a.variables();
    a.extend(b.variables());
    return a;
}
