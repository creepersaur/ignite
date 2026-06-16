use crate::language::{nodes::Node, token::TokenKind};

pub struct AST {
    pub nodes: Vec<Node>,
}

impl AST {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub fn is_terminator(node: &Node) -> bool {
        match node {
            Node::ExprStmt(x) => Self::is_terminator(&**x),

            Node::BreakStatement(_) => true,
            Node::ReturnStatement(_) => true,
            Node::OutStatement { .. } => true,
            Node::ContinueStatement => true,

            Node::Block { body, .. } => {
                if let Some(last) = body.last() {
                    return Self::is_terminator(last);
                }
                false
            }

            _ => false,
        }
    }

    pub fn optimize(&mut self) {
        let mut terminator = None;

        for (i, node) in self.nodes.iter_mut().enumerate() {
            Self::prune_node(node);
            *node = Self::fold_constants(node.clone());

            if Self::is_terminator(node) {
                terminator = Some(i);
                break;
            }
        }

        if let Some(last) = terminator {
            self.nodes.truncate(last + 1);
        }
    }

    pub fn prune_node(node: &mut Node) {
        match node {
            Node::ExprStmt(node) => {
                Self::prune_node(node);
            }
            Node::Block { body, .. } => {
                Self::prune_block(body);
                if body.len() == 0 {
                    *node = Node::NIL;
                }
            }
            Node::IfStatement {
                block,
                elifs,
                else_block,
                condition,
            } => {
                if let Node::BooleanLiteral(x) = **condition {
                    if x == true {
                        Self::prune_node(block);
                        *node = *block.clone();
                        return;
                    } else if let Some(else_block) = else_block {
                        *node = *else_block.clone();
                        return;
                    } else {
                        *node = Node::NIL;
                        return;
                    }
                }

                Self::prune_node(condition);
                for (a, b) in elifs {
                    Self::prune_node(a);
                    Self::prune_node(b);
                }
                if let Some(e_block) = else_block {
                    Self::prune_node(e_block)
                }
            }
            Node::LetStatement { values, .. } => {
                for val in values {
                    if let Some(v) = val {
                        Self::prune_node(v);
                    }
                }
            }
            Node::ReturnStatement(value) => {
                if let Some(v) = value {
                    Self::prune_node(v);
                }
            }
            Node::OutStatement { value, .. } => {
                if let Some(v) = value {
                    Self::prune_node(v);
                }
            }
            Node::BreakStatement(value) => {
                if let Some(v) = value {
                    Self::prune_node(v);
                }
            }
            Node::FunctionDefinition { block, .. } => {
                Self::prune_node(block);
            }
            Node::Loop { block, .. } => {
                Self::prune_node(block);
            }
            Node::ForLoop { block, expr, .. } => {
                Self::prune_node(expr);
                Self::prune_node(block);
            }
            Node::WhileLoop {
                block, condition, ..
            } => {
                Self::prune_node(condition);
                Self::prune_node(block);
            }
            Node::BinOp { left, right, .. } => {
                Self::prune_node(left);
                Self::prune_node(right);
            }
            Node::UnaryOp { right, .. } => {
                Self::prune_node(right);
            }
            Node::DictNode(values) => {
                for (a, b) in values {
                    Self::prune_node(a);
                    Self::prune_node(b);
                }
            }
            Node::ListNode(values) => {
                for val in values {
                    Self::prune_node(val);
                }
            }
            Node::TupleNode(values) => {
                for val in values {
                    Self::prune_node(val);
                }
            }
            Node::RangeNode {
                start, step, end, ..
            } => {
                Self::prune_node(start);
                Self::prune_node(end);
                if let Some(v) = step {
                    Self::prune_node(v);
                }
            }
            Node::FunctionCall { target, args } => {
                Self::prune_node(target);

                for val in args {
                    Self::prune_node(val);
                }
            }
            Node::MemberAccess { expr, member } => {
                Self::prune_node(expr);
                Self::prune_node(member);
            }
            Node::SetVariable { target, value } => {
                Self::prune_node(target);
                Self::prune_node(value);
            }
            Node::ShorthandAssignment { target, value, .. } => {
                Self::prune_node(target);
                Self::prune_node(value);
            }
            Node::EnumDef { items, .. } => {
                for (_, value) in items.iter_mut() {
                    Self::prune_node(value);
                }
            }
            Node::MatchStatement { expr, branches } => {
                Self::prune_node(expr);
                for (condition, value) in branches {
                    Self::prune_node(condition);
                    Self::prune_node(value);
                }
            }
            Node::ClassInit { target, parameters } => {
                Self::prune_node(target);
                for i in parameters {
                    Self::prune_node(i);
                }
            }
            Node::ComparisonChain { expressions, .. } => {
                expressions.iter_mut().for_each(|x| Self::prune_node(x));
            }
            Node::NullCoalesce { left, right } => {
                Self::prune_node(left);
                Self::prune_node(right);
            }
            Node::ElvisCoalesce { left, right } => {
                Self::prune_node(left);
                Self::prune_node(right);
            }
            Node::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                Self::prune_node(condition);
                Self::prune_node(true_expr);
                Self::prune_node(false_expr);
            }
            Node::FString(parts) => {
                for part in parts {
                    Self::prune_node(part);
                }
            }
            Node::ClassDef {
                let_statements,
                functions,
                constructor,
                ..
            } => {
                for stmt in let_statements {
                    Self::prune_node(stmt);
                }

                for func in functions {
                    Self::prune_node(func);
                }

                if let Some(cons) = constructor {
                    Self::prune_node(cons);
                }
            }
            Node::StructInit { target, fields } => {
                Self::prune_node(target);

                for (_, value) in fields {
                    Self::prune_node(value);
                }
            }
            Node::InterfaceDef {
                let_statements,
                functions,
                ..
            } => {
                for stmt in let_statements {
                    Self::prune_node(stmt);
                }

                for func in functions {
                    Self::prune_node(func);
                }
            }

            _ => {}
        }
    }

    pub fn prune_block(body: &mut Vec<Node>) {
        for (idx, i) in body.iter_mut().enumerate() {
            Self::prune_node(i);
            if let Node::ExprStmt(i) = i {
                if Self::is_terminator(&*i) {
                    body.truncate(idx + 1);
                    return;
                }
            } else if Self::is_terminator(&*i) {
                body.truncate(idx + 1);
                return;
            }
        }
    }

    pub fn fold_constants(node: Node) -> Node {
        match node {
            Node::ExprStmt(n) => Node::ExprStmt(Box::new(Self::fold_constants(*n))),

            Node::BinOp { left, right, op } => {
                // Desugar chained comparison: (a < b) < c  →  (a < b) && (b < c)
                if matches!(
                    op,
                    TokenKind::LT | TokenKind::LE | TokenKind::GT | TokenKind::GE
                ) {
                    if let Node::BinOp {
                        left: ll,
                        right: lr,
                        op: inner_op,
                    } = &*left
                    {
                        if matches!(
                            inner_op,
                            TokenKind::LT | TokenKind::LE | TokenKind::GT | TokenKind::GE
                        ) {
                            let and_node = Node::BinOp {
                                left: Box::new(Node::BinOp {
                                    left: ll.clone(),
                                    right: lr.clone(),
                                    op: inner_op.clone(),
                                }),
                                right: Box::new(Node::BinOp {
                                    left: lr.clone(), // middle value
                                    right,
                                    op,
                                }),
                                op: TokenKind::AND,
                            };
                            return Self::fold_constants(and_node); // re-fold the AND
                        }
                    }
                }

                let folded_left = Self::fold_constants(*left);
                let folded_right = Self::fold_constants(*right);

                match (&folded_left, &folded_right) {
                    (Node::NumberLiteral(l), Node::NumberLiteral(r)) => match op {
                        // Arithmetic
                        TokenKind::PLUS => Node::NumberLiteral(l + r),
                        TokenKind::MINUS => Node::NumberLiteral(l - r),
                        TokenKind::STAR => Node::NumberLiteral(l * r),
                        TokenKind::SLASH if *r != 0.0 => Node::NumberLiteral(l / r),
                        TokenKind::MOD if *r != 0.0 => Node::NumberLiteral(l % r),
                        TokenKind::POW => Node::NumberLiteral(l.powf(*r)),

                        // Comparisons
                        TokenKind::EQ => Node::BooleanLiteral(l == r),
                        TokenKind::NEQ => Node::BooleanLiteral(l != r),
                        TokenKind::LT => Node::BooleanLiteral(l < r),
                        TokenKind::LE => Node::BooleanLiteral(l <= r),
                        TokenKind::GT => Node::BooleanLiteral(l > r),
                        TokenKind::GE => Node::BooleanLiteral(l >= r),

                        // Logical
                        TokenKind::OR => Node::BooleanLiteral(true),
                        TokenKind::AND => Node::BooleanLiteral(true),

                        _ => Node::BinOp {
                            left: Box::new(folded_left),
                            right: Box::new(folded_right),
                            op,
                        },
                    },

                    (Node::BooleanLiteral(l), Node::BooleanLiteral(r)) => match op {
                        TokenKind::EQ => Node::BooleanLiteral(l == r),
                        TokenKind::NEQ => Node::BooleanLiteral(l != r),

                        TokenKind::OR => Node::BooleanLiteral(*l || *r),
                        TokenKind::AND => Node::BooleanLiteral(*l && *r),

                        _ => Node::BinOp {
                            left: Box::new(folded_left),
                            right: Box::new(folded_right),
                            op,
                        },
                    },

                    (Node::StringLiteral(l), Node::StringLiteral(r)) => match op {
                        TokenKind::PLUS => Node::StringLiteral(format!("{}{}", l, r)),
                        TokenKind::EQ => Node::BooleanLiteral(l == r),
                        TokenKind::NEQ => Node::BooleanLiteral(l != r),

                        _ => Node::BinOp {
                            left: Box::new(folded_left),
                            right: Box::new(folded_right),
                            op,
                        },
                    },

                    (&Node::NIL, &Node::NIL) => match op {
                        TokenKind::OR => Node::BooleanLiteral(false),
                        TokenKind::AND => Node::BooleanLiteral(false),

                        _ => Node::BinOp {
                            left: Box::new(folded_left),
                            right: Box::new(folded_right),
                            op,
                        },
                    },

                    (&Node::BooleanLiteral(x), &Node::NIL) => match op {
                        TokenKind::OR => Node::BooleanLiteral(x),
                        TokenKind::AND => Node::BooleanLiteral(false),

                        _ => Node::BinOp {
                            left: Box::new(folded_left),
                            right: Box::new(folded_right),
                            op,
                        },
                    },

                    (&Node::NIL, &Node::BooleanLiteral(x)) => match op {
                        TokenKind::OR => Node::BooleanLiteral(x),
                        TokenKind::AND => Node::BooleanLiteral(false),

                        _ => Node::BinOp {
                            left: Box::new(folded_left),
                            right: Box::new(folded_right),
                            op,
                        },
                    },

                    _ => Node::BinOp {
                        left: Box::new(folded_left),
                        right: Box::new(folded_right),
                        op,
                    },
                }
            }

            Node::UnaryOp {
                op,
                right,
                is_prefix,
            } => {
                let folded_right = Self::fold_constants(*right);

                match folded_right {
                    Node::NumberLiteral(x) => match op {
                        TokenKind::MINUS => Node::NumberLiteral(-x),
                        TokenKind::PLUS => Node::NumberLiteral(x),
                        TokenKind::BANG => Node::BooleanLiteral(false),

                        _ => Node::UnaryOp {
                            op,
                            right: Box::new(Node::NumberLiteral(x)),
                            is_prefix,
                        },
                    },
                    Node::BooleanLiteral(x) => match op {
                        TokenKind::BANG => Node::BooleanLiteral(!x),

                        _ => Node::UnaryOp {
                            op,
                            right: Box::new(Node::BooleanLiteral(x)),
                            is_prefix,
                        },
                    },
                    Node::NIL => match op {
                        TokenKind::BANG => Node::BooleanLiteral(true),

                        _ => Node::UnaryOp {
                            op,
                            right: Box::new(Node::NIL),
                            is_prefix,
                        },
                    },

                    _ => Node::UnaryOp {
                        op,
                        right: Box::new(folded_right),
                        is_prefix,
                    },
                }
            }

            // --- DEEP RECURSION FOR ALL OTHER NODES ---
            Node::Block { body, name } => Node::Block {
                name,
                body: body.into_iter().map(Self::fold_constants).collect(),
            },

            Node::ListNode(items) => {
                Node::ListNode(items.into_iter().map(Self::fold_constants).collect())
            }
            Node::TupleNode(items) => {
                Node::TupleNode(items.into_iter().map(Self::fold_constants).collect())
            }
            Node::DictNode(items) => Node::DictNode(
                items
                    .into_iter()
                    .map(|(k, v)| (Self::fold_constants(k), Self::fold_constants(v)))
                    .collect(),
            ),
            Node::RangeNode {
                start,
                end,
                step,
                inclusive,
            } => Node::RangeNode {
                start: Box::new(Self::fold_constants(*start)),
                end: Box::new(Self::fold_constants(*end)),
                step: step.map(|s| Box::new(Self::fold_constants(*s))),
                inclusive,
            },

            Node::MemberAccess { expr, member } => Node::MemberAccess {
                expr: Box::new(Self::fold_constants(*expr)),
                member: Box::new(Self::fold_constants(*member)),
            },

            Node::LetStatement {
                names,
                values,
                is_const,
            } => Node::LetStatement {
                names,
                values: values
                    .into_iter()
                    .map(|v| v.map(|inner| Box::new(Self::fold_constants(*inner))))
                    .collect(),
                is_const,
            },
            Node::SetVariable { target, value } => Node::SetVariable {
                target: Box::new(Self::fold_constants(*target)),
                value: Box::new(Self::fold_constants(*value)),
            },
            Node::ShorthandAssignment {
                token,
                target,
                value,
            } => Node::ShorthandAssignment {
                token,
                target: Box::new(Self::fold_constants(*target)),
                value: Box::new(Self::fold_constants(*value)),
            },

            Node::FunctionDefinition {
                name,
                return_type,
                args,
                is_const,
                block,
            } => Node::FunctionDefinition {
                name,
                return_type,
                args,
                is_const,
                block: Box::new(Self::fold_constants(*block)),
            },
            Node::FunctionCall { target, args } => Node::FunctionCall {
                target: Box::new(Self::fold_constants(*target)),
                args: args.into_iter().map(Self::fold_constants).collect(),
            },

            Node::ReturnStatement(val) => {
                Node::ReturnStatement(val.map(|v| Box::new(Self::fold_constants(*v))))
            }
            Node::OutStatement { block_name, value } => Node::OutStatement {
                block_name,
                value: value.map(|v| Box::new(Self::fold_constants(*v))),
            },
            Node::BreakStatement(val) => {
                Node::BreakStatement(val.map(|v| Box::new(Self::fold_constants(*v))))
            }

            Node::IfStatement {
                condition,
                block,
                elifs,
                else_block,
            } => Node::IfStatement {
                condition: Box::new(Self::fold_constants(*condition)),
                block: Box::new(Self::fold_constants(*block)),
                elifs: elifs
                    .into_iter()
                    .map(|(c, b)| (Self::fold_constants(c), Self::fold_constants(b)))
                    .collect(),
                else_block: else_block.map(|b| Box::new(Self::fold_constants(*b))),
            },

            Node::Loop { block } => Node::Loop {
                block: Box::new(Self::fold_constants(*block)),
            },
            Node::WhileLoop {
                condition,
                block,
                else_block,
            } => Node::WhileLoop {
                condition: Box::new(Self::fold_constants(*condition)),
                block: Box::new(Self::fold_constants(*block)),
                else_block: else_block.map(|x| Box::new(Self::fold_constants(*x))),
            },
            Node::ForLoop {
                var_name,
                expr,
                block,
            } => Node::ForLoop {
                var_name,
                expr: Box::new(Self::fold_constants(*expr)),
                block: Box::new(Self::fold_constants(*block)),
            },

            Node::MatchStatement { expr, branches } => Node::MatchStatement {
                expr: Box::new(Self::fold_constants(*expr)),
                branches: branches
                    .into_iter()
                    .map(|(p, b)| (Self::fold_constants(p), Self::fold_constants(b)))
                    .collect(),
            },

            Node::EnumDef { name, items } => Node::EnumDef {
                name,
                items: items
                    .into_iter()
                    .map(|(k, v)| (k, Self::fold_constants(v)))
                    .collect(),
            },

            Node::ComparisonChain {
                expressions,
                operators,
            } => Node::ComparisonChain {
                expressions: expressions.into_iter().map(Self::fold_constants).collect(),
                operators,
            },

            Node::NullCoalesce { left, right } => {
                let left = Self::fold_constants(*left);
                let right = Self::fold_constants(*right);

                match left {
                    Node::NIL => right,
                    other => Node::NullCoalesce {
                        left: Box::new(other),
                        right: Box::new(right),
                    },
                }
            }

            Node::ElvisCoalesce { left, right } => Node::ElvisCoalesce {
                left: Box::new(Self::fold_constants(*left.clone())),
                right: Box::new(Self::fold_constants(*right.clone())),
            },

            Node::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                let condition = Self::fold_constants(*condition);

                match condition {
                    Node::BooleanLiteral(true) => Self::fold_constants(*true_expr),

                    Node::BooleanLiteral(false) => Self::fold_constants(*false_expr),

                    condition => Node::TernaryOp {
                        condition: Box::new(condition),
                        true_expr: Box::new(Self::fold_constants(*true_expr)),
                        false_expr: Box::new(Self::fold_constants(*false_expr)),
                    },
                }
            }

            Node::FString(parts) => {
                Node::FString(parts.into_iter().map(Self::fold_constants).collect())
            }

            Node::ClassDef {
                name,
                interfaces,
                let_statements,
                functions,
                constructor,
            } => Node::ClassDef {
                name,
                interfaces,
                let_statements: let_statements
                    .iter()
                    .map(|x| Self::fold_constants(x.clone()))
                    .collect(),
                functions: functions
                    .iter()
                    .map(|x| Self::fold_constants(x.clone()))
                    .collect(),
                constructor: constructor.map(|x| Box::new(Self::fold_constants(*x))),
            },

            Node::StructInit { target, fields } => Node::StructInit {
                target: Box::new(Self::fold_constants(*target)),
                fields: fields
                    .iter()
                    .map(|(name, value)| (name.clone(), Self::fold_constants(value.clone())))
                    .collect(),
            },

            Node::InterfaceDef {
                name,
                let_statements,
                functions,
            } => Node::InterfaceDef {
                name,
                let_statements: let_statements
                    .iter()
                    .map(|x| Self::fold_constants(x.clone()))
                    .collect(),
                functions: functions
                    .iter()
                    .map(|x| Self::fold_constants(x.clone()))
                    .collect(),
            },

            Node::ClassInit { target, parameters } => Node::ClassInit {
                target: Box::new(Self::fold_constants(*target.clone())),
                parameters: parameters
                    .iter()
                    .map(|x| Self::fold_constants(x.clone()))
                    .collect(),
            },

            other => other,
        }
    }
}
