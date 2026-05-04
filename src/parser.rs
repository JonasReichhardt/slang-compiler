/*
 * Note: This code was partially created by LLMs
 */

use crate::Scanner;
use crate::structs::*;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

pub struct ParseResult {
    pub ast: Vec<Declaration>,
    pub errors: Vec<ParseError>,
}

pub struct Parser<'a> {
    scanner: Scanner<'a>,
    current: SpannedToken,
    errors: Vec<ParseError>,
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

    fn synchronize(&mut self) {
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

    fn expect_ident(&mut self) -> String {
        if let Token::Ident(s) = self.current.token.clone() {
            self.advance();
            s
        } else {
            self.error(format!("Expected identifier"));
            "".to_owned()
        }
    }

    pub fn parse_program(&mut self) -> ParseResult {
        let mut decls = Vec::new();

        while self.current.token != Token::EOF {
            if let Some(d) = self.parse_declaration() {
                decls.push(d);
            } else {
                self.synchronize();
            }
        }

        ParseResult {
            ast: decls,
            errors: self.errors.clone(),
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
                Some(Declaration::Var(name, ty))
            }

            Token::Fn => {
                self.advance();
                let name = self.expect_ident();
                let (params, ret) = self.parse_parameters()?;

                self.expect(Token::LBrace);

                let mut locals = Vec::new();
                while self.current.token == Token::Var {
                    if let Declaration::Var(n, t) = self.parse_declaration()? {
                        locals.push((n, t));
                    }
                }

                let body = self.parse_stat_seq()?;
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
                self.error(format!("Expected declaration"));
                None
            }
        }
    }

    fn parse_parameters(&mut self) -> Result<(Vec<(String, String)>, Option<String>), ()> {
        let mut params = Vec::new();

        self.expect(Token::LParen);
        if self.current.token != Token::RParen {
            loop {
                let name = self.expect_ident();
                self.expect(Token::Colon);
                let ty = self.expect_ident();
                params.push((name, ty));

                if self.current.token != Token::Comma {
                    break;
                }
                self.advance();
            }
        }
        self.expect(Token::RParen);

        let ret = if self.current.token == Token::Colon {
            self.advance();
            Some(self.expect_ident())
        } else {
            None
        };

        Ok((params, ret))
    }

    fn parse_stat_seq(&mut self) -> Result<Vec<Statement>, ()> {
        let mut stmts = Vec::new();
        while self.current.token != Token::RBrace {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, ()> {
        match self.current.token.clone() {
            Token::Ident(name) => {
                self.advance();

                if self.current.token == Token::Assign {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Assign(name, expr))
                } else {
                    let args = self.parse_act_parameters()?;
                    self.expect(Token::Semicolon)?;
                    Ok(Statement::Call(name, args))
                }
            }

            Token::If => {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_condition()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let body = self.parse_stat_seq()?;
                self.expect(Token::RBrace)?;

                let mut branches = vec![(cond, body)];

                while self.current.token == Token::ElseIf {
                    self.advance();
                    self.expect(Token::LParen)?;
                    let cond = self.parse_condition()?;
                    self.expect(Token::RParen)?;
                    self.expect(Token::LBrace)?;
                    let body = self.parse_stat_seq()?;
                    self.expect(Token::RBrace)?;
                    branches.push((cond, body));
                }

                let else_branch = if self.current.token == Token::Else {
                    self.advance();
                    self.expect(Token::LBrace)?;
                    let body = self.parse_stat_seq()?;
                    self.expect(Token::RBrace)?;
                    Some(body)
                } else {
                    None
                };

                Ok(Statement::If {
                    branches,
                    else_branch,
                })
            }

            Token::While => {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_condition()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let body = self.parse_stat_seq()?;
                self.expect(Token::RBrace)?;
                Ok(Statement::While { cond, body })
            }

            Token::Return => {
                self.advance();
                let expr = if self.current.token != Token::Semicolon {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Statement::Return(expr))
            }

            _ => {
                self.error(format!("Invalid statement"));
                Err(())
            }
        }
    }

    fn parse_condition(&mut self) -> Result<Expr, ()> {
        let left = self.parse_expression()?;
        let op = self.parse_relop()?;
        let right = self.parse_expression()?;

        Ok(Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn parse_relop(&mut self) -> Result<BinaryOp, ()> {
        let op = match self.current.token {
            Token::Assign => BinaryOp::Eq,
            Token::Neq => BinaryOp::Neq,
            Token::Lt => BinaryOp::Lt,
            Token::Gt => BinaryOp::Gt,
            Token::Le => BinaryOp::Le,
            Token::Ge => BinaryOp::Ge,
            _ => {
                self.error(format!("Expected relop"));
                return Err(());
            }
        };
        self.advance();
        Ok(op)
    }

    fn parse_expression(&mut self) -> Result<Expr, ()> {
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

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ()> {
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

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ()> {
        match self.current.token.clone() {
            Token::Ident(name) => {
                self.advance();
                if self.current.token == Token::LParen {
                    let args = self.parse_act_parameters()?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Ident(name))
                }
            }

            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }

            Token::Char(c) => {
                self.advance();
                Ok(Expr::Char(c))
            }

            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }

            _ => {
                self.error(format!("Invalid factor"));
                Err(())
            }
        }
    }

    fn parse_act_parameters(&mut self) -> Result<Vec<Expr>, ()> {
        let mut args = Vec::new();
        self.expect(Token::LParen)?;

        if self.current.token != Token::RParen {
            loop {
                args.push(self.parse_expression()?);
                if self.current.token != Token::Comma {
                    break;
                }
                self.advance();
            }
        }

        self.expect(Token::RParen)?;
        Ok(args)
    }
}
