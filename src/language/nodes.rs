use std::rc::Rc;

use crate::{language::token::TokenKind, virtual_machine::libs::types::TypeValue};

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Node {
    // Expressions vs Statements
    ExprStmt(Box<Node>),
    Multiple(Vec<Node>),

    // LITERALS
    NIL,
    Variable(Rc<String>),
    Symbol(String),
    Type(TypeValue),

    NumberLiteral(f64),
    BooleanLiteral(bool),
    StringLiteral(String),
    FString(Vec<Node>),

    // COLLECTIONS
    ListNode(Vec<Node>),
    TupleNode(Vec<Node>),
    DictNode(Vec<(Node, Node)>),

    // Range
    RangeNode {
        start: Box<Node>,
        end: Box<Node>,
        step: Option<Box<Node>>,
        inclusive: bool,
    },

    // OPERATORS
    BinOp {
        left: Box<Node>,
        right: Box<Node>,
        op: TokenKind,
    },
    UnaryOp {
        op: TokenKind,
        right: Box<Node>,
        is_prefix: bool,
    },
    ComparisonChain {
        expressions: Vec<Node>,
        operators: Vec<TokenKind>,
    },

    // Dedicated Coalescing
    NullCoalesce {
        left: Box<Node>,
        right: Box<Node>,
    }, // ??
    ElvisCoalesce {
        left: Box<Node>,
        right: Box<Node>,
    }, // ?:
    TernaryOp {
        condition: Box<Node>,
        true_expr: Box<Node>,
        false_expr: Box<Node>,
    },

    // MEMBER ACCESS
    MemberAccess {
        expr: Box<Node>,
        member: Box<Node>,
    },

    // STATEMENTS
    Exported(Box<Node>),
    ImportStatement {
        files: Vec<(String, Option<String>)>,
        pop_module: bool,
    },

    LetStatement {
        names: Vec<Rc<String>>,
        values: Vec<Option<Box<Node>>>,
        is_const: bool,
    },
    UsingStatement {
        sequence: Vec<String>,
        imports: Vec<(String, Option<String>)>,
        wildcard: bool,
    },

    SetVariable {
        target: Box<Node>,
        value: Box<Node>,
    },
    ShorthandAssignment {
        token: TokenKind,
        target: Box<Node>,
        value: Box<Node>,
    },

    Block {
        name: Option<String>,
        body: Vec<Node>,
    },

    // Arguments are in the tuple -> (name: String, type: Option<String>)
    FunctionDefinition {
        name: Option<Rc<String>>,
        return_type: Option<Rc<String>>,
        args: Vec<(Rc<String>, Option<Rc<String>>, Option<Node>)>,
        is_const: bool,
        block: Box<Node>,
    },

    FunctionCall {
        target: Box<Node>,
        args: Vec<Node>,
    },

    ReturnStatement(Option<Box<Node>>),
    BreakStatement(Option<Box<Node>>),
    OutStatement {
        block_name: Option<String>,
        value: Option<Box<Node>>,
    },
    ContinueStatement,

    // Loops
    Loop {
        block: Box<Node>,
    },
    WhileLoop {
        condition: Box<Node>,
        block: Box<Node>,
        else_block: Option<Box<Node>>,
    },
    ForLoop {
        var_name: Rc<String>,
        expr: Box<Node>,
        block: Box<Node>,
        else_block: Option<Box<Node>>,
    },

    // Logical Operations
    IfStatement {
        condition: Box<Node>,
        block: Box<Node>,
        elifs: Vec<(Node, Node)>,
        else_block: Option<Box<Node>>,
    },

    // Class stuff
    ClassDef {
        name: String,
        interfaces: Vec<Rc<String>>,
        let_statements: Vec<Node>,
        functions: Vec<Node>,
        constructor: Option<Box<Node>>,
    },

    ClassInit {
        target: Box<Node>,
        parameters: Vec<Node>,
    },

    StructDef {
        name: String,
        fields: Vec<(String, String)>, // (key, type)
    },

    StructInit {
        target: Box<Node>,
        fields: Vec<(String, Node)>,
    },

    InterfaceDef {
        name: Rc<String>,
        let_statements: Vec<Node>,
        functions: Vec<Node>,
    },

    EnumDef {
        name: String,
        items: Vec<(String, Node)>,
    },

    MatchStatement {
        expr: Box<Node>,
        branches: Vec<(Node, Node)>,
    },
}

impl Node {
    pub fn requires_scope(&self) -> bool {
        match self {
            // Wrappers
            Node::ExprStmt(node) | Node::Exported(node) => node.requires_scope(),

            // Multiple nodes
            Node::Multiple(nodes) => nodes.iter().any(Node::requires_scope),
            Node::Block { body, .. } => body.iter().any(Node::requires_scope),
            Node::ComparisonChain { expressions, .. } => {
                expressions.iter().any(Node::requires_scope)
            }

            // Operations
            Node::UnaryOp { right, .. } => right.requires_scope(),
            Node::BinOp { left, right, .. }
            | Node::NullCoalesce { left, right }
            | Node::ElvisCoalesce { left, right } => {
                left.requires_scope() || right.requires_scope()
            }
            Node::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => {
                condition.requires_scope()
                    || true_expr.requires_scope()
                    || false_expr.requires_scope()
            }

            // Values
            Node::ListNode(nodes) => nodes.iter().any(Node::requires_scope),
            Node::TupleNode(nodes) => nodes.iter().any(Node::requires_scope),
            Node::DictNode(nodes) => nodes
                .iter()
                .any(|(k, v)| k.requires_scope() || v.requires_scope()),
            Node::FString(nodes) => nodes.iter().any(Node::requires_scope),
            Node::RangeNode {
                start, end, step, ..
            } => {
                start.requires_scope()
                    || end.requires_scope()
                    || step.as_deref().is_some_and(Node::requires_scope)
            }

            // Setting stuff
            Node::SetVariable { target, value }
            | Node::ShorthandAssignment { target, value, .. } => {
                target.requires_scope() || value.requires_scope()
            }

            _ => matches!(
                self,
                Node::LetStatement { .. }
                    | Node::ImportStatement { .. }
                    | Node::FunctionDefinition { .. }
                    | Node::ClassDef { .. }
                    | Node::EnumDef { .. }
                    | Node::StructDef { .. }
                    | Node::InterfaceDef { .. }
            ),
        }
    }
}
