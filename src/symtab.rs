pub use crate::structs::*;
use std::collections::HashMap;

type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Var { typ: Type, loc: VarLocation },
    Fn { params: Vec<VarDecl>, ret: Type },
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if matches!(self, Symbol::Fn { .. }) {
            write!(f, "Fn")
        } else {
            write!(f, "Var")
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    builtins: Scope,
    pub builtins_used: Vec<String>,
}

// creates symbols for the builtin functions
// put(e)
// putLN
// ORD(char)
// CHR(int)
// returns the prefilled global scope
fn create_global_scope() -> Scope {
    let mut glob_scope: Scope = HashMap::new();
    glob_scope.insert(
        "put".to_string(),
        Symbol::Fn {
            params: vec![VarDecl {
                name: "e".to_string(),
                typ: Type::Char,
                loc: None,
            }],
            ret: Type::Void,
        },
    );
    glob_scope.insert(
        "putLn".to_string(),
        Symbol::Fn {
            params: Vec::new(),
            ret: Type::Void,
        },
    );
    glob_scope.insert(
        "ORD".to_string(),
        Symbol::Fn {
            params: vec![VarDecl {
                name: "ch".to_string(),
                typ: Type::Char,
                loc: None,
            }],
            ret: Type::Int,
        },
    );
    glob_scope.insert(
        "CHR".to_string(),
        Symbol::Fn {
            params: vec![VarDecl {
                name: "i".to_string(),
                typ: Type::Int,
                loc: None,
            }],
            ret: Type::Char,
        },
    );
    glob_scope
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        let builtins = create_global_scope();
        Self {
            scopes: vec![builtins.clone()],
            builtins,
            builtins_used: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, sym: Symbol) -> bool {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name) {
            return false; // duplicate in same scope
        }

        scope.insert(name, sym);
        true
    }

    pub fn lookup(&mut self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if self.is_builtin(name) && !self.builtins_used.contains(&name.to_string()) {
                self.builtins_used.push(name.to_string());
            }
            if let Some(s) = scope.get(name) {
                return Some(s);
            }
        }
        None
    }

    // checks if a variable is in the global scope
    pub fn is_global(&self, name: &str) -> bool {
        let global_scope = self
            .scopes
            .first()
            .expect("Could not retrieve global scope");
        global_scope.get(name).is_some()
    }

    fn is_builtin(&self, name: &str) -> bool {
        if self.is_global(name) {
            return self.builtins.get(name).is_some();
        }
        return false;
    }
}
