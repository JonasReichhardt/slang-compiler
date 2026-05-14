use crate::{Declaration, Expr, Statement};
use std::fmt;

#[rustfmt::skip]
enum Register { T0,T1,T2,T3,T4,T5,T6,}

impl fmt::Display for Register {
    #[rustfmt::skip]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {

        let name = match self {
            Register::T0 => "t0",Register::T1 => "t1",Register::T2 => "t2",
            Register::T3 => "t3",Register::T4 => "t4",Register::T5 => "t5",
            Register::T6 => "t6",
        };

        write!(f, "{}", name)
    }
}

struct RegisterAllocator {
    free: Vec<Register>,
}

impl RegisterAllocator {
    #[rustfmt::skip]
    pub fn new() -> RegisterAllocator {
        Self {
            free: vec![
                Register::T0,Register::T1,Register::T2,
                Register::T3,Register::T4,Register::T5,Register::T6,
            ],
        }
    }

    pub fn alloc(&mut self) -> Register {
        self.free.pop().expect("RegisterAllocator out of regs")
    }

    pub fn free(&mut self, reg: Register) {
        self.free.push(reg);
    }
}

// emit RISC-V assembler instructions
pub struct Codegen {
    pub code: Vec<String>,
    regs: RegisterAllocator,
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            code: vec![
                format!(".global _start"),
                format!(".text"),
                format!("_start:"),
                format!("call main"),
                format!("li a7,93"),
                format!("ecall"),
            ],
            regs: RegisterAllocator::new(),
        }
    }
    fn emit(&mut self, text: impl Into<String>) {
        self.code.push(text.into());
    }
    pub fn generate_asm(&mut self, ast: &[Declaration]) -> String {
        for decl in ast {
            self.gen_decl(decl);
        }
        let asm = self.code.join("\n");
        println!("ASM:");
        println!("{asm}");
        asm
    }

    fn gen_decl(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Fn {
                name,
                params,
                ret,
                locals,
                body,
            } => {
                self.emit(format!("{name}:"));

                for stmt in body {
                    self.gen_statement(stmt);
                }
            }
            _ => todo!(),
        }
    }

    fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Return(expr) => {
                if let Some(ex) = expr {
                    let reg = self.gen_expression(ex);
                    self.emit(format!("addi a0,{reg},0")); //move to a0
                }
                self.emit("ret");
            }
            _ => todo!(),
        }
    }

    fn gen_expression(&mut self, expr: &Expr) -> Register {
        match expr {
            Expr::Number(num) => {
                let reg = self.regs.alloc();
                self.emit(format!("li {reg},{num}"));
                reg
            }
            _ => todo!(),
        }
    }
}
