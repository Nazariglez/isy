use crate::token;
use crate::token::{Token, TokenType};
use std::str::{CharIndices, Chars};

pub struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    pos: usize,
    read_position: usize,
    ch: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        let mut lexer = Lexer {
            input,
            chars: input.char_indices(),
            pos: 0,
            read_position: 0,
            ch: None,
        };

        lexer.next_char();

        lexer
    }

    fn next_char(&mut self) {
        match self.chars.next() {
            Some((pos, ch)) => {
                self.ch = Some(ch);
                self.pos = pos;
            }
            _ => {
                self.ch = None;
                self.pos = self.input.len();
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let mut read_next = true;
        let typ = match self.ch {
            Some(':') => TokenType::Colon,
            Some('+') => TokenType::Plus,
            Some('-') => TokenType::Minus,
            Some('/') => TokenType::Slash,
            Some('*') => TokenType::Asterisk,
            Some('%') => TokenType::Module,
            Some('"') => self.read_string().unwrap(),
            Some('=') => match self.peek_char() {
                Some('=') => TokenType::Equal,
                _ => TokenType::Assign,
            },
            Some('!') => match self.peek_char() {
                Some('=') => TokenType::NotEqual,
                _ => TokenType::Bang,
            },
            Some(ch) => {
                if is_letter(self.ch) {
                    read_next = false;
                    self.read_identifier().unwrap()
                } else if is_digit(self.ch) {
                    read_next = false;
                    self.read_number().unwrap()
                } else {
                    TokenType::Illegal(ch)
                }
            }
            None => TokenType::EOF,
        };

        if read_next {
            self.next_char();
        }

        Token::new(typ)
    }

    fn peek_char(&self) -> Option<char> {
        if self.pos >= self.input.len() {
            return None;
        }

        self.input.chars().nth(self.pos + 1)
    }

    fn prev_char(&self) -> Option<char> {
        if self.pos == 0 {
            return None;
        }

        self.input.chars().nth(self.pos - 1)
    }

    fn read_string(&mut self) -> Result<TokenType, String> {
        let initial_pos = self.pos + 1;
        let mut escape = false;
        while self.peek_char().is_some() {
            self.next_char();

            match self.ch {
                Some('\\') if !escape => {
                    escape = true;
                    continue;
                }
                Some('"') => {
                    if !escape {
                        break;
                    }
                }
                _ => {}
            }

            escape = false;
        }

        Ok(TokenType::String(
            self.input[initial_pos..self.pos].to_string(),
        ))
    }

    fn read_number(&mut self) -> Result<TokenType, String> {
        let mut is_float = false;
        let initial_pos = self.pos;
        loop {
            if is_digit(self.ch) {
                self.next_char();
                continue;
            }

            if !is_float {
                if let Some('.') = self.ch {
                    if is_digit(self.peek_char()) {
                        is_float = true;
                        self.next_char();
                        continue;
                    }
                }
            }

            break;
        }

        let num = &self.input[initial_pos..self.pos];
        if is_float {
            let float_num = num.parse::<f32>().map_err(|e| e.to_string())?;
            return Ok(TokenType::Float(float_num));
        }

        let int_num = num.parse::<i32>().map_err(|e| e.to_string())?;
        Ok(TokenType::Int(int_num))
    }

    fn read_identifier(&mut self) -> Result<TokenType, String> {
        let initial_pos = self.pos;
        while is_letter(self.ch) || is_digit(self.ch) {
            self.next_char();
        }

        let ident = &self.input[initial_pos..self.pos];
        Ok(lookup_ident(ident))
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ') = self.ch {
            self.next_char();
        }
    }
}

fn lookup_ident(ident: &str) -> TokenType {
    match ident {
        "true" => TokenType::Bool(true),
        "false" => TokenType::Bool(false),
        "bool" | "int" | "float" | "string" => TokenType::Type(ident.to_string()),
        _ => TokenType::Ident(ident.to_string()),
    }
}

fn is_new_line(ch: char) -> bool {
    ch == '\n' // \t? \r?
}

fn is_letter(ch: Option<char>) -> bool {
    match ch {
        Some('a'..='z') | Some('A'..='Z') | Some('_') => true,
        _ => false,
    }
}

