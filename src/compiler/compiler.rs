use crate::{
    compiler::native_functions::NativeFunction,
    hash_u64,
    language::{nodes::Node, token::TokenKind},
    patch, patch_execute, rc,
    virtual_machine::{
        inst::Inst::{self, RANGE_EXCLUSIVE, RANGE_INCLUSIVE},
        types::{list::TList, structdef::TStructDef},
        value::Value,
    },
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub struct Compiler {
    pub constants: Vec<Value>,
    pub offset: usize,
    pub instructions: Vec<Inst>,
    pub intern_table: HashMap<u64, Rc<str>>,
    pub scopes: Vec<HashSet<String>>,
    pub scope_base: usize,
    pub current_captures: Vec<usize>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            offset: 0,
            constants: vec![],
            instructions: vec![],
            scopes: vec![HashSet::new()],
            intern_table: HashMap::new(),
            scope_base: 0,
            current_captures: vec![],
        }
    }

    pub fn intern(&mut self, s: &str) -> u64 {
        let id = hash_u64!(s);
        self.intern_table.entry(id).or_insert_with(|| Rc::from(s));
        id
    }

    pub fn comment(&mut self, data: &str) {
        self.instructions.push(Inst::COMMENT(data.into()))
    }

    pub fn push_scope(&mut self) {
        self.instructions.push(Inst::PUSH_SCOPE);
        self.scopes.push(HashSet::new());
    }

    pub fn pop_scope(&mut self) {
        self.instructions.push(Inst::POP_SCOPE);
        self.scopes.pop();
    }

    pub fn emit_store_local(&mut self, name: &str, is_const: bool) {
        let id = self.intern(name);
        let depth = (self.scopes.len() - 1 - self.scope_base) as u16;

        if self.scopes.len() == 1 {
            if is_const {
                self.instructions.push(Inst::STORE_GLOBAL_CONST(id));
            } else {
                self.instructions.push(Inst::STORE_GLOBAL(id));
            }
            self.scopes[0].insert(name.to_string());
            return;
        }

        self.instructions.push(if is_const {
            Inst::STORE_LOCAL_CONST { id, depth }
        } else {
            Inst::STORE_LOCAL { id, depth }
        });

        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name.to_string());
        }
    }

    pub fn emit_load_local(&mut self, name: &str) {
        for (depth, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains(name) {
                let id = self.intern(name);

                if depth == 0 {
                    self.instructions.push(Inst::LOAD_GLOBAL(id));
                } else if depth < self.scope_base {
                    let absolute = depth; // absolute index into locals at runtime
                    if !self.current_captures.contains(&absolute) {
                        self.current_captures.push(absolute);
                    }

                    let scope_idx = self
                        .current_captures
                        .iter()
                        .position(|&d| d == absolute)
                        .unwrap() as u16;
                    self.instructions.push(Inst::LOAD_UPVALUE { id, scope_idx });
                } else {
                    let relative = depth - self.scope_base;
                    self.instructions.push(Inst::LOAD_LOCAL {
                        id,
                        depth: relative as u16,
                    });
                }
                return;
            }
        }

        let id = self.intern(name);
        self.instructions.push(Inst::LOAD_GLOBAL(id));
    }

    pub fn emit_load_const(&mut self, value: Value) {
        if let Some((idx, _)) = self
            .constants
            .iter()
            .enumerate()
            .find(|(_, thing)| thing == &&value)
        {
            self.instructions.push(Inst::LOAD_CONST(idx as u32));
            return;
        }
        self.constants.push(value);
        self.instructions
            .push(Inst::LOAD_CONST(self.constants.len() as u32 - 1));
    }

    fn emit_get_prop(&mut self, member: &Node) {
        match member {
            Node::Symbol(name) => {
                let id = self.intern(name);
                self.instructions.push(Inst::GET_PROP_BY_ID(id));
            }
            _ => {
                self.compile_node(member);
                self.instructions.push(Inst::GET_PROP);
            }
        }
    }

    fn emit_set_prop(&mut self, member: &Node) {
        match member {
            Node::Symbol(name) => {
                let id = self.intern(name);
                self.instructions.push(Inst::SET_PROP_BY_ID(id));
            }
            _ => {
                self.compile_node(member);
                self.instructions.push(Inst::SET_PROP);
            }
        }
    }

    pub fn compile_node(&mut self, node: &Node) {
        match node {
            Node::NIL => self.instructions.push(Inst::PUSH(Value::NIL)),
            Node::Type(t) => self.instructions.push(Inst::PUSH(Value::Type(*t))),
            Node::Variable(x) => self.emit_load_local(x.as_str()),
            Node::NumberLiteral(x) => self.instructions.push(Inst::PUSH(Value::Number(*x))),
            Node::BooleanLiteral(x) => self.instructions.push(Inst::PUSH(Value::Bool(*x))),
            Node::StringLiteral(x) => self.emit_load_const(Value::string(x)),
            Node::FString(values) => self.compile_fstring(values),

            Node::ListNode(values) => self.compile_list(values, false),
            Node::TupleNode(values) => self.compile_list(values, true),
            Node::DictNode(values) => self.compile_dict(values),

            Node::RangeNode {
                start,
                end,
                step,
                inclusive,
            } => self.compile_range(start, end, step, *inclusive),

            Node::ExprStmt(x) => {
                self.compile_node(&*x);
                self.instructions.push(Inst::TRY_POP);
            }

            Node::UnaryOp {
                op,
                right,
                is_prefix,
            } => self.compile_unary_op(op, right, *is_prefix),
            Node::BinOp { left, right, op } => self.compile_bin_op(left, right, op),
            Node::ComparisonChain {
                expressions,
                operators,
            } => self.compile_comparison_chain(expressions, operators),

            Node::NullCoalesce { left, right } => self.compile_null_coalesce(left, right),
            Node::ElvisCoalesce { left, right } => self.compile_elvis_coalesce(left, right),
            Node::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => self.compile_ternary_op(condition, true_expr, false_expr),

            Node::LetStatement {
                names,
                values,
                is_const,
            } => self.compile_let(names, &mut values.clone(), *is_const),

            Node::UsingStatement {
                sequence,
                imports,
                wildcard,
            } => self.compile_using(sequence, imports, *wildcard),

            Node::SetVariable { target, value } => self.compile_set_variable(target, value),

            Node::MemberAccess { expr, member } => self.compile_member_access(expr, member),

            Node::ShorthandAssignment {
                token,
                target,
                value,
            } => self.compile_shorthand_assignment(target, value, token),

            Node::FunctionCall { target, args } => self.compile_function_call(target, args),

            Node::FunctionDefinition {
                name,
                return_type,
                args,
                is_const,
                block,
            } => self.compile_function_def(name, return_type, args, *is_const, block),

            Node::ReturnStatement(value) => self.compile_return(value),

            Node::IfStatement {
                condition,
                block,
                elifs,
                else_block,
            } => self.compile_if_statement(condition, block, elifs, else_block),

            Node::Block { name, body } => self.compile_block(name, body),

            Node::SingleLineBlock { body } => {
                self.push_scope();
                self.compile_node(&**body);
                self.pop_scope();
            }

            Node::Loop { block } => self.compile_loop(block),

            Node::WhileLoop { condition, block } => self.compile_while_loop(condition, block),

            Node::BreakStatement(value) => self.compile_break(value),

            Node::ContinueStatement => self.compile_continue(),

            Node::ForLoop {
                var_name,
                expr,
                block,
            } => self.compile_for(var_name, expr, block),

            Node::MatchStatement { expr, branches } => self.compile_match(expr, branches),

            Node::EnumDef { name, items } => self.compile_enum_def(name, items),

            Node::StructDef { name, fields } => self.compile_struct_def(name, fields),
            Node::StructInit { target, fields } => self.compile_struct_init(target, fields),

            Node::ClassDef {
                name,
                let_statements,
                functions,
                constructor,
                ..
            } => self.compile_class_def(name, let_statements, functions, constructor),
            Node::ClassInit { target, parameters } => self.compile_class_init(target, parameters),

            _ => panic!("Unknown node: `{node:?}`"),
        }
    }
}

