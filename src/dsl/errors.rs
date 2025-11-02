/// Error types for DSL processing

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DslError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Validation error in {context}: {message}")]
    ValidationError {
        context: String,
        message: String,
    },

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Undefined function '{name}' in state '{context}'")]
    UndefinedFunction {
        name: String,
        context: String,
    },

    #[error("Undefined variable '{name}' at line {line}")]
    UndefinedVariable {
        name: String,
        line: usize,
    },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition {
        from: String,
        to: String,
    },

    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type DslResult<T> = Result<T, DslError>;
