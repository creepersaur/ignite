use crate::{
    hash_u64,
    language::{nodes::Node, token::TokenKind},
    patch, patch_execute, rc,
    virtual_machine::{
        inst::Inst,
        types::{function::TFunction, list::TList, string::TString},
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
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            offset: 0,
            constants: vec![],
            instructions: vec![],
            scopes: vec![HashSet::new()],
            intern_table: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> u64 {
        let id = hash_u64!(s);
        self.intern_table.entry(id).or_insert_with(|| Rc::from(s));
        id
    }

    pub fn comment(&mut self, data: &str) {
        self.instructions.push(Inst::COMMENT(data.to_string()))
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
        let depth = self.scopes.len() - 1;

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
        let mut found_depth = None;

        for (depth, scope) in self.scopes.iter().enumerate().rev() {
            if let Some(_) = scope.get(name) {
                found_depth = Some(depth);
                break;
            }
        }

        let id = self.intern(name);
        if let Some(depth) = found_depth {
            self.instructions.push(Inst::LOAD_LOCAL { id, depth });
        } else {
            self.instructions.push(Inst::LOAD_GLOBAL(id));
        }
    }

    pub fn compile_node(&mut self, node: &Node) {
        match node {
            Node::NIL => self.instructions.push(Inst::PUSH(Value::NIL)),
            Node::Variable(x) => self.emit_load_local(x.as_str()),
            Node::NumberLiteral(x) => self.instructions.push(Inst::PUSH(Value::Number(*x))),
            Node::BooleanLiteral(x) => self.instructions.push(Inst::PUSH(Value::Bool(*x))),
            Node::StringLiteral(x) => {
                if let Some((idx, _)) = self
                    .constants
                    .iter()
                    .enumerate()
                    .find(|(_, thing)| thing == &&Value::String(TString::new(x.clone())))
                {
                    self.instructions.push(Inst::LOAD_CONST(idx));
                    return;
                }
                self.constants
                    .push(Value::String(TString::new(x.to_string())));
                self.instructions
                    .push(Inst::LOAD_CONST(self.constants.len() - 1));
            }
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
                block,
            } => self.compile_function_def(name, return_type, args, block),

            Node::ReturnStatement(value) => self.compile_return(value),

            Node::IfStatement {
                condition,
                block,
                elifs,
                else_block,
            } => self.compile_if_statement(condition, block, elifs, else_block),

            Node::Block { body } => self.compile_block(body),

            Node::SingleLineBlock { body } => {
                self.push_scope();
                self.compile_node(&**body);
                self.pop_scope();
            }

            Node::Loop { block } => self.compile_loop(block),

            Node::WhileLoop { condition, block } => self.compile_while_loop(condition, block),

            Node::BreakStatement(value) => self.compile_break(value),

            Node::ForLoop {
                var_name,
                expr,
                block,
            } => self.compile_for(var_name, expr, block),

            Node::MatchStatement { expr, branches } => self.compile_match(expr, branches),

            _ => panic!("Unknown node: `{node:?}`"),
        }
    }
}

impl Compiler {
    pub fn compile_fstring(&mut self, values: &Vec<Node>) {
        for i in values.iter().rev() {
            self.compile_node(i);
        }
        self.instructions.push(Inst::CONCAT_STR(values.len()));
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
        self.instructions.push(Inst::PUSH(Value::Bool(inclusive)));
        self.instructions.push(Inst::RANGE);
    }

    pub fn compile_list(&mut self, values: &Vec<Node>, is_tuple: bool) {
        let mut folded_values = Vec::with_capacity(values.len());
        let mut all_literals = true;

        for node in values {
            match node {
                Node::NumberLiteral(n) => folded_values.push(Value::Number(*n)),
                Node::BooleanLiteral(b) => folded_values.push(Value::Bool(*b)),
                Node::StringLiteral(s) => {
                    folded_values.push(Value::String(TString::new(s.clone())))
                }
                Node::NIL => folded_values.push(Value::NIL),

                _ => {
                    all_literals = false;
                    break;
                }
            }
        }

        if all_literals && !values.is_empty() {
            self.instructions
                .push(Inst::PUSH(Value::List(TList::new(rc!(RefCell::new(
                    folded_values
                ))))));
            return;
        }

        values.iter().rev().for_each(|node| self.compile_node(node));
        self.instructions.push(if is_tuple {
            Inst::TUPLE(values.len())
        } else {
            Inst::LIST(values.len())
        });
    }

