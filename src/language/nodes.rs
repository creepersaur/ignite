use std::rc::Rc;

use crate::language::token::TokenKind;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Node {
    // Expressions vs Statements
    ExprStmt(Box<Node>),

    // LITERALS
    NIL,
    Variable(Rc<String>),

    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),

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
    UsingStatement { sequence: Vec<String>, imports: Vec<String>, wildcard: bool },

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
        block: Box<Node>,
    },

    FunctionCall {
        target: Box<Node>,
        args: Vec<Node>,
    },

    ReturnStatement(Option<Box<Node>>),
    BreakStatement(Option<Box<Node>>),
    OutStatement(Option<Box<Node>>),
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
    },

    StructDef {
        name: String,
        types: Vec<(Rc<String>, Rc<String>)>, // (key, type)
    },

    InterfaceDef {
        name: Rc<String>,
        let_statements: Vec<Node>,
        functions: Vec<Node>,
    },

    MatchStatement {
        expr: Box<Node>,
        branches: Vec<(Node, Node)>,
    },
}
