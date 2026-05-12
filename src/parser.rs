/*
 * Note: This code was partially created by LLMs
 */

use crate::Scanner;
use crate::structs::*;

type FnSignature = (Vec<(String, Type)>, Type);

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "slang: {}:{} {}", self.line, self.col, self.message)
    }
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current: SpannedToken,
    errors: Vec<ParseError>,
}

fn is_declaration_start(tok: &Token) -> bool {
    matches!(tok, Token::Var | Token::Fn)
}

impl<'a> Parser<'a> {
    pub fn new(mut scanner: Scanner<'a>) -> Self {
        let current = scanner.next_token();
        let errors: Vec<ParseError> = Vec::new();
        Self {
            scanner,
            current,
            errors,
        }
    }

    fn advance(&mut self) {
        self.current = self.scanner.next_token();
    }

    fn expect(&mut self, expected: Token) {
        if std::mem::discriminant(&self.current.token) == std::mem::discriminant(&expected) {
            self.advance();
        } else {
            self.error(format!(
                "Expected {:?}, got {:?}",
                expected, self.current.token
            ));
            self.advance();
        }
    }

    fn error(&mut self, msg: String) {
        self.errors.push(ParseError {
            message: msg,
            line: self.current.line,
            col: self.current.col,
        });
    }

    fn synchronize_statement(&mut self) {
        while self.current.token != Token::Semicolon
            && self.current.token != Token::RBrace
            && self.current.token != Token::EOF
        {
            self.advance();
        }

        if self.current.token == Token::Semicolon {
            self.advance();
        }
    }

    fn synchronize_declaration(&mut self) {
        while !is_declaration_start(&self.current.token) && self.current.token != Token::EOF {
            self.advance();
        }
    }

