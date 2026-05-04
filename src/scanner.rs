/*
 * Note: This code was partially created by LLMs
 */

use crate::{Token, structs::SpannedToken};

pub struct Scanner<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.peek();
        if let Some(c) = ch {
            self.pos += 1;

            if c == b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }

        ch
    }

    fn match_next(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => {
                    self.advance();
                }
                Some(b'/') => {
                    if self.input.get(self.pos + 1) == Some(&b'/') {
                        // line comment
                        while let Some(c) = self.peek() {
                            self.advance();
                            if c == b'\n' {
                                break;
                            }
                        }
                    } else if self.input.get(self.pos + 1) == Some(&b'*') {
                        // block comment
                        self.advance();
                        self.advance();
                        while !(self.peek() == Some(b'*')
                            && self.input.get(self.pos + 1) == Some(&b'/'))
                        {
                            if self.advance().is_none() {
                                break;
                            }
                        }
                        self.advance();
                        self.advance();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> SpannedToken {
        let line = self.line;
        let col = self.col;
        self.skip_whitespace();

        let token: Token = match self.peek() {
            Some(c) if c.is_ascii_digit() => {
                let start = self.pos;
                while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                    self.advance();
                }
                let num = std::str::from_utf8(&self.input[start..self.pos])
                    .unwrap()
                    .parse()
                    .unwrap();
                Token::Number(num)
            }

            Some(c) if c.is_ascii_alphabetic() || c == b'_' || c == b'$' => {
                let start = self.pos;
                while matches!(self.peek(), Some(ch)
                    if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'$')
                {
                    self.advance();
                }

                let s = std::str::from_utf8(&self.input[start..self.pos]).unwrap();

                match s {
                    "var" => Token::Var,
                    "fn" => Token::Fn,
                    "if" => Token::If,
                    "elseif" => Token::ElseIf,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "return" => Token::Return,
                    _ => Token::Ident(s.to_string()),
                }
            }

            Some(b'\'') => {
                self.advance();
                let c = match self.advance() {
                    Some(b'\\') => match self.advance() {
                        Some(b'n') => '\n',
                        Some(b't') => '\t',
                        Some(b'r') => '\r',
                        Some(b'\\') => '\\',
                        Some(b'\'') => '\'',
                        Some(b'"') => '"',
                        _ => '?',
                    },
                    Some(ch) => ch as char,
                    None => '?',
                };
                self.advance(); // closing '
                Token::Char(c)
            }

            Some(b'=') => {
                self.advance();
                Token::Assign
            }

            Some(b'#') => {
                self.advance();
                Token::Neq
            }

            Some(b'<') => {
                self.advance();
                if self.match_next(b'=') {
                    Token::Le
                } else {
                    Token::Lt
                }
            }

            Some(b'>') => {
                self.advance();
                if self.match_next(b'=') {
                    Token::Ge
                } else {
                    Token::Gt
                }
            }

            Some(b'+') => {
                self.advance();
                Token::Plus
            }
            Some(b'-') => {
                self.advance();
                Token::Minus
            }
            Some(b'*') => {
                self.advance();
                Token::Star
            }
            Some(b'/') => {
                self.advance();
                Token::Slash
            }
            Some(b'%') => {
                self.advance();
                Token::Percent
            }

            Some(b'(') => {
                self.advance();
                Token::LParen
            }
            Some(b')') => {
                self.advance();
                Token::RParen
            }
            Some(b'{') => {
                self.advance();
                Token::LBrace
            }
            Some(b'}') => {
                self.advance();
                Token::RBrace
            }

            Some(b':') => {
                self.advance();
                Token::Colon
            }
            Some(b';') => {
                self.advance();
                Token::Semicolon
            }
            Some(b',') => {
                self.advance();
                Token::Comma
            }

            None => Token::EOF,

            _ => {
                println!("slang: unknown token skipped");
                self.advance();
                return self.next_token();
            }
        };
        SpannedToken {
            token,
            line,
            col,
        }
    }
}
