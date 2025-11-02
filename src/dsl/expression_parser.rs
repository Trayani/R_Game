/// Expression parser - converts tokens to AST
///
/// Implements recursive descent parsing for expressions like:
/// - distance_to_target()
/// - NDT.x + 10
/// - actor.pos.x

use crate::dsl::ast::{ExpressionAST, LiteralValue};
use crate::dsl::errors::{DslError, DslResult};
use crate::dsl::lexer::Token;

pub struct ExpressionParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExpressionParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        ExpressionParser { tokens, pos: 0 }
    }

    /// Parse an expression from a string
    pub fn parse_from_str(input: &str) -> DslResult<ExpressionAST> {
        let tokens = crate::dsl::lexer::tokenize(input)
            .map_err(|e| DslError::ParseError {
                line: 0,
                column: 0,
                message: e,
            })?;

        let mut parser = ExpressionParser::new(tokens);
        parser.parse()
    }

    /// Parse the expression
    pub fn parse(&mut self) -> DslResult<ExpressionAST> {
        self.parse_expression()
    }

    // Grammar (operator precedence, lowest to highest):
    // expression := comparison
    // comparison := additive (('==' | '!=' | '<' | '>' | '<=' | '>=') additive)?
    // additive := multiplicative (('+' | '-') multiplicative)*
    // multiplicative := unary (('*' | '/') unary)*
    // unary := ('!' | '-') unary | postfix
    // postfix := primary ('.' IDENTIFIER | '(' args ')')*
    // primary := LITERAL | IDENTIFIER | '(' expression ')'

    fn parse_expression(&mut self) -> DslResult<ExpressionAST> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> DslResult<ExpressionAST> {
        let left = self.parse_additive()?;

        if let Some(op) = self.match_comparison_op() {
            let right = self.parse_additive()?;
            return Ok(ExpressionAST::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    fn match_comparison_op(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Eq) => {
                self.advance();
                Some("==".to_string())
            }
            Some(Token::Neq) => {
                self.advance();
                Some("!=".to_string())
            }
            Some(Token::Lt) => {
                self.advance();
                Some("<".to_string())
            }
            Some(Token::Gt) => {
                self.advance();
                Some(">".to_string())
            }
            Some(Token::Leq) => {
                self.advance();
                Some("<=".to_string())
            }
            Some(Token::Geq) => {
                self.advance();
                Some(">=".to_string())
            }
            _ => None,
        }
    }

    fn parse_additive(&mut self) -> DslResult<ExpressionAST> {
        let mut left = self.parse_multiplicative()?;

        while matches!(self.peek(), Some(Token::Plus) | Some(Token::Minus)) {
            let op = match self.advance() {
                Token::Plus => "+",
                Token::Minus => "-",
                _ => unreachable!(),
            }
            .to_string();

            let right = self.parse_multiplicative()?;
            left = ExpressionAST::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> DslResult<ExpressionAST> {
        let mut left = self.parse_unary()?;

        while matches!(self.peek(), Some(Token::Star) | Some(Token::Slash)) {
            let op = match self.advance() {
                Token::Star => "*",
                Token::Slash => "/",
                _ => unreachable!(),
            }
            .to_string();

            let right = self.parse_unary()?;
            left = ExpressionAST::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> DslResult<ExpressionAST> {
        if matches!(self.peek(), Some(Token::Bang) | Some(Token::Minus)) {
            let op = match self.advance() {
                Token::Bang => "!",
                Token::Minus => "-",
                _ => unreachable!(),
            }
            .to_string();

            let operand = self.parse_unary()?;
            return Ok(ExpressionAST::UnaryOp {
                op,
                operand: Box::new(operand),
            });
        }

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> DslResult<ExpressionAST> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                Some(Token::Dot) => {
                    self.advance(); // consume '.'
                    let field = self.expect_identifier()?;
                    expr = ExpressionAST::FieldAccess {
                        object: Box::new(expr),
                        field,
                    };
                }
                Some(Token::LParen) if matches!(expr, ExpressionAST::Variable(_)) => {
                    // Function call
                    let name = match expr {
                        ExpressionAST::Variable(n) => n,
                        _ => unreachable!(),
                    };

                    self.advance(); // consume '('
                    let args = self.parse_args()?;
                    self.expect(Token::RParen)?;

                    expr = ExpressionAST::FunctionCall { name, args };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> DslResult<ExpressionAST> {
        match self.peek() {
            Some(Token::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                Ok(ExpressionAST::Variable(name))
            }
            Some(Token::Int(value)) => {
                let value = *value;
                self.advance();
                Ok(ExpressionAST::Literal(LiteralValue::Int(value)))
            }
            Some(Token::Float(value)) => {
                let value = *value;
                self.advance();
                Ok(ExpressionAST::Literal(LiteralValue::Float(value)))
            }
            Some(Token::True) => {
                self.advance();
                Ok(ExpressionAST::Literal(LiteralValue::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(ExpressionAST::Literal(LiteralValue::Bool(false)))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(ExpressionAST::Literal(LiteralValue::String(s)))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(DslError::ParseError {
                line: 0,
                column: self.pos,
                message: format!("Unexpected token: {:?}", self.peek()),
            }),
        }
    }

    fn parse_args(&mut self) -> DslResult<Vec<ExpressionAST>> {
        let mut args = Vec::new();

        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expression()?);

            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(args)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> DslResult<()> {
        let actual = self.peek();
        if actual == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(DslError::ParseError {
                line: 0,
                column: self.pos,
                message: format!("Expected {:?}, got {:?}", expected, actual),
            })
        }
    }

    fn expect_identifier(&mut self) -> DslResult<String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            other => Err(DslError::ParseError {
                line: 0,
                column: self.pos - 1,
                message: format!("Expected identifier, got {:?}", other),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function_call() {
        let expr = ExpressionParser::parse_from_str("distance_to_target()").unwrap();
        match expr {
            ExpressionAST::FunctionCall { name, args } => {
                assert_eq!(name, "distance_to_target");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        let expr = ExpressionParser::parse_from_str("NDT.x").unwrap();
        match expr {
            ExpressionAST::FieldAccess { object, field } => {
                assert!(matches!(*object, ExpressionAST::Variable(_)));
                assert_eq!(field, "x");
            }
            _ => panic!("Expected field access"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let expr = ExpressionParser::parse_from_str("NDT.x < 0.5").unwrap();
        match expr {
            ExpressionAST::BinaryOp { op, .. } => {
                assert_eq!(op, "<");
            }
            _ => panic!("Expected binary op"),
        }
    }

    #[test]
    fn test_parse_nested_field_access() {
        let expr = ExpressionParser::parse_from_str("actor.pos.x").unwrap();
        // Should be: FieldAccess(FieldAccess(Variable("actor"), "pos"), "x")
        match expr {
            ExpressionAST::FieldAccess { object, field } => {
                assert_eq!(field, "x");
                match *object {
                    ExpressionAST::FieldAccess { ref field, .. } => {
                        assert_eq!(field, "pos");
                    }
                    _ => panic!("Expected nested field access"),
                }
            }
            _ => panic!("Expected field access"),
        }
    }
}
