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

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub constants: HashMap<String, ConstantDef>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, ParameterDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredConfig {
    pub pp_release_threshold: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredProcedures {
    pub native: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dsl: HashMap<String, StructuredProcedure>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredProcedure {
    pub params: Vec<(String, String)>,  // [(param_name, type)]
    pub return_type: Option<String>,
    pub body: Vec<StructuredAction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructuredTypeDef {
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstantDef {
    pub type_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParameterDef {
    pub type_name: String,
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

    Tuple {
        elements: Vec<StructuredExpression>,
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

    Return {
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<StructuredExpression>,
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
