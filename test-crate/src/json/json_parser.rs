//! Basic JSON parser

use std::{io::Error, string::ParseError};

use crate::json::{
    errors::JSONError,
    structs::{JSONToken, TokenMetaData},
    tokenizer::tokenize,
};

#[derive(Debug)]
pub struct JSONMember {
    key: String,
    value: JSONValue,
}

#[derive(Debug)]
pub struct JSONNumber {
    raw: String,
}

#[derive(Debug)]
pub enum JSONValue {
    Object { members: Vec<JSONMember> },
    Array { elements: Vec<JSONValue> },
    Bool(bool),
    String(String),
    Number(JSONNumber),
    Null,
}

/// Parses a JSON number from a raw string
/// Allowed numbers in JSON:
/// * 123
/// * 1.23
/// * -45
/// * -0.34
/// * 3.6e3
/// * 5.6e+3
/// * 5e-3
/// * -5e+3
///
/// Disallowed number types:
/// * 012
/// * +45
/// * 5.6e+3.6
fn parse_number(raw: String) -> Result<JSONNumber, JSONError> {
    // TODO: Implement the parsing/validation logic here for numbers and
    // Determine the correct internal representation for numbers
    return Ok(JSONNumber { raw });
}

fn construct_next_node(
    iter: &mut std::slice::Iter<'_, TokenMetaData>,
) -> Result<JSONValue, JSONError> {
    let Some(token_data) = iter.next() else {
        return Err(JSONError::UnexpectedEOF);
    };

    match &token_data.token {
        JSONToken::CurlyOpen => construct_object(iter),
        JSONToken::BracketOpen => construct_array(iter),
        JSONToken::String(value) => Ok(JSONValue::String(value.clone())),
        JSONToken::Literal(_) => construct_literal(token_data.clone()),
        _ => Err(JSONError::ParserError {
            message: String::from("Unexpected token"),
            token: token_data.clone(),
        }),
    }
}

fn construct_literal(token_data: TokenMetaData) -> Result<JSONValue, JSONError> {
    let JSONToken::Literal(value) = token_data.token else {
        return Err(JSONError::ParserError {
            message: String::from("Unexpected literal token"),
            token: token_data,
        });
    };

    if value == "null" {
        Ok(JSONValue::Null)
    } else if value == "true" {
        Ok(JSONValue::Bool(true))
    } else if value == "false" {
        Ok(JSONValue::Bool(false))
    } else {
        // TODO: Parse number value
        Ok(JSONValue::Number(parse_number(value)?))
    }
}

fn construct_array(iter: &mut std::slice::Iter<'_, TokenMetaData>) -> Result<JSONValue, JSONError> {
    let mut elements = Vec::new();

    match construct_next_node(iter) {
        Ok(value) => elements.push(value),
        Err(err) => {
            if let JSONError::ParserError {
                token:
                    TokenMetaData {
                        token: JSONToken::BracketClose,
                        ..
                    },
                ..
            } = err
            {
                return Ok(JSONValue::Array { elements });
            } else {
                return Err(err);
            }
        }
    }

    while let Some(token_data) = iter.next() {
        match &token_data.token {
            JSONToken::BracketClose => return Ok(JSONValue::Array { elements }),
            JSONToken::Comma => {
                elements.push(construct_next_node(iter)?);
            }
            _ => {
                return Err(JSONError::ParserError {
                    message: String::from("Unexpected token in array"),
                    token: token_data.clone(),
                });
            }
        }
    }
    return Err(JSONError::UnexpectedEOF);
}

fn construct_next_object_member(
    iter: &mut std::slice::Iter<'_, TokenMetaData>,
) -> Result<Option<JSONMember>, JSONError> {
    let Some(token_data) = iter.next() else {
        return Err(JSONError::UnexpectedEOF);
    };

    let key = match &token_data.token {
        JSONToken::CurlyClose => return Ok(None),
        JSONToken::String(key) => key.clone(),
        _ => {
            return Err(JSONError::UnexpectedToken {
                token: token_data.clone(),
            });
        }
    };

    let Some(token_data) = iter.next() else {
        return Err(JSONError::UnexpectedEOF);
    };

    match token_data.token {
        JSONToken::Colon => {}
        _ => {
            return Err(JSONError::UnexpectedToken {
                token: token_data.clone(),
            });
        }
    };

    let value = construct_next_node(iter)?;

    return Ok(Some(JSONMember { key, value }));
}

fn construct_object(
    iter: &mut std::slice::Iter<'_, TokenMetaData>,
) -> Result<JSONValue, JSONError> {
    let mut members = Vec::new();

    while let Some(member) = construct_next_object_member(iter)? {
        members.push(member);
        let Some(token_data) = iter.next() else {
            return Err(JSONError::UnexpectedEOF);
        };

        match &token_data.token {
            JSONToken::Comma => {}
            JSONToken::CurlyClose => {
                return Ok(JSONValue::Object { members });
            }
            _ => {
                return Err(JSONError::UnexpectedToken {
                    token: token_data.clone(),
                });
            }
        }
    }
    Ok(JSONValue::Object { members })
}

pub fn parse_data(raw: &str) -> Result<JSONValue, JSONError> {
    let tokens = tokenize(raw)?;
    let mut iter = tokens.iter();
    let root = construct_next_node(&mut iter)?;

    if let Some(token_data) = iter.next() {
        return Err(JSONError::UnexpectedToken {
            token: token_data.clone(),
        });
    }

    return Ok(root);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let data = parse_data("{\"key\": 123}").unwrap();
        println!("{:?}", data);

        let data2 = parse_data("{\"key\": [1, 2.0, true, false, null]}").unwrap();
        println!("{:?}", data2);
    }

    #[test]
    fn test_should_fail_multiple_roots() {
        let data = parse_data("{\"key\": 123}123");
        assert!(data.is_err());
    }

    // #[test]
    // fn test_parse_number() {
    //     /// Allowed numbers in JSON:
    //     /// * 123
    //     /// * 1.23
    //     /// * -45
    //     /// * -0.34
    //     /// * 3.6e3
    //     /// * 5.6e+3
    //     /// * 5e-3
    //     /// * -5e+3
    //     ///
    //     /// Disallowed number types:
    //     /// * 012
    //     /// * +45
    //     /// * 5.6e+3.6
    //     assert_eq!(parse_number(String::from("123")).unwrap().raw, "123");
    // }
}
