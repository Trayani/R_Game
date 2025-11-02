/// Compact YAML format - what humans write
///
/// This format uses concise syntax with expression strings that get parsed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// COMPACT BEHAVIOR SPECIFICATION
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactBehaviorSpec {
    pub version: String,
    pub config: CompactConfig,
    pub states: HashMap<String, Vec<CompactAction>>,
    pub procedures: CompactProcedures,

    #[serde(default)]
    pub types: HashMap<String, CompactTypeDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactConfig {
    pub pp_release_threshold: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactProcedures {
    pub native: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactTypeDef {
    pub fields: HashMap<String, String>,  // field_name -> type_name
}

// ============================================================================
// COMPACT ACTIONS
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CompactAction {
    /// do: action_name
    Do {
        #[serde(rename = "do")]
        action: String,
    },

    /// set: {var_name: "expression"}
    /// Can have multiple assignments: set: {a: "1", b: "2"}
    Set {
        set: HashMap<String, String>,
    },

    /// set_state: STATE_NAME
    SetState {
        set_state: String,
    },

    /// if: "condition"
    /// or if: {condition: "...", then: [...], else: [...]}
    /// or if: {and: [...], then: [...]}  (structured condition)
    If {
        #[serde(flatten)]
        if_data: CompactIfData,
    },

    /// pass: true
    Pass {
        pass: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactIfData {
    /// Condition can be a string or structured
    #[serde(flatten)]
    pub condition: CompactCondition,

    /// Then branch
    pub then: Vec<CompactAction>,

    /// Optional else branch
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#else: Option<Vec<CompactAction>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CompactCondition {
    /// String condition: "AND(a < b, c > d)"
    String(String),

    /// Structured condition: {and: [...]}
    Structured {
        #[serde(flatten)]
        op: CompactConditionOp,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompactConditionOp {
    And(Vec<CompactComparison>),
    Or(Vec<CompactComparison>),
    // Not is less common, can add if needed
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompactComparison {
    #[serde(flatten)]
    pub op: CompactComparisonOp,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompactComparisonOp {
    Lt([String; 2]),    // {lt: ["a", "b"]}
    Gt([String; 2]),
    Eq([String; 2]),
    Neq([String; 2]),
    Leq([String; 2]),
    Geq([String; 2]),
}

impl CompactBehaviorSpec {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let spec: CompactBehaviorSpec = serde_yaml::from_str(&content)?;
        Ok(spec)
    }

    pub fn load_from_str(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let spec: CompactBehaviorSpec = serde_yaml::from_str(yaml)?;
        Ok(spec)
    }
}