impl Compiler {
    pub fn compile_fstring(&mut self, values: &Vec<Node>) {
        for i in values.iter().rev() {
            self.compile_node(i);
        }
        self.instructions
            .push(Inst::CONCAT_STR(values.len() as u16));
    }

    pub fn compile_range(
        &mut self,
        start: &Box<Node>,
        end: &Box<Node>,
        step: &Option<Box<Node>>,
        inclusive: bool,
    ) {
        self.compile_node(&**start);
        self.compile_node(&**end);
        if let Some(step) = step {
            self.compile_node(&**step);
        } else {
            self.instructions.push(Inst::PUSH(Value::Number(1.0)))
        }
        self.instructions.push(match inclusive {
            true => RANGE_INCLUSIVE,
            false => RANGE_EXCLUSIVE,
        });
    }

    pub fn compile_list(&mut self, values: &Vec<Node>, is_tuple: bool) {
        let mut folded_values = Vec::with_capacity(values.len());
        let mut all_literals = true;

        for node in values {
            match node {
                Node::NumberLiteral(n) => folded_values.push(Value::Number(*n)),
                Node::BooleanLiteral(b) => folded_values.push(Value::Bool(*b)),
                Node::StringLiteral(s) => folded_values.push(Value::string(s)),
                Node::NIL => folded_values.push(Value::NIL),

                _ => {
                    all_literals = false;
                    break;
                }
            }
        }

        if all_literals && !values.is_empty() {
            if is_tuple {
                self.instructions
                    .push(Inst::PUSH(Value::Tuple(TList::new(rc!(RefCell::new(
                        folded_values
                    ))))));
            } else {
                self.instructions
                    .push(Inst::PUSH(Value::List(TList::new(rc!(RefCell::new(
                        folded_values
                    ))))));
            }
            return;
        }

        values.iter().rev().for_each(|node| self.compile_node(node));
        self.instructions.push(if is_tuple {
            Inst::TUPLE(values.len() as u16)
        } else {
            Inst::LIST(values.len() as u16)
        });
    }

