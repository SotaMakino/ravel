use crate::lexer::token::Token;
use crate::parser::ast::{AstNode, BinaryOp, UnaryOp, VarKind};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(mut self) -> Result<AstNode, String> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.statement()?);
        }
        Ok(AstNode::Block(stmts))
    }

    fn statement(&mut self) -> Result<AstNode, String> {
        match self.peek() {
            Token::Let => self.var_decl(VarKind::Let),
            Token::Const => self.var_decl(VarKind::Const),
            Token::Var => self.var_decl(VarKind::Var),
            Token::If => self.if_stmt(),
            Token::While => self.while_stmt(),
            Token::For => self.for_stmt(),
            Token::Return => self.return_stmt(),
            Token::Func => self.func_decl(),
            Token::LBrace => self.block_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn var_decl(&mut self, kind: VarKind) -> Result<AstNode, String> {
        self.advance();
        let name = match self.advance() {
            Token::Ident(n) => n,
            t => return Err(format!("Expected identifier, got {}", t)),
        };

        let init = if self.match_token(&Token::Eq) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.consume_semicolon()?;
        Ok(AstNode::VarDecl { name, kind, init })
    }

    fn if_stmt(&mut self) -> Result<AstNode, String> {
        self.advance();
        self.consume(&Token::LParen)?;
        let cond = self.expression()?;
        self.consume(&Token::RParen)?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&Token::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(AstNode::If {
            cond: Box::new(cond),
            then_branch,
            else_branch,
        })
    }

    fn while_stmt(&mut self) -> Result<AstNode, String> {
        self.advance();
        self.consume(&Token::LParen)?;
        let cond = self.expression()?;
        self.consume(&Token::RParen)?;
        let body = Box::new(self.statement()?);
        Ok(AstNode::While {
            cond: Box::new(cond),
            body,
        })
    }

    fn for_stmt(&mut self) -> Result<AstNode, String> {
        self.advance();
        self.consume(&Token::LParen)?;

        let init = if self.match_token(&Token::Semicolon) {
            None
        } else {
            let stmt = if self.matches(&[Token::Let, Token::Const, Token::Var]) {
                let kind = match self.advance() {
                    Token::Let => VarKind::Let,
                    Token::Const => VarKind::Const,
                    Token::Var => VarKind::Var,
                    _ => unreachable!(),
                };
                let name = match self.advance() {
                    Token::Ident(n) => n,
                    t => return Err(format!("Expected identifier, got {}", t)),
                };
                let init = if self.match_token(&Token::Eq) {
                    Some(Box::new(self.expression()?))
                } else {
                    None
                };
                AstNode::VarDecl { name, kind, init }
            } else {
                self.expression()?
            };
            self.consume(&Token::Semicolon)?;
            Some(Box::new(stmt))
        };

        let cond = if self.match_token(&Token::Semicolon) {
            None
        } else {
            let expr = self.expression()?;
            self.consume(&Token::Semicolon)?;
            Some(Box::new(expr))
        };

        let update = if self.match_token(&Token::RParen) {
            None
        } else {
            let expr = self.expression()?;
            self.consume(&Token::RParen)?;
            Some(Box::new(expr))
        };

        let body = Box::new(self.statement()?);
        Ok(AstNode::For {
            init,
            cond,
            update,
            body,
        })
    }

    fn return_stmt(&mut self) -> Result<AstNode, String> {
        self.advance();
        if self.match_token(&Token::Semicolon) || self.is_at_end() || self.matches(&[Token::RBrace])
        {
            Ok(AstNode::Return(None))
        } else {
            let expr = self.expression()?;
            self.consume_semicolon_optional();
            Ok(AstNode::Return(Some(Box::new(expr))))
        }
    }

    fn func_decl(&mut self) -> Result<AstNode, String> {
        self.advance();
        let name = match self.advance() {
            Token::Ident(n) => n,
            t => return Err(format!("Expected function name, got {}", t)),
        };

        self.consume(&Token::LParen)?;
        let mut params = Vec::new();
        if !self.match_token(&Token::RParen) {
            loop {
                let p = match self.advance() {
                    Token::Ident(n) => n,
                    t => return Err(format!("Expected parameter, got {}", t)),
                };
                params.push(p);
                if self.match_token(&Token::Comma) {
                    continue;
                }
                break;
            }
            self.consume(&Token::RParen)?;
        }

        let body = self.block()?;
        Ok(AstNode::FuncDecl {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn block_stmt(&mut self) -> Result<AstNode, String> {
        self.block()
    }

    fn block(&mut self) -> Result<AstNode, String> {
        self.consume(&Token::LBrace)?;
        let mut stmts = Vec::new();
        while !self.is_at_end() && !self.matches(&[Token::RBrace]) {
            stmts.push(self.statement()?);
        }
        self.consume(&Token::RBrace)?;
        Ok(AstNode::Block(stmts))
    }

    fn expr_stmt(&mut self) -> Result<AstNode, String> {
        let expr = self.expression()?;
        self.consume_semicolon_optional();
        Ok(AstNode::Expr(Box::new(expr)))
    }

    fn expression(&mut self) -> Result<AstNode, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<AstNode, String> {
        let expr = self.or()?;

        if self.match_token(&Token::Eq) {
            let name = match &expr {
                AstNode::Ident(n) => n.clone(),
                AstNode::MemberAccess {
                    object: _,
                    property: _,
                } => {
                    return Ok(AstNode::Assign {
                        name: format!("[member]"),
                        value: Box::new(self.assignment()?),
                    });
                }
                _ => return Err("Invalid assignment target".into()),
            };
            let value = Box::new(self.assignment()?);
            return Ok(AstNode::Assign { name, value });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<AstNode, String> {
        let mut left = self.and()?;
        while self.match_token(&Token::PipePipe) {
            let right = self.and()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<AstNode, String> {
        let mut left = self.equality()?;
        while self.match_token(&Token::AmpAmp) {
            let right = self.equality()?;
            left = AstNode::BinaryOp {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn equality(&mut self) -> Result<AstNode, String> {
        let mut left = self.comparison()?;
        loop {
            let op = if self.match_token(&Token::EqEq) {
                Some(BinaryOp::Eq)
            } else if self.match_token(&Token::EqEqEq) {
                Some(BinaryOp::StrictEq)
            } else if self.match_token(&Token::BangEq) {
                Some(BinaryOp::NotEq)
            } else if self.match_token(&Token::BangEqEq) {
                Some(BinaryOp::StrictNotEq)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.comparison()?;
                left = AstNode::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn comparison(&mut self) -> Result<AstNode, String> {
        let mut left = self.term()?;
        loop {
            let op = if self.match_token(&Token::Lt) {
                Some(BinaryOp::Lt)
            } else if self.match_token(&Token::Gt) {
                Some(BinaryOp::Gt)
            } else if self.match_token(&Token::LtEq) {
                Some(BinaryOp::LtEq)
            } else if self.match_token(&Token::GtEq) {
                Some(BinaryOp::GtEq)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.term()?;
                left = AstNode::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<AstNode, String> {
        let mut left = self.factor()?;
        loop {
            let op = if self.match_token(&Token::Plus) {
                Some(BinaryOp::Add)
            } else if self.match_token(&Token::Minus) {
                Some(BinaryOp::Sub)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.factor()?;
                left = AstNode::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<AstNode, String> {
        let mut left = self.unary()?;
        loop {
            let op = if self.match_token(&Token::Star) {
                Some(BinaryOp::Mul)
            } else if self.match_token(&Token::Slash) {
                Some(BinaryOp::Div)
            } else if self.match_token(&Token::Percent) {
                Some(BinaryOp::Mod)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.unary()?;
                left = AstNode::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<AstNode, String> {
        if self.match_token(&Token::Bang) {
            let operand = self.unary()?;
            return Ok(AstNode::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            });
        }
        if self.match_token(&Token::Minus) {
            let operand = self.unary()?;
            return Ok(AstNode::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<AstNode, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&Token::LParen) {
                let mut args = Vec::new();
                if !self.match_token(&Token::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::RParen)?;
                }
                expr = AstNode::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.match_token(&Token::LBracket) {
                let index = self.expression()?;
                self.consume(&Token::RBracket)?;
                expr = AstNode::IndexAccess {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::Dot) {
                let prop = match self.advance() {
                    Token::Ident(n) => n,
                    t => return Err(format!("Expected property name, got {}", t)),
                };
                expr = AstNode::MemberAccess {
                    object: Box::new(expr),
                    property: prop,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<AstNode, String> {
        match self.advance() {
            Token::Number(n) => Ok(AstNode::NumberLit(n)),
            Token::Str(s) => Ok(AstNode::StrLit(s)),
            Token::True => Ok(AstNode::BoolLit(true)),
            Token::False => Ok(AstNode::BoolLit(false)),
            Token::Null => Ok(AstNode::NullLit),
            Token::Undefined => Ok(AstNode::UndefinedLit),
            Token::Ident(s) => Ok(AstNode::Ident(s)),

            Token::LParen => {
                let expr = self.expression()?;
                self.consume(&Token::RParen)?;
                Ok(expr)
            }

            Token::LBracket => {
                let mut items = Vec::new();
                if !self.match_token(&Token::RBracket) {
                    loop {
                        items.push(self.expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::RBracket)?;
                }
                Ok(AstNode::ArrayLit(items))
            }

            Token::LBrace => {
                let mut entries = Vec::new();
                if !self.match_token(&Token::RBrace) {
                    loop {
                        let key = match self.advance() {
                            Token::Ident(s) => s,
                            Token::Str(s) => s,
                            t => return Err(format!("Expected object key, got {}", t)),
                        };
                        self.consume(&Token::Colon)?;
                        let value = self.expression()?;
                        entries.push((key, value));
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                    self.consume(&Token::RBrace)?;
                }
                Ok(AstNode::ObjectLit(entries))
            }

            t => Err(format!("Unexpected token: {}", t)),
        }
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        t
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len() || matches!(self.tokens[self.pos], Token::Eof)
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if !self.is_at_end() && &self.tokens[self.pos] == token {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn matches(&self, tokens: &[Token]) -> bool {
        if self.is_at_end() {
            return false;
        }
        tokens.iter().any(|t| t == &self.tokens[self.pos])
    }

    fn consume(&mut self, expected: &Token) -> Result<(), String> {
        if self.match_token(expected) {
            Ok(())
        } else {
            Err(format!("Expected {}, got {}", expected, self.peek()))
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), String> {
        if !self.is_at_end() && self.match_token(&Token::Semicolon) {
            return Ok(());
        }
        Err(format!("Expected ';', got {}", self.peek()))
    }

    fn consume_semicolon_optional(&mut self) {
        if !self.is_at_end() {
            self.match_token(&Token::Semicolon);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lexer::Lexer;

    fn parse(source: &str) -> AstNode {
        let tokens = Lexer::new(source).tokenize().unwrap();
        Parser::new(tokens).parse().unwrap()
    }

    #[test]
    fn test_number_literal() {
        let ast = parse("42;");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                assert_eq!(stmts[0], AstNode::Expr(Box::new(AstNode::NumberLit(42.0))));
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_var_decl() {
        let ast = parse("let x = 10;");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                assert_eq!(
                    stmts[0],
                    AstNode::VarDecl {
                        name: "x".into(),
                        kind: VarKind::Let,
                        init: Some(Box::new(AstNode::NumberLit(10.0))),
                    }
                );
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_binary_op() {
        let ast = parse("1 + 2 * 3;");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::Expr(e) => match e.as_ref() {
                        AstNode::BinaryOp { left, op, right } => {
                            assert_eq!(*op, BinaryOp::Add);
                            assert_eq!(**left, AstNode::NumberLit(1.0));
                            match right.as_ref() {
                                AstNode::BinaryOp {
                                    left: l2,
                                    op: op2,
                                    right: r2,
                                } => {
                                    assert_eq!(*op2, BinaryOp::Mul);
                                    assert_eq!(**l2, AstNode::NumberLit(2.0));
                                    assert_eq!(**r2, AstNode::NumberLit(3.0));
                                }
                                _ => panic!("Expected binary op"),
                            }
                        }
                        _ => panic!("Expected binary op"),
                    },
                    _ => panic!("Expected expr"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_if_stmt() {
        let ast = parse("if (true) { 1; }");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::If {
                        cond,
                        then_branch: _,
                        else_branch,
                    } => {
                        assert_eq!(**cond, AstNode::BoolLit(true));
                        assert!(else_branch.is_none());
                    }
                    _ => panic!("Expected if"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_func_decl() {
        let ast = parse("function add(a, b) { return a + b; }");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::FuncDecl {
                        name,
                        params,
                        body: _,
                    } => {
                        assert_eq!(name, "add");
                        assert_eq!(params, &["a".to_string(), "b".to_string()]);
                    }
                    _ => panic!("Expected func decl"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_call_expr() {
        let ast = parse("console.log(42);");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::Expr(e) => match e.as_ref() {
                        AstNode::Call { callee, args } => {
                            match callee.as_ref() {
                                AstNode::MemberAccess { object, property } => {
                                    assert_eq!(**object, AstNode::Ident("console".into()));
                                    assert_eq!(property, "log");
                                }
                                _ => panic!("Expected member access"),
                            }
                            assert_eq!(args.len(), 1);
                            assert_eq!(args[0], AstNode::NumberLit(42.0));
                        }
                        _ => panic!("Expected call"),
                    },
                    _ => panic!("Expected expr"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_object_literal() {
        let ast = parse("({ x: 1, y: 2 });");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::Expr(e) => match e.as_ref() {
                        AstNode::ObjectLit(entries) => {
                            assert_eq!(entries.len(), 2);
                            assert_eq!(entries[0].0, "x");
                            assert_eq!(entries[0].1, AstNode::NumberLit(1.0));
                            assert_eq!(entries[1].0, "y");
                            assert_eq!(entries[1].1, AstNode::NumberLit(2.0));
                        }
                        _ => panic!("Expected object lit"),
                    },
                    _ => panic!("Expected expr"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_array_literal() {
        let ast = parse("[1, 2, 3];");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::Expr(e) => match e.as_ref() {
                        AstNode::ArrayLit(items) => {
                            assert_eq!(items.len(), 3);
                            assert_eq!(items[0], AstNode::NumberLit(1.0));
                            assert_eq!(items[1], AstNode::NumberLit(2.0));
                            assert_eq!(items[2], AstNode::NumberLit(3.0));
                        }
                        _ => panic!("Expected array lit"),
                    },
                    _ => panic!("Expected expr"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_while_loop() {
        let ast = parse("while (true) { 1; }");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::While { cond, body: _ } => {
                        assert_eq!(**cond, AstNode::BoolLit(true));
                    }
                    _ => panic!("Expected while"),
                }
            }
            _ => panic!("Expected block"),
        }
    }

    #[test]
    fn test_unary_not() {
        let ast = parse("!true;");
        match ast {
            AstNode::Block(stmts) => {
                assert_eq!(stmts.len(), 1);
                match &stmts[0] {
                    AstNode::Expr(e) => match e.as_ref() {
                        AstNode::UnaryOp { op, operand } => {
                            assert_eq!(*op, UnaryOp::Not);
                            assert_eq!(**operand, AstNode::BoolLit(true));
                        }
                        _ => panic!("Expected unary"),
                    },
                    _ => panic!("Expected expr"),
                }
            }
            _ => panic!("Expected block"),
        }
    }
}
