use crate::{
    language::{nodes::Node, token::TokenKind},
    patch, patch_execute, rc,
    virtual_machine::{
        builtin::BUILTINS,
        inst::Inst,
        types::{function::TFunction, string::TString},
        value::Value,
    },
};
use std::{cell::RefCell, rc::Rc};

pub struct Compiler {
    pub constants: Vec<Value>,
    pub offset: usize,
    pub instructions: Vec<Inst>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            offset: 0,
            constants: vec![],
            instructions: vec![],
        }
    }

    pub fn comment(&mut self, data: &str) {
        self.instructions.push(Inst::COMMENT(data.to_string()))
    }

    pub fn compile_node(&mut self, node: &Node) {
        match node {
            Node::NIL => self.instructions.push(Inst::PUSH(Value::NIL)),
            Node::Variable(x) => self.instructions.push(Inst::LOAD(x.clone())),
            Node::NumberLiteral(x) => self.instructions.push(Inst::PUSH(Value::Number(*x))),
            Node::BooleanLiteral(x) => self.instructions.push(Inst::PUSH(Value::Bool(*x))),
            Node::StringLiteral(x) => {
                if let Some((idx, _)) = self.constants.iter().enumerate().find(|(_, thing)| {
                    thing == &&Value::String(TString(rc!(RefCell::new(x.clone()))))
                }) {
                    self.instructions.push(Inst::LOAD_CONST(idx));
                    return;
                }
                self.constants
                    .push(Value::String(TString(rc!(RefCell::new(x.to_string())))));
                self.instructions
                    .push(Inst::LOAD_CONST(self.constants.len() - 1));
            }

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
                self.instructions.push(Inst::POP)
            }

            Node::UnaryOp {
                op,
                right,
                is_prefix,
            } => self.compile_unary_op(op, right, *is_prefix),
            Node::BinOp { left, right, op } => self.compile_bin_op(left, right, op),

            Node::LetStatement {
                names,
                values,
                is_const,
            } => self.compile_let(names, &mut values.clone(), *is_const),

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
                self.instructions.push(Inst::PUSH_SCOPE);
                self.compile_node(&**body);
                self.instructions.push(Inst::POP_SCOPE);
            }

            Node::Loop { block } => self.compile_loop(block),

            Node::WhileLoop { condition, block } => self.compile_while_loop(condition, block),

            Node::BreakStatement(value) => self.compile_break(value),

            Node::ForLoop {
                var_name,
                expr,
                block,
            } => self.compile_for(var_name, expr, block),

            _ => panic!("Unknown node: `{node:?}`"),
        }
    }
}

impl Compiler {
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
        for i in values.iter().rev() {
            self.compile_node(i);
        }
        if is_tuple {
            self.instructions.push(Inst::TUPLE(values.len()));
        } else {
            self.instructions.push(Inst::LIST(values.len()));
        }
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
                self.instructions.push(Inst::SET_VAR(x.clone()));
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

        self.instructions.push(match op {
            TokenKind::MINUS => Inst::NEG,
            TokenKind::PLUS => Inst::POS,
            TokenKind::BANG => Inst::NOT,

            _ => panic!("Tried compiling unknown unary op: {op:?}"),
        });
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

            if is_const {
                self.instructions
                    .push(Inst::STORE_LOCAL_CONST(names[i].clone()));
            } else {
                self.instructions.push(Inst::STORE_LOCAL(names[i].clone()));
            }
        }

        self.instructions.push(Inst::LOAD(names[0].clone()));
    }

    pub fn compile_block(&mut self, body: &Vec<Node>) {
        let mut outs = vec![];

        self.instructions.push(Inst::PUSH_SCOPE);
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

        self.instructions.push(Inst::POP_SCOPE);
    }

    pub fn compile_member_access(&mut self, expr: &Box<Node>, member: &Box<Node>) {
        self.compile_node(&**expr);
        self.compile_node(&**member);
        self.instructions.push(Inst::GET_PROP);
    }

    pub fn compile_function_call(&mut self, target: &Box<Node>, args: &Vec<Node>) {
        for i in args {
            self.compile_node(i);
        }

        if let Node::Variable(x) = &**target
            && BUILTINS.contains(&&***x)
        {
            self.instructions
                .push(Inst::CALL_BUILTIN(x.clone(), args.len()));
        } else {
            self.compile_node(&**target);
            self.instructions.push(Inst::CALL);
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

            self.instructions.push(Inst::PUSH_SCOPE);

            self.compile_node(&condition);
            let jump_if_false = patch!(self.instructions);

            self.compile_node(&block);

            self.instructions.push(Inst::POP_SCOPE);

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
            self.instructions.push(Inst::SET_VAR(x.clone()));
        } else if let Node::MemberAccess { expr, member } = &**target {
            self.compile_node(&**value);
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
            self.instructions.push(Inst::SET_VAR(x.clone()));
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
            self.instructions.push(Inst::STORE_LOCAL(name.clone()));
        }
        let func_jump_to_end = patch!(self.instructions);

        let func_start = self.offset + self.instructions.len();

        self.comment("Function def start:");

        self.instructions.push(Inst::PUSH_SCOPE);

        for (arg_name, _, default_value) in args.iter().rev() {
            self.instructions.push(Inst::DEFAULT_NIL);

            if let Some(def) = default_value {
                self.compile_node(def);
                self.instructions.push(Inst::DEFAULT);
            }

            self.instructions.push(Inst::STORE_LOCAL(arg_name.clone()));
        }

        self.compile_node(block);

        self.instructions.push(Inst::RETURN);
        self.instructions.push(Inst::POP_SCOPE);

        self.comment("Function def end");

        patch_execute!(
            self.instructions,
            func_value,
            Inst::PUSH(Value::Function(TFunction::new(func_start, args.len())))
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

        self.instructions.push(Inst::PUSH_SCOPE);

        self.compile_node(&*expr);
        self.instructions.push(Inst::GET_ITER);

        let loop_start_index = self.instructions.len();

        let for_iter = patch!(self.instructions);
        self.instructions.push(Inst::STORE_LOCAL(var_name.clone()));

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

        self.instructions.push(Inst::POP_SCOPE);
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
