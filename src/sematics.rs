use crate::symtab::*;

pub struct SemanticError {
    message: String,
}

pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    errors: Vec<SemanticError>,
}
