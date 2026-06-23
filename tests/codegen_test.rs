// LLM-generated test for codegen
// Invokes gcc and checks
// if the resulting assembler compiles sucessfully

#[cfg(test)]
mod codegen_tests {
    pub use slang::*;
    use std::fs;
    use std::process::{Command, Output};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_name(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("{prefix}_{nanos}")
    }

    fn check_stdout(program: &str) -> String {
        let res = compile_and_run(program);
        let ret = String::from_utf8_lossy(&res.stdout);
        let err = String::from_utf8_lossy(&res.stderr);
        println!("STDOUT: {ret}");
        println!("STDERR: {err}");
        dbg!(res.status);
        ret.into_owned()
    }

    fn check_ret(program: &str) -> i32 {
        let res = compile_and_run(program);
        res.status.code().unwrap_or(-1)
    }

    fn compile_and_run(program: &str) -> Output {
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
        let asm = cg.generate_asm(&ast, &analyzer.symbols);

        link_run(asm)
    }

    fn link_run(asm: String) -> Output {
        let base = unique_name("test");
        let asm_file = format!("tmp/{base}.s");
        let exe_file = format!("tmp/{base}");

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

        Command::new("qemu-riscv64")
            .arg("-strace")
            .arg(&exe_file)
            .output()
            .expect("failed to execute qemu")
    }

    #[test]
    fn test_setup() {
        let code = ".global _start

        .section .bss
        put_buffer:
            .space 1

        .section .text

        _start:
            li a0, 65
            call put

            li a7, 93
            li a0, 0
            ecall

        put:
            la t0, put_buffer
            sb a0, 0(t0)

            li a0, 1
            la a1, put_buffer
            li a2, 1
            li a7, 64
            ecall

            ret"
        .to_string();
        assert_eq!(link_run(code).status.code().unwrap_or(-1), 0);
    }

    #[test]
    fn test_return_literal() {
        let code = "
            fn main(): int {
                return 42;
            }
        ";
        assert_eq!(check_ret(code), 42);
    }

    #[test]
    fn test_return_zero() {
        let code = "
            fn main(): int {
                return 0;
            }
        ";
        assert_eq!(check_ret(code), 0);
    }

    #[test]
    fn test_return_expr() {
        let code = "
            fn main(): int {
                return 5+10;
            }
        ";
        assert_eq!(check_ret(code), 15);
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
        assert_eq!(check_ret(code), 42);
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
        assert_eq!(check_ret(code), 42);
    }

    #[test]
    fn test_return_unused_var() {
        let code = "
            fn main(): int {
                var y: int;
                return y;
            }
        ";
        assert_eq!(check_ret(code), 0);
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
        assert_eq!(check_ret(code), 3);
    }

    #[test]
    fn test_addition() {
        let code = "
            fn main(): int {
                return 5+10;
            }
        ";
        assert_eq!(check_ret(code), 15);
    }

    #[test]
    fn test_minus() {
        let code = "
            fn main(): int {
                return 10-5;
            }
        ";
        assert_eq!(check_ret(code), 5);
    }

    #[test]
    fn test_mul() {
        let code = "
            fn main(): int {
                return 5*10;
            }
        ";
        assert_eq!(check_ret(code), 50);
    }

    #[test]
    fn test_div() {
        let code = "
            fn main(): int {
                return 50/10;
            }
        ";
        assert_eq!(check_ret(code), 5);
    }

    #[test]
    fn test_mod() {
        let code = "
            fn main(): int {
                return 50%10;
            }
        ";
        assert_eq!(check_ret(code), 0);
    }

    #[test]
    fn test_operater_precedence() {
        let code = "
            fn main(): int {
                return 1+10*5;
            }
        ";
        assert_eq!(check_ret(code), 51);
    }

    #[test]
    fn test_parentheses() {
        let code = "
            fn main(): int {
                return (1+10)*5;
            }
        ";
        assert_eq!(check_ret(code), 55);
    }

    #[test]
    fn test_unary() {
        let code = "
            fn main(): int {
                return -5;
            }
        ";
        assert_eq!(check_ret(code), 4);
    }

    #[test]
    fn test_large_expr() {
        let code = "
            fn main(): int {
              return 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8;
            }
        ";
        assert_eq!(check_ret(code), 36);
    }

    #[test]
    fn test_put() {
        let code = "
            fn main(): int {
                put('A');
                return 0;
            }
        ";
        assert_eq!(check_ret(code), 0);
    }

    #[test]
    fn test_put_ln() {
        let code = "
            fn main(): int {
                putLn();
                return 0;
            }
        ";
        assert_eq!(check_ret(code), 0);
    }

    #[test]
    fn test_basic_fn_call() {
        let code = "
            fn add(x: int, y: int): int{
                return x+y;
            }

            fn main(): int {
                return add(10,10);
            }
        ";
        assert_eq!(check_ret(code), 20);
    }

    #[test]
    fn test_multiple_fn_calls() {
        let code = "
            fn add(x: int, y: int): int{
                return x+y;
            }

            fn main(): int {
                var n1: int;
                var n2: int;
                var n3: int;
                var n4: int;
                n1 = 5;
                n2 = 10;
                n3 = add(n1,n2);
                n4 = add(n1,n3);
                n1 = add(n2,n3);
                n2 = add(n3,n4);
                return n1+n2+n3+n4;
            }
        ";
        //15+20+25+35
        assert_eq!(check_ret(code), 95);
    }

    #[test]
    fn test_multiple_fn_expr() {
        let code = "
            fn foo(x: int): int{
                return x;
            }

            fn main(): int {
                return foo(foo(foo(foo(foo(foo(foo(foo(foo(foo(foo(foo(5))))))))))));
            }
        ";
        assert_eq!(check_ret(code), 5);
    }

    #[test]
    fn test_stdout() {
        let code = "
            fn main(): void {
                var c: char;
                c = '1';
                put(c);
                putLn();
            }
        ";
        assert_eq!(check_stdout(code), "1\n");
    }

    #[test]
    fn test_if() {
        let code = "
            fn main(): void {
                var c: char;
                c = '1';
                if(c > '0'){
                    c='2';
                }
                put(c);
                putLn();
            }
        ";
        assert_eq!(check_stdout(code), "2\n");
    }

    #[test]
    fn test_chr() {
        let code = "
            fn main(): void {
                var a: int;
                a = 65;
                while(a<70){
                    put(CHR(a));
                    putLn();
                    a=a+1;
                }
            }
        ";
        assert_eq!(check_stdout(code), "A\nB\nC\nD\nE\n");
    }

    #[test]
    fn test_final() {
        let code = "
            var i: int;

            fn putInt(x: int): void {
                var c0: char;
                var c1: char;
                var c2: char;
                var c3: char;

                c3 = CHR(48 + x % 10); x = x / 10;
                c2 = CHR(48 + x % 10); x = x / 10;
                c1 = CHR(48 + x % 10); x = x / 10;
                c0 = CHR(48 + x % 10);

                if (c0 > '0') { put(c0); put(c1); put(c2); }
                elseif (c1 > '0') { put(c1); put(c2); }
                elseif (c2 > '0') { put(c2); }
                put(c3);
            }

            fn main(): void { /* print odd numbers */
    i = 1;
                while (i < 10) {
                    putInt(i);
                    putLn();
                    i = i + 2;
                }
            }
        ";
        assert_eq!(check_stdout(code), "1\n3\n5\n7\n9\n");
    }
}
