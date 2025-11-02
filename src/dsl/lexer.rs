/// Lexer for DSL expressions and conditions
///
/// Tokenizes strings like "distance_to_target()" and "AND(NDT.x < 0.5, NDT.y < 0.5)"

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]  // Skip whitespace
pub enum Token {
    // Keywords
    #[token("AND")]
    And,

    #[token("OR")]
    Or,

    #[token("NOT")]
    Not,

    #[token("true")]
    True,

    #[token("false")]
    False,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Numbers
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse().ok())]
    Float(f32),

    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    Int(i32),

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Remove surrounding quotes
        s[1..s.len()-1].to_string()
    })]
    String(String),

    // Operators
    #[token("==")]
    Eq,

    #[token("!=")]
    Neq,

    #[token("<=")]
    Leq,

    #[token(">=")]
    Geq,

    #[token("<")]
    Lt,

    #[token(">")]
    Gt,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("!")]
    Bang,

    // Delimiters
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,
}

/// Tokenize an expression string
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(input);

    while let Some(token_result) = lexer.next() {
        match token_result {
            Ok(token) => tokens.push(token),
            Err(_) => {
                return Err(format!(
                    "Unexpected character '{}' at position {}",
                    lexer.slice(),
                    lexer.span().start
                ));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("distance_to_target()").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[2], Token::RParen);
    }

    #[test]
    fn test_tokenize_field_access() {
        let tokens = tokenize("NDT.x").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert_eq!(tokens[1], Token::Dot);
        assert!(matches!(tokens[2], Token::Identifier(_)));
    }

    #[test]
    fn test_tokenize_comparison() {
        let tokens = tokenize("NDT.x < 0.5").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Identifier(_)));
        assert_eq!(tokens[1], Token::Dot);
        assert!(matches!(tokens[2], Token::Identifier(_)));
        assert_eq!(tokens[3], Token::Lt);
        assert!(matches!(tokens[4], Token::Float(_)));
    }

    #[test]
    fn test_tokenize_logical() {
        let tokens = tokenize("AND(a < b, c > d)").unwrap();
        assert_eq!(tokens[0], Token::And);
        assert_eq!(tokens[1], Token::LParen);
    }
}
