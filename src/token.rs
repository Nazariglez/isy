#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
}

impl Token {
    pub fn new(typ: TokenType) -> Token {
        Token { typ }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Illegal(char),
    EOF,

    Type(String),

    Ident(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),

    Assign,
    Colon,

    Equal,
    NotEqual,
    Bang,

    Minus,
    Plus,
    Asterisk,
    Slash,
    Module,
}