    pub fn compile_dict(&mut self, values: &Vec<(Node, Node)>) {
        for (k, v) in values.iter().rev() {
            self.compile_node(k);
            self.compile_node(v);
        }
        self.instructions.push(Inst::DICT(values.len() as u16));
    }

    pub fn compile_unary_op(&mut self, op: &TokenKind, target: &Box<Node>, is_prefix: bool) {
        // increment/decrement
        if matches!(op, TokenKind::INCREMENT | TokenKind::DECREMENT) {
            let operator_inst = match op {
                TokenKind::INCREMENT => Inst::ADD,
                TokenKind::DECREMENT => Inst::SUB,

                _ => unreachable!(),
            };

            if let Node::Variable(x) = &**target {
                self.compile_node(&**target);
                if !is_prefix {
                    self.instructions.push(Inst::DUP);
                }
                self.instructions.push(Inst::PUSH(Value::Number(1.0)));
                self.instructions.push(operator_inst);
                if is_prefix {
                    self.instructions.push(Inst::DUP);
                }
                let id = self.intern(x.as_str());
                self.instructions.push(Inst::SET_VAR(id))
            } else if let Node::MemberAccess { expr, member } = &**target {
                self.compile_node(&**expr);
                self.emit_get_prop(member);
                if !is_prefix {
                    self.instructions.push(Inst::DUP);
                }
                self.instructions.push(Inst::PUSH(Value::Number(1.0)));
                self.instructions.push(operator_inst);
                if !is_prefix {
                    self.instructions.push(Inst::DUP);
                }
                self.compile_node(&**expr);
                self.emit_set_prop(member);
            } else {
                panic!("Cannot set equal a value to `{:?}`", **target);
            }

            return;
        }

        // minus/plus/bang
        self.compile_node(target);

        match op {
            TokenKind::MINUS => {
                if let Inst::PUSH(Value::Number(x)) = self.instructions[self.instructions.len() - 1]
                {
                    self.instructions.pop();
                    self.instructions.push(Inst::PUSH(Value::Number(-x)));
                } else {
                    self.instructions.push(Inst::NEG);
                }
            }
            TokenKind::PLUS => self.instructions.push(Inst::POS),
            TokenKind::BANG => self.instructions.push(Inst::NOT),

            _ => panic!("Tried compiling unknown unary op: {op:?}"),
        }
    }

