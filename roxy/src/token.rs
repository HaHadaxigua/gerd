use std::any::Any;
use crate::token::TokenType::Eof;

#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Box<dyn Any>>,
    pub line: usize,
}

impl Token {
    pub fn new() -> Self {
        Token {
            token_type: TokenType::LeftParen,
            lexeme: "".to_string(),
            literal: None,
            line: 0,
        }
    }

    pub fn end(line: Option<usize>) -> Self {
        Token {
            token_type: Eof,
            lexeme: "".to_string(),
            literal: None,
            line: match line {
                Some(l) => l,
                None => 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token() {
        println!("{:?}", TokenType::And);
        println!("{:?}", Token::new())
    }
}