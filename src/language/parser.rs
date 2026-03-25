#![allow(unused)]

use std::collections::HashMap;

use crate::{
    language::{
        nodes::Node,
        token::{Token, TokenKind},
    },
    rc,
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

    pub fn parse(&mut self) -> NodeResult {
        self.skip_new_lines();

        match self.current()?.kind {
            TokenKind::CLASS => self.parse_class_def(),
            TokenKind::FN => self.parse_function_def(false),

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
                right: Box::new(self.parse_exponent()?),
                is_prefix: true,
            });
        }

        let expr = self.parse_exponent()?;

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

    fn parse_exponent(&mut self) -> NodeResult {
        let mut left = self.parse_primary()?;

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

    fn parse_primary(&mut self) -> NodeResult {
        self.skip_new_lines();
        let start_pos = self.pos;
        let current = self.current()?;

        let node = match current.kind {
            TokenKind::NIL => Ok(Node::NIL),
            TokenKind::NumberLiteral(x) => Ok(Node::NumberLiteral(x)),
            TokenKind::BooleanLiteral(x) => Ok(Node::BooleanLiteral(x)),
            TokenKind::StringLiteral(_) => Ok(Node::StringLiteral({
                let text = current.get_text(&self.source);
                text[1..text.len() - 1].to_string()
            })),

            TokenKind::Identifier => Ok(Node::Variable(rc!(current
                .get_text(&self.source)
                .to_string()))),

            TokenKind::AT => self.parse_explicit_block(),

            // COLLECTIONS
            TokenKind::LPAREN => self.parse_tuple(),
            TokenKind::LBRACK => self.parse_list(),
            TokenKind::LBRACE => self.parse_dict(),

            TokenKind::LET => self.parse_let(false),
            TokenKind::CONST => self.parse_let(true),
            TokenKind::FN => self.parse_function_def(true),
            TokenKind::LOOP => self.parse_loop(),
            TokenKind::WHILE => self.parse_while(),
            TokenKind::FOR => self.parse_for(),
            TokenKind::RETURN => self.parse_return(),
            TokenKind::OUT => self.parse_out(),
            TokenKind::BREAK => self.parse_break(),
            TokenKind::CONTINUE => self.simple_parse_keyword(Node::ContinueStatement),
            TokenKind::IF => self.parse_if(),

            other => Err(format!(
                "Got unexpected token `{other:?}` while parsing primary."
            )),
        }?;

        if self.pos == start_pos {
            self.advance()?;
        }

        self.parse_postfix(node)
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
        self.advance()?;

        let mut data = vec![];

        loop {
            self.skip_new_lines();

            if let Ok(next) = self.current() {
                if next.kind == TokenKind::RBRACE {
                    break;
                }
            } else {
                return Err(format!(
                    "Unexpected end of input [EOF] while parsing dict. Expected `]`."
                ));
            }

            let key_base = self.parse_expression()?;
            self.skip_new_lines();

            let (key, value) = if let Ok(next) = self.current()
                && (next.kind == TokenKind::COMMA || next.kind == TokenKind::RBRACE)
            {
                if let Node::Variable(ref x) = key_base {
                    (Node::StringLiteral(x.to_string()), key_base)
                } else {
                    return Err(format!(
                        "Can only use variables in dict field init shorthand."
                    ));
                }
            } else if let Ok(next) = self.current()
                && next.kind == TokenKind::EQUAL
            {
                self.advance();
                if let Node::Variable(x) = key_base {
                    (Node::StringLiteral(x.to_string()), self.parse_expression()?)
                } else {
                    return Err(format!(
                        "Expected identifier when parsing dict key. Have you tried using a colon (:)?"
                    ));
                }
            } else {
                self.expect_and_consume(TokenKind::COLON)?;
                (key_base, self.parse_expression()?)
            };

            data.push((key, value));

            if let Ok(next) = self.current()
                && next.kind == TokenKind::COMMA
            {
                self.advance()?;
            } else {
                break;
            }
        }

        self.advance()?;

        Ok(Node::DictNode(data))
    }

    fn parse_expression(&mut self) -> NodeResult {
        self.skip_new_lines();

        let expr = self.parse_logical()?;

        self.skip_new_lines();

        if let Ok(next) = self.current() {
            match next.kind {
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

    fn parse_postfix(&mut self, mut expr: Node) -> NodeResult {
        loop {
            if let Ok(x) = self.current() {
                match x.kind {
                    TokenKind::LPAREN => {
                        expr = self.parse_function_call(expr)?;
                        continue;
                    }
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
            if !matches!(next.kind, TokenKind::EQ | TokenKind::NEQ) {
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
        let mut left = self.parse_range()?;

        while let Ok(next) = self.current() {
            if !matches!(
                next.kind,
                TokenKind::LT | TokenKind::LE | TokenKind::GT | TokenKind::GE
            ) {
                break;
            }

            let op = self.advance()?.kind;
            let right = self.parse_range()?;

            left = Node::BinOp {
                left: Box::new(left),
                right: Box::new(right),
                op,
            };
        }

        Ok(left)
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
                member: Box::new(Node::StringLiteral(
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
                && x.kind == TokenKind::COMMA
            {
                self.advance();
            } else {
                self.skip_new_lines();
                break;
            }
        }

        self.expect_and_consume(TokenKind::EQUAL)?;

        let mut values = vec![];

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

        if values.len() < names.len() {
            for i in 0..(names.len() - values.len()) {
                values.push(None);
            }
        }

        Ok(Node::LetStatement {
            names,
            values,
            is_const,
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

        Ok(Node::Block { body })
    }

    fn parse_function_def(&mut self, is_lambda: bool) -> NodeResult {
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

        self.skip_new_lines();
        self.expect_and_consume(TokenKind::LPAREN)?;

        let mut args = vec![];

        loop {
            self.skip_new_lines();

            if let Ok(next) = self.current() {
                if next.kind == TokenKind::RPAREN {
                    break;
                }
            } else {
                return Err(format!(
                    "Unexpected end of input while parsing function arguments."
                ));
            }

            let arg_name = self
                .expect_and_consume(TokenKind::Identifier)?
                .get_text(&self.source)
                .to_string();

            let arg_type = if let Ok(next) = self.current()
                && next.kind == TokenKind::COLON
            {
                self.expect_and_consume(TokenKind::COLON)?;
                Some(rc!(self
                    .expect_and_consume(TokenKind::Identifier)?
                    .get_text(&self.source)
                    .to_string()))
            } else {
                None
            };

            let mut default_value = None;
            if let Ok(x) = self.current()
                && x.kind == TokenKind::EQUAL
            {
                self.advance();
                default_value = Some(self.parse_expression()?);
            }

            args.push((rc!(arg_name), arg_type, default_value));

            self.skip_new_lines();
            if let Ok(next) = self.current()
                && next.kind == TokenKind::COMMA
            {
                self.advance()?;
            } else {
                break;
            }
        }

        self.expect_and_consume(TokenKind::RPAREN)?;

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
            condition: Box::new(self.parse_expression()?),
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

        let expr = Box::new(self.parse_expression()?);
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
                    TokenKind::FN => functions.push(self.parse_function_def(false)?),
                    TokenKind::LET => let_statements.push(self.parse_let(false)?),
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
}
