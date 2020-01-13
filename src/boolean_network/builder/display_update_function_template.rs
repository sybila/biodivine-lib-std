use crate::boolean_network::builder::UpdateFunctionTemplate;
use crate::boolean_network::builder::UpdateFunctionTemplate::*;
use std::fmt::{Display, Error, Formatter};

impl Display for UpdateFunctionTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            UpdateFunctionTemplate::Variable { name } => {
                write!(f, "{}", name)?;
            }
            UpdateFunctionTemplate::Parameter { name, inputs } => {
                write!(f, "{}", name)?;
                if inputs.len() > 0 {
                    write!(f, "({}", inputs[0])?;
                    for i in 1..inputs.len() {
                        write!(f, ", {}", inputs[i])?;
                    }
                    write!(f, ")")?;
                }
            }
            Not(inner) => write!(f, "!{}", inner)?,
            And(a, b) => write!(f, "({} & {})", a, b)?,
            Or(a, b) => write!(f, "({} | {})", a, b)?,
            Imp(a, b) => write!(f, "({} => {})", a, b)?,
            Iff(a, b) => write!(f, "({} <=> {})", a, b)?,
            Xor(a, b) => write!(f, "({} ^ {})", a, b)?,
        }
        Ok(())
    }
}
