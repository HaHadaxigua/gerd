use crate::token::{Token, TokenType};
use crate::err::RoxyErr::{self, CrossBorder, LoadSubString, UnexpectedCharacter};
use crate::token;
use std::str;


#[derive(Debug)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    // points to the first character in the lexeme being scanned
    start: usize,
    //  points at the character currently being considered
    current: usize,
    // tracks what source line current is on
    line: usize,
}

impl Scanner {
    pub fn new() -> Self {
        Scanner {
            source: "".to_string(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), RoxyErr> {
        // in each turn of the loop, we scan a single token
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::end(Some(self.line)));
        Ok(())
    }

    fn scan_token(&mut self) -> Result<(), RoxyErr> {
        let c: char = self.advance()?;
        match c {
            '(' => {
                self.add_token(TokenType::LeftParen, None)
            }
            ')' => {
                self.add_token(TokenType::RightParen, None)
            }
            '{' => {
                self.add_token(TokenType::LeftBrace, None)
            }
            '}' => {
                self.add_token(TokenType::RightBrace, None)
            }
            ',' => {
                self.add_token(TokenType::Comma, None)
            }
            '.' => {
                self.add_token(TokenType::Dot, None)
            }
            '-' => {
                self.add_token(TokenType::Minus, None)
            }
            '+' => {
                self.add_token(TokenType::Plus, None)
            }
            ';' => {
                self.add_token(TokenType::Semicolon, None)
            }
            '*' => {
                self.add_token(TokenType::Star, None)
            }
            // need to look at the second character
            '!' => {
                let x = self.match_second('=')?;
                self.add_token(if x {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }, None)
            }
            '=' => {
                let x = self.match_second('=')?;
                self.add_token(if x {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }, None)
            }
            '<' => {
                let x = self.match_second('=')?;
                self.add_token(if x {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }, None)
            }
            '>' => {
                let x = self.match_second('=')?;
                self.add_token(if x {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }, None)
            }
            '/' => {
                let x = self.match_second('/')?;
                if !x {
                    return self.add_token(TokenType::Slash, None);
                }

                while self.peek()? != '\n' && !self.is_at_end() {
                    self.advance()?;
                }
                return Ok(());
            }
            _ => {
                Err(UnexpectedCharacter)
            }
        }
    }

    /// consumes the next character in the source file and returns it.
    fn advance(&mut self) -> Result<char, RoxyErr> {
        if self.current > self.source.len() {
            return Err(CrossBorder);
        }
        if let Some(c) = self.source.chars().nth(self.current) {
            self.current += self.current;
            Ok(c)
        } else {
            Err(RoxyErr::CharNotFound)
        }
    }

    /// peek: lookahead, get but not consume the character
    fn peek(&mut self) -> Result<char, RoxyErr> {
        if self.is_at_end() {
            return Ok('\0');
        }
        if let Some(e) = self.source.chars().nth(self.current) {
            Ok(e)
        } else {
            Err(CrossBorder)
        }
    }

    /// grabs the text of the current lexeme and creates a new token for it
    fn add_token(&mut self, tt: TokenType, literal: Option<Box<dyn std::any::Any>>) -> Result<(), RoxyErr> {
        let text = match str::from_utf8(&self.source.as_bytes()[self.start..self.current]) {
            Ok(e) => { e }
            Err(e) => {
                return Err(RoxyErr::Utf8Error(e));
            }
        };
        self.tokens.push(Token {
            token_type: tt,
            lexeme: String::from(text),
            literal,
            line: self.line,
        });
        Ok(())
    }

    /// check if the second is we expected
    fn match_second(&mut self, expected: char) -> Result<bool, RoxyErr> {
        if self.is_at_end() {
            return Ok(false);
        }

        if let Some(next) = self.source.chars().nth(self.current) {
            if next != expected {
                return Ok(false);
            }
        } else {
            return Err(CrossBorder);
        };

        self.current += self.current;
        return Ok(true);
    }


    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_substring() {
        let s = String::from("hello world");
        let ss = &s.as_bytes()[1..3];
        let ss = str::from_utf8(ss).unwrap();
        println!("{}", ss);
        println!("{}", s)
    }
}