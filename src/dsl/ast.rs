/// Abstract Syntax Tree node definitions for the DSL
///
/// These represent the parsed structure of expressions and statements
/// before conversion to the structured YAML format.

use serde::{Deserialize, Serialize};

// ============================================================================
// LITERAL VALUES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
}

// ============================================================================
// EXPRESSION AST
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionAST {
    /// Literal value: 42, 3.14, true, "hello"
    Literal(LiteralValue),

    /// Variable reference: x, NDT, actor
    Variable(String),

    /// Field access: actor.pos, NDT.x
    FieldAccess {
        object: Box<ExpressionAST>,
        field: String,
    },

    /// Function call: distance_to_target(), opt_dir()
    FunctionCall {
        name: String,
        args: Vec<ExpressionAST>,
    },

    /// Binary operation: a + b, x < y
    BinaryOp {
        op: String,  // "+", "-", "*", "/", "<", ">", "==", "!=", "<=", ">="
        left: Box<ExpressionAST>,
        right: Box<ExpressionAST>,
    },

    /// Unary operation: !x, -x
    UnaryOp {
        op: String,  // "!", "-"
        operand: Box<ExpressionAST>,
    },

    /// Tuple literal: (a, b, c)
    Tuple {
        elements: Vec<ExpressionAST>,
    },
}

// ============================================================================
// CONDITION AST (for IF statements)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionAST {
    /// Logical AND: AND(a, b, c)
    And(Vec<ConditionAST>),

    /// Logical OR: OR(a, b, c)
    Or(Vec<ConditionAST>),

    /// Logical NOT: NOT(a)
    Not(Box<ConditionAST>),

    /// Comparison: a < b, x == y
    Comparison {
        op: String,
        left: ExpressionAST,
        right: ExpressionAST,
    },

    /// Single expression (must evaluate to bool)
    Expression(ExpressionAST),
}

// ============================================================================
// STATEMENT AST
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementAST {
    /// DO: action_name
    Do {
        action: String,
    },

    /// SET variable: expression
    Set {
        variable: String,
        value: ExpressionAST,
    },

    /// SET_STATE: state_name
    SetState {
        state: String,
    },

    /// IF condition: ... ELSE: ...
    If {
        condition: ConditionAST,
        then_body: Vec<StatementAST>,
        else_body: Option<Vec<StatementAST>>,
    },

    /// PASS (early return)
    Pass,
}

// ============================================================================
// STATE DEFINITION AST
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAST {
    pub name: String,
    pub statements: Vec<StatementAST>,
}

// ============================================================================
// BEHAVIOR SPECIFICATION AST
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorAST {
    pub version: String,
    pub config: ConfigAST,
    pub states: Vec<StateAST>,
    pub procedures: Vec<String>,  // List of native procedure names
    pub types: Vec<TypeDefAST>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAST {
    pub pp_release_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefAST {
    pub name: String,
    pub fields: Vec<FieldDefAST>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefAST {
    pub name: String,
    pub type_name: String,
}