fn is_digit(ch: Option<char>) -> bool {
    match ch {
        Some('0'..='9') => true,
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_tokens {
        ($input:expr, $tokens:expr) => {{
            let mut lexer = Lexer::new($input);
            $tokens.iter().enumerate().for_each(|(i, t)| {
                let tok = lexer.next_token();
                assert_eq!(tok.typ, *t, "Wrong token type at index: {}", i);
            });
        }};
    }

    #[test]
    fn test_next_token_var_int() {
        let input = "my_var := 10";
        let tests = vec![
            TokenType::Ident(String::from("my_var")),
            TokenType::Colon,
            TokenType::Assign,
            TokenType::Int(10),
            TokenType::EOF,
        ];

        assert_tokens!(input, tests);
    }

    #[test]
    fn test_next_token_var_float() {
        let input = "my_var3 := 99.0";
        let tokens = vec![
            TokenType::Ident(String::from("my_var3")),
            TokenType::Colon,
            TokenType::Assign,
            TokenType::Float(99.0),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_string() {
        let input = "my_var := \"hello\"";
        let tokens = vec![
            TokenType::Ident(String::from("my_var")),
            TokenType::Colon,
            TokenType::Assign,
            TokenType::String("hello".to_string()),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_true() {
        let input = "my_var3 := true";
        let tokens = vec![
            TokenType::Ident(String::from("my_var3")),
            TokenType::Colon,
            TokenType::Assign,
            TokenType::Bool(true),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_false() {
        let input = "my_var3 := false";
        let tokens = vec![
            TokenType::Ident(String::from("my_var3")),
            TokenType::Colon,
            TokenType::Assign,
            TokenType::Bool(false),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_type_bool() {
        let input = "my_var : bool = false";
        let tokens = vec![
            TokenType::Ident(String::from("my_var")),
            TokenType::Colon,
            TokenType::Type("bool".to_string()),
            TokenType::Assign,
            TokenType::Bool(false),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_type_string() {
        let input = "my_var : string = \"hello\"";
        let tokens = vec![
            TokenType::Ident(String::from("my_var")),
            TokenType::Colon,
            TokenType::Type("string".to_string()),
            TokenType::Assign,
            TokenType::String("hello".to_string()),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_type_int() {
        let input = "my_var : int = 10";
        let tokens = vec![
            TokenType::Ident(String::from("my_var")),
            TokenType::Colon,
            TokenType::Type("int".to_string()),
            TokenType::Assign,
            TokenType::Int(10),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_var_type_float() {
        let input = "my_var : float = 10.123456";
        let tokens = vec![
            TokenType::Ident(String::from("my_var")),
            TokenType::Colon,
            TokenType::Type("float".to_string()),
            TokenType::Assign,
            TokenType::Float(10.123456),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }
    #[test]
    fn test_next_token_int_operators() {
        let input = "1 + 2 - 3 * 4 / 5 % 6";
        let tokens = vec![
            TokenType::Int(1),
            TokenType::Plus,
            TokenType::Int(2),
            TokenType::Minus,
            TokenType::Int(3),
            TokenType::Asterisk,
            TokenType::Int(4),
            TokenType::Slash,
            TokenType::Int(5),
            TokenType::Module,
            TokenType::Int(6),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_float_operators() {
        let input = "1.12 + 2.23 - 3.34 * 4.45 / 5.56 % 6.67";
        let tokens = vec![
            TokenType::Float(1.12),
            TokenType::Plus,
            TokenType::Float(2.23),
            TokenType::Minus,
            TokenType::Float(3.34),
            TokenType::Asterisk,
            TokenType::Float(4.45),
            TokenType::Slash,
            TokenType::Float(5.56),
            TokenType::Module,
            TokenType::Float(6.67),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_string() {
        let input = r#""hello!" + "bye!""#;
        let tokens = vec![
            TokenType::String("hello!".to_string()),
            TokenType::Plus,
            TokenType::String("bye!".to_string()),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }

    #[test]
    fn test_next_token_string_escape() {
        let input = r#""escape this \" please""#;
        let tokens = vec![
            TokenType::String(r#"escape this \" please"#.to_string()),
            TokenType::EOF,
        ];

        assert_tokens!(input, tokens);
    }
}
