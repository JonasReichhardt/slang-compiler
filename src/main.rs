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
        dbg!(ast.unwrap());
    } else {
        let errors = ast.err().unwrap();
        for err in &errors {
            println!("{}", err);
        }
        println!("slang: Compilation failed with {} errors.", errors.len());
    }
}
/*
 * Test suite for auto-generated programs located in
 * -> tests/suite/invalid
 * -> test/suite/valid
 */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParseError;

    fn get_test_programs(path: &str) -> Vec<PathBuf> {
        let dir_path = PathBuf::from(path);
        let entries = fs::read_dir(dir_path).expect("slang test: could not read tests dir");
        let mut programs: Vec<PathBuf> = Vec::new();
        for entry in entries {
            let entry = entry.unwrap();
            if entry
                .file_name()
                .to_str()
                .is_some_and(|s| s.to_lowercase().ends_with(".sl"))
            {
                programs.push(entry.path());
            }
        }
        programs
    }

    fn parse_program(path: PathBuf) -> Result<Vec<Declaration>, Vec<ParseError>> {
        let src_str = fs::read_to_string(&path).expect("slang: could not read file");
        let scanner = Scanner::new(&src_str);
        let mut parser = Parser::new(scanner);
        parser.parse_program()
    }

    #[test]
    fn check_valid_programs() {
        let programs = get_test_programs("tests/suite/valid");
        for program in programs {
            let result = parse_program(program);
            assert!(result.is_ok(), "{}", true);
        }
    }

    #[test]
    fn check_invalid_programs() {
        let programs = get_test_programs("tests/suite/invalid");
        for program in programs {
            let result = parse_program(program);
            assert!(result.is_err(), "{}", true);
        }
    }
}
