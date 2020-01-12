use crate::boolean_network::builder::UpdateFunctionTemplate;
use crate::boolean_network::builder::UpdateFunctionTemplate::*;
use crate::boolean_network::UpdateFunction;
use std::convert::TryFrom;
use std::fmt::{Display, Error, Formatter};
use std::iter::Peekable;
use std::str::Chars;

impl TryFrom<&str> for UpdateFunctionTemplate {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = tokenize_function_group(&mut value.chars().peekable(), true)?;
        return Ok(*(parse_update_function(&tokens)?));
    }
}

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

#[derive(Debug, Eq, PartialEq)]
enum Token {
    Not,                // '!'
    And,                // '&'
    Or,                 // '|'
    Xor,                // '^'
    Imp,                // '=>'
    Iff,                // '<=>'
    Comma,              // ','
    Name(String),       // 'name'
    Tokens(Vec<Token>), // A block of tokens inside parentheses
}

/// **(internal)** Process a peekable iterator of characters into a vector of `Token`s.
///
/// The outer method always consumes the opening parenthesis and the recursive call consumes the
/// closing parenthesis. Use `top_level` to indicate that there will be no closing parenthesis.
fn tokenize_function_group(
    data: &mut Peekable<Chars>,
    top_level: bool,
) -> Result<Vec<Token>, String> {
    let mut output = Vec::new();
    while let Some(c) = data.next() {
        match c {
            c if c.is_whitespace() => { /* Skip whitespace */ }
            // single char tokens
            '!' => output.push(Token::Not),
            ',' => output.push(Token::Comma),
            '&' => output.push(Token::And),
            '|' => output.push(Token::Or),
            '^' => output.push(Token::Xor),
            '=' => {
                if Some('>') == data.next() {
                    output.push(Token::Imp);
                } else {
                    return Result::Err("Expected '>' after '='.".to_string());
                }
            }
            '<' => {
                if Some('=') == data.next() {
                    if Some('>') == data.next() {
                        output.push(Token::Iff)
                    } else {
                        return Result::Err("Expected '>' after '='.".to_string());
                    }
                } else {
                    return Result::Err("Expected '=' after '<'.".to_string());
                }
            }
            // '>' is invalid as a start of a token
            '>' => return Result::Err("Unexpected '>'.".to_string()),
            ')' => {
                return if !top_level {
                    Result::Ok(output)
                } else {
                    Result::Err("Unexpected ')'.".to_string())
                }
            }
            '(' => {
                // start a nested token group
                let tokens = tokenize_function_group(data, false)?;
                output.push(Token::Tokens(tokens));
            }
            c if is_valid_in_name(c) => {
                // start of a variable name
                let mut name = vec![c];
                while let Some(c) = data.peek() {
                    if c.is_whitespace() || !is_valid_in_name(*c) {
                        break;
                    } else {
                        name.push(*c);
                        data.next(); // advance iterator
                    }
                }
                output.push(Token::Name(name.into_iter().collect()));
            }
            _ => return Result::Err(format!("Unexpected '{}'.", c)),
        }
    }
    return if top_level {
        Result::Ok(output)
    } else {
        Result::Err("Expected ')'.".to_string())
    };
}

fn is_valid_in_name(c: char) -> bool {
    return c.is_alphanumeric() || c == '_';
}

fn parse_update_function(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    return iff(data);
}

/// **(internal)** Utility method to find first occurrence of a specific token in the token tree.
fn index_of_first(data: &[Token], token: Token) -> Option<usize> {
    return data.iter().position(|t| *t == token);
}

/// **(internal)** Recursive parsing step 1: extract `<=>` operators.
fn iff(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    let iff_token = index_of_first(data, Token::Iff);
    return Ok(if let Some(iff_token) = iff_token {
        Box::new(Iff(
            imp(&data[..iff_token])?,
            iff(&data[(iff_token + 1)..])?,
        ))
    } else {
        imp(data)?
    });
}

