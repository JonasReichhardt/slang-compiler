use crate::{Declaration, Expr, FuncDecl, Statement, UnaryOp, VarLocation};
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

        write!(f, "{name}")
    }
}

struct RegisterAllocator {
    free: Vec<Register>,
    args: Vec<String>,
}

impl RegisterAllocator {
    #[rustfmt::skip]
    pub fn new() -> RegisterAllocator {
        Self {
            free: vec![
                Register::T0,Register::T1,Register::T2,
                Register::T3,Register::T4,Register::T5,Register::T6,
            ],
            args: vec![format!("a0"),format!("a1"),format!("a2"),format!("a3"),
                format!("a4"),format!("a5"),format!("a6"),format!("a7")
            ]
        }
    }

    pub fn alloc(&mut self) -> Register {
        let reg = self.free.pop().expect("RegisterAllocator out of regs");
        reg
    }

    pub fn free(&mut self, reg: Register) {
        self.free.push(reg);
    }
}

// emit RISC-V assembler instructions
pub struct Codegen {
    code: Vec<String>,
    glob_data: Vec<String>,
    builtin_func: Vec<String>,
    regs: RegisterAllocator,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            code: vec![
                format!(".global _start"),
                format!(".section .bss"),
                format!("put_buf: .space 1"),
                format!(".text"),
                format!("_start:"),
                format!("addi sp,sp,-16"),
                format!("sd ra,8(sp)"),
                format!("call main"),
                format!("ld ra,8(sp)"),
                format!("addi sp,sp,16"),
                format!("li a7,93"),
                format!("ecall"),
            ],
            glob_data: Vec::new(),
            builtin_func: builtin_funcs(),
            regs: RegisterAllocator::new(),
        }
    }

    fn emit_glob_var(&mut self, name: &str) {
        // emit a .data segment
        if self.glob_data.is_empty() {
            self.glob_data.push(".data".to_string());
        }
        self.glob_data.push(format!("{name}: .quad 0"));
    }

    // stores the variable value inside the register into the given location
    // frees the register aftwerwards
    fn store(&mut self, reg: Register, loc: &VarLocation) {
        match loc {
            VarLocation::Stack(offset) => {
                self.emit(format!("sd {reg}, {offset}(sp)"));
            }

            VarLocation::Global(label) => {
                let addr = self.regs.alloc();
                self.emit(format!("la {addr},{label}"));
                self.emit(format!("sd {reg},0({addr})"));
                self.regs.free(addr);
            }
        }
        self.regs.free(reg);
    }

    // loads the variable from the given location into a register
    fn load(&mut self, loc: &VarLocation) -> Register {
        let reg = self.regs.alloc();
        match loc {
            VarLocation::Stack(offset) => {
                self.emit(format!("ld {reg},{offset}(sp)"));
            }

            VarLocation::Global(label) => {
                let addr = self.regs.alloc();
                self.emit(format!("la {addr},{label}"));
                self.emit(format!("ld {reg},0({addr})"));
                self.regs.free(addr);
            }
        }
        reg
    }

    fn emit(&mut self, text: impl Into<String>) {
        self.code.push(text.into());
    }

    pub fn generate_asm(&mut self, ast: &[Declaration]) -> String {
        for decl in ast {
            self.gen_decl(decl);
        }
        self.glob_data.append(&mut self.code);
        self.glob_data.append(&mut self.builtin_func);
        let asm = self.glob_data.join("\n");
        println!("ASM:");
        println!("{asm}");
        asm
    }

    fn gen_func_prologue(&mut self, func: &FuncDecl) {
        let stack_frame_size: i32 = get_stack_frame(func.locals.len().try_into().unwrap());
        self.emit(format!("addi sp,sp,{}", -stack_frame_size));
        self.emit(format!("sd ra,0(sp)")); // preserve ra
    }

    fn gen_func_epilogue(&mut self, func: &FuncDecl) {
        let stack_frame_size: i32 = get_stack_frame(func.locals.len().try_into().unwrap());
        self.emit(format!("ld ra,0(sp)")); // restore ra
        self.emit(format!("addi sp,sp,{}", stack_frame_size));
        self.emit("ret");
    }

    fn gen_decl(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Fn(func) => {
                self.emit(format!("{}:", func.name));

                self.gen_func_prologue(func);

                for stmt in &func.body {
                    self.gen_statement(stmt);
                }

                self.gen_func_epilogue(func);
            }
            Declaration::Var(var) => match &var.loc {
                Some(VarLocation::Global(label)) => self.emit_glob_var(label),
                // init local variable with 0
                Some(VarLocation::Stack(offset)) => self.emit(format!("sd zero, {offset}(fp)")),
                None => unreachable!(),
            },
        }
    }

    fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Return(expr) => {
                if let Some(ex) = expr {
                    let reg = self.gen_expression(ex);
                    self.emit(format!("addi a0,{reg},0")); //move to a0
                    self.regs.free(reg);
                }
            }
            Statement::Assign { name: _, loc, expr } => {
                if let Some(var_loc) = loc {
                    let val = self.gen_expression(expr);
                    self.store(val, var_loc);
                } else {
                    unreachable!()
                }
            }
            Statement::Call(name, args) => {
                if args.len() > self.regs.args.len() {
                    unreachable!()
                }
                for (idx, arg) in args.iter().enumerate() {
                    let reg = self.gen_expression(arg);
                    //move from temp reg into arg reg
                    self.emit(format!("addi {},{reg},0", self.regs.args[idx]));
                    self.regs.free(reg);
                }

                self.emit(format!("call {name}"));
            }
            _ => todo!(),
        }
    }

    // returns the register the expr result is stored in
    fn gen_expression(&mut self, expr: &Expr) -> Register {
        match expr {
            Expr::Number(num) => {
                let reg = self.regs.alloc();
                self.emit(format!("li {reg},{num}"));
                reg
            }
            Expr::Char(c) => {
                let reg = self.regs.alloc();
                self.emit(format!("li {reg},{}", *c as u32)); // this should be byte load and store later
                reg
            }
            Expr::Binary { left, op, right } => {
                let rs1 = self.gen_expression(left);
                let rs2 = self.gen_expression(right);
                self.emit(format!("{op} {rs1},{rs1},{rs2}"));
                self.regs.free(rs2);
                rs1
            }
            Expr::Unary { op, expr } => {
                let rs1 = self.gen_expression(expr);
                let imm = match op {
                    UnaryOp::Plus => 1,
                    UnaryOp::Minus => -1,
                };
                self.emit(format!("addi {rs1},{rs1},{imm}"));
                rs1
            }
            Expr::Ident { name: _, loc } => {
                if let Some(var_loc) = loc {
                    let rd = self.load(var_loc);
                    rd
                } else {
                    unreachable!()
                }
            }
            _ => todo!(),
        }
    }
}

// calculates the stack frame size needed for the given number of bytes
fn get_stack_frame(num_vars: usize) -> i32 {
    if num_vars == 0 {
        return 16;
    }
    let frame_size: i32 = (1 + num_vars as i32) * 8; // reserve one extra space for return addr
    ((frame_size + 15) / 16) * 16
}

fn builtin_funcs() -> Vec<String> {
    let ret = vec![
        format!("put:"),
        format!("addi sp,sp,-16"),
        format!("sd ra,8(sp)"),
        format!("la t0, put_buf"),
        format!("sb a0, 0(t0)"),
        format!("li a0, 1"),
        format!("la a1, put_buf"),
        format!("li a2, 1"),
        format!("li a7, 64"),
        format!("ecall"),
        format!("ld ra,8(sp)"),
        format!("addi sp,sp,16"),
        format!("ret"),
        format!("putLn:"),
        format!("addi sp, sp, -16"),
        format!("sd ra, 8(sp)"),
        format!("li a0,10"),
        format!("call put"),
        format!("ld ra, 8(sp)"),
        format!("addi sp, sp, 16"),
        format!("ret"),
    ];
    ret
}