    fn expect_ident(&mut self) -> String {
        if let Token::Ident(s) = self.current.token.clone() {
            self.advance();
            s
        } else {
            self.error("Expected identifier".to_string());
            "".to_owned()
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Declaration>, Vec<ParseError>> {
        let mut decls = Vec::new();

        while self.current.token != Token::EOF {
            if let Some(d) = self.parse_declaration() {
                decls.push(d);
            } else {
                self.synchronize_declaration();
            }
        }

        if self.errors.is_empty() {
            Ok(decls)
        } else {
            Err(self.errors.clone())
        }
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        match self.current.token {
            Token::Var => {
                self.advance();
                let name = self.expect_ident();
                self.expect(Token::Colon);
                let ty = self.expect_ident();
                self.expect(Token::Semicolon);
                Some(Declaration::Var(name, ty.into()))
            }

            Token::Fn => {
                self.advance();
                let name = self.expect_ident();
                let (params, ret) = self.parse_parameters().ok()?;

                self.expect(Token::LBrace);

                let mut locals = Vec::new();
                while self.current.token == Token::Var {
                    if let Some(Declaration::Var(n, t)) = self.parse_declaration() {
                        locals.push((n, t));
                    } else {
                        self.synchronize_declaration();
                    }
                }

                let body = self.parse_stat_seq();
                self.expect(Token::RBrace);

                Some(Declaration::Fn {
                    name,
                    params,
                    ret,
                    locals,
                    body,
                })
            }

            _ => {
                self.error("Expected declaration".to_string());
                None
            }
        }
    }

    fn parse_parameters(&mut self) -> Result<FnSignature, ()> {
        let mut params = Vec::new();

        self.expect(Token::LParen);
        if self.current.token != Token::RParen {
            loop {
                let name = self.expect_ident();
                self.expect(Token::Colon);
                let ty = self.expect_ident();
                params.push((name, ty.into()));

                if self.current.token != Token::Comma {
                    break;
                }
                self.advance();
            }
        }
        self.expect(Token::RParen);

        let ret: Type = if self.current.token == Token::Colon {
            self.advance();
            self.expect_ident().into()
        } else {
            Type::Void
        };

        Ok((params, ret))
    }

    fn parse_stat_seq(&mut self) -> Vec<Statement> {
        let mut stmts = Vec::new();

        while self.current.token != Token::RBrace && self.current.token != Token::EOF {
            if let Some(stmt) = self.parse_statement() {
                stmts.push(stmt);
            } else {
                self.synchronize_statement();
            }
        }

        stmts
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current.token.clone() {
            Token::Ident(name) => {
                self.advance();

                if self.current.token == Token::Assign {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect(Token::Semicolon);
                    Some(Statement::Assign(name, expr))
                } else {
                    let args = self.parse_act_parameters()?;
                    self.expect(Token::Semicolon);
                    Some(Statement::Call(name, args))
                }
            }

            Token::If => {
                self.advance();
                self.expect(Token::LParen);
                let cond = self.parse_condition()?;
                self.expect(Token::RParen);
                self.expect(Token::LBrace);
                let body = self.parse_stat_seq();
                self.expect(Token::RBrace);

                let mut branches = vec![(cond, body)];

                while self.current.token == Token::ElseIf {
                    self.advance();
                    self.expect(Token::LParen);
                    let cond = self.parse_condition()?;
                    self.expect(Token::RParen);
                    self.expect(Token::LBrace);
                    let body = self.parse_stat_seq();
                    self.expect(Token::RBrace);
                    branches.push((cond, body));
                }

                let else_branch = if self.current.token == Token::Else {
                    self.advance();
                    self.expect(Token::LBrace);
                    let body = self.parse_stat_seq();
                    self.expect(Token::RBrace);
                    Some(body)
                } else {
                    None
                };

                Some(Statement::If {
                    branches,
                    else_branch,
                })
            }

            Token::While => {
                self.advance();
                self.expect(Token::LParen);
                let cond = self.parse_condition()?;
                self.expect(Token::RParen);
                self.expect(Token::LBrace);
                let body = self.parse_stat_seq();
                self.expect(Token::RBrace);
                Some(Statement::While { cond, body })
            }

            Token::Return => {
                self.advance();
                let expr = if self.current.token != Token::Semicolon {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Some(Statement::Return(expr))
            }

            _ => {
                self.error("Invalid statement".to_string());
                None
            }
        }
    }

    fn parse_condition(&mut self) -> Option<Condition> {
        let left = self.parse_expression()?;
        let op = self.parse_relop()?;
        let right = self.parse_expression()?;

        Some(Condition { left, op, right })
    }

    fn parse_relop(&mut self) -> Option<RelOp> {
        let op = match self.current.token {
            Token::Assign => RelOp::Eq,
            Token::Neq => RelOp::Neq,
            Token::Lt => RelOp::Lt,
            Token::Gt => RelOp::Gt,
            Token::Le => RelOp::Le,
            Token::Ge => RelOp::Ge,
            _ => {
                self.error("Expected relop".to_string());
                return None;
            }
        };
        self.advance();
        Some(op)
    }

    fn parse_expression(&mut self) -> Option<Expr> {
        let mut expr = self.parse_term()?;

        while matches!(self.current.token, Token::Plus | Token::Minus) {
            let op = match self.current.token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.advance();

            let right = self.parse_term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn parse_term(&mut self) -> Option<Expr> {
        let mut expr = self.parse_factor()?;

        while matches!(
            self.current.token,
            Token::Star | Token::Slash | Token::Percent
        ) {
            let op = match self.current.token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Some(expr)
    }

    fn parse_factor(&mut self) -> Option<Expr> {
        match self.current.token.clone() {
            Token::Ident(name) => {
                self.advance();
                if self.current.token == Token::LParen {
                    let args = self.parse_act_parameters()?;
                    Some(Expr::Call(name, args))
                } else {
                    Some(Expr::Ident(name))
                }
            }

            Token::Number(n) => {
                self.advance();
                Some(Expr::Number(n))
            }

            Token::Char(c) => {
                self.advance();
                Some(Expr::Char(c))
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen);
                Some(expr)
            }

            _ => {
                self.error("Invalid factor".to_string());
                None
            }
        }
    }

    fn parse_act_parameters(&mut self) -> Option<Vec<Expr>> {
        let mut args = Vec::new();
        self.expect(Token::LParen);

        if self.current.token != Token::RParen {
            loop {
                args.push(self.parse_expression()?);
                if self.current.token != Token::Comma {
                    break;
                }
                self.advance();
            }
        }

        self.expect(Token::RParen);
        Some(args)
    }
}
