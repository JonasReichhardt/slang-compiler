use crate::{
    Condition, Declaration, Expr, FuncDecl, Statement, UnaryOp, VarDecl, VarLocation,
    symtab::SymbolTable,
};
use std::fmt;

#[rustfmt::skip]
#[derive(Debug, Clone,PartialEq)]
enum Register { T0,T1,T2,T3,T4,T5,T6,A0,A1,A2,A3,A4,A5,A6,A7,Zero}

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
            Register::Zero => "zero",
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
        self.free.pop().expect("RegisterAllocator out of regs")
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
    next_label: usize,
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
                format!("[BUILTINS]"), // placeholder for optional bss segment
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
            next_label: 0,
        }
    }

    fn get_label(&mut self) -> String {
        let label = format!(".L{}", self.next_label);
        self.next_label += 1;
        label
    }

    fn emit_glob_var(&mut self, name: &str, size: &u8) {
        // emit a .data segment
        if self.glob_data.is_empty() {
            self.glob_data.push(".data".to_string());
        }
        let cmd = match size {
            1 => ".byte",
            8 => ".quad",
            _ => unreachable!(),
        };
        self.glob_data.push(format!("{name}: {cmd} 0"));
    }

    // stores the variable value inside the register into the given location
    // frees the register aftwerwards
    fn store(&mut self, reg: Register, loc: &VarLocation) {
        match loc {
            VarLocation::Stack((size, offset)) => {
                let cmd = get_load_cmd(size);
                self.emit(format!("s{cmd} {reg}, {offset}(sp)"));
            }
            VarLocation::Global((size, label)) => {
                let addr = self.regs.alloc();
                let cmd = get_load_cmd(size);
                self.emit(format!("la {addr},{label}"));
                self.emit(format!("s{cmd} {reg},0({addr})"));
            }
        }
        self.regs.free(reg);
    }

    // loads the variable from the given location into a register
    fn load(&mut self, loc: &VarLocation) -> Register {
        let reg = self.regs.alloc();
        match loc {
            VarLocation::Stack((size, offset)) => {
                let cmd = get_load_cmd(size);
                self.emit(format!("l{cmd} {reg},{offset}(sp)"));
            }
            VarLocation::Global((size, label)) => {
                let addr = self.regs.alloc();
                let cmd = get_load_cmd(size);
                self.emit(format!("la {addr},{label}"));
                self.emit(format!("l{cmd} {reg},0({addr})"));
            }
        }
        reg
    }

    fn emit(&mut self, text: impl Into<String>) {
        self.code.push(text.into());
    }

    pub fn generate_asm(&mut self, ast: &[Declaration], sym: &SymbolTable) -> String {
        // Generate code
        for decl in ast {
            self.gen_decl(decl);
        }

        let mut code = self.glob_data.clone();

        let mut builtins = builtin_funcs(sym);
        // if builtins are used add bss segment
        if builtins.is_empty() {
            self.code[1] = "\n".to_string();
        } else {
            self.code[1] = ".section .bss\nput_buf: .space 1".to_string();
        }
        code.append(&mut self.code);
        code.append(&mut builtins);

        code.join("\n")
    }

    fn gen_func_prologue(&mut self, func: &FuncDecl, frame_size: usize) {
        self.emit(format!("addi sp,sp,-{frame_size}"));
        self.emit("sd ra,0(sp)".to_string()); // preserve ra
        self.emit("sd s0,8(sp)".to_string()); // preserve s0
        self.emit("sd s1,16(sp)".to_string()); // preserve s1

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
            match local.loc.clone() {
                Some(loc) => self.store(Register::Zero, &loc),
                None => unreachable!(),
            }
        }
    }

    fn gen_func_epilogue(&mut self, frame_size: usize) {
        self.emit("ld s1,16(sp)".to_string()); // preserve s1
        self.emit("ld s0,8(sp)".to_string()); // preserve s0
        self.emit("ld ra,0(sp)".to_string()); // restore ra
        self.emit(format!("addi sp,sp,{frame_size}"));
        self.emit("ret");
    }

    fn gen_decl(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Fn(func) => {
                self.emit(format!("{}:", func.name));

                let stack_frame_size = get_stack_frame(
                    get_bytes(func.locals.clone()) + get_bytes(func.params.clone()),
                );
                self.gen_func_prologue(func, stack_frame_size);
                self.emit("  ");

                for stmt in &func.body {
                    self.gen_statement(stmt);
                }

                self.emit("  ");

                self.gen_func_epilogue(stack_frame_size);
            }
            Declaration::Var(var) => match &var.loc {
                Some(VarLocation::Global((size, _))) => self.emit_glob_var(&var.name, size),
                _ => unreachable!(),
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
            Statement::If {
                branches,
                else_branch,
            } => {
                let end_label = self.get_label();

                // generate all ifelse branches
                for (cond, body) in branches {
                    let next_label = self.get_label();

                    self.gen_condition(cond, &next_label);

                    for stmt in body {
                        self.gen_statement(stmt);
                    }

                    self.emit(format!("j {end_label}"));
                    self.emit(format!("{next_label}:"));
                }

                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        self.gen_statement(stmt);
                    }
                }

                self.emit(format!("{end_label}:"));
            }
            Statement::While { cond, body } => {
                let start_label = self.get_label();
                let end_label = self.get_label();

                self.emit(format!("{start_label}:"));

                self.gen_condition(cond, &end_label);

                for stmt in body {
                    self.gen_statement(stmt);
                }

                self.emit(format!("j {start_label}"));

                self.emit(format!("{end_label}:"));
            }
        }
    }

    fn gen_condition(&mut self, cond: &Condition, flabel: &str) {
        let left = self.gen_expression(&cond.left);
        let right = self.gen_expression(&cond.right);

        self.emit(format!("{} {},{},{}", cond.op, left, right, flabel));

        self.regs.free(left);
        self.regs.free(right);
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
                    self.load(var_loc)
                } else {
                    unreachable!()
                }
            }
            Expr::Call(name, args) => self.gen_call(name, args),
        }
    }

    fn gen_call(&mut self, name: &str, args: &Vec<Expr>) -> Register {
        self.emit(format!("# {name}()"));

        if name == "ORD" || name == "CHR" {
            return self.gen_expression(args.first().unwrap());
        }

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
        self.emit(format!("addi {reg},a0,0\n"));
        reg
    }
}

