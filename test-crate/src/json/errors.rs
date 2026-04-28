use std::{error::Error, fmt::Display};

use crate::json::structs::TokenMetaData;

#[derive(Debug)]
pub enum JSONError {
    TokenizerError {
        message: String,
        line: i32,
        col: i32,
    },
    ParserError {
        message: String,
        token: TokenMetaData,
    },
    UnexpectedToken {
        token: TokenMetaData,
    },
    UnexpectedEOF,
}

impl Display for JSONError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JSONError::UnexpectedEOF => write!(f, "Unexpected end of file"),
            JSONError::UnexpectedToken { .. } => write!(f, "Unexpected token"),
            JSONError::ParserError { message, .. } => write!(f, "{}", message),
            JSONError::TokenizerError { message, .. } => write!(f, "{}", message),
        }
    }
}

impl Error for JSONError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