    pub fn compile_bin_op(&mut self, left: &Box<Node>, right: &Box<Node>, op: &TokenKind) {
        self.compile_node(&**left);
        self.compile_node(&**right);

        self.instructions.push(match op {
            TokenKind::PLUS => Inst::ADD,
            TokenKind::MINUS => Inst::SUB,
            TokenKind::STAR => Inst::MUL,
            TokenKind::SLASH => Inst::DIV,
            TokenKind::POW => Inst::POW,
            TokenKind::MOD => Inst::MOD,

            TokenKind::GT => Inst::GT,
            TokenKind::LT => Inst::LT,
            TokenKind::GE => Inst::GE,
            TokenKind::LE => Inst::LE,

            TokenKind::EQ => Inst::EQ,
            TokenKind::NEQ => Inst::NEQ,

            TokenKind::AND => Inst::AND,
            TokenKind::OR => Inst::OR,

            TokenKind::IS => Inst::IS_INSTANCE_OF,

            _ => panic!("Cannot compile unknown bin-op: `{op:?}`"),
        });
    }

    pub fn compile_comparison_chain(
        &mut self,
        expressions: &Vec<Node>,
        operators: &Vec<TokenKind>,
    ) {
        self.compile_node(&expressions[0]);

        let n = operators.len();
        for i in 0..n {
            let is_first = i == 0;
            let is_last = i == n - 1;

            self.compile_node(&expressions[i + 1]);

            // Save a copy of the right operand as carry for the next comparison
            if !is_last {
                self.instructions.push(Inst::DUP);
                self.instructions.push(Inst::ROT3);
            }

            self.instructions.push(match operators[i] {
                TokenKind::LT => Inst::LT,
                TokenKind::LE => Inst::LE,
                TokenKind::GT => Inst::GT,
                TokenKind::GE => Inst::GE,
                _ => unreachable!(),
            });

            if !is_last {
                // Re-establish invariant: [accumulated, carry]
                self.instructions.push(Inst::SWAP);
                if !is_first {
                    // Fold in prior accumulated result
                    self.instructions.push(Inst::ROT3);
                    self.instructions.push(Inst::AND);
                    self.instructions.push(Inst::SWAP);
                }
            } else if !is_first {
                self.instructions.push(Inst::AND);
            }
        }
    }

    pub fn compile_null_coalesce(&mut self, left: &Box<Node>, right: &Box<Node>) {
        self.compile_node(&**left);

        self.instructions.push(Inst::DUP);

        let jump_patch = patch!(self.instructions);

        self.instructions.push(Inst::POP);
        self.compile_node(&**right);

        patch_execute!(
            self.instructions,
            jump_patch,
            Inst::JUMP_IF_NOT_NIL(self.instructions.len() as u32)
        );
    }

    pub fn compile_elvis_coalesce(&mut self, left: &Box<Node>, right: &Box<Node>) {
        self.compile_node(&**left);

        self.instructions.push(Inst::DUP);

        let jump_patch = patch!(self.instructions);

        self.instructions.push(Inst::POP);
        self.compile_node(&**right);

        patch_execute!(
            self.instructions,
            jump_patch,
            Inst::JUMP_IF_TRUE(self.instructions.len() as u32)
        );
    }

    pub fn compile_ternary_op(
        &mut self,
        condition: &Box<Node>,
        true_expr: &Box<Node>,
        false_expr: &Box<Node>,
    ) {
        self.compile_node(&**condition);

        let jump_if_false = patch!(self.instructions);

        self.compile_node(&**true_expr);

        let jump_end = patch!(self.instructions);

        patch_execute!(
            self.instructions,
            jump_if_false,
            Inst::JUMP_IF_FALSE(self.instructions.len() as u32)
        );

        self.compile_node(&**false_expr);

        patch_execute!(
            self.instructions,
            jump_end,
            Inst::JUMP(self.instructions.len() as u32)
        );
    }

    pub fn compile_let(
        &mut self,
        names: &Vec<Rc<String>>,
        values: &mut Vec<Option<Box<Node>>>,
        is_const: bool,
    ) {
        values.resize(names.len(), Some(Box::new(Node::NIL)));

        for (i, value) in values.iter().enumerate() {
            if let Some(val) = value {
                self.compile_node(&**val)
            } else {
                self.instructions.push(Inst::PUSH(Value::NIL));
            };

            self.emit_store_local(names[i].as_str(), is_const);
        }

        self.emit_load_local(names[0].as_str());
    }