// calculates the stack frame size needed for the given number of bytes
fn get_stack_frame(bytes: usize) -> usize {
    let frame_size: usize = bytes + (3 * 8); // reserve three extra space for ra, s0 and s1
    frame_size.div_ceil(16) * 16
}

// calculates how many bytes are needed by the given VarDecls
fn get_bytes(vars: Vec<VarDecl>) -> usize {
    let mut bytes: usize = 0;
    for mut var in vars {
        bytes += var.typ.get_size() as usize;
    }
    bytes
}

fn get_load_cmd(size: &u8) -> char {
    match size {
        1 => 'b',
        8 => 'd',
        _ => unreachable!(),
    }
}

// Checks if a builtin function is used and generates the asm accordingly
fn builtin_funcs(sym: &SymbolTable) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    ret.push("\n# BUILTIN FUNCTIONS".to_string());
    let mut put_gen = false;
    for func in &sym.builtins_used {
        // skip intrinsic functions
        if func == "CHR" || func == "ORD" {
            continue;
        }

        let mut code = String::new();

        if func == "putLn" {
            //putLn needs put
            if !put_gen {
                code.push_str(&put());
                code.push('\n');
                put_gen = true;
            }
            code.push_str(&put_ln());
            code.push('\n');
        }
        if func == "put" && !put_gen {
            code.push_str(&put());
            code.push('\n');
            put_gen = true;
        }
        ret.push(code);
    }
    ret
}

fn put() -> String {
    [
        format!("put:"),
        format!(".option push"),
        format!(".option norelax"),
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
        format!(".option pop"),
        format!("ret"),
    ]
    .join("\n")
}

fn put_ln() -> String {
    [
        "putLn:".to_string(),
        "addi sp, sp, -16".to_string(),
        "sd ra, 8(sp)".to_string(),
        "li a0,10".to_string(),
        "call put".to_string(),
        "ld ra, 8(sp)".to_string(),
        "addi sp, sp, 16".to_string(),
        "ret".to_string(),
    ]
    .join("\n")
}
