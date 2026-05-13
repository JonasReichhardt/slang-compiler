use crate::Declaration;

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
            ],
        }
    }
    fn _emit(&mut self, text: impl Into<String>) {
        self.code.push(text.into());
    }
    pub fn generate(&mut self, _ast: &[Declaration]) -> String {
        "
            li a0, 42

            li a7, 93
            ecall"
            .to_owned()
    }
}