    pub fn compile_block(&mut self, name: &Option<String>, body: &Vec<Node>) {
        let out_text = format!("out:{}", name.clone().unwrap_or("".to_string()));
        let block_start = self.instructions.len();

        self.push_scope();
        for i in body {
            if let Node::ExprStmt(expr) = i {
                if let Node::OutStatement { block_name, value } = expr.as_ref() {
                    if let Some(v) = value {
                        self.compile_node(&*v);
                    } else {
                        self.instructions.push(Inst::PUSH(Value::NIL));
                    }
                    let _ = patch!(
                        self.instructions,
                        format!("out:{}", block_name.clone().unwrap_or("".to_string()))
                    ); // Jump to scope cleanup
                } else {
                    self.compile_node(i);
                }
            } else {
                if let Node::OutStatement { block_name, value } = i {
                    if let Some(v) = value {
                        self.compile_node(&*v);
                    } else {
                        self.instructions.push(Inst::PUSH(Value::NIL));
                    }
                    let _ = patch!(
                        self.instructions,
                        format!("out:{}", block_name.clone().unwrap_or("".to_string()))
                    ); // Jump to scope cleanup
                } else {
                    self.compile_node(i);
                }
            }
        }

        self.instructions.push(Inst::PUSH(Value::NIL));

        let block_end = self.instructions.len() as u32;
        patch_execute!(
            self.instructions,
            out_text.as_str(),
            Inst::JUMP(block_end),
            block_start
        );
        patch_execute!(
            self.instructions,
            "out:",
            Inst::JUMP(block_end),
            block_start
        );

        self.pop_scope();
    }

    pub fn compile_member_access(&mut self, expr: &Box<Node>, member: &Box<Node>) {
        self.compile_node(&**expr);
        self.emit_get_prop(member);
    }

    pub fn compile_function_call(&mut self, target: &Box<Node>, args: &Vec<Node>) {
        for i in args.iter() {
            self.compile_node(i);
        }

        if let Node::Variable(x) = &**target
            && let Some(f) = NativeFunction::is_native(&x)
            && {
                let mut is_native = true;
                for scope in self.scopes.iter() {
                    if scope.contains(x.as_str()) {
                        is_native = false;
                    }
                }
                is_native
            }
        {
            self.instructions
                .push(Inst::FAST_CALL(f, args.len() as u16))
        } else {
            self.compile_node(&**target);
            self.instructions.push(Inst::CALL(args.len() as u16));
        }
    }

    pub fn compile_if_statement(
        &mut self,
        condition: &Box<Node>,
        block: &Box<Node>,
        elifs: &Vec<(Node, Node)>,
        else_block: &Option<Box<Node>>,
    ) {
        let mut if_end_jumps = vec![];

        self.comment("If statement start:");

        let mut handler = |condition: &Node, block: &Node| {
            self.comment("If statement handler start:");

            self.push_scope();

            self.compile_node(&condition);
            let jump_if_false = patch!(self.instructions);

            self.compile_node(&block);

            self.pop_scope();

            if_end_jumps.push(patch!(self.instructions));

            self.comment("If branch end");

            patch_execute!(
                self.instructions,
                jump_if_false,
                Inst::JUMP_IF_FALSE(self.instructions.len() as u32)
            );
        };

        handler(&**condition, &**block);

        if elifs.len() > 0 {
            for (condition, block) in elifs {
                handler(&*condition, &*block);
            }
        }

        if let Some(body) = else_block {
            self.compile_node(&**body);
        } else {
            self.instructions.push(Inst::PUSH(Value::NIL));
        }

        let if_end = self.instructions.len() as u32;
        for x in if_end_jumps {
            patch_execute!(self.instructions, x, Inst::JUMP(if_end));
        }

        self.comment("If statement end:");
    }

