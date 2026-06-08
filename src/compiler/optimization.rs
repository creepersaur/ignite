use crate::{
    compiler::compiler::Compiler,
    hash_u64,
    virtual_machine::{inst::Inst, types::function::TFunction, value::Value},
};

impl Compiler {
    pub fn optimize(&mut self) {
        self.replace_tostring();
        self.remove_load_pops();

        self.compress_multiple_pushes();
        self.compress_multiple_load_consts();

        self.replace(Inst::PUSH(Value::NIL), Inst::PUSH_NIL);
        self.replace(Inst::PUSH(Value::Bool(true)), Inst::PUSH_TRUE);
        self.replace(Inst::PUSH(Value::Bool(false)), Inst::PUSH_FALSE);
        self.replace(Inst::PUSH(Value::Number(0.0)), Inst::PUSH_0);
        self.replace(Inst::PUSH(Value::Number(1.0)), Inst::PUSH_1);

        // Push type
        self.replace_with(|_, x| {
            if let Inst::PUSH(Value::Type(t)) = x {
                Some(Inst::PUSH_TYPE(*t))
            } else {
                None
            }
        });

        // Jump straight ahead
        self.replace_with(|i, v| {
            if let Inst::JUMP(n) = v
                && *n == i as u32 + 1
            {
                Some(Inst::COMMENT("optimized away jump straight ahead".into()))
            } else {
                None
            }
        });

        // FAST_CALL_VOID
        self.replace_pattern_2_with(|a, b| {
            if let Inst::FAST_CALL(func, args) = a
                && let Inst::TRY_POP = b
            {
                Some(Inst::FAST_CALL_VOID(*func, *args))
            } else {
                None
            }
        });

        // Remove DUP-SET-TRY_POP
        self.replace_pattern_3_with(|a, b, c| {
            if let Inst::DUP = a {
                if matches!(
                    b,
                    Inst::SET_GLOBAL(_) | Inst::SET_LOCAL { .. } | Inst::SET_UPVALUE { .. }
                ) && matches!(c, Inst::TRY_POP | Inst::POP)
                {
                    return Some(b.clone());
                }
            }

            None
        });
    }

