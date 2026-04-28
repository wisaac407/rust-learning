#[derive(PartialEq, Eq, Debug, Clone)]
pub enum JSONToken {
    CurlyOpen,
    CurlyClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    String(String),

    // Can be bool, number, or null
    Literal(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct TokenMetaData {
    pub line: i32,
    pub col: i32,
    pub token: JSONToken,
}