    pub fn compile_set_variable(&mut self, target: &Box<Node>, value: &Box<Node>) {
        if let Node::Variable(x) = &**target {
            self.compile_node(&**value);
            self.instructions.push(Inst::DUP);
            self.instructions.push(Inst::SET_VAR(hash_u64!(x.as_str())));
        } else if let Node::MemberAccess { expr, member } = &**target {
            self.compile_node(&**value);
            self.instructions.push(Inst::DUP);
            self.compile_node(&**expr);
            self.emit_set_prop(member);
        } else {
            panic!("Cannot set equal a value to `{:?}`", **target);
        }
    }

    pub fn compile_shorthand_assignment(
        &mut self,
        target: &Box<Node>,
        value: &Box<Node>,
        token: &TokenKind,
    ) {
        let operator_inst = match token {
            TokenKind::ADD_SH => Inst::ADD,
            TokenKind::SUB_SH => Inst::SUB,
            TokenKind::MUL_SH => Inst::MUL,
            TokenKind::DIV_SH => Inst::DIV,
            TokenKind::POW_SH => Inst::POW,
            TokenKind::MOD_SH => Inst::MOD,

            _ => panic!("Unknown shorthand assignment token: {token:?}"),
        };

        if let Node::Variable(x) = &**target {
            self.compile_node(&**target);
            self.compile_node(&**value);
            self.instructions.push(operator_inst);
            self.instructions.push(Inst::SET_VAR(hash_u64!(x.as_str())));
        } else if let Node::MemberAccess { expr, member } = &**target {
            self.compile_node(&**expr);
            self.emit_get_prop(member); // LOAD expr.member
            self.compile_node(&**value); // LOAD 1
            self.instructions.push(operator_inst); // ADD

            self.compile_node(&**expr);
            self.emit_set_prop(member); // SET expr.member
        } else {
            panic!("Cannot set equal a value to `{:?}`", **target);
        }
    }

    pub fn compile_return(&mut self, value: &Option<Box<Node>>) {
        if let Some(val) = value {
            self.compile_node(val);
        } else {
            self.instructions.push(Inst::PUSH(Value::NIL));
        }
        self.instructions.push(Inst::RETURN);
    }

    pub fn compile_function_def(
        &mut self,
        name: &Option<Rc<String>>,
        _return_type: &Option<Rc<String>>,
        args: &Vec<(Rc<String>, Option<Rc<String>>, Option<Node>)>,
        is_const: bool,
        block: &Box<Node>,
    ) {
        let saved_captures = std::mem::take(&mut self.current_captures);

        self.comment(&format!("New function (const: {is_const}):"));
        let func_value = patch!(self.instructions);
        if let Some(name) = name {
            self.emit_store_local(name.as_str(), is_const);
        }
        let func_jump_to_end = patch!(self.instructions);

        let func_start = self.offset + self.instructions.len();

        self.comment("Function def start:");

        let saved_base = self.scope_base; // <-- save
        self.scope_base = self.scopes.len(); // <-- reset: depths start fresh here

        self.push_scope();

        for (arg_name, _, default_value) in args.iter() {
            if let Some(def) = default_value {
                self.compile_node(def);
                self.instructions.push(Inst::DEFAULT);
            } else {
                self.instructions.push(Inst::DEFAULT_NIL);
            }

            self.emit_store_local(arg_name.as_str(), false);
        }

        self.compile_node(block);

        self.instructions.push(Inst::RETURN);
        self.pop_scope();

        self.scope_base = saved_base; // <-- restore

        self.comment("Function def end");

        let captures = std::mem::take(&mut self.current_captures)
            .iter()
            .map(|x| *x as u32)
            .collect();
        self.current_captures = saved_captures;

        patch_execute!(
            self.instructions,
            func_value,
            Inst::MAKE_CLOSURE {
                entry: func_start as u32,
                captures
            }
        );

        patch_execute!(
            self.instructions,
            func_jump_to_end,
            Inst::JUMP(self.instructions.len() as u32)
        );
    }

    pub fn compile_while_loop(&mut self, condition: &Box<Node>, block: &Box<Node>) {
        let loop_start_index = self.instructions.len();

        self.compile_node(&*condition);

        let end_loop_jump = patch!(self.instructions);

        self.compile_node(&*block);

        self.instructions.push(Inst::JUMP(loop_start_index as u32));

        patch_execute!(
            self.instructions,
            end_loop_jump,
            Inst::JUMP_IF_FALSE(self.instructions.len() as u32)
        );

        patch_execute!(
            self.instructions,
            "break",
            Inst::JUMP(self.instructions.len() as u32),
            loop_start_index
        );

        patch_execute!(
            self.instructions,
            "continue",
            Inst::JUMP(loop_start_index as u32),
            loop_start_index
        );

        self.instructions.push(Inst::DEFAULT_NIL);
    }