    pub fn finalize_bytecode(&mut self) {
        for inst in self.instructions.iter_mut() {
            if matches!(inst, Inst::COMMENT(_)) {
                *inst = Inst::NOP;
            }
        }

        self.remove_nops();
        self.trim_end_pops();

        self.remove_store_load_pairs();
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

    pub fn replace_with(&mut self, replacer: impl Fn(usize, &Inst) -> Option<Inst>) {
        self.instructions.iter_mut().enumerate().for_each(|(i, v)| {
            if let Some(new) = replacer(i, v) {
                *v = new;
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
            self.instructions[i + 1] = Inst::NOP;
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
            self.instructions[i + 1] = Inst::NOP;
        }
    }

    fn replace_pattern_3_with(&mut self, replacer: impl Fn(&Inst, &Inst, &Inst) -> Option<Inst>) {
        let indices: Vec<_> = self
            .instructions
            .windows(3)
            .enumerate()
            .filter_map(|(i, w)| replacer(&w[0], &w[1], &w[2]).map(|r| (i, r)))
            .collect();

        for (i, replacement) in indices {
            self.instructions[i] = replacement;
            self.instructions[i + 1] = Inst::NOP;
            self.instructions[i + 2] = Inst::NOP;
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
                    *target = old_to_new[*target as usize] as u32;
                }

                // Functions
                Inst::PUSH(Value::Function(f)) => {
                    *f = Box::new(TFunction {
                        entry: old_to_new[f.entry],
                        handler: f.handler,
                        this: f.this.clone(),
                        target: f.target.clone(),
                        upvalues: f.upvalues.clone(),
                    })
                }
                Inst::MAKE_CLOSURE { entry, .. } => *entry = old_to_new[*entry as usize] as u32,

                _ => {}
            }
        }

        self.instructions.retain(|inst| !matches!(inst, Inst::NOP));
    }

    pub fn compress_multiple_pushes(&mut self) {
        const MIN_COMPRESS_LENGTH: usize = 3;

        let mut i = 0;
        let mut start = 0;
        let mut push_length = 0;
        let mut push_value = None;

        while i < self.instructions.len() {
            if let Inst::PUSH(x) = &self.instructions[i] {
                if let Some(y) = &push_value {
                    if x == y {
                        push_length += 1;
                    } else {
                        start = i;
                        push_length = 1;
                        push_value = Some(x.clone());
                        if push_length >= MIN_COMPRESS_LENGTH {
                            self.instructions
                                .insert(i + 1, Inst::DUP_N(push_length as u16 - 1));
                        }
                    }
                } else {
                    start = i;
                    push_length = 1;
                    push_value = Some(x.clone());
                }
            } else if let Some(_) = push_value {
                if push_length >= MIN_COMPRESS_LENGTH {
                    self.instructions.drain(start + 1..start + push_length);
                    self.instructions
                        .insert(start + 1, Inst::DUP_N(push_length as u16 - 1));
                }
                push_value = None;
            }

            i += 1;
        }

        if let Some(_) = push_value
            && push_length >= MIN_COMPRESS_LENGTH
        {
            self.instructions.drain(start + 1..start + push_length);
            self.instructions
                .insert(start + 1, Inst::DUP_N(push_length as u16 - 1));
        }
    }

    pub fn compress_multiple_load_consts(&mut self) {
        const MIN_COMPRESS_LENGTH: usize = 3;

        let mut i = 0;
        let mut start = 0;
        let mut push_length = 0;
        let mut push_value = None;

        while i < self.instructions.len() {
            if let Inst::LOAD_CONST(x) = &self.instructions[i] {
                if let Some(y) = &push_value {
                    if x == y {
                        push_length += 1;
                    } else {
                        start = i;
                        push_length = 1;
                        push_value = Some(x.clone());
                        if push_length >= MIN_COMPRESS_LENGTH {
                            self.instructions
                                .insert(i + 1, Inst::DUP_N(push_length as u16 - 1));
                        }
                    }
                } else {
                    start = i;
                    push_length = 1;
                    push_value = Some(x.clone());
                }
            } else if let Some(_) = push_value {
                if push_length >= MIN_COMPRESS_LENGTH {
                    self.instructions.drain(start + 1..start + push_length);
                    self.instructions
                        .insert(start + 1, Inst::DUP_N(push_length as u16 - 1));
                }
                push_value = None;
            }

            i += 1;
        }

        if let Some(_) = push_value
            && push_length >= MIN_COMPRESS_LENGTH
        {
            self.instructions.drain(start + 1..start + push_length);
            self.instructions
                .insert(start + 1, Inst::DUP_N(push_length as u16 - 1));
        }
    }

    // LOAD followed by POP instantly
    pub fn remove_load_pops(&mut self) {
        let mut i = 0;
        while i < self.instructions.len().saturating_sub(1) {
            match (&self.instructions[i], &self.instructions[i + 1]) {
                (Inst::LOAD(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD".into());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".into());
                    i += 2;
                }
                (Inst::LOAD_CONST(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD_CONST".into());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".into());
                    i += 2;
                }
                (Inst::LOAD_LOCAL { .. }, Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD_LOCAL".into());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".into());
                    i += 2;
                }
                (Inst::LOAD_GLOBAL(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away LOAD_GLOBAL".into());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".into());
                    i += 2;
                }
                (Inst::PUSH(_), Inst::POP | Inst::TRY_POP) => {
                    self.instructions[i] = Inst::COMMENT("optimized away PUSH".into());
                    self.instructions[i + 1] = Inst::COMMENT("optimized away POP".into());
                    i += 2;
                }

                _ => i += 1,
            }
        }
    }

    // STORE/SET followed by LOAD instantly
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

                (Inst::STORE_GLOBAL(id_a), Inst::LOAD_GLOBAL(id_b)) => *id_a == *id_b,

                (Inst::SET_UPVALUE { id: id_a, .. }, Inst::LOAD_UPVALUE { id: id_b, .. }) => {
                    *id_a == *id_b
                }

                (Inst::SET_GLOBAL(id_a), Inst::LOAD_GLOBAL(id_b)) => *id_a == *id_b,

                (Inst::SET_LOCAL { id: id_a, .. }, Inst::LOAD_LOCAL { id: id_b, .. }) => {
                    *id_a == *id_b
                }

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