    pub fn compile_dict(&mut self, values: &Vec<(Node, Node)>) {
        for (k, v) in values.iter().rev() {
            self.compile_node(k);
            self.compile_node(v);
        }
        self.instructions.push(Inst::DICT(values.len()));
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
                if !is_prefix {
                    self.instructions.push(Inst::DUP);
                }
                self.instructions.push(Inst::PUSH(Value::Number(1.0)));
                self.instructions.push(operator_inst);
                if !is_prefix {
                    self.instructions.push(Inst::DUP);
                }
                self.compile_node(&**expr);
                self.compile_node(&**member);
                self.instructions.push(Inst::SET_PROP);
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

            _ => panic!("Cannot compile unknown bin-op: `{op:?}`"),
        });
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
            Inst::JUMP_IF_NOT_NIL(self.instructions.len())
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
            Inst::JUMP_IF_TRUE(self.instructions.len())
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
            Inst::JUMP_IF_FALSE(self.instructions.len())
        );

        self.compile_node(&**false_expr);

        patch_execute!(
            self.instructions,
            jump_end,
            Inst::JUMP(self.instructions.len())
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

    pub fn compile_block(&mut self, body: &Vec<Node>) {
        let mut outs = vec![];

        self.push_scope();
        for i in body {
            if let Node::ExprStmt(expr) = i {
                if let Node::OutStatement(val) = expr.as_ref() {
                    if let Some(v) = val {
                        self.compile_node(&*v);
                    } else {
                        self.instructions.push(Inst::PUSH(Value::NIL));
                    }
                    outs.push(patch!(self.instructions)); // Jump to scope cleanup
                } else {
                    self.compile_node(i);
                }
            } else {
                self.compile_node(i);
            }
        }

        self.instructions.push(Inst::PUSH(Value::NIL));

        let block_end = self.instructions.len();
        for i in outs {
            patch_execute!(self.instructions, i, Inst::JUMP(block_end));
        }

        self.pop_scope();
    }

    pub fn compile_member_access(&mut self, expr: &Box<Node>, member: &Box<Node>) {
        self.compile_node(&**expr);
        self.compile_node(&**member);
        self.instructions.push(Inst::GET_PROP);
    }

    pub fn compile_function_call(&mut self, target: &Box<Node>, args: &Vec<Node>) {
        for i in args.iter() {
            self.compile_node(i);
        }

        self.compile_node(&**target);
        self.instructions.push(Inst::CALL(args.len()));
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
                Inst::JUMP_IF_FALSE(self.instructions.len())
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

        let if_end = self.instructions.len();
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
            self.compile_node(&**member);
            self.instructions.push(Inst::SET_PROP);
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
            self.compile_node(&**value);
            self.compile_node(&**expr);
            self.instructions.push(operator_inst);
            self.compile_node(&**expr);
            self.compile_node(&**member);
            self.instructions.push(Inst::SET_PROP);
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
        block: &Box<Node>,
    ) {
        self.comment("New function:");
        let func_value = patch!(self.instructions);
        if let Some(name) = name {
            self.emit_store_local(name.as_str(), false);
        }
        let func_jump_to_end = patch!(self.instructions);

        let func_start = self.offset + self.instructions.len();

        self.comment("Function def start:");

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

        self.comment("Function def end");

        patch_execute!(
            self.instructions,
            func_value,
            Inst::PUSH(Value::Function(TFunction::new(func_start)))
        );

        patch_execute!(
            self.instructions,
            func_jump_to_end,
            Inst::JUMP(self.instructions.len())
        );
    }

    pub fn compile_while_loop(&mut self, condition: &Box<Node>, block: &Box<Node>) {
        let loop_start_index = self.instructions.len();

        self.compile_node(&*condition);

        let end_loop_jump = patch!(self.instructions);

        self.compile_node(&*block);

        self.instructions.push(Inst::JUMP(loop_start_index));

        patch_execute!(
            self.instructions,
            end_loop_jump,
            Inst::JUMP_IF_FALSE(self.instructions.len())
        );

        patch_execute!(
            self.instructions,
            "break",
            Inst::JUMP(self.instructions.len()),
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

        self.instructions.push(Inst::JUMP(loop_start_index));

        patch_execute!(
            self.instructions,
            for_iter,
            Inst::FOR_ITER(self.instructions.len())
        );

        patch_execute!(
            self.instructions,
            "break",
            Inst::JUMP(self.instructions.len()),
            loop_start_index
        );

        self.pop_scope();
        self.instructions.push(Inst::DEFAULT_NIL);
        self.comment("For loop end");
    }

    pub fn compile_loop(&mut self, block: &Box<Node>) {
        let loop_start_index = self.instructions.len();

        self.compile_node(&*block);

        self.instructions.push(Inst::JUMP(loop_start_index));

        patch_execute!(
            self.instructions,
            "break",
            Inst::JUMP(self.instructions.len()),
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
                Inst::JUMP_IF_FALSE(self.instructions.len())
            );
        }

        self.instructions.push(Inst::DEFAULT_NIL);
    }

    pub fn compile_using(
        &mut self,
        sequence: &Vec<String>,
        imports: &Vec<String>,
        _wildcard: bool,
    ) {
        for (index, item) in sequence.iter().enumerate() {
            if index == 0 {
                let id = self.intern(item);
                self.instructions.push(Inst::LOAD(id));
            } else {
                self.instructions
                    .push(Inst::PUSH(Value::String(TString::new(item.clone()))));
                self.instructions.push(Inst::GET_PROP);
            }
        }

        if imports.len() > 0 {
            for (idx, item) in imports.iter().enumerate() {
                if idx < imports.len() - 1 {
                    self.instructions.push(Inst::DUP)
                }
                self.instructions
                    .push(Inst::PUSH(Value::String(TString::new(item.clone()))));
                self.instructions.push(Inst::GET_PROP);
                self.emit_store_local(item, false);
            }
        } else {
            self.emit_store_local(&sequence[sequence.len() - 1], false);
        }
    }
}

#[macro_export]
macro_rules! patch {
    ($instructions:expr, $expr:expr) => {{
        $instructions.push(Inst::PATCH_ME($expr.to_string()));
        $instructions.len() - 1
    }};

    ($instructions:expr) => {{
        $instructions.push(Inst::PATCH_ME(String::new()));
        $instructions.len() - 1
    }};
}

#[macro_export]
macro_rules! patch_execute {
    ($instructions:expr, $expr:literal, $new_expr:expr, $from:expr) => {
        for i in $from..$instructions.len() {
            if $instructions[i] == Inst::PATCH_ME($expr.to_string()) {
                $instructions[i] = $new_expr;
            }
        }
    };

    ($instructions:expr, $expr:literal, $new_expr:expr) => {
        for i in 0..$instructions.len() {
            if $instructions[i] == Inst::PATCH_ME($expr.to_string()) {
                $instructions[i] = $new_expr;
            }
        }
    };

    ($instructions:expr, $i:expr, $new_expr:expr) => {{
        $instructions[$i] = $new_expr;
    }};
}