    pub fn compile_for(&mut self, var_name: &Rc<String>, expr: &Box<Node>, block: &Box<Node>) {
        self.comment("For loop start:");

        self.push_scope();

        self.compile_node(&*expr);
        self.instructions.push(Inst::GET_ITER);

        let loop_start_index = self.instructions.len();

        let for_iter = patch!(self.instructions);
        self.emit_store_local(var_name.as_str(), false);

        self.compile_node(&*block);

        self.instructions.push(Inst::JUMP(loop_start_index as u32));

        patch_execute!(
            self.instructions,
            for_iter,
            Inst::FOR_ITER(self.instructions.len() as u32)
        );

        patch_execute!(
            self.instructions,
            "break",
            Inst::JUMP(self.instructions.len() as u32),
            loop_start_index
        );

        patch_execute!(
            self.instructions,
            "continue",
            Inst::JUMP(loop_start_index as u32),
            loop_start_index
        );

        self.pop_scope();
        self.instructions.push(Inst::DEFAULT_NIL);
        self.comment("For loop end");
    }

    pub fn compile_loop(&mut self, block: &Box<Node>) {
        let loop_start_index = self.instructions.len();

        self.compile_node(&*block);

        self.instructions.push(Inst::JUMP(loop_start_index as u32));

        patch_execute!(
            self.instructions,
            "break",
            Inst::JUMP(self.instructions.len() as u32),
            loop_start_index
        );

        patch_execute!(
            self.instructions,
            "continue",
            Inst::JUMP(loop_start_index as u32),
            loop_start_index
        );

        self.instructions.push(Inst::DEFAULT_NIL);
    }

    pub fn compile_break(&mut self, value: &Option<Box<Node>>) {
        if let Some(val) = value {
            self.compile_node(&*val);
        } else {
            self.instructions.push(Inst::PUSH(Value::NIL));
        }
        let _ = patch!(self.instructions, "break");
    }

    pub fn compile_continue(&mut self) {
        let _ = patch!(self.instructions, "continue");
    }

    pub fn compile_match(&mut self, expr: &Box<Node>, branches: &Vec<(Node, Node)>) {
        self.compile_node(expr);

        for (idx, (condition, value)) in branches.iter().enumerate() {
            if idx <= branches.len() {
                self.instructions.push(Inst::DUP);
            }
            self.compile_node(condition);
            self.instructions.push(Inst::MATCH);

            let jump_if_false = patch!(self.instructions);

            self.compile_node(value);

            patch_execute!(
                self.instructions,
                jump_if_false,
                Inst::JUMP_IF_FALSE(self.instructions.len() as u32)
            );
        }

        self.instructions.push(Inst::DEFAULT_NIL);
    }

    pub fn compile_using(
        &mut self,
        sequence: &Vec<String>,
        imports: &Vec<(String, Option<String>)>,
        _wildcard: bool,
    ) {
        for (index, item) in sequence.iter().enumerate() {
            if index == 0 {
                let id = self.intern(item);
                self.instructions.push(Inst::LOAD(id));
            } else {
                self.instructions
                    .push(Inst::PUSH(Value::string(item.clone())));
                self.instructions.push(Inst::GET_PROP);
            }
        }

        if imports.len() > 0 {
            for (idx, (item, alias)) in imports.iter().enumerate() {
                if idx < imports.len() - 1 {
                    self.instructions.push(Inst::DUP)
                }
                self.instructions
                    .push(Inst::PUSH(Value::string(item.clone())));
                self.instructions.push(Inst::GET_PROP);
                if let Some(alias) = alias {
                    self.emit_store_local(alias, false);
                } else {
                    self.emit_store_local(item, false);
                }
            }
        } else {
            self.emit_store_local(&sequence[sequence.len() - 1], false);
        }
    }

