use std::collections::HashMap;

type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone)]
pub enum Symbol {
    Var {
        typ: String,
    },
    Fn {
        params: Vec<(String, String)>,
        ret: Option<String>,
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
            params: vec![("e".to_string(), "char".to_string())],
            ret: None,
        },
    );
    glob_scope.insert(
        "putLn".to_string(),
        Symbol::Fn {
            params: Vec::new(),
            ret: None,
        },
    );
    glob_scope.insert(
        "ORD".to_string(),
        Symbol::Fn {
            params: vec![("ch".to_string(), "char".to_string())],
            ret: Some("int".to_string()),
        },
    );
    glob_scope.insert(
        "CHR".to_string(),
        Symbol::Fn {
            params: vec![("i".to_string(), "int".to_string())],
            ret: Some("char".to_string()),
        },
    );
    return glob_scope;
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![create_global_scope()],
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
        println!("");
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
}
