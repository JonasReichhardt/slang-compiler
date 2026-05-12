use crate::{structs::*, symtab::*};


#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
}

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<SemanticError>,
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
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

        if self.errors.is_empty() {
            Ok(self.symbols.clone())
        } else {
            Err(self.errors.clone())
        }
    }

    fn analyze_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Var(name, ty) => {
                if !self.symbols.insert(name.clone(), Symbol::Var { typ: *ty }) {
                    self.error(format!("Duplicate variable {name}"));
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
                        ret: *ret,
                    },
                ) {
                    self.error(format!("Redefined function {name}"));
                }

                // new scope for function body
                self.symbols.enter_scope();

                // parameters
                for (pname, ptype) in params {
                    if !self
                        .symbols
                        .insert(pname.clone(), Symbol::Var { typ: *ptype })
                    {
                        self.error(format!("Redefined parameter {name}"));
                    }
                }

                // local variables
                for (lname, ltype) in locals {
                    if !self
                        .symbols
                        .insert(lname.clone(), Symbol::Var { typ: *ltype })
                    {
                        self.error(format!("Redefined local variable {name}"));
                    }
                }

                // analyze body
                match self.analyze_stat_seq(body) {
                    Some(ret_type) => {
                        if ret_type != Type::Error && ret_type != ret.to_owned() {
                            self.error(format!(
                                "Function {} needs to return {:?} not {:?}",
                                name,
                                ret.to_owned(),
                                ret_type
                            ));
                        }
                    }
                    None => {
                        if ret.to_owned() != Type::Void {
                            self.error(format!("Function {} does not have a return value", name));
                        }
                    }
                }

                self.symbols.exit_scope();
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Option<Type> {
        match stmt {
            Statement::Assign(name, expr) => match self.symbols.lookup(name) {
                Some(sym) => match sym {
                    Symbol::Var { typ } => {
                        let ltype = *typ;
                        let rtype = self.analyze_expr(expr);
                        if ltype != Type::Error && rtype != Type::Error && ltype != rtype {
                            self.error(format!("Cannot assign {rtype:?} to {ltype:?}"));
                        }
                        None
                    }
                    Symbol::Fn { params: _, ret: _ } => {
                        self.error(format!("Cannot assign to function {name}"));
                        None
                    }
                },
                None => {
                    self.error(format!("Undefined variable {name}"));
                    None
                }
            },

            Statement::Call(name, args) => Some(self.analyze_func_call(name, args)),

            Statement::While { cond, body } => {
                self.analyze_expr(cond);

                self.symbols.enter_scope();
                for s in body {
                    self.analyze_statement(s);
                }
                self.symbols.exit_scope();

                None
            }

            Statement::Return(ret_val) => match ret_val {
                Some(ret) => return Some(self.analyze_expr(ret)),
                None => Some(Type::Void),
            },

            Statement::If {
                branches,
                else_branch,
            } => {
                let mut ret = None;
                for (_, stmts) in branches {
                    let ret_val = self.analyze_stat_seq(stmts);
                    ret = self.merge_return_types(ret, ret_val);
                }
                if let Some(else_stmts) = else_branch {
                    let ret_val = self.analyze_stat_seq(else_stmts);
                    ret = self.merge_return_types(ret, ret_val);
                }
                ret
            }
        }
    }

    fn merge_return_types(&mut self, a: Option<Type>, b: Option<Type>) -> Option<Type> {
        match (a, b) {
            (None, t) => t,
            (t, None) => t,

            (Some(Type::Error), _) | (_, Some(Type::Error)) => Some(Type::Error),

            (Some(x), Some(y)) if x == y => Some(x),

            (Some(x), Some(y)) => {
                self.error(format!("Mismatched return types: {:?} vs {:?}", x, y));

                Some(Type::Error)
            }
        }
    }

    // Returns the type if a return statement is inside the sequence
    // else returns None
    fn analyze_stat_seq(&mut self, stmts: &Vec<Statement>) -> Option<Type> {
        for (idx, stmt) in stmts.iter().enumerate() {
            if let Some(return_val) = self.analyze_statement(stmt) {
                if idx != stmts.len() - 1 {
                    self.error(format!("Dead CODE bruada"));
                    return Some(Type::Error);
                } else {
                    return Some(return_val);
                }
            }
        }
        None
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Number(..) => Type::Int,
            Expr::Char(..) => Type::Char,

            Expr::Ident(name) => {
                if let Some(sym) = self.symbols.lookup(name) {
                    match sym {
                        Symbol::Var { typ } => typ.to_owned(),
                        Symbol::Fn { params: _, ret: _ } => Type::Error,
                    }
                } else {
                    self.error(format!("Undefined variable {name}"));
                    Type::Error
                }
            }

            Expr::Call(name, args) => self.analyze_func_call(name, args),

            Expr::Binary { left, right, op } => {
                let ltype = self.analyze_expr(left);
                let rtype = self.analyze_expr(right);
                if ltype != Type::Int || rtype != Type::Int {
                    self.error(format!("Cannot perform {op:?} on {ltype:?} and {rtype:?}"));
                    return Type::Error;
                }
                Type::Int
            }

            Expr::Unary { expr, op } => {
                if self.analyze_expr(expr) != Type::Int {
                    self.error(format!("Cannot perform {op:?} on non-integer types"));
                    Type::Error
                } else {
                    Type::Int
                }
            }
        }
    }

    fn analyze_func_call(&mut self, name: &str, args: &[Expr]) -> Type {
        let (params, ret) = match self.symbols.lookup(name) {
            Some(Symbol::Var { .. }) => {
                self.error(format!("{} is not a function", name));

                return Type::Error;
            }

            Some(Symbol::Fn { params, ret }) => (params.clone(), *ret),

            None => {
                self.error(format!("Undefined function {}", name));

                return Type::Error;
            }
        };

        if params.len() != args.len() {
            self.error(format!(
                "Function {} expects {} arguments, got {}",
                name,
                params.len(),
                args.len()
            ));

            return Type::Error;
        }

        for (arg, (_, expected)) in args.iter().zip(params.iter()) {
            let actual = self.analyze_expr(arg);

            if actual != *expected && actual != Type::Error {
                self.error(format!(
                    "Function {} expected argument type {:?}, got {:?}",
                    name, expected, actual
                ));
            }
        }

        ret
    }
}