    pub fn compile_enum_def(&mut self, name: &String, items: &Vec<(String, Node)>) {
        let mut name_vec = vec![];

        for (name, value) in items {
            name_vec.push(Value::string(name));
            self.compile_node(value);
        }
        self.instructions
            .push(Inst::ENUM(name.as_str().into(), name_vec));
        self.emit_store_local(name, false);
    }

    pub fn compile_struct_def(&mut self, name: &String, fields: &Vec<(String, String)>) {
        let mut field_map = HashMap::new();
        for (k, v) in fields {
            field_map.insert(
                self.intern(k.as_str()),
                (v.as_str().into(), k.as_str().into()),
            );
        }

        self.instructions
            .push(Inst::PUSH(Value::StructDef(rc!(TStructDef::new(
                name.as_str().into(),
                rc!(field_map),
            )))));

        self.emit_store_local(name, false);
    }

    pub fn compile_struct_init(&mut self, target: &Box<Node>, fields: &Vec<(String, Node)>) {
        let mut field_names = vec![];
        for (name, value) in fields {
            field_names.push(name.clone());
            self.compile_node(value);
        }
        self.compile_node(&**target);

        let field_name_ids = field_names
            .iter()
            .rev()
            .map(|x| self.intern(x.as_str()))
            .collect();
        self.instructions.push(Inst::STRUCT(field_name_ids));
    }

    pub fn compile_class_def(
        &mut self,
        name: &String,
        values: &Vec<Node>,
        functions: &Vec<Node>,
        constructor: &Option<Box<Node>>,
    ) {
        if let Some(constructor) = constructor {
            self.compile_node(&**constructor);
        }

        let mut field_names = vec![];

        for node in values {
            if let Node::LetStatement {
                names: field_name_list,
                values: field_values,
                is_const,
            } = node
            {
                for (i, field_name) in field_name_list.iter().enumerate() {
                    field_names.push((self.intern(field_name), *is_const));

                    match field_values.get(i).and_then(|v| v.as_deref()) {
                        Some(val) => self.compile_node(val),
                        None => self.instructions.push(Inst::PUSH(Value::NIL)),
                    }
                }
            }
        }

        let mut method_names = vec![];

        for node in functions {
            if let Node::FunctionDefinition {
                name: method_name,
                return_type,
                args,
                is_const,
                block,
            } = node
            {
                let method_str = method_name
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                method_names.push(self.intern(&method_str));

                self.compile_function_def(&None, return_type, args, *is_const, block);
            }
        }

        self.instructions.push(Inst::MAKE_CLASS {
            name: name.as_str().into(),
            field_names,
            method_names,
            has_constructor: constructor.is_some(),
        });

        self.emit_store_local(name, false);
    }

    pub fn compile_class_init(&mut self, target: &Box<Node>, parameters: &Vec<Node>) {
        for i in parameters.iter().rev() {
            self.compile_node(i);
        }

        self.compile_node(&**target);
        self.instructions
            .push(Inst::INIT_CLASS(parameters.len() as u16));
    }
}

#[macro_export]
macro_rules! patch {
    ($instructions:expr, $expr:expr) => {{
        $instructions.push(Inst::PATCH_ME($expr.into()));
        $instructions.len() - 1
    }};

    ($instructions:expr) => {{
        $instructions.push(Inst::PATCH_ME("".into()));
        $instructions.len() - 1
    }};
}

#[macro_export]
macro_rules! patch_execute {
    ($instructions:expr, $expr:literal, $new_expr:expr, $from:expr) => {
        for i in $from..$instructions.len() {
            if $instructions[i] == Inst::PATCH_ME($expr.into()) {
                $instructions[i] = $new_expr;
            }
        }
    };

    ($instructions:expr, $expr:literal, $new_expr:expr) => {
        for i in 0..$instructions.len() {
            if $instructions[i] == Inst::PATCH_ME($expr.into()) {
                $instructions[i] = $new_expr;
            }
        }
    };

    ($instructions:expr, $expr:expr, $new_expr:expr, $block_start:expr) => {
        for i in $block_start..$instructions.len() {
            if $instructions[i] == Inst::PATCH_ME($expr.into()) {
                $instructions[i] = $new_expr;
            }
        }
    };

    ($instructions:expr, $i:expr, $new_expr:expr) => {{
        $instructions[$i] = $new_expr;
    }};
}