/// **(internal)** Recursive parsing step 2: extract `=>` operators.
fn imp(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    let imp_token = index_of_first(data, Token::Imp);
    return Ok(if let Some(imp_token) = imp_token {
        Box::new(Imp(or(&data[..imp_token])?, imp(&data[(imp_token + 1)..])?))
    } else {
        or(data)?
    });
}

/// **(internal)** Recursive parsing step 3: extract `|` operators.
fn or(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    let or_token = index_of_first(data, Token::Or);
    return Ok(if let Some(or_token) = or_token {
        Box::new(Or(and(&data[..or_token])?, or(&data[(or_token + 1)..])?))
    } else {
        and(data)?
    });
}

/// **(internal)** Recursive parsing step 4: extract `&` operators.
fn and(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    let and_token = index_of_first(data, Token::And);
    return Ok(if let Some(and_token) = and_token {
        Box::new(And(
            xor(&data[..and_token])?,
            and(&data[(and_token + 1)..])?,
        ))
    } else {
        xor(data)?
    });
}

/// **(internal)** Recursive parsing step 5: extract `^` operators.
fn xor(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    let xor_token = index_of_first(data, Token::Xor);
    return Ok(if let Some(xor_token) = xor_token {
        Box::new(Xor(
            terminal(&data[..xor_token])?,
            xor(&data[(xor_token + 1)..])?,
        ))
    } else {
        terminal(data)?
    });
}

/// **(internal)** Recursive parsing step 6: extract terminals and negations.
fn terminal(data: &[Token]) -> Result<Box<UpdateFunctionTemplate>, String> {
    return if data.is_empty() {
        Err("Expected formula, found nothing :(".to_string())
    } else {
        if data[0] == Token::Not {
            Ok(Box::new(Not(terminal(&data[1..])?)))
        } else if data.len() == 1 {
            // This should be either a name or a parenthesis group, everything else does not make sense.
            match &data[0] {
                Token::Name(name) => Ok(Box::new(Variable { name: name.clone() })),
                Token::Tokens(inner) => Ok(parse_update_function(inner)?),
                _ => Err(format!(
                    "Unexpected token: {:?}. Expecting formula.",
                    data[0]
                )),
            }
        } else if data.len() == 2 {
            // If more tokens remain, it means this should be a parameter (function call).
            // Anything else is invalid.
            if let Token::Name(name) = &data[0] {
                if let Token::Tokens(args) = &data[1] {
                    let inputs = read_args(args)?;
                    Ok(Box::new(Parameter {
                        name: name.clone(),
                        inputs,
                    }))
                } else {
                    Err(format!("Unexpected: {:?}. Expecting formula.", data))
                }
            } else {
                Err(format!("Unexpected: {:?}. Expecting formula.", data))
            }
        } else {
            Err(format!("Unexpected: {:?}. Expecting formula.", data))
        }
    };
}

/// Parse a list of function arguments. All arguments must be names separated by commas.
fn read_args(data: &[Token]) -> Result<Vec<String>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let mut i = 0;
    while let Token::Name(name) = &data[i] {
        result.push(name.clone());
        i += 1;
        if data.len() == i {
            return Ok(result);
        }
        if data[i] != Token::Comma {
            return Err(format!("Expected ',', found {:?}.", data[i]));
        }
        i += 1;
        if data.len() == i {
            return Err("Unexpected ',' at the end of an argument list.".to_string());
        }
    }
    return Err(format!("Unexpected token {:?} in argument list.", data[i]));
}

#[cfg(test)]
mod tests {
    use crate::boolean_network::builder::UpdateFunctionTemplate;
    use std::convert::TryFrom;

    #[test]
    fn parse_update_function_basic() {
        let inputs = vec![
            "var",
            "var1(a, b, c)",
            "!foo(a)",
            "(var(a, b) | x)",
            "(xyz123 & abc)",
            "(a ^ b)",
            "(a => b)",
            "(a <=> b)",
            "(a <=> !(f(a, b) => (c ^ d)))",
        ];
        for str in inputs {
            assert_eq!(
                str,
                format!("{}", UpdateFunctionTemplate::try_from(str).unwrap())
            )
        }
    }

}
