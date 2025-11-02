/// Condition parser - parses logical conditions for IF statements
///
/// Handles:
/// - AND(a < b, c > d)
/// - OR(a, b, c)
/// - NOT(x)
/// - Simple comparisons: a < b

use crate::dsl::ast::{ConditionAST, ExpressionAST};
use crate::dsl::errors::{DslError, DslResult};
use crate::dsl::expression_parser::ExpressionParser;
use crate::dsl::lexer::Token;

pub struct ConditionParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ConditionParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        ConditionParser { tokens, pos: 0 }
    }

    /// Parse a condition from a string
    pub fn parse_from_str(input: &str) -> DslResult<ConditionAST> {
        let tokens = crate::dsl::lexer::tokenize(input)
            .map_err(|e| DslError::ParseError {
                line: 0,
                column: 0,
                message: e,
            })?;

        let mut parser = ConditionParser::new(tokens);
        parser.parse()
    }

    /// Parse the condition
    pub fn parse(&mut self) -> DslResult<ConditionAST> {
        self.parse_logical_or()
    }

    // Grammar:
    // condition := logical_or
    // logical_or := logical_and (('||' | OR) logical_and)*
    // logical_and := logical_not (('&&' | AND) logical_not)*
    // logical_not := NOT logical_not | '!' logical_not | primary_condition
    // primary_condition := AND '(' condition_list ')' | OR '(' condition_list ')' | comparison | '(' condition ')'

    fn parse_logical_or(&mut self) -> DslResult<ConditionAST> {
        let mut left = self.parse_logical_and()?;

        while matches!(self.peek(), Some(Token::OrOr) | Some(Token::Or)) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = ConditionAST::Or(vec![left, right]);
        }

        Ok(left)
    }

    fn parse_logical_and(&mut self) -> DslResult<ConditionAST> {
        let mut left = self.parse_logical_not()?;

        while matches!(self.peek(), Some(Token::AndAnd) | Some(Token::And)) {
            // Check if this is infix && or prefix AND(
            if matches!(self.peek(), Some(Token::And)) {
                // Peek ahead to see if next token is '('
                if matches!(self.peek_ahead(1), Some(Token::LParen)) {
                    // This is prefix AND(...), stop parsing infix
                    break;
                }
            }

            self.advance();
            let right = self.parse_logical_not()?;
            left = ConditionAST::And(vec![left, right]);
        }

        Ok(left)
    }

    fn parse_logical_not(&mut self) -> DslResult<ConditionAST> {
        // Handle prefix NOT(...) or !
        if matches!(self.peek(), Some(Token::Not)) {
            self.advance();
            self.expect(Token::LParen)?;
            let operand = self.parse()?;
            self.expect(Token::RParen)?;
            return Ok(ConditionAST::Not(Box::new(operand)));
        }

        if matches!(self.peek(), Some(Token::Bang)) {
            self.advance();
            let operand = self.parse_logical_not()?;  // Allow chaining: !!x
            return Ok(ConditionAST::Not(Box::new(operand)));
        }

        self.parse_primary_condition()
    }

    fn parse_primary_condition(&mut self) -> DslResult<ConditionAST> {
        match self.peek() {
            Some(Token::And) => {
                self.advance();
                self.expect(Token::LParen)?;
                let operands = self.parse_condition_list()?;
                self.expect(Token::RParen)?;
                Ok(ConditionAST::And(operands))
            }
            Some(Token::Or) => {
                self.advance();
                self.expect(Token::LParen)?;
                let operands = self.parse_condition_list()?;
                self.expect(Token::RParen)?;
                Ok(ConditionAST::Or(operands))
            }
            Some(Token::LParen) => {
                self.advance();
                let cond = self.parse()?;
                self.expect(Token::RParen)?;
                Ok(cond)
            }
            _ => {
                // Parse as comparison or expression
                self.parse_comparison()
            }
        }
    }

    fn parse_condition_list(&mut self) -> DslResult<Vec<ConditionAST>> {
        let mut conditions = Vec::new();

        if matches!(self.peek(), Some(Token::RParen)) {
            return Ok(conditions);
        }

        loop {
            conditions.push(self.parse()?);

            if matches!(self.peek(), Some(Token::Comma)) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(conditions)
    }

    fn parse_comparison(&mut self) -> DslResult<ConditionAST> {
        // Save position to try parsing as expression first
        let start_pos = self.pos;

        // Try to parse as expression with comparison operator
        let expr = self.parse_expression_from_tokens()?;

        // Check if there's a comparison operator
        if let Some(op) = self.match_comparison_op() {
            let right = self.parse_expression_from_tokens()?;
            return Ok(ConditionAST::Comparison {
                op,
                left: expr,
                right,
            });
        }

        // No comparison operator, treat as boolean expression
        Ok(ConditionAST::Expression(expr))
    }

    fn parse_expression_from_tokens(&mut self) -> DslResult<ExpressionAST> {
        // Collect tokens until we hit a comparison operator or end
        let start = self.pos;
        let mut depth = 0;
        let mut end = start;

        while end < self.tokens.len() {
            match &self.tokens[end] {
                Token::LParen => depth += 1,
                Token::RParen if depth > 0 => depth -= 1,
                Token::RParen if depth == 0 => break,
                Token::Comma if depth == 0 => break,
                Token::Eq | Token::Neq | Token::Lt | Token::Gt | Token::Leq | Token::Geq
                    if depth == 0 =>
                {
                    break
                }
                _ => {}
            }
            end += 1;
        }

        if end == start {
            return Err(DslError::ParseError {
                line: 0,
                column: self.pos,
                message: "Expected expression".to_string(),
            });
        }

        let expr_tokens = self.tokens[start..end].to_vec();
        self.pos = end;

        let mut expr_parser = ExpressionParser::new(expr_tokens);
        expr_parser.parse()
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and() {
        let cond = ConditionParser::parse_from_str("AND(NDT.x < 0.5, NDT.y < 0.5)").unwrap();
        match cond {
            ConditionAST::And(operands) => {
                assert_eq!(operands.len(), 2);
            }
            _ => panic!("Expected AND condition"),
        }
    }

    #[test]
    fn test_parse_or() {
        let cond = ConditionParser::parse_from_str("OR(a < b, c > d)").unwrap();
        match cond {
            ConditionAST::Or(operands) => {
                assert_eq!(operands.len(), 2);
            }
            _ => panic!("Expected OR condition"),
        }
    }

    #[test]
    fn test_parse_simple_comparison() {
        let cond = ConditionParser::parse_from_str("actor.speed > 0").unwrap();
        match cond {
            ConditionAST::Comparison { op, .. } => {
                assert_eq!(op, ">");
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_not() {
        let cond = ConditionParser::parse_from_str("NOT(actor.dir.set)").unwrap();
        match cond {
            ConditionAST::Not(_) => {}
            _ => panic!("Expected NOT condition"),
        }
    }
}
