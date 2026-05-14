use crate::{Declaration, Expr, Statement};

// emit RISC-V assembler instructions
pub struct Codegen {
    pub code: Vec<String>,
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
                    self.gen_expression(ex);
                }
                self.emit("ret");
            }
            _ => todo!(),
        }
    }

    fn gen_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(num) => self.emit(format!("li a0,{num}")),
            _ => todo!(),
        }
    }
}
