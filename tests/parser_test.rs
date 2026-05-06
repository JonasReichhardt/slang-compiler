// LLM-generated test for scanner and parser
// Test harness is handwritten

#[cfg(test)]
mod parser_tests {
    pub use slang::parser::*;
    pub use slang::scanner::*;

    fn parse_ok(input: &str) {
        let scanner = Scanner::new(input);
        let mut parser = Parser::new(scanner);

        match parser.parse_program() {
            Ok(_) => {
                assert!(true);
            }
            Err(errors) => {
                for err in &errors {
                    println!("{}{}:{}", err.line, err.col, err.message);
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
    fn test_invalid_deep_shadowing() {
        parse_err(
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
