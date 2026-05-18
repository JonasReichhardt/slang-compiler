pub use crate::structs::*;
use std::collections::HashMap;

type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Var {
        typ: Type,
        loc: VarLocation,
    },
    Fn {
        params: Vec<(String, Type)>,
        ret: Type,
    },
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
    builtin: Scope,
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
            params: vec![("e".to_string(), Type::Char)],
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
            params: vec![("ch".to_string(), Type::Char)],
            ret: Type::Int,
        },
    );
    glob_scope.insert(
        "CHR".to_string(),
        Symbol::Fn {
            params: vec![("i".to_string(), Type::Int)],
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
        let builtin = create_global_scope();
        Self {
            scopes: vec![builtin.clone()],
            builtin,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        print!("Closing scope: ");
        for symbol in self.scopes.last().unwrap() {
            print!("[{}|{}],", symbol.0, symbol.1);
        }
        println!();
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

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(s) = scope.get(name) {
                return Some(s);
            }
        }
        None
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.builtin.contains_key(name)
    }

    // checks if a variable is in the global scope
    pub fn is_global(&self, name: &str) -> bool {
        let global_scope = self
            .scopes
            .first()
            .expect("Could not retrieve global scope");
        global_scope.get(name).is_some()
    }
}
