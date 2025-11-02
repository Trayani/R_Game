/// Structured YAML format - what the runtime interprets
///
/// This format has all expressions fully parsed into AST structures.
/// It's more verbose but enables static analysis and type checking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// STRUCTURED BEHAVIOR SPECIFICATION
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredBehaviorSpec {
    pub version: String,
    pub config: StructuredConfig,
    pub states: HashMap<String, Vec<StructuredAction>>,
    pub procedures: StructuredProcedures,

    #[serde(default)]
    pub types: HashMap<String, StructuredTypeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredConfig {
    pub pp_release_threshold: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredProcedures {
    pub native: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredTypeDef {
    pub fields: HashMap<String, String>,
}

// ============================================================================
// STRUCTURED EXPRESSIONS
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StructuredExpression {
    Literal {
        value: StructuredLiteral,
    },

    Variable {
        name: String,
    },

    FieldAccess {
        object: Box<StructuredExpression>,
        field: String,
    },

    FunctionCall {
        name: String,
        args: Vec<StructuredExpression>,
    },

    BinaryOp {
        op: String,
        left: Box<StructuredExpression>,
        right: Box<StructuredExpression>,
    },

    UnaryOp {
        op: String,
        operand: Box<StructuredExpression>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StructuredLiteral {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
}

// ============================================================================
// STRUCTURED CONDITIONS
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StructuredCondition {
    And {
        operands: Vec<StructuredCondition>,
    },

    Or {
        operands: Vec<StructuredCondition>,
    },

    Not {
        operand: Box<StructuredCondition>,
    },

    Comparison {
        op: String,
        left: StructuredExpression,
        right: StructuredExpression,
    },

    Expression {
        expr: StructuredExpression,
    },
}

// ============================================================================
// STRUCTURED ACTIONS
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum StructuredAction {
    Do {
        call: String,
    },

    Set {
        variable: String,
        value: StructuredExpression,
    },

    SetState {
        state: String,
    },

    If {
        condition: StructuredCondition,
        then_body: Vec<StructuredAction>,

        #[serde(skip_serializing_if = "Option::is_none")]
        else_body: Option<Vec<StructuredAction>>,
    },

    Pass,
}

impl StructuredBehaviorSpec {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let spec: StructuredBehaviorSpec = serde_yaml::from_str(&content)?;
        Ok(spec)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}
