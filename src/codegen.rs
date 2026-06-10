use crate::{Declaration, Expr, FuncDecl, Statement, UnaryOp, VarLocation, symtab::SymbolTable};
use std::{collections::HashMap, fmt};

#[rustfmt::skip]
#[derive(Debug, Clone,PartialEq)]
enum Register { T0,T1,T2,T3,T4,T5,T6,A0,A1,A2,A3,A4,A5,A6,A7}

impl fmt::Display for Register {
    #[rustfmt::skip]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {

        let name = match self {
            Register::T0 => "t0",Register::T1 => "t1",Register::T2 => "t2",
            Register::T3 => "t3",Register::T4 => "t4",Register::T5 => "t5",
            Register::T6 => "t6",Register::A0 => "a0",Register::A1 => "a1",
            Register::A2 => "a2",Register::A3 => "a3",Register::A4 => "a4",
            Register::A5 => "a5",Register::A6 => "a6",Register::A7 => "a7",
        };

        write!(f, "{name}")
    }
}

impl Register {
    pub fn get_arg_reglist() -> Vec<Register> {
        vec![
            Register::A7,
            Register::A6,
            Register::A5,
            Register::A4,
            Register::A3,
            Register::A2,
            Register::A1,
            Register::A0,
        ]
    }

    pub fn get_temp_reglist() -> Vec<Register> {
        vec![
            Register::T0,
            Register::T1,
            Register::T2,
            Register::T3,
            Register::T4,
            Register::T5,
            Register::T6,
        ]
    }
}

struct RegisterAllocator {
    free: Vec<Register>,  // Currently free temp regs
    tregs: Vec<Register>, // All temp regs
    args: Vec<Register>,  // All argument regs
}

impl RegisterAllocator {
    #[rustfmt::skip]
    pub fn new() -> RegisterAllocator {
        Self {
            tregs: Register::get_temp_reglist(),
            free: Register::get_temp_reglist(),
            args: Register::get_arg_reglist(),
        }
    }

    pub fn alloc(&mut self) -> Register {
        let reg = self.free.pop().expect("RegisterAllocator out of regs");
        reg
    }

    pub fn free(&mut self, reg: Register) {
        if self.tregs.contains(&reg) {
            self.free.push(reg);
        }
    }

    pub fn get_next_arg_reg(&mut self) -> Register {
        self.args.pop().expect("UNIMPLEMENTED ARGUMENT SPILL")
    }

    pub fn reset_arg_regs(&mut self) {
        self.args = Register::get_arg_reglist();
    }
}

// emit RISC-V assembler instructions
pub struct Codegen {
    code: Vec<String>,
    glob_data: Vec<String>,
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
                self.emit(format!("sd {reg}, {}(sp)", offset));
            }
            VarLocation::Global(label) => {
                let addr = self.regs.alloc();
                self.emit(format!("la {addr},{label}"));
                self.emit(format!("sd {reg},0({addr})"));
            }
        }
        self.regs.free(reg);
    }

    // loads the variable from the given location into a register
    fn load(&mut self, loc: &VarLocation) -> Register {
        let reg = self.regs.alloc();
        match loc {
            VarLocation::Stack(offset) => {
                self.emit(format!("ld {reg},{}(sp)", offset));
            }
            VarLocation::Global(label) => {
                let addr = self.regs.alloc();
                self.emit(format!("la {addr},{label}"));
                self.emit(format!("ld {reg},0({addr})"));
            }
        }
        reg
    }

    fn emit(&mut self, text: impl Into<String>) {
        self.code.push(text.into());
    }

    pub fn generate_asm(&mut self, ast: &[Declaration], sym: &SymbolTable) -> String {
        for decl in ast {
            self.gen_decl(decl);
        }
        self.glob_data.append(&mut self.code);
        self.glob_data.append(&mut builtin_funcs(sym));
        let asm = self.glob_data.join("\n");
        println!("ASM:");
        println!("{asm}");
        asm
    }

    fn gen_func_prologue(&mut self, func: &FuncDecl) {
        let var_num = func.locals.len() + func.params.len();
        let stack_frame_size: i32 = get_stack_frame(var_num.try_into().unwrap());
        self.emit(format!("addi sp,sp,-{}", stack_frame_size));
        self.emit(format!("sd ra,0(sp)")); // preserve ra

        // spill function arguments onto stack
        for param in &func.params {
            let reg = self.regs.get_next_arg_reg();
            match &param.loc {
                Some(loc) => self.store(reg, loc),
                None => unreachable!(),
            };
        }
        self.regs.reset_arg_regs();

        // create space for local variables
        for local in &func.locals {
            match &local.loc {
                Some(loc) => match loc {
                    VarLocation::Stack(offset) => self.emit(format!("sd zero,{}(sp)", offset)),
                    VarLocation::Global(_) => unreachable!(),
                },
                None => unreachable!(),
            };
        }
    }

    fn gen_func_epilogue(&mut self, func: &FuncDecl) {
        let var_num = func.locals.len() + func.params.len();
        let stack_frame_size: i32 = get_stack_frame(var_num.try_into().unwrap());
        self.emit(format!("ld ra,0(sp)")); // restore ra
        self.emit(format!("addi sp,sp,{}", stack_frame_size));
        self.emit("ret");
    }

    fn gen_decl(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Fn(func) => {
                self.emit(format!("{}:", func.name));

                self.gen_func_prologue(func);

                self.emit("  ");

                for stmt in &func.body {
                    self.gen_statement(stmt);
                }

                self.emit("  ");

                self.gen_func_epilogue(&func);
            }
            Declaration::Var(var) => match &var.loc {
                Some(VarLocation::Global(_)) => self.emit_glob_var(&var.name),
                // do nothing as everything regarding local var is done in the function declaration
                Some(VarLocation::Stack(_)) => (),
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
                let reg = self.gen_call(name, args);
                self.regs.free(reg); // throw away result
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
            Expr::Call(name, args) => self.gen_call(name, args),
        }
    }

    fn gen_call(&mut self, name: &str, args: &Vec<Expr>) -> Register {
        // move args into a0-a7 TODO: spill onto stack if more args
        for arg in args {
            let reg = self.gen_expression(arg);
            let arg_reg = self.regs.get_next_arg_reg();
            self.emit(format!("addi {arg_reg},{reg},0"));
            self.regs.free(reg);
        }
        self.regs.reset_arg_regs();

        self.emit(format!("call {name}"));
        // move return value from a0 into temp reg
        let reg = self.regs.alloc();
        self.emit(format!("addi {reg},a0,0"));
        reg
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

fn builtin_funcs(sym: &SymbolTable) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let builtints = HashMap::from([("put".to_string(), put()), ("putLn".to_string(), put_ln())]);
    for func in &sym.builtins_used {
        let mut code = String::new();
        if func == "putLn" {
            //putLn needs put
            code.push_str(&put());
            code.push_str("\n");
        }
        code.push_str(
            builtints
                .get(func)
                .expect("Builtin function does not have and implementation"),
        );
        ret.push(code);
    }
    ret
}

fn put() -> String {
    vec![
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
    ]
    .join("\n")
}

fn put_ln() -> String {
    vec![
        format!("putLn:"),
        format!("addi sp, sp, -16"),
        format!("sd ra, 8(sp)"),
        format!("li a0,10"),
        format!("call put"),
        format!("ld ra, 8(sp)"),
        format!("addi sp, sp, 16"),
        format!("ret"),
    ]
    .join("\n")
}
