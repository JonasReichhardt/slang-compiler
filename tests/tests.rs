// LLM-generated test for scanner, parser and semantic analyzer
// Test harness is handwritten

#[cfg(test)]
mod tests {
    pub use slang::parser::*;
    pub use slang::scanner::*;
    pub use slang::sematics::*;

    fn analyze_ok(input: &str) {
        let mut parser = Parser::new(Scanner::new(input));
        let parse_res = parser.parse_program();

        assert!(parse_res.is_ok());

        let mut analyzer = SemanticAnalyzer::new();
        let semantic_res = analyzer.analyze_program(&parse_res.unwrap());

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

    fn parse_ok(input: &str) {
        let scanner = Scanner::new(input);
        let mut parser = Parser::new(scanner);

        match parser.parse_program() {
            Ok(_) => {
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

    fn parse_err(input: &str) {
        let scanner = Scanner::new(input);
        let mut parser = Parser::new(scanner);

        match parser.parse_program() {
            Ok(ast) => {
                println!("{:?}", ast);
                assert!(false);
            }
            Err(_) => {
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

                if (1 = 1) {
                    var y: int;
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
    fn test_valid_deep_shadowing() {
        analyze_ok(
            "
            fn main() {
                var x: int;

                if (1 = 1) {
                    var x: int;
                    x = 2;
                }

                x = 3;
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
    fn test_variable_out_of_scope() {
        analyze_err(
            "
            fn main() {
                if (1 = 1) {
                    var x: int;
                }

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

    // Parse tests - START

    #[test]
    fn test_minimal_var() {
        parse_ok("var x: int;");
    }

    #[test]
    fn test_multiple_vars() {
        parse_ok(
            "
                var x: int;
                var y: int;
                var z: char;
            ",
        );
    }

    #[test]
    fn test_empty_function() {
        parse_ok(
            "
                fn main() {
                }
            ",
        );
    }

    #[test]
    fn test_function_return() {
        parse_ok(
            "
                fn main() {
                    return;
                }
            ",
        );
    }

    #[test]
    fn test_function_with_params() {
        parse_ok(
            "
                fn add(a: int, b: int): int {
                    return a + b;
                }
            ",
        );
    }

    #[test]
    fn test_local_vars() {
        parse_ok(
            "
                fn main() {
                    var x: int;
                    var y: int;
                    x = 5;
                    y = x;
                }
            ",
        );
    }

    #[test]
    fn test_function_call() {
        parse_ok(
            "
                fn main() {
                    print(5);
                }
            ",
        );
    }

    #[test]
    fn test_expression_precedence() {
        parse_ok(
            "
                fn main() {
                    var x: int;
                    x = 1 + 2 * 3 - 4 / 2 % 2;
                }
            ",
        );
    }

    #[test]
    fn test_parentheses() {
        parse_ok(
            "
                fn main() {
                    var x: int;
                    x = (1 + 2) * (3 - 4);
                }
            ",
        );
    }

    #[test]
    fn test_if_statement() {
        parse_ok(
            "
                fn main() {
                    if (1 = 1) {
                        return;
                    }
                }
            ",
        );
    }

    #[test]
    fn test_if_elseif_else() {
        parse_ok(
            "
                fn main() {
                    if (x = 1) {
                        return;
                    } elseif (x # 2) {
                        return;
                    } else {
                        return;
                    }
                }
            ",
        );
    }

    #[test]
    fn test_while_loop() {
        parse_ok(
            "
                fn main() {
                    while (x < 10) {
                        x = x + 1;
                    }
                }
            ",
        );
    }

    #[test]
    fn test_call_with_args() {
        parse_ok(
            "
                fn main() {
                    foo(1, 2, 3);
                }
            ",
        );
    }

    #[test]
    fn test_char_literal() {
        parse_ok(
            "
                var c: char;
                fn main() {
                    c = 'a';
                }
            ",
        );
    }

    #[test]
    fn test_escaped_char() {
        parse_ok(
            "
                fn main() {
                    var c: char;
                    c = '\\n';
                }
            ",
        );
    }

    #[test]
    fn test_comments() {
        parse_ok(
            "
                // line comment
                var x: int;

                /* block comment */
                fn main() {
                    x = 5;
                }
            ",
        );
    }

    #[test]
    fn test_nested_control() {
        parse_ok(
            "
                fn main() {
                    if (1 = 1) {
                        while (2 = 2) {
                            if (3 = 3) {
                                return;
                            }
                        }
                    }
                }
            ",
        );
    }

    #[test]
    fn test_many_params() {
        parse_ok(
            "
                fn test(a:int,b:int,c:int,d:int,e:int,f:int): int {
                    return a;
                }
            ",
        );
    }

    #[test]
    fn test_missing_semicolon() {
        parse_err("var x: int");
    }

    #[test]
    fn test_missing_colon() {
        parse_err("var x int;");
    }

    #[test]
    fn test_invalid_assignment() {
        parse_err(
            "
                fn main() {
                    = 5;
                }
            ",
        );
    }

    #[test]
    fn test_broken_expression() {
        parse_err(
            "
                fn main() {
                    x = 1 + ;
                }
            ",
        );
    }

    #[test]
    fn test_unclosed_block() {
        parse_err(
            "
                fn main() {
                    if (1 = 1) {
                        return;
                }
            ",
        );
    }

    #[test]
    fn test_invalid_function_syntax() {
        parse_err(
            "
                fn main( {
                    return;
                }
            ",
        );
    }

    #[test]
    fn test_invalid_char() {
        parse_err(
            "
                fn main() {
                    var c: char;
                    c = '';
                }
            ",
        );
    }

    // Parse tests - END
}
