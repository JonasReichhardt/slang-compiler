use crate::{structs::*, symtab::*};

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
}

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    pub fn error(&mut self, msg: String) {
        self.errors.push(SemanticError { message: msg });
    }

    pub fn analyze_program(
        &mut self,
        decls: &[Declaration],
    ) -> Result<SymbolTable, Vec<SemanticError>> {
        for decl in decls {
            self.analyze_declaration(decl);
        }

        if self.errors.len() == 0 {
            Ok(self.symbols.clone())
        } else {
            Err(self.errors.clone())
        }
    }

    fn analyze_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Var(name, ty) => {
                if !self
                    .symbols
                    .insert(name.clone(), Symbol::Var { typ: ty.clone() })
                {
                    self.error(format!("Duplicate variable {}", name));
                }
            }

            Declaration::Fn {
                name,
                params,
                ret,
                body,
                locals,
            } => {
                // register function first (important for recursion)
                if !self.symbols.insert(
                    name.clone(),
                    Symbol::Fn {
                        params: params.clone(),
                        ret: ret.clone(),
                    },
                ) {
                    self.error(format!("Redefined function {}", name));
                }

                // new scope for function body
                self.symbols.enter_scope();

                // parameters
                for (pname, ptype) in params {
                    if !self
                        .symbols
                        .insert(pname.clone(), Symbol::Var { typ: ptype.clone() })
                    {
                        self.error(format!("Redefined parameter {}", name));
                    }
                }

                // local variables
                for (lname, ltype) in locals {
                    if !self
                        .symbols
                        .insert(lname.clone(), Symbol::Var { typ: ltype.clone() })
                    {
                        self.error(format!("Redefined local variable {}", name));
                    }
                }

                // analyze body
                for stmt in body {
                    self.analyze_statement(stmt);
                }

                self.symbols.exit_scope();
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assign(name, expr) => {
                match self.symbols.lookup(name) {
                    Some(sym) => {
                        if matches!(sym, Symbol::Fn { .. }) {
                            self.error(format!("Cannot assign to function {}", name))
                        }
                    }
                    None => self.error(format!("Undefined variable {}", name)),
                }

                self.analyze_expr(expr);
            }

            Statement::Call(name, args) => {
                match self.symbols.lookup(name) {
                    Some(Symbol::Fn { params, .. }) => {
                        if params.len() != args.len() {
                            self.error(format!("Wrong number of arguments in call to {}", name));
                        }
                    }
                    _ => {
                        self.error(format!("Undefined function {}", name));
                    }
                }

                for arg in args {
                    self.analyze_expr(arg);
                }
            }

            Statement::While { cond, body } => {
                self.analyze_expr(cond);

                self.symbols.enter_scope();
                for s in body {
                    self.analyze_statement(s);
                }
                self.symbols.exit_scope();
            }

            _ => {}
        }
    }

    fn analyze_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name) => {
                if self.symbols.lookup(name).is_none() {
                    self.error(format!("Undefined variable {}", name));
                }
            }

            Expr::Call(name, args) => {
                if self.symbols.lookup(name).is_none() {
                    self.error(format!("Undefined function {}", name));
                }

                for a in args {
                    self.analyze_expr(a);
                }
            }

            Expr::Binary { left, right, .. } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }

            Expr::Unary { expr, .. } => {
                self.analyze_expr(expr);
            }

            _ => {}
        }
    }
}
