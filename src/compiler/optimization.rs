use crate::{compiler::compiler::Compiler, virtual_machine::inst::Inst};

impl Compiler {
    pub fn optimize(&mut self) {
        self.remove_load_pops();
    }

    pub fn finalize_bytecode(&mut self) {
        for inst in self.instructions.iter_mut() {
            if matches!(inst, Inst::COMMENT(_)) {
                *inst = Inst::NOP;
            }
        }
    }

    // LOAD followed by POP instantly
    pub fn remove_load_pops(&mut self) {
        let mut i = 0;
        while i < self.instructions.len().saturating_sub(1) {
            match (&self.instructions[i], &self.instructions[i + 1]) {
                (Inst::LOAD(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD".to_string());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".to_string());
                    i += 2;
                }
                (Inst::LOAD_CONST(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD_CONST".to_string());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".to_string());
                    i += 2;
                }
                (Inst::LOAD_LOCAL { .. }, Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD_LOCAL".to_string());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".to_string());
                    i += 2;
                }
                (Inst::LOAD_GLOBAL(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD_GLOBAL".to_string());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".to_string());
                    i += 2;
                }

                _ => i += 1,
            }
        }
    }
}
