use crate::boolean_network::UpdateFunction::*;
use crate::boolean_network::{
    BooleanNetwork, Effect, Parameter, Regulation, UpdateFunction, Variable, VariableId,
};
use std::fmt::{Display, Error, Formatter};

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return write!(f, "{}", self.name);
    }
}

impl Display for BooleanNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for reg in &self.regulations {
            // print all the regulations
            write!(f, "{}\n", Reg(self, reg))?;
        }
        for var in self.variable_ids() {
            // print all update functions
            if let Some(fun) = self.get_update_function(var) {
                write!(f, "${}: {}\n", self.get_variable(var), Fun(self, fun))?;
            }
        }
        return Ok(());
    }
}

struct Reg<'a>(&'a BooleanNetwork, &'a Regulation);
struct Fun<'a>(&'a BooleanNetwork, &'a UpdateFunction);

impl Display for Reg<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let Reg(bn, reg) = self;
        let effect = match reg.effect {
            None => "?",
            Some(Effect::ACTIVATION) => ">",
            Some(Effect::INHIBITION) => "|",
        };
        let observable = if reg.observable { "" } else { "?" };
        write!(
            f,
            "{} -{}{} {}",
            bn.get_variable(reg.source),
            effect,
            observable,
            bn.get_variable(reg.target)
        )
    }
}

impl Display for Fun<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let Fun(bn, fun) = self;
        match fun {
            UpdateFunction::Variable { id } => {
                write!(f, "{}", bn.get_variable(*id))?;
            }
            UpdateFunction::Parameter { id, inputs } => {
                let parameter = bn.get_parameter(*id);
                assert_eq!(inputs.len(), parameter.cardinality);
                write!(f, "{}", parameter.name)?;
                if inputs.len() > 0 {
                    write!(f, "({}", bn.get_variable(inputs[0]))?;
                    for i in 1..inputs.len() {
                        write!(f, ", {}", bn.get_variable(inputs[i]))?;
                    }
                    write!(f, ")")?;
                }
            }
            Not(inner) => write!(f, "!{}", Fun(bn, inner))?,
            And(a, b) => write!(f, "({} & {})", Fun(bn, a), Fun(bn, b))?,
            Or(a, b) => write!(f, "({} | {})", Fun(bn, a), Fun(bn, b))?,
            Imp(a, b) => write!(f, "({} => {})", Fun(bn, a), Fun(bn, b))?,
            Iff(a, b) => write!(f, "({} <=> {})", Fun(bn, a), Fun(bn, b))?,
            Xor(a, b) => write!(f, "({} ^ {})", Fun(bn, a), Fun(bn, b))?,
        }
        Ok(())
    }
}
