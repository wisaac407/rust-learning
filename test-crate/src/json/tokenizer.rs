use std::{error::Error, fmt::Display};

use crate::json::{
    errors::JSONError,
    structs::{JSONToken, TokenMetaData},
};

const CHAR_NEW_LINE: char = '\n';
const CHAR_CURLY_OPEN: char = '{';
const CHAR_CURLY_CLOSE: char = '}';
const CHAR_BRACKET_OPEN: char = '[';
const CHAR_BRACKET_CLOSE: char = ']';
const CHAR_COLON: char = ':';
const CHAR_COMMA: char = ',';
const CHAR_QUOTE: char = '"';
const CHAR_ESCAPE: char = '\\';

const CHAR_A_LOWER: char = 'a';
const CHAR_Z_LOWER: char = 'z';
const CHAR_E_UPPER: char = 'E';

const CHAR_ZERO: char = '0';
const CHAR_NINE: char = '9';
const CHAR_MINUS: char = '-';
const CHAR_PLUS: char = '+';
const CHAR_PERIOD: char = '.';

fn is_valid_literal_char(c: char) -> bool {
    (CHAR_A_LOWER <= c && c <= CHAR_Z_LOWER)
        || (CHAR_ZERO <= c && c <= CHAR_NINE)
        || c == CHAR_MINUS
        || c == CHAR_PLUS
        || c == CHAR_PERIOD
        || c == CHAR_E_UPPER
}

fn get_escaped_char(c: char) -> char {
    match c {
        'n' => CHAR_NEW_LINE,
        _ => c,
    }
}

/// Tokenize the raw text into JSON tokens
pub fn tokenize(raw: &str) -> Result<Vec<TokenMetaData>, JSONError> {
    let mut in_quotes = false;
    let mut escaped = false;
    let mut is_literal = false;

    let mut line = 0;
    let mut col = 0;

    let mut cur_line = 0;
    let mut cur_col = 0;

    let mut cur = String::new();
    let mut tokens = Vec::new();

    for c in raw.chars() {
        if c == CHAR_NEW_LINE {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        if in_quotes {
            if escaped {
                cur.push(get_escaped_char(c));
                escaped = false
            } else if c == CHAR_QUOTE {
                tokens.push(TokenMetaData {
                    line: cur_line,
                    col: cur_col,
                    token: JSONToken::String(cur),
                });
                cur = String::new();
                in_quotes = false;
            } else if c == CHAR_ESCAPE {
                escaped = true
            } else {
                cur.push(c);
            }
            continue;
        } else if is_literal {
            if is_valid_literal_char(c) {
                cur.push(c);
                continue;
            } else {
                tokens.push(TokenMetaData {
                    line: cur_line,
                    col: cur_col,
                    token: JSONToken::Literal(cur),
                });
                cur = String::new();
                is_literal = false
            }
        }

        if is_valid_literal_char(c) {
            cur_line = line;
            cur_col = col;
            is_literal = true;

            // TOOD: Should we do cur.push instead?
            cur = String::from(c);
        } else if c == CHAR_QUOTE {
            cur_line = line;
            cur_col = col;
            in_quotes = true;
            cur = String::new();
        } else if c.is_whitespace() {
            // Ignore whitespace
        } else {
            let token = match c {
                CHAR_BRACKET_OPEN => JSONToken::BracketOpen,
                CHAR_BRACKET_CLOSE => JSONToken::BracketClose,
                CHAR_CURLY_OPEN => JSONToken::CurlyOpen,
                CHAR_CURLY_CLOSE => JSONToken::CurlyClose,
                CHAR_COLON => JSONToken::Colon,
                CHAR_COMMA => JSONToken::Comma,
                _ => {
                    return Err(JSONError::TokenizerError {
                        message: format!("Invalid char: {}", c),
                        line,
                        col,
                    });
                }
            };

            tokens.push(TokenMetaData { line, col, token });
        }
    }

    if cur.len() > 0 {
        if in_quotes {
            Err(JSONError::TokenizerError {
                message: String::from("Unclosed string"),
                line: cur_line,
                col: cur_col,
            })
        } else if is_literal {
            tokens.push(TokenMetaData {
                line: cur_line,
                col: cur_col,
                token: JSONToken::Literal(cur),
            });
            Ok(tokens)
        } else {
            Err(JSONError::TokenizerError {
                line,
                col,
                message: String::from("Unknown parsing error"),
            })
        }
    } else {
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let s = "{\"bar\": [107, true, false, null, -4.5e+7, 3E-8]}";
        let result = tokenize(s);

        // Get rid of token data to make matching easier
        let tokens: Vec<JSONToken> = result.unwrap().into_iter().map(|data| data.token).collect();

        assert_eq!(
            tokens,
            vec![
                JSONToken::CurlyOpen,
                JSONToken::String(String::from("bar")),
                JSONToken::Colon,
                JSONToken::BracketOpen,
                JSONToken::Literal(String::from("107")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("true")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("false")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("null")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("-4.5e+7")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("3E-8")),
                JSONToken::BracketClose,
                JSONToken::CurlyClose,
            ]
        );
    }

    #[test]
    fn test_tokenize_literal() {
        let s = "null";
        let result = tokenize(s);

        // Get rid of token data to make matching easier
        let token = result.unwrap().pop().unwrap().token;

        assert_eq!(token, JSONToken::Literal(String::from("null")));
    }

    #[test]
    fn test_tokenize_string() {
        let s = "\"Hello World\"";
        let result = tokenize(s);

        // Get rid of token data to make matching easier
        let token = result.unwrap().pop().unwrap().token;

        assert_eq!(token, JSONToken::String(String::from("Hello World")));
    }

    #[test]
    fn test_tokenize_array() {
        let s = "[1,2,    3]";
        let result = tokenize(s);

        // Get rid of token data to make matching easier
        let tokens: Vec<JSONToken> = result.unwrap().into_iter().map(|data| data.token).collect();

        assert_eq!(
            tokens,
            vec![
                JSONToken::BracketOpen,
                JSONToken::Literal(String::from("1")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("2")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("3")),
                JSONToken::BracketClose,
            ]
        );
    }

    #[test]
    fn test_tokenize_multiline() {
        let s = "
    {
        \"key\": [
            1,
            2,
            3
        ]
    }
        ";
        let result = tokenize(s);

        // Get rid of token data to make matching easier
        let tokens: Vec<JSONToken> = result.unwrap().into_iter().map(|data| data.token).collect();

        assert_eq!(
            tokens,
            vec![
                JSONToken::CurlyOpen,
                JSONToken::String(String::from("key")),
                JSONToken::Colon,
                JSONToken::BracketOpen,
                JSONToken::Literal(String::from("1")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("2")),
                JSONToken::Comma,
                JSONToken::Literal(String::from("3")),
                JSONToken::BracketClose,
                JSONToken::CurlyClose,
            ]
        );
    }

    #[test]
    fn test_tokenize_empty_string() {
        let s = "";
        let result = tokenize(s);

        // Get rid of token data to make matching easier
        let tokens: Vec<JSONToken> = result.unwrap().into_iter().map(|data| data.token).collect();

        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_tokenize_string_after_literal() {
        let s = "45HELLO";
        let result = tokenize(s);

        assert!(result.is_err())
    }

    #[test]
    fn test_tokenize_invalid_syntax_no_comma() {
        let s = "[1,2{\"a\": 4}]";
        let result = tokenize(s);

        println!("{:?}", result);

        assert!(result.is_err())
    }

    #[test]
    fn test_unclosed_string_should_fail() {
        let s = "\"test";
        let result = tokenize(s);

        assert!(result.is_err())
    }
}
