/// Original actor.yaml format - as written by the user
///
/// This format uses:
/// - Uppercase keywords: DO:, SET, IF, ELSE, PROC:, etc.
/// - Implicit function calls (no parens needed for known functions)
/// - Nested PROC blocks
/// - Config variables with CFG_ prefix
/// - Type definitions
/// - proc: (executable DSL) and proc-ai: (native Rust specs)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete behavior specification in original format
#[derive(Debug, Clone, Default)]
pub struct OriginalBehaviorSpec {
    pub config: HashMap<String, ConfigVar>,
    pub states: HashMap<String, Vec<OriginalStatement>>,
    pub types: HashMap<String, TypeDef>,
    pub procedures: HashMap<String, ProcedureDef>,           // proc:
    pub native_procedures: HashMap<String, NativeProcSpec>,  // proc-ai:
    pub constants: HashMap<String, String>,                  // global: constants: (name -> type_name)
    pub parameters: HashMap<String, String>,                 // global: parameters: (name -> type_name)
}

/// Configuration variable (e.g., CFG_PP_RELEASE AT: float)
#[derive(Debug, Clone)]
pub struct ConfigVar {
    pub name: String,          // "CFG_PP_RELEASE"
    pub attr: Option<String>,  // "AT"
    pub type_hint: String,     // "float"
    pub comment: Option<String>, // Comments after #
}

/// Type definition
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub name: String,
    pub fields: Vec<(String, String)>,  // [(field_name, type_name)]
    pub constants: Vec<TypeConstant>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TypeConstant {
    pub name: String,
    pub params: Vec<String>,
}

/// Procedure definition (from proc: section - executable DSL)
#[derive(Debug, Clone)]
pub struct ProcedureDef {
    pub name: String,
    pub params: Vec<(String, String)>,  // [(param_name, type)]
    pub return_type: Option<String>,
    pub body: Vec<OriginalStatement>,
}

/// Native procedure specification (from proc-ai: section - documentation)
#[derive(Debug, Clone)]
pub struct NativeProcSpec {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub return_type: Option<String>,
    pub doc: String,  // Full documentation text
}

/// Statement in original format (before conversion to AST)
#[derive(Debug, Clone, PartialEq)]
pub enum OriginalStatement {
    /// DO: action_name
    Do {
        action: String,
    },

    /// SET variable: expression
    /// or SET object.field: expression
    Set {
        target: String,  // Will be parsed as expression later
        value: String,   // Will be parsed as expression later
    },

    /// IF condition: body
    If {
        condition: String,  // Will be parsed as condition later
        then_body: Vec<OriginalStatement>,
        else_body: Option<Vec<OriginalStatement>>,
    },

    /// ELSE IF condition: body (chained conditional)
    ElseIf {
        condition: String,
        then_body: Vec<OriginalStatement>,
        else_body: Option<Vec<OriginalStatement>>,
    },

    /// PASS (early exit)
    Pass,

    /// RETURN or RETURN: value
    Return {
        value: Option<String>,  // Will be parsed as expression later
    },

    /// PANIC: "message"
    Panic {
        message: String,
    },

    /// PROC:name: body (named procedure definition + call)
    ProcDef {
        name: String,
        body: Vec<OriginalStatement>,
    },

    /// SET state: STATE_NAME (special case for state transitions)
    SetState {
        state: String,
    },
}

impl OriginalStatement {
    /// Check if this is a SET statement
    pub fn is_set(&self) -> bool {
        matches!(self, OriginalStatement::Set { .. })
    }

    /// Check if this is a DO statement
    pub fn is_do(&self) -> bool {
        matches!(self, OriginalStatement::Do { .. })
    }

    /// Check if this is an IF statement
    pub fn is_if(&self) -> bool {
        matches!(self, OriginalStatement::If { .. })
    }

    /// Get the statement type as a string (for error messages)
    pub fn type_name(&self) -> &'static str {
        match self {
            OriginalStatement::Do { .. } => "DO",
            OriginalStatement::Set { .. } => "SET",
            OriginalStatement::If { .. } => "IF",
            OriginalStatement::ElseIf { .. } => "ELSE IF",
            OriginalStatement::Pass => "PASS",
            OriginalStatement::Return { .. } => "RETURN",
            OriginalStatement::Panic { .. } => "PANIC",
            OriginalStatement::ProcDef { .. } => "PROC",
            OriginalStatement::SetState { .. } => "SET state",
        }
    }
}
