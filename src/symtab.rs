use std::collections::HashMap;

type Scope = HashMap<String, Symbol>;

#[derive(Debug, Clone)]
pub enum Symbol {
    Var {
        name: String,
    },
    Fn {
        params: Vec<(String, String)>,
        ret: Option<String>,
    },
}

pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
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
        scope.insert(name, sym).is_none()
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
