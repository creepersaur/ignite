use crate::{
    compiler::compiler::Compiler,
    virtual_machine::{inst::Inst, value::Value},
};

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
        
		self.remove_nops();
		
		// remove last POP
        if matches!(self.instructions.last(), Some(Inst::TRY_POP | Inst::POP)) {
            self.instructions.pop();
        }
    }

    pub fn remove_nops(&mut self) {
        let mut old_to_new: Vec<usize> = Vec::with_capacity(self.instructions.len());
        let mut new_idx = 0;
        for inst in &self.instructions {
            old_to_new.push(new_idx);
            if !matches!(inst, Inst::NOP) {
                new_idx += 1;
            }
        }

        for inst in &mut self.instructions {
            match inst {
                Inst::JUMP(target) => *target = old_to_new[*target],
                Inst::JUMP_IF_FALSE(target) => *target = old_to_new[*target],
                Inst::FOR_ITER(target) => *target = old_to_new[*target],

                // Functions
                Inst::PUSH(Value::Function(f)) => f.entry = old_to_new[f.entry],

                _ => {}
            }
        }

        self.instructions.retain(|inst| !matches!(inst, Inst::NOP));
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
                (Inst::PUSH(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away PUSH".to_string());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".to_string());
                    i += 2;
                }
                (Inst::CALL(args), Inst::TRY_POP | Inst::POP) => {
                    self.instructions[i] = Inst::CALL_VOID(*args);
                    self.instructions[i + 1] = Inst::NOP;
                    i += 2;
                }

                _ => i += 1,
            }
        }
    }
}
