use std::rc::Rc;

use crate::{language::token::TokenKind, virtual_machine::libs::types::TypeValue};

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Node {
    // Expressions vs Statements
    ExprStmt(Box<Node>),

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
    SingleLineBlock {
        body: Box<Node>,
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
    },
    ForLoop {
        var_name: Rc<String>,
        expr: Box<Node>,
        block: Box<Node>,
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
