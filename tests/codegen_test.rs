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
        let ast = parse_result.unwrap();

        let mut analyzer = SemanticAnalyzer::new();
        let semantic_res = analyzer.analyze_program(&ast);
        analyzer.pring_warnings();
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
}
