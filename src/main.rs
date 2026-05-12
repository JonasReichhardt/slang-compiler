mod parser;
mod scanner;
mod sematics;
mod structs;
mod symtab;

pub use crate::parser::*;
pub use crate::scanner::*;
pub use crate::sematics::*;
pub use crate::structs::*;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path: PathBuf = PathBuf::from(args.get(1).expect("slang: path to file expected"));

    let src_str = fs::read_to_string(&path).expect("slang: could not read file");
    let scanner = Scanner::new(&src_str);
    let mut parser = Parser::new(scanner);
    let ast = parser.parse_program();
    if ast.is_ok() {
        let mut semantics = SemanticAnalyzer::new();
        //dbg!(&ast);
        let res = semantics.analyze_program(&ast.unwrap());
        match res {
            Ok(_) => (),
            Err(errors) => {
                for err in &errors {
                    println!("{}", err.message);
                }
                println!("slang: Compilation failed with {} errors.", errors.len());
            }
        }
    } else {
        let errors = ast.err().unwrap();
        for err in &errors {
            println!("{err}");
        }
        println!("slang: Compilation failed with {} errors.", errors.len());
    }
}
