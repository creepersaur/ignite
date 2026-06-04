#![allow(unused)]

use std::{collections::HashMap, thread::current};

use crate::{
    language::{
        lexer::Lexer,
        nodes::Node,
        token::{Token, TokenKind},
    },
    rc,
    virtual_machine::libs::types::TypeValue,
};

type TokenResult = Result<Token, String>;
type NodeResult = Result<Node, String>;

#[derive(Debug, Clone)]
pub struct Parser {
    source: String,
    tokens: Vec<Token>,
    pos: i32,
}

impl Parser {
    pub fn new(source: String, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            pos: 0,
        }
    }

    fn advance(&mut self) -> TokenResult {
        let current = self.current();
        self.pos += 1;

        current
    }

    #[allow(unused)]
    pub fn current(&self) -> TokenResult {
        if self.pos < self.tokens.len() as i32 {
            Ok(self.tokens[self.pos as usize].clone())
        } else {
            Err("Expected more tokens. Got [EOF]. (Current)".to_string())
        }
    }

    pub fn peek(&self) -> Option<Token> {
        if self.pos + 1 < self.tokens.len() as i32 {
            Some(self.tokens[(self.pos + 1) as usize].clone())
        } else {
            None
        }
    }

    fn expect_and_consume(&mut self, kind: TokenKind) -> Result<Token, String> {
        self.skip_new_lines();

        if let Ok(next) = self.current() {
            if next.kind != kind {
                Err(format!("Expected `{kind:?}`, got `{:?}`", next.kind))
            } else {
                self.advance()?;
                self.skip_new_lines();

                Ok(next)
            }
        } else {
            Err(format!("Expected `{kind:?}`, got [EOF]."))
        }
    }

    fn parse_surrounded(
        &mut self,
        left: TokenKind,
        right: TokenKind,
        separator: Option<TokenKind>,
        mut f: impl FnMut(&mut Self) -> Result<(), String>,
    ) -> Result<(), String> {
        self.expect_and_consume(left)?;

        loop {
            self.skip_new_lines();
            if let Ok(next) = self.current()
                && next.kind == right
            {
                break;
            }

            f(self)?;

            if let Some(ref sep) = separator {
                if let Ok(next) = self.current()
                    && next.kind == *sep
                {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }

        self.expect_and_consume(right)?;

        Ok(())
    }

    pub fn parse(&mut self) -> NodeResult {
        self.skip_new_lines();

        match self.current()?.kind {
            TokenKind::CLASS => self.parse_class_def(),
            TokenKind::FN => self.parse_function_def(false, false),
            TokenKind::USING => self.parse_using(),
            TokenKind::ENUM => self.parse_enum(),
            TokenKind::STRUCT => self.parse_struct_def(),

            _ => {
                let expr = self.parse_expression()?;

                Ok(Node::ExprStmt(Box::new(expr)))
            }
        }
    }
}

// EXPRESSIONS
impl Parser {
    fn parse_explicit_block(&mut self) -> NodeResult {
        self.advance();
        self.parse_block()
    }

    fn parse_set_variable(&mut self, expr: Node) -> NodeResult {
        self.advance()?;

        Ok(Node::SetVariable {
            target: Box::new(expr),
            value: Box::new(self.parse_expression()?),
        })
    }

    fn parse_shorthand_assignment(&mut self, expr: Node, token: TokenKind) -> NodeResult {
        self.advance()?;

        Ok(Node::ShorthandAssignment {
            token,
            target: Box::new(expr),
            value: Box::new(self.parse_expression()?),
        })
    }

    fn parse_add_sub(&mut self) -> NodeResult {
        let mut left = self.parse_mul_div()?;

        while let Ok(next) = self.current() {
            if !matches!(next.kind, TokenKind::PLUS | TokenKind::MINUS) {
                break;
            }

            self.skip_new_lines();
            let op = self.advance()?.kind;
            let right = self.parse_mul_div()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
    }

    fn parse_mul_div(&mut self) -> NodeResult {
        let mut left = self.parse_unary()?;

        while let Ok(next) = self.current() {
            if !matches!(
                next.kind,
                TokenKind::STAR | TokenKind::SLASH | TokenKind::MOD
            ) {
                break;
            }

            self.skip_new_lines();
            let op = self.advance()?.kind;
            let right = self.parse_unary()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> NodeResult {
        self.skip_new_lines();

        if let Ok(next) = self.current()
            && matches!(
                next.kind,
                TokenKind::PLUS
                    | TokenKind::MINUS
                    | TokenKind::BANG
                    | TokenKind::INCREMENT
                    | TokenKind::DECREMENT
            )
        {
            let op = self.advance()?.kind;

            return Ok(Node::UnaryOp {
                op,
                right: Box::new(self.parse_fstring()?),
                is_prefix: true,
            });
        }

        let expr = self.parse_fstring()?;

        if let Ok(next) = self.current()
            && matches!(next.kind, TokenKind::INCREMENT | TokenKind::DECREMENT)
        {
            let op = self.advance()?.kind;

            return Ok(Node::UnaryOp {
                op,
                right: Box::new(expr),
                is_prefix: false,
            });
        }

        Ok(expr)
    }

    fn parse_fstring(&mut self) -> NodeResult {
        self.skip_new_lines();

        if let Ok(next) = self.current()
            && matches!(next.kind, TokenKind::DOLLAR)
        {
            self.advance()?;

            let expr = self.parse_exponent()?;
            if let Node::StringLiteral(s) = expr {
                let chars = s.chars().collect::<Vec<char>>();
                let mut idx = 0;
                let mut current_value = String::new();
                let mut values = vec![];
                let mut depth: i32 = 0;

                while idx < chars.len() {
                    let ch = chars[idx];

                    // Handle escape sequences for braces
                    if ch == '\\'
                        && idx + 1 < chars.len()
                        && (chars[idx + 1] == '{' || chars[idx + 1] == '}')
                    {
                        current_value.push(chars[idx + 1]);
                        idx += 2;
                        continue;
                    }

                    if ch == '{' {
                        if depth == 0 {
                            // Flush accumulated string before the interpolation
                            if !current_value.is_empty() {
                                values.push(Node::StringLiteral(current_value.clone()));
                                current_value.clear();
                            }
                        } else {
                            // Nested brace — keep it as part of the expression
                            current_value.push(ch);
                        }
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            // Parse whatever was inside ${ ... }
                            values.push(
                                Parser::new(
                                    current_value.clone(),
                                    Lexer::new(&current_value).get_tokens(),
                                )
                                .parse_expression()?,
                            );
                            current_value.clear();
                        } else {
                            // Still inside a nested expression
                            current_value.push(ch);
                        }
                    } else {
                        current_value.push(ch);
                    }

                    idx += 1;
                }

                if depth != 0 {
                    return Err("Unclosed `{` in f-string interpolation.".to_string());
                }

                // Flush any trailing string literal after the last `}`
                if !current_value.is_empty() {
                    values.push(Node::StringLiteral(current_value));
                }

                return Ok(Node::FString(values));
            } else {
                panic!("`$` can only be applied to string literals")
            }
        } else {
            self.parse_exponent()
        }
    }

    fn parse_exponent(&mut self) -> NodeResult {
        let mut left = self.parse_call()?;

        while let Ok(next) = self.current() {
            if !matches!(next.kind, TokenKind::POW) {
                break;
            }

            self.skip_new_lines();
            let op = self.advance()?.kind;
            let right = self.parse_primary()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
    }

    fn parse_call(&mut self) -> NodeResult {
        let mut expr = self.parse_member()?;

        loop {
            if let Ok(x) = self.current() {
                match x.kind {
                    TokenKind::LPAREN => {
                        expr = self.parse_function_call(expr)?;
                        continue;
                    }

                    _ => {}
                }
            }

            break;
        }

        return Ok(expr);
    }

    fn parse_member(&mut self) -> NodeResult {
        let mut expr = self.parse_primary()?;

        loop {
            if let Ok(x) = self.current() {
                match x.kind {
                    TokenKind::DOT => {
                        expr = self.parse_member_access(expr)?;
                        continue;
                    }
                    TokenKind::DOUBLECOLON => {
                        expr = self.parse_member_access(expr)?;
                        continue;
                    }
                    TokenKind::LBRACK => {
                        expr = self.parse_member_access(expr)?;
                        continue;
                    }

                    _ => {}
                }
            }

            break;
        }

        return Ok(expr);
    }

    fn parse_primary(&mut self) -> NodeResult {
        self.skip_new_lines();
        let start_pos = self.pos;
        let current = self.current()?;

        let node = match current.kind {
            TokenKind::NIL => Ok(Node::NIL),
            TokenKind::NumberLiteral(x) => Ok(Node::NumberLiteral(x)),
            TokenKind::BooleanLiteral(x) => Ok(Node::BooleanLiteral(x)),
            TokenKind::StringLiteral(x) => Ok(Node::StringLiteral(x)),

            TokenKind::Identifier => {
                let text = current.get_text(&self.source);

                if let Some(t) = self.is_type(&text) {
                    Ok(Node::Type(t))
                } else {
                    Ok(Node::Variable(rc!(text)))
                }
            }

            TokenKind::AT => self.parse_explicit_block(),

            // COLLECTIONS
            TokenKind::LPAREN => self.parse_tuple(),
            TokenKind::LBRACK => self.parse_list(),
            TokenKind::LBRACE => self.parse_dict(),

            TokenKind::LET => self.parse_let(false),
            TokenKind::CONST => self.parse_const(),
            TokenKind::FN => self.parse_function_def(true, false),
            TokenKind::NEW => self.parse_new(),
            TokenKind::LOOP => self.parse_loop(),
            TokenKind::WHILE => self.parse_while(),
            TokenKind::FOR => self.parse_for(),
            TokenKind::RETURN => self.parse_return(),
            TokenKind::OUT => self.parse_out(),
            TokenKind::BREAK => self.parse_break(),
            TokenKind::CONTINUE => self.simple_parse_keyword(Node::ContinueStatement),
            TokenKind::IF => self.parse_if(),
            TokenKind::MATCH => self.parse_match(),

            other => Err(format!(
                "Got unexpected token `{other:?}` while parsing primary."
            )),
        }?;

        if self.pos == start_pos {
            self.advance()?;
        }

        Ok(node)
    }

    fn is_type(&mut self, text: &str) -> Option<TypeValue> {
        match text {
            "number" => Some(TypeValue::Number),
            "bool" => Some(TypeValue::Bool),
            "char" => Some(TypeValue::Char),
            "string" => Some(TypeValue::String),
            "dict" => Some(TypeValue::Dict),
            "function" => Some(TypeValue::Function),
            "list" => Some(TypeValue::List),
            "tuple" => Some(TypeValue::Tuple),

            _ => None,
        }
    }

    fn skip_new_lines(&mut self) {
        while let Ok(next) = self.current() {
            if matches!(next.kind, TokenKind::NEWLINE) {
                self.advance().unwrap();
            } else {
                break;
            }
        }
    }

    fn parse_tuple(&mut self) -> NodeResult {
        self.advance()?;
        self.skip_new_lines();

        let first = if self.current().is_ok() {
            self.parse_expression()
        } else {
            return Err("Unexpected end of input inside parentheses".to_string());
        };

        if let Ok(x) = self.current()
            && x.kind == TokenKind::COMMA
        {
            self.advance()?;

            let mut values = vec![first?];

            loop {
                self.skip_new_lines();
                if let Ok(x) = self.current()
                    && x.kind == TokenKind::RPAREN
                {
                    break;
                }

                values.push(self.parse_expression()?);

                self.skip_new_lines();

                if let Ok(x) = self.current()
                    && x.kind == TokenKind::COMMA
                {
                    self.advance()?;
                } else {
                    break;
                }
            }

            self.expect_and_consume(TokenKind::RPAREN)?;

            return Ok(Node::TupleNode(values));
        } else {
            self.expect_and_consume(TokenKind::RPAREN)?;
            first
        }
    }

    fn parse_list(&mut self) -> NodeResult {
        self.advance()?;

        let mut values = vec![];

        loop {
            self.skip_new_lines();

            if let Ok(next) = self.current() {
                if next.kind == TokenKind::RBRACK {
                    break;
                }
            } else {
                return Err(format!(
                    "Unexpected end of input [EOF] while parsing list. Expected `]`."
                ));
            }

            values.push(self.parse_expression()?);

            if let Ok(next) = self.current()
                && next.kind == TokenKind::COMMA
            {
                self.advance()?;
            } else {
                break;
            }
        }

        self.advance()?;

        Ok(Node::ListNode(values))
    }

    fn parse_dict(&mut self) -> NodeResult {
        let mut data = vec![];

        self.parse_surrounded(TokenKind::LBRACE, TokenKind::RBRACE, Some(TokenKind::COMMA), |this| {
			// USE parse_ternary_op BECAUSE PARSE_EXPRESSION IS TOO HIGH LEVEL
            let key_base = this.parse_ternary_op()?;
            this.skip_new_lines();

            let (key, value) = if let Ok(next) = this.current()
                && (next.kind == TokenKind::COMMA || next.kind == TokenKind::RBRACE)
            {
                if let Node::Variable(ref x) = key_base {
                    (Node::StringLiteral(x.to_string()), key_base)
                } else {
                    return Err(format!(
                        "Can only use variables in dict field init shorthand."
                    ));
                }
            } else if let Ok(next) = this.current()
                && next.kind == TokenKind::EQUAL
            {
                this.advance();
                if let Node::Variable(x) = key_base {
                    (Node::StringLiteral(x.to_string()), this.parse_expression()?)
                } else {
                    return Err(format!(
                        "Expected identifier when parsing dict key. Have you tried using a colon (:)?"
                    ));
                }
            } else {
                this.expect_and_consume(TokenKind::COLON)?;
                if let Node::Variable(x) = key_base {
                    (Node::StringLiteral(x.to_string()), this.parse_expression()?)
                } else {
                    unreachable!()
                }
            };

            data.push((key, value));

			Ok(())
		})?;

        Ok(Node::DictNode(data))
    }

    fn parse_struct_init(&mut self, target: Node) -> NodeResult {
        let mut data = vec![];

        if self.pos > 0 {
            if !matches!(
                self.tokens[self.pos as usize - 1].kind,
                TokenKind::Identifier
            ) {
                return Ok(target);
            }
        }

        self.parse_surrounded(
            TokenKind::LBRACE,
            TokenKind::RBRACE,
            Some(TokenKind::COMMA),
            |this| {

			// USE parse_ternary_op BECAUSE PARSE_EXPRESSION IS TOO HIGH LEVEL
            let key_base = this.parse_ternary_op()?;
            this.skip_new_lines();

            let (key, value) = if let Ok(next) = this.current()
                && (next.kind == TokenKind::COMMA || next.kind == TokenKind::RBRACE)
            {
                if let Node::Variable(ref x) = key_base {
                    (x.to_string(), key_base)
                } else {
                    return Err(format!(
                        "Can only use variables in struct field field init shorthand."
                    ));
                }
            } else {
				this.advance();
                if let Node::Variable(x) = key_base {
                    (x.to_string(), this.parse_expression()?)
                } else {
                    return Err(format!(
                        "Expected identifier when parsing struct field key. Have you tried using a colon (:)?"
                    ));
                }
            };

            data.push((key, value));

			Ok(())
	 } )?;

        Ok(Node::StructInit {
            target: Box::new(target),
            fields: data,
        })
    }

    fn parse_expression(&mut self) -> NodeResult {
        self.skip_new_lines();

        let expr = self.parse_ternary_op()?;

        self.skip_new_lines();

        if let Ok(next) = self.current() {
            match next.kind {
                TokenKind::LBRACE => return self.parse_struct_init(expr),
                TokenKind::EQUAL => return self.parse_set_variable(expr),

                TokenKind::ADD_SH
                | TokenKind::SUB_SH
                | TokenKind::MUL_SH
                | TokenKind::DIV_SH
                | TokenKind::MOD_SH
                | TokenKind::POW_SH => {
                    return self.parse_shorthand_assignment(expr, next.kind);
                }

                _ => {}
            }
        }

        Ok(expr)
    }

    fn parse_ternary_op(&mut self) -> NodeResult {
        let mut condition = self.parse_elvis_coalescing()?;

        while let Ok(next) = self.current() {
            if next.kind != TokenKind::QUESTION {
                break;
            }

            self.advance()?;
            let true_expr = self.parse_elvis_coalescing()?;

            if let Ok(next) = self.current()
                && next.kind != TokenKind::COLON
            {
                break;
            }

            self.advance()?;
            let false_expr = self.parse_elvis_coalescing()?;

            condition = Node::TernaryOp {
                condition: Box::new(condition),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            };
        }

        Ok(condition)
    }

    fn parse_elvis_coalescing(&mut self) -> NodeResult {
        let mut left = self.parse_null_coalescing()?;

        while let Ok(next) = self.current() {
            if next.kind != TokenKind::ELVIS {
                break;
            }

            self.advance()?;
            let right = self.parse_null_coalescing()?;

            left = Node::ElvisCoalesce {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_null_coalescing(&mut self) -> NodeResult {
        let mut left = self.parse_logical()?;

        while let Ok(next) = self.current() {
            if next.kind != TokenKind::DOUBLEQUESTION {
                break;
            }

            self.advance()?;
            let right = self.parse_logical()?;

            left = Node::NullCoalesce {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical(&mut self) -> NodeResult {
        self.parse_or()
    }

    fn parse_or(&mut self) -> NodeResult {
        let mut left = self.parse_and()?;

        while let Ok(next) = self.current() {
            if next.kind != TokenKind::OR {
                break;
            }

            let op = self.advance()?.kind;
            let right = self.parse_and()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> NodeResult {
        let mut left = self.parse_equality()?;

        while let Ok(next) = self.current() {
            if next.kind != TokenKind::AND {
                break;
            }

            let op = self.advance()?.kind;
            let right = self.parse_equality()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
    }

    fn parse_equality(&mut self) -> NodeResult {
        let mut left = self.parse_comparison()?;

        while let Ok(next) = self.current() {
            if !matches!(next.kind, TokenKind::EQ | TokenKind::NEQ | TokenKind::IS) {
                break;
            }

            let op = self.advance()?.kind;
            let right = self.parse_comparison()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> NodeResult {
        let first = self.parse_range()?;

        let mut expressions = vec![first];
        let mut operators = vec![];

        while let Ok(next) = self.current() {
            if !matches!(
                next.kind,
                TokenKind::LT | TokenKind::LE | TokenKind::GT | TokenKind::GE
            ) {
                break;
            }

            operators.push(self.advance()?.kind);
            expressions.push(self.parse_range()?);
        }

        if operators.is_empty() {
            return Ok(expressions.remove(0));
        }

        Ok(Node::ComparisonChain {
            expressions,
            operators,
        })
    }

    fn parse_function_call(&mut self, expr: Node) -> NodeResult {
        self.advance()?; // consume LPAREN

        let mut args = vec![];

        loop {
            self.skip_new_lines();

            let next = match self.current() {
                Ok(x) => x,
                Err(_) => {
                    return {
                        Err(format!(
                            "Unexpected end of input in function call. Expected `)`."
                        ))
                    };
                }
            };

            if next.kind == TokenKind::RPAREN {
                break;
            }

            args.push(self.parse_expression()?);

            if let Ok(next) = self.current()
                && next.kind == TokenKind::COMMA
            {
                self.advance()?;
                continue;
            } else {
                break;
            }
        }

        self.expect_and_consume(TokenKind::RPAREN)?;

        Ok(Node::FunctionCall {
            target: Box::new(expr),
            args,
        })
    }

    fn parse_member_access(&mut self, expr: Node) -> NodeResult {
        let x = self.advance()?;

        if x.kind == TokenKind::DOT || x.kind == TokenKind::DOUBLECOLON {
            let member = self.expect_and_consume(TokenKind::Identifier)?;

            return Ok(Node::MemberAccess {
                expr: Box::new(expr),
                member: Box::new(Node::Symbol(
                    member.get_text(&self.source).to_string(),
                )),
            });
        } else if x.kind == TokenKind::LBRACK {
            let member = self.parse_expression()?;
            self.expect_and_consume(TokenKind::RBRACK)?;

            return Ok(Node::MemberAccess {
                expr: Box::new(expr),
                member: Box::new(member),
            });
        }

        panic!("Unknown member access parse token")
    }
}

// STATEMENTS
impl Parser {
    fn parse_const(&mut self) -> NodeResult {
        if let Some(x) = self.peek()
            && x.kind == TokenKind::FN
        {
            self.advance()?;
            self.parse_function_def(false, true)
        } else {
            self.parse_let(true)
        }
    }

    fn parse_let(&mut self, is_const: bool) -> NodeResult {
        self.advance()?;

        let mut names = vec![];

        loop {
            self.skip_new_lines();

            if let Ok(x) = self.current()
                && x.kind == TokenKind::EQUAL
            {
                break;
            }

            names.push(rc!(self
                .expect_and_consume(TokenKind::Identifier)?
                .get_text(&self.source)
                .to_string()));

            self.skip_new_lines();

            if let Ok(x) = self.current()
                && x.kind == TokenKind::COLON
            {
                self.advance()?;
                self.expect_and_consume(TokenKind::Identifier)?;
            }

            self.skip_new_lines();

            if let Ok(x) = self.current()
                && x.kind == TokenKind::COMMA
            {
                self.advance();
            } else {
                self.skip_new_lines();
                break;
            }
        }

        let mut values = vec![];

        if let Ok(next) = self.current()
            && next.kind == TokenKind::EQUAL
        {
            self.advance()?;

            for i in names.iter() {
                values.push(Some(Box::new(self.parse_expression()?)));

                if let Ok(next) = self.current()
                    && next.kind == TokenKind::COMMA
                {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(Node::LetStatement {
            names,
            values,
            is_const,
        })
    }

    fn parse_using(&mut self) -> NodeResult {
        self.advance()?;
        self.skip_new_lines();

        let mut sequence = vec![];
        let mut wildcard = false;
        let mut imports = vec![];

        loop {
            if let Ok(next) = self.current()
                && next.kind == TokenKind::LBRACE
            {
                self.advance()?;
                loop {
                    let token = self.expect_and_consume(TokenKind::Identifier)?;
                    let mut alias = None;

                    if let Ok(next) = self.current()
                        && next.kind == TokenKind::AS
                    {
                        self.advance()?;
                        let new_name = self.expect_and_consume(TokenKind::Identifier)?;

                        alias = Some(new_name.get_text(&self.source));
                    }

                    imports.push((token.get_text(&self.source), alias));

                    if let Ok(next) = self.current()
                        && next.kind == TokenKind::COMMA
                    {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
                self.expect_and_consume(TokenKind::RBRACE)?;
            } else {
                let token = self.expect_and_consume(TokenKind::Identifier)?;
                sequence.push(token.get_text(&self.source));
            }

            if let Ok(next) = self.current() {
                if matches!(next.kind, TokenKind::NEWLINE | TokenKind::SEMI) {
                    self.advance()?;
                    break;
                } else if matches!(next.kind, TokenKind::DOUBLECOLON | TokenKind::DOT) {
                    self.advance()?;
                    if let Ok(next) = self.current()
                        && matches!(next.kind, TokenKind::STAR)
                    {
                        wildcard = true;
                        self.advance()?;
                        break;
                    }
                    continue;
                }
            }

            break;
        }

        Ok(Node::UsingStatement {
            sequence,
            imports,
            wildcard,
        })
    }

    fn parse_block(&mut self) -> NodeResult {
        if let Ok(x) = self.current()
            && x.kind == TokenKind::FATARROW
        {
            self.advance();
            return Ok(Node::SingleLineBlock {
                body: Box::new(self.parse_expression()?),
            });
        }

        self.expect_and_consume(TokenKind::LBRACE)?;

        let mut body = vec![];

        loop {
            self.skip_new_lines();

            let next = match self.current() {
                Ok(tok) => tok,
                Err(_) => return Err("Unexpected end of input inside block.".to_string()),
            };

            match next.kind {
                TokenKind::RBRACE => break,
                TokenKind::SEMI => {
                    self.advance()?;
                }

                _ => body.push(self.parse()?),
            }
        }

        self.expect_and_consume(TokenKind::RBRACE)?;

        self.parse_implicit_return(body.last_mut());

        Ok(Node::Block { body })
    }

    fn parse_function_def(&mut self, is_lambda: bool, is_const: bool) -> NodeResult {
        self.advance()?;
        self.skip_new_lines();

        let name = if is_lambda {
            None
        } else {
            Some(rc!(self
                .expect_and_consume(TokenKind::Identifier)?
                .get_text(&self.source)
                .to_string()))
        };

        let mut args = vec![];

        self.parse_surrounded(
            TokenKind::LPAREN,
            TokenKind::RPAREN,
            Some(TokenKind::COMMA),
            |this| {
                let arg_name = this
                    .expect_and_consume(TokenKind::Identifier)?
                    .get_text(&this.source)
                    .to_string();

                let arg_type = if let Ok(next) = this.current()
                    && next.kind == TokenKind::COLON
                {
                    this.expect_and_consume(TokenKind::COLON)?;
                    Some(rc!(this
                        .expect_and_consume(TokenKind::Identifier)?
                        .get_text(&this.source)
                        .to_string()))
                } else {
                    None
                };

                let mut default_value = None;
                if let Ok(x) = this.current()
                    && x.kind == TokenKind::EQUAL
                {
                    this.advance();
                    default_value = Some(this.parse_expression()?);
                }

                args.push((rc!(arg_name), arg_type, default_value));

                Ok(())
            },
        )?;

        let return_type = if let Ok(next) = self.current()
            && next.kind == TokenKind::ARROW
        {
            self.advance()?;
            Some(rc!(self
                .expect_and_consume(TokenKind::Identifier)?
                .get_text(&self.source)
                .to_string()))
        } else {
            None
        };

        Ok(Node::FunctionDefinition {
            name,
            args,
            return_type,
            is_const,
            block: Box::new(self.parse_block()?),
        })
    }

    fn parse_return(&mut self) -> NodeResult {
        self.advance()?;

        if let Ok(next) = self.current()
            && !matches!(next.kind, TokenKind::NEWLINE | TokenKind::SEMI)
        {
            Ok(Node::ReturnStatement(Some(Box::new(
                self.parse_expression()?,
            ))))
        } else {
            Ok(Node::ReturnStatement(None))
        }
    }
    fn parse_break(&mut self) -> NodeResult {
        self.advance()?;

        if let Ok(next) = self.current()
            && !matches!(next.kind, TokenKind::NEWLINE | TokenKind::SEMI)
        {
            Ok(Node::BreakStatement(Some(Box::new(
                self.parse_expression()?,
            ))))
        } else {
            Ok(Node::BreakStatement(None))
        }
    }
    fn parse_out(&mut self) -> NodeResult {
        self.advance()?;

        if let Ok(next) = self.current()
            && !matches!(next.kind, TokenKind::NEWLINE | TokenKind::SEMI)
        {
            Ok(Node::OutStatement(Some(Box::new(self.parse_expression()?))))
        } else {
            Ok(Node::OutStatement(None))
        }
    }

    fn parse_if(&mut self) -> NodeResult {
        self.advance()?;

        let condition = self.parse_expression()?;
        let main_block = self.parse_block()?;
        let mut elifs = vec![];
        let mut else_block = None;

        loop {
            if let Ok(next) = self.current()
                && next.kind == TokenKind::ELSE
            {
                self.advance()?;

                if let Ok(next) = self.current()
                    && next.kind == TokenKind::IF
                {
                    self.advance()?;

                    let elif_condition = self.parse_expression()?;
                    let elif_block = self.parse_block()?;

                    elifs.push((elif_condition, elif_block));
                } else {
                    else_block = Some(Box::new(self.parse_block()?));
                }
            } else {
                break;
            }
        }

        Ok(Node::IfStatement {
            condition: Box::new(condition),
            block: Box::new(main_block),
            elifs,
            else_block,
        })
    }

    /// Just advance and return whatever you want.
    fn simple_parse_keyword(&mut self, node: Node) -> NodeResult {
        self.advance()?;
        Ok(node)
    }

    fn parse_loop(&mut self) -> NodeResult {
        self.advance()?;

        Ok(Node::Loop {
            block: Box::new(self.parse_block()?),
        })
    }

    fn parse_while(&mut self) -> NodeResult {
        self.advance()?;

        Ok(Node::WhileLoop {
            condition: Box::new(self.parse_ternary_op()?),
            block: Box::new(self.parse_block()?),
        })
    }

    fn parse_for(&mut self) -> NodeResult {
        self.advance()?;

        let var_name = rc!(self
            .expect_and_consume(TokenKind::Identifier)?
            .get_text(&self.source)
            .to_string());

        self.expect_and_consume(TokenKind::IN)?;

        let expr = Box::new(self.parse_ternary_op()?);
        let block = Box::new(self.parse_block()?);

        Ok(Node::ForLoop {
            var_name,
            expr,
            block,
        })
    }

    fn parse_class_def(&mut self) -> NodeResult {
        self.advance()?;
        self.skip_new_lines();

        let name = self
            .expect_and_consume(TokenKind::Identifier)?
            .get_text(&self.source)
            .to_string();

        self.skip_new_lines();

        let mut interfaces = vec![];
        if let Ok(next) = self.current()
            && next.kind == TokenKind::COLON
        {
            self.expect_and_consume(TokenKind::COLON)?;
            interfaces.push(rc!(self
                .expect_and_consume(TokenKind::Identifier)?
                .get_text(&self.source)
                .to_string()));

            loop {
                self.skip_new_lines();

                if let Ok(next) = self.current() {
                    if next.kind == TokenKind::LBRACE {
                        break;
                    }
                } else {
                    return Err(
                        "Unexpected end of input while parsing class interfaces.".to_string()
                    );
                }

                self.expect_and_consume(TokenKind::COMMA)?;

                interfaces.push(rc!(self
                    .expect_and_consume(TokenKind::Identifier)?
                    .get_text(&self.source)
                    .to_string()));
            }
        }

        self.expect_and_consume(TokenKind::LBRACE)?;

        let mut let_statements = vec![];
        let mut functions = vec![];
        let mut constructor = None;

        loop {
            self.skip_new_lines();

            if let Ok(next) = self.current() {
                if next.kind == TokenKind::RBRACE {
                    break;
                }
            } else {
                return Err("Unexpected end of input while parsing class.".to_string());
            }

            if let Ok(next) = self.current() {
                match next.kind {
                    TokenKind::LET => let_statements.push(self.parse_let(false)?),
                    TokenKind::CONST => let_statements.push(self.parse_let(true)?),
                    TokenKind::CONSTRUCTOR => {
                        constructor = Some(Box::new(self.parse_function_def(true, false)?))
                    }
                    TokenKind::FN => functions.push(self.parse_function_def(false, false)?),
                    TokenKind::SEMI => {
                        self.advance()?;
                    }

                    _ => {
                        return Err(format!(
                            "Class definitions can only take functions or let statements."
                        ));
                    }
                }
            }
        }

        self.expect_and_consume(TokenKind::RBRACE)?;

        Ok(Node::ClassDef {
            name,
            interfaces,
            let_statements,
            functions,
            constructor,
        })
    }

    fn parse_new(&mut self) -> NodeResult {
        self.advance()?;

        let target = self.parse_member()?;
        let mut parameters = vec![];

        self.parse_surrounded(
            TokenKind::LPAREN,
            TokenKind::RPAREN,
            Some(TokenKind::COMMA),
            |this| {
                parameters.push(this.parse_expression()?);
                Ok(())
            },
        )?;

        Ok(Node::ClassInit {
            target: Box::new(target),
            parameters,
        })
    }

    fn parse_range(&mut self) -> NodeResult {
        let left = self.parse_add_sub()?;

        if let Ok(x) = self.current()
            && x.kind == TokenKind::DOUBLEDOT
        {
            self.advance();
            self.skip_new_lines();

            let start = left;

            let inclusive = if let Ok(x) = self.current()
                && x.kind == TokenKind::EQUAL
            {
                self.advance();
                true
            } else {
                false
            };

            self.skip_new_lines();
            let end = self.parse_add_sub()?;

            let step = if let Ok(x) = self.current()
                && x.kind == TokenKind::DOUBLEDOT
            {
                self.advance();
                Some(Box::new(self.parse_add_sub()?))
            } else {
                None
            };

            self.skip_new_lines();
            return Ok(Node::RangeNode {
                start: Box::new(start),
                end: Box::new(end),
                step,
                inclusive,
            });
        }

        return Ok(left);
    }

    fn parse_implicit_return(&mut self, last: Option<&mut Node>) {
        if let Some(last) = last {
            if let Node::ExprStmt(expr) = last {
                let is_returnable = !matches!(
                    **expr,
                    Node::LetStatement { .. }
                        | Node::SetVariable { .. }
                        | Node::ShorthandAssignment { .. }
                        | Node::WhileLoop { .. }
                        | Node::ForLoop { .. }
                        | Node::Loop { .. }
                        | Node::OutStatement(..)
                        | Node::ReturnStatement(..)
                        | Node::BreakStatement(..)
                );

                if is_returnable {
                    *last = Node::ExprStmt(Box::new(Node::OutStatement(Some(expr.clone()))));
                }
            }
        }
    }

    fn parse_match(&mut self) -> NodeResult {
        self.advance()?;

        let expr = self.parse_ternary_op()?;

        let mut branches = vec![];

        self.parse_surrounded(
            TokenKind::LBRACE,
            TokenKind::RBRACE,
            Some(TokenKind::COMMA),
            |this| {
                let condition = this.parse_expression()?;
                this.skip_new_lines();
                this.expect_and_consume(TokenKind::FATARROW);

                let value = this.parse_expression()?;

                branches.push((condition, value));

                Ok(())
            },
        )?;

        Ok(Node::MatchStatement {
            expr: Box::new(expr),
            branches,
        })
    }

    fn parse_enum(&mut self) -> NodeResult {
        self.advance()?;

        let name = self
            .expect_and_consume(TokenKind::Identifier)?
            .get_text(&self.source);

        self.expect_and_consume(TokenKind::LBRACE)?;

        let mut items = vec![];
        let mut id = 0;

        self.parse_surrounded(
            TokenKind::LBRACE,
            TokenKind::RBRACE,
            Some(TokenKind::COMMA),
            |this| {
                let item_name = this
                    .expect_and_consume(TokenKind::Identifier)?
                    .get_text(&this.source);

                let item_value = if let Ok(next) = this.current()
                    && next.kind == TokenKind::EQUAL
                {
                    this.advance()?;
                    this.parse_expression()?
                } else {
                    let v = Node::NumberLiteral(id as f64);
                    id += 1;
                    v
                };

                items.push((item_name, item_value));

                Ok(())
            },
        )?;

        Ok(Node::EnumDef { name, items })
    }

    fn parse_struct_def(&mut self) -> NodeResult {
        self.advance()?;

        let name = self
            .expect_and_consume(TokenKind::Identifier)?
            .get_text(&self.source);

        let mut fields = vec![];

        self.parse_surrounded(
            TokenKind::LBRACE,
            TokenKind::RBRACE,
            Some(TokenKind::COMMA),
            |this| {
                let field_name = this
                    .expect_and_consume(TokenKind::Identifier)?
                    .get_text(&this.source);

                this.expect_and_consume(TokenKind::COLON);

                let field_type = this
                    .expect_and_consume(TokenKind::Identifier)?
                    .get_text(&this.source);

                fields.push((field_name, field_type));

                Ok(())
            },
        )?;

        Ok(Node::StructDef { name, fields })
    }
}
