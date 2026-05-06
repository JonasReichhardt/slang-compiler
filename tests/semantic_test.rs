// LLM-generated test for semantic analyzer
// Test harness is handwritten

#[cfg(test)]
mod semantic_tests {
    pub use slang::parser::*;
    pub use slang::scanner::*;
    pub use slang::sematics::*;

    fn analyze_ok(input: &str) {
        let mut parser = Parser::new(Scanner::new(input));
        let parse_result = parser.parse_program();

        if let Err(errors) = parse_result {
            for err in &errors {
                println!("{}:{}:{}", err.line, err.col, err.message);
            }
            panic!()
        }

        let mut analyzer = SemanticAnalyzer::new();
        let semantic_res = analyzer.analyze_program(&parse_result.unwrap());

        match semantic_res {
            Ok(sym) => {
                println!("Incorrect symbol table {:?}", sym);
                assert!(true);
            }
            Err(errors) => {
                for err in &errors {
                    println!("{}", err.message);
                }
                assert!(false)
            }
        }
    }

    fn analyze_err(input: &str) {
        let mut parser = Parser::new(Scanner::new(input));
        let parse_result = parser.parse_program();

        if let Err(errors) = parse_result {
            for err in &errors {
                println!("{}", err.message);
            }
            panic!()
        }

        let ast = parse_result.unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let semantic_res = analyzer.analyze_program(&ast);

        match semantic_res {
            Ok(sym) => {
                println!("Incorrect symbol table {:?}", sym);
                assert!(false);
            }
            Err(errors) => {
                for err in &errors {
                    println!("{}", err.message);
                }
                assert!(true)
            }
        }
    }

    // Semantic tests - START

    #[test]
    fn test_valid_simple_assignment() {
        analyze_ok(
            "
            var x: int;

            fn main() {
                x = 5;
            }
        ",
        );
    }

    #[test]
    fn test_valid_function_params() {
        analyze_ok(
            "
            fn add(a: int, b: int): int {
                return a;
            }
        ",
        );
    }

    #[test]
    fn test_valid_shadowing() {
        analyze_ok(
            "
            var x: int;

            fn main() {
                var x: int;
                x = 10;
            }
        ",
        );
    }

    #[test]
    fn test_valid_nested_scope() {
        analyze_ok(
            "
            fn main() {
                var x: int;
                var y: int;

                if (1 = 1) {
                    y = 5;
                }

                x = 3;
            }
        ",
        );
    }

    #[test]
    fn test_valid_function_call() {
        analyze_ok(
            "
            fn foo(a: int, b: int) {
                return;
            }

            fn main() {
                foo(1, 2);
            }
        ",
        );
    }

    #[test]
    fn test_valid_recursive_function() {
        analyze_ok(
            "
            fn fact(n: int): int {
                if (n = 0) {
                    return 1;
                } else {
                    return n;
                }
            }
        ",
        );
    }

    #[test]
    fn test_valid_while() {
        analyze_ok(
            "
            fn main() {
                var x: int;
                x = 0;

                while (x < 10) {
                    x = x + 1;
                }
            }
        ",
        );
    }

    #[test]
    fn test_undefined_variable() {
        analyze_err(
            "
            fn main() {
                x = 5;
            }
        ",
        );
    }

    #[test]
    fn test_duplicate_variable() {
        analyze_err(
            "
            fn main() {
                var x: int;
                var x: int;
            }
        ",
        );
    }

    #[test]
    fn test_duplicate_function() {
        analyze_err(
            "
            fn foo() {}
            fn foo() {}
        ",
        );
    }

    #[test]
    fn test_undefined_function_call() {
        analyze_err(
            "
            fn main() {
                foo(1);
            }
        ",
        );
    }

    #[test]
    fn test_wrong_argument_count() {
        analyze_err(
            "
            fn foo(a: int, b: int) {}

            fn main() {
                foo(1);
            }
        ",
        );
    }

    #[test]
    fn test_variable_not_global() {
        analyze_err(
            "
            fn foo() {
                var x: int;
            }

            fn main() {
                x = 5;
            }
        ",
        );
    }

    #[test]
    fn test_duplicate_parameter() {
        analyze_err(
            "
            fn foo(a: int, a: int) {}
        ",
        );
    }

    #[test]
    fn test_param_local_conflict() {
        analyze_err(
            "
            fn foo(a: int) {
                var a: int;
            }
        ",
        );
    }

    #[test]
    fn test_assign_to_function() {
        analyze_err(
            "
            fn foo() {}

            fn main() {
                foo = 5;
            }
        ",
        );
    }

    #[test]
    fn test_call_variable_as_function() {
        analyze_err(
            "
            var x: int;

            fn main() {
                x(1);
            }
        ",
        );
    }

    #[test]
    fn test_multiple_errors() {
        analyze_err(
            "
            fn main() {
                x = 5;
                y = 6;
                foo(1);
            }
        ",
        );
    }

    // Semantic tests - END
}
