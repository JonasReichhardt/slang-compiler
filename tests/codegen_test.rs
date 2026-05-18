// LLM-generated test for codegen
// Invokes gcc and checks
// if the resulting assembler compiles sucessfully

#[cfg(test)]
mod codegen_tests {
    pub use slang::*;
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_name(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("{prefix}_{nanos}")
    }

    fn compile_and_run(program: &str) -> i32 {
        let mut parser = Parser::new(Scanner::new(program));
        let parse_result = parser.parse_program();

        if let Err(errors) = parse_result {
            for err in &errors {
                println!("{}:{}:{}", err.line, err.col, err.message);
            }
            panic!()
        }
        let mut ast = parse_result.unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let semantic_res = analyzer.analyze_program(&mut ast);
        analyzer.print_warnings();
        if !semantic_res {
            analyzer.print_errors();
            panic!()
        }

        // ---------------------------------------------
        // Code generation
        // ---------------------------------------------

        let mut cg = Codegen::new();
        let asm = cg.generate_asm(&ast);

        let base = unique_name("test");
        let asm_file = format!("/tmp/{base}.s");
        let exe_file = format!("/tmp/{base}");

        fs::write(&asm_file, asm).expect("failed to write asm");

        // ---------------------------------------------
        // Assemble + link
        // ---------------------------------------------

        let gcc = Command::new("riscv64-linux-gnu-gcc")
            .arg("-nostdlib")
            .arg("-static")
            .arg(&asm_file)
            .arg("-o")
            .arg(&exe_file)
            .output()
            .expect("failed to invoke riscv gcc");

        assert!(
            gcc.status.success(),
            "gcc failed:\n{}",
            String::from_utf8_lossy(&gcc.stderr)
        );

        // ---------------------------------------------
        // Run via qemu
        // ---------------------------------------------

        let run = Command::new("qemu-riscv64")
            .arg(&exe_file)
            .output()
            .expect("failed to execute qemu");

        run.status.code().unwrap_or(-1)
    }

    #[test]
    fn test_return_literal() {
        let code = "
            fn main(): int {
                return 42;
            }
        ";
        assert_eq!(compile_and_run(code), 42);
    }

    #[test]
    fn test_return_zero() {
        let code = "
            fn main(): int {
                return 0;
            }
        ";
        assert_eq!(compile_and_run(code), 0);
    }

    #[test]
    fn test_return_expr() {
        let code = "
            fn main(): int {
                return 5+10;
            }
        ";
        assert_eq!(compile_and_run(code), 15);
    }

    #[test]
    fn test_return_glob_var() {
        let code = "
            var x: int;

            fn main(): int {
                x = 42;
                return x;
            }
        ";
        assert_eq!(compile_and_run(code), 42);
    }

    #[test]
    fn test_return_local_var() {
        let code = "
            fn main(): int {
                var x: int;
                x = 42;
                return x;
            }
        ";
        assert_eq!(compile_and_run(code), 42);
    }

    #[test]
    fn test_return_unused_var() {
        let code = "
            fn main(): int {
                var y: int;
                return y;
            }
        ";
        assert_eq!(compile_and_run(code), 0);
    }

    #[test]
    fn test_local_vars() {
        let code = "
            fn main(): int {
                var y0: int;
                var y1: int;
                var y2: int;
                var y3: int;
                var y4: int;
                var y5: int;
                var y6: int;
                var y7: int;
                var y8: int;
                var y9: int;
                y0=1;
                y1=2;
                y2=3;
                y3=4;
                y4=5;
                y5=6;
                y6=7;
                y7=8;
                y8=9;
                y9=10;
                return y0+y1;
            }
        ";
        assert_eq!(compile_and_run(code), 3);
    }

    #[test]
    fn test_addition() {
        let code = "
            fn main(): int {
                return 5+10;
            }
        ";
        assert_eq!(compile_and_run(code), 15);
    }

    #[test]
    fn test_minus() {
        let code = "
            fn main(): int {
                return 10-5;
            }
        ";
        assert_eq!(compile_and_run(code), 5);
    }

    #[test]
    fn test_mul() {
        let code = "
            fn main(): int {
                return 5*10;
            }
        ";
        assert_eq!(compile_and_run(code), 50);
    }

    #[test]
    fn test_div() {
        let code = "
            fn main(): int {
                return 50/10;
            }
        ";
        assert_eq!(compile_and_run(code), 5);
    }

    #[test]
    fn test_mod() {
        let code = "
            fn main(): int {
                return 50%10;
            }
        ";
        assert_eq!(compile_and_run(code), 0);
    }

    #[test]
    fn test_operater_precedence() {
        let code = "
            fn main(): int {
                return 1+10*5;
            }
        ";
        assert_eq!(compile_and_run(code), 51);
    }

    #[test]
    fn test_parentheses() {
        let code = "
            fn main(): int {
                return (1+10)*5;
            }
        ";
        assert_eq!(compile_and_run(code), 55);
    }

    #[test]
    fn test_unary() {
        let code = "
            fn main(): int {
                return -5;
            }
        ";
        assert_eq!(compile_and_run(code), 4);
    }
}
