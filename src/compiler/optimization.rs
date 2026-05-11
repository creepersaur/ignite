use crate::{
    compiler::compiler::Compiler,
    hash_u64,
    virtual_machine::{inst::Inst, value::Value},
};

impl Compiler {
    pub fn optimize(&mut self) {
        self.replace_tostring();
        self.remove_load_pops();
        self.remove_store_load_pairs();
    }

    pub fn finalize_bytecode(&mut self) {
        for inst in self.instructions.iter_mut() {
            if matches!(inst, Inst::COMMENT(_)) {
                *inst = Inst::NOP;
            }
        }

        self.remove_nops();

        self.trim_end_pops();
    }

    pub fn trim_end_pops(&mut self) {
        while let Some(last) = self.instructions.last() {
            if matches!(
                last,
                Inst::NOP | Inst::TRY_POP | Inst::POP | Inst::DEFAULT_NIL
            ) {
                self.instructions.pop();
            } else {
                break;
            }
        }
    }

    pub fn replace_tostring(&mut self) {
        self.replace_pattern_2(
            Inst::LOAD_GLOBAL(hash_u64!("string")),
            Inst::CALL(1),
            Inst::TO_STRING,
        );

        self.replace(Inst::CONCAT_STR(1), Inst::TO_STRING);

        self.replace_pattern_2_with(|a, b| {
            if *b == Inst::TO_STRING {
                if let Inst::PUSH(value) = a {
                    match value {
                        Value::NIL => Some(Inst::PUSH(Value::string("nil"))),
                        Value::Bool(x) => Some(Inst::PUSH(Value::string(x))),
                        Value::Number(x) => Some(Inst::PUSH(Value::string(x))),
                        Value::Char(x) => Some(Inst::PUSH(Value::string(x))),
                        Value::String(x) => Some(Inst::PUSH(Value::String(x.clone()))),

                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    pub fn replace(&mut self, old: Inst, new_value: Inst) {
        self.instructions.iter_mut().for_each(|x| {
            if *x == old {
                *x = new_value.clone()
            }
        });
    }

    fn replace_pattern_2(&mut self, a: Inst, b: Inst, replacement: Inst) {
        let indices: Vec<usize> = self
            .instructions
            .windows(2)
            .enumerate()
            .filter(|(_, w)| w[0] == a && w[1] == b)
            .map(|(i, _)| i)
            .collect();

        for i in indices {
            self.instructions[i] = replacement.clone();
            self.instructions.remove(i + 1);
        }
    }

    fn replace_pattern_2_with(&mut self, replacer: impl Fn(&Inst, &Inst) -> Option<Inst>) {
        let indices: Vec<_> = self
            .instructions
            .windows(2)
            .enumerate()
            .filter_map(|(i, w)| replacer(&w[0], &w[1]).map(|r| (i, r)))
            .collect();

        for (i, replacement) in indices {
            self.instructions[i] = replacement;
            self.instructions.remove(i + 1);
        }
    }

    pub fn remove_nops(&mut self) {
        let mut old_to_new: Vec<usize> = Vec::with_capacity(self.instructions.len());
        let mut new_idx = 0;
        for inst in &self.instructions {
            old_to_new.push(new_idx);
            if !matches!(inst, Inst::NOP | Inst::COMMENT(..)) {
                new_idx += 1;
            }
        }

		old_to_new.push(new_idx);

        for inst in &mut self.instructions {
            match inst {
                Inst::JUMP(target)
                | Inst::JUMP_IF_FALSE(target)
                | Inst::JUMP_IF_TRUE(target)
                | Inst::JUMP_IF_NOT_NIL(target)
                | Inst::FOR_ITER(target) => {
                    *target = old_to_new[*target];
                }

                // Functions
                Inst::PUSH(Value::Function(f)) => f.entry = old_to_new[f.entry],
                Inst::MAKE_CLOSURE { entry, .. } => *entry = old_to_new[*entry],

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

                _ => i += 1,
            }
        }
    }

    // STORE followed by LOAD instantly
    pub fn remove_store_load_pairs(&mut self) {
        let mut i = 0;
        while i < self.instructions.len().saturating_sub(1) {
            let is_redundant = match (&self.instructions[i], &self.instructions[i + 1]) {
                (
                    Inst::STORE_LOCAL {
                        id: id_a,
                        depth: depth_a,
                    },
                    Inst::LOAD_LOCAL {
                        id: id_b,
                        depth: depth_b,
                    },
                ) => *id_a == *id_b && *depth_a == *depth_b,
                _ => false,
            };

            if is_redundant {
                self.instructions[i + 1] = self.instructions[i].clone(); // shift STORE down
                self.instructions[i] = Inst::DUP;
                i += 2;
            } else {
                i += 1;
            }
        }
    }
}
