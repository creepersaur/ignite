#![allow(unused)]

use crate::language::token::{
    Token,
    TokenKind::{self, *},
    TokenRange,
};

const PUNCTUATION: &str = "!@#$%^&*()-+[]{}|:;,./<>?=\n";
const DOUBLE: [&str; 17] = [
    "->", "||", "&&", "<=", ">=", "==", "!=", "=>", "::", "..", "++", "--", "+=", "-=", "*=", "/=",
    "%=",
];

#[derive(Debug)]
pub struct Lexer {
    chars: Vec<char>,
    pos: i32,
    cur_char: Option<char>,
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        let mut new_lexer = Self {
            chars: Self::apply_comments(text),
            pos: -1,
            cur_char: None,
        };

        new_lexer.advance();
        new_lexer
    }

    pub fn apply_comments(text: &str) -> Vec<char> {
        text.lines()
            .map(|x| {
                if let Some((left, _)) = x.split_once("//") {
                    left
                } else {
                    x
                }
            })
            .collect::<Vec<&str>>()
            .join("\n")
            .chars()
            .collect()
    }

    pub fn advance(&mut self) {
        self.pos += 1;

        if self.pos < self.chars.len() as i32 {
            self.cur_char = Some(self.chars[self.pos as usize])
        } else {
            self.cur_char = None
        }
    }

    pub fn get_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        let mut instr = None;

        loop {
            if self.cur_char.is_none() {
                break;
            }

            // Skip tabs and spaces
            while let Some(c) = self.cur_char {
                if c == '\t' || c == ' ' {
                    self.advance();
                } else {
                    break;
                }
            }

            let start_pos = self.pos;
            let mut current_token = String::new();

            while let Some(c) = self.cur_char {
                if instr.is_none() && (c == '\t' || c == ' ') {
                    break;
                }

                if instr.is_none() && PUNCTUATION.contains(c) {
                    if c == '.' && current_token.parse::<i32>().is_ok() {
                        let next_char = self.chars.get((self.pos + 1) as usize);

                        if next_char == Some(&'.') {
                            if !current_token.is_empty() {
                                tokens.push(Self::identify(&current_token, start_pos));
                                current_token.clear();
                            }
                        } else if current_token.parse::<i32>().is_ok() {
                            current_token.push(c);
                            self.advance();
                            continue;
                        }
                    }

                    if tokens.len() > 0
                        && DOUBLE
                            .contains(&format!("{}{c}", self.chars[self.pos as usize - 1]).as_str())
                    {
                        tokens.pop();
                        current_token = format!("{}{c}", self.chars[self.pos as usize - 1]);

                        tokens.push(Self::identify(&current_token, start_pos - 1));
                        current_token.clear();
                        break;
                    }

                    if !current_token.is_empty() {
                        tokens.push(Self::identify(&current_token, start_pos));
                    }

                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    tokens.push(Self::identify(s, self.pos));

                    current_token.clear();
                    break;
                } else {
                    current_token.push(c);

                    if c == '"' || c == '\'' {
                        if let Some(s) = instr {
                            if s == c {
                                instr = None;
                            }
                        } else {
                            instr = Some(c);
                        }
                    }
                }

                self.advance();
            }

            if !current_token.is_empty() {
                tokens.push(Self::identify(&current_token, start_pos));
            }

            self.advance();
        }

        tokens
    }

    pub fn identify(text: &str, start: i32) -> Token {
        let start = start as usize;
        let end = start + text.len();

        let kind = match text {
            "\n" => NEWLINE,
            "nil" => NIL,

            // Keywords
            "let" => LET,
            "const" => CONST,
            "fn" => FN,
            "return" => RETURN,
            "for" => FOR,
            "in" => IN,
            "break" => BREAK,
            "continue" => CONTINUE,
            "out" => OUT,
            "loop" => LOOP,
            "while" => WHILE,
            "if" => IF,
            "else" => ELSE,
            "class" => CLASS,
            // "struct" => STRUCT,
            // "interface" => INTERFACE,

            // Punctuation
            "(" => LPAREN, // Parenthesis ()
            ")" => RPAREN,
            "[" => LBRACK, // Brackets []
            "]" => RBRACK,
            "{" => LBRACE, // Braces {}
            "}" => RBRACE,
            "+" => PLUS,
            "-" => MINUS,
            "*" => STAR,
            "/" => SLASH,
            "%" => MOD,
            "^" => POW,
            "$" => DOLLAR,
            "#" => HASH,
            "@" => AT,
            "!" => BANG,
            "=" => EQUAL,
            "==" => EQ,
            ">" => GT,
            "<" => LT,
            ">=" => GE,
            "<=" => LE,
            "!=" => NEQ,
            "or" => OR,
            "||" => OR,
            "and" => AND,
            "&&" => AND,
            ":" => COLON,
            "::" => DOUBLECOLON,
            ";" => SEMI,
            "?" => QUESTION,
            "~" => TILDA,
            "`" => BACKTICK,
            "|" => PIPE,
            "." => DOT,
            ".." => DOUBLEDOT,
            "," => COMMA,
            "->" => ARROW,
            "=>" => FATARROW,
			"++" => INCREMENT,
			"--" => DECREMENT,
			"+=" => ADD_SH,
			"-=" => SUB_SH,
			"*=" => MUL_SH,
			"/=" => DIV_SH,
			"%=" => MOD_SH,

            _ => Self::identify_other(text),
        };

        Token::new(kind, TokenRange { start, end })
    }

    pub fn identify_other(text: &str) -> TokenKind {
        if let Ok(x) = text.parse::<f32>() {
            return NumberLiteral(x);
        } else if let Ok(x) = text.parse::<bool>() {
            return BooleanLiteral(x);
        } else if text.starts_with('"') && text.ends_with('"') {
            return StringLiteral(text[1..text.len() - 1].to_string());
        } else if text.starts_with('\'') && text.ends_with('\'') {
            return StringLiteral(text[1..text.len() - 1].to_string());
        } else if text.starts_with('`') && text.ends_with('`') {
            return StringLiteral(text[1..text.len() - 1].to_string());
        }

        Identifier
    }
}
