/// DSL (Domain Specific Language) module for actor behavior specification
///
/// This module provides a two-stage processing system:
/// 1. Compact YAML (human-friendly) → Converter → Structured YAML (machine-friendly)
/// 2. Structured YAML → Interpreter → Runtime execution
///
/// Architecture:
/// - AST: Abstract Syntax Tree node definitions
/// - Lexer: Tokenization of expression strings
/// - Parser: Parse expressions and conditions into AST
/// - Converter: Transform compact YAML to structured YAML
/// - Validator: Static analysis and error checking
/// - Interpreter: Runtime execution of structured YAML

pub mod ast;
pub mod compact_format;
pub mod structured_format;
pub mod lexer;
pub mod expression_parser;
pub mod condition_parser;
pub mod converter;
pub mod validator;
pub mod interpreter;
pub mod native_functions;
pub mod errors;

// Re-export commonly used types
pub use ast::*;
pub use compact_format::*;
pub use structured_format::*;
pub use converter::ActorYAMLConverter;
pub use validator::Validator;
pub use interpreter::Interpreter;
pub use errors::*;
