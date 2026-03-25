#![allow(unused)]

#[derive(Debug, Clone, Copy)]
pub struct TokenRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub range: TokenRange,
}

impl Token {
    pub fn new(kind: TokenKind, range: TokenRange) -> Self {
        Self { kind, range }
    }

    pub fn get_text<'a>(&self, source: &'a str) -> String {
        source
            .chars()
            .skip(self.range.start)
            .take(self.range.end - self.range.start)
            .collect()
    }
}

#[allow(nonstandard_style)]
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    NEWLINE,
    Identifier,

    // LITERALS
    NIL,
    IntLiteral(i32),
    NumberLiteral(f32),
    StringLiteral(String),
    BooleanLiteral(bool),

    // Keywords
    LET,
    CONST,
    FN,
    RETURN,
    LOOP,
    WHILE,
    FOR,
    IN,
    BREAK,
    CONTINUE,
    OUT,
    IF,
    ELSE,
    CLASS,
    // INTERFACE,
    // STRUCT,

    // Punctuation
    LPAREN, // Parenthesis ()
    RPAREN,
    LBRACK, // Brackets []
    RBRACK,
    LBRACE, // Braces {}
    RBRACE,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    MOD,
    POW,
    DOLLAR, // $
    HASH,   // #
    AT,     // @
    BANG,   // !
    EQUAL,  // =
    EQ,     // ==
    NEQ,    // !=
    GT,     // >
    LT,     // <
    GE,     // >=
    LE,     // <=
    NE,     // !=
    OR,     // or, ||
    AND,    // and, &&
    COLON,
    DOUBLECOLON, // ::
    SEMI,        // ;
    QUESTION,    // ?
    TILDA,       // ~
    BACKTICK,
    PIPE,
    DOT,
    DOUBLEDOT, // ..
    COMMA,
    ARROW,     // ->
    FATARROW,  // =>
    INCREMENT, // ++
    DECREMENT, // --
    ADD_SH,    // += shorthand
    SUB_SH,    // -= shorthand
    MUL_SH,    // *= shorthand
    DIV_SH,    // /= shorthand
    MOD_SH,    // %= shorthand
}
