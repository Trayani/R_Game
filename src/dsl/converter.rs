/// Converter - transforms compact YAML to structured YAML
///
/// This is the core of the two-stage processing system.
/// Takes human-friendly compact YAML and expands it into machine-friendly structured YAML.

use crate::dsl::ast::*;
use crate::dsl::compact_format::*;
use crate::dsl::original_format::*;
use crate::dsl::condition_parser::ConditionParser;
use crate::dsl::errors::{DslError, DslResult};
use crate::dsl::expression_parser::ExpressionParser;
use crate::dsl::structured_format::*;
use crate::dsl::validator::Validator;
use std::collections::HashMap;

pub struct ActorYAMLConverter {
    validator: Option<Validator>,
}

impl ActorYAMLConverter {
    pub fn new() -> Self {
        ActorYAMLConverter { validator: None }
    }

    pub fn with_validator(mut self, validator: Validator) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Convert compact YAML string to structured YAML string
    pub fn convert_str(&self, compact_yaml: &str) -> DslResult<String> {
        // 1. Parse compact YAML
        let compact = CompactBehaviorSpec::load_from_str(compact_yaml)
            .map_err(|e| DslError::ConversionError(format!("Failed to parse compact YAML: {}", e)))?;

        // 2. Convert to structured format
        let structured = self.expand_to_structured(compact)?;

        // 3. Validate (if validator is set)
        if let Some(ref validator) = self.validator {
            validator.validate(&structured)?;
        }

        // 4. Serialize to YAML
        let structured_yaml = serde_yaml::to_string(&structured)
            .map_err(|e| DslError::ConversionError(format!("Failed to serialize structured YAML: {}", e)))?;

        Ok(structured_yaml)
    }

    /// Convert compact spec to structured spec
    pub fn expand_to_structured(
        &self,
        compact: CompactBehaviorSpec,
    ) -> DslResult<StructuredBehaviorSpec> {
        let mut structured = StructuredBehaviorSpec {
            version: compact.version,
            config: StructuredConfig {
                pp_release_threshold: compact.config.pp_release_threshold,
            },
            states: Default::default(),
            procedures: StructuredProcedures {
                native: compact.procedures.native,
            },
            types: compact.types.into_iter().map(|(name, typedef)| {
                (name, StructuredTypeDef { fields: typedef.fields })
            }).collect(),
        };

        // Convert each state
        for (state_name, actions) in compact.states {
            let expanded_actions = actions
                .into_iter()
                .map(|action| self.expand_action(action))
                .collect::<DslResult<Vec<_>>>()?;

            structured.states.insert(state_name, expanded_actions);
        }

        Ok(structured)
    }

    /// Convert a compact action to structured action
    fn expand_action(&self, action: CompactAction) -> DslResult<StructuredAction> {
        match action {
            CompactAction::Do { action } => Ok(StructuredAction::Do { call: action }),

            CompactAction::Set { set } => {
                // For now, take the first assignment
                // (Multiple assignments in one `set` are rare, can handle later)
                let (var_name, expr_str) = set
                    .into_iter()
                    .next()
                    .ok_or_else(|| DslError::ConversionError("Empty set statement".to_string()))?;

                // Parse expression string
                let expr_ast = ExpressionParser::parse_from_str(&expr_str)?;

                // Convert AST to structured format
                let structured_expr = self.ast_to_structured_expr(expr_ast)?;

                Ok(StructuredAction::Set {
                    variable: var_name,
                    value: structured_expr,
                })
            }

            CompactAction::SetState { set_state } => Ok(StructuredAction::SetState {
                state: set_state,
            }),

            CompactAction::If { if_data } => {
                // Parse condition
                let structured_cond = self.expand_condition(if_data.condition)?;

                // Recursively expand then/else bodies
                let expanded_then = if_data
                    .then
                    .into_iter()
                    .map(|a| self.expand_action(a))
                    .collect::<DslResult<Vec<_>>>()?;

                let expanded_else = if let Some(else_actions) = if_data.r#else {
                    Some(
                        else_actions
                            .into_iter()
                            .map(|a| self.expand_action(a))
                            .collect::<DslResult<Vec<_>>>()?,
                    )
                } else {
                    None
                };

                Ok(StructuredAction::If {
                    condition: structured_cond,
                    then_body: expanded_then,
                    else_body: expanded_else,
                })
            }

            CompactAction::Pass { .. } => Ok(StructuredAction::Pass),
        }
    }

    /// Expand a compact condition to structured condition
    fn expand_condition(&self, condition: CompactCondition) -> DslResult<StructuredCondition> {
        match condition {
            CompactCondition::String(cond_str) => {
                // Parse condition string
                let cond_ast = ConditionParser::parse_from_str(&cond_str)?;
                self.ast_to_structured_condition(cond_ast)
            }
            CompactCondition::Structured { op } => {
                // Already structured, just convert
                match op {
                    CompactConditionOp::And(comparisons) => {
                        let operands = comparisons
                            .into_iter()
                            .map(|c| self.expand_comparison(c))
                            .collect::<DslResult<Vec<_>>>()?;
                        Ok(StructuredCondition::And { operands })
                    }
                    CompactConditionOp::Or(comparisons) => {
                        let operands = comparisons
                            .into_iter()
                            .map(|c| self.expand_comparison(c))
                            .collect::<DslResult<Vec<_>>>()?;
                        Ok(StructuredCondition::Or { operands })
                    }
                }
            }
        }
    }

    /// Expand a compact comparison to structured condition
    fn expand_comparison(&self, comparison: CompactComparison) -> DslResult<StructuredCondition> {
        let (op, left_str, right_str) = match comparison.op {
            CompactComparisonOp::Lt([left, right]) => ("<", left, right),
            CompactComparisonOp::Gt([left, right]) => (">", left, right),
            CompactComparisonOp::Eq([left, right]) => ("==", left, right),
            CompactComparisonOp::Neq([left, right]) => ("!=", left, right),
            CompactComparisonOp::Leq([left, right]) => ("<=", left, right),
            CompactComparisonOp::Geq([left, right]) => (">=", left, right),
        };

        let left_ast = ExpressionParser::parse_from_str(&left_str)?;
        let right_ast = ExpressionParser::parse_from_str(&right_str)?;

        let left_expr = self.ast_to_structured_expr(left_ast)?;
        let right_expr = self.ast_to_structured_expr(right_ast)?;

        Ok(StructuredCondition::Comparison {
            op: op.to_string(),
            left: left_expr,
            right: right_expr,
        })
    }

    /// Convert expression AST to structured expression
    fn ast_to_structured_expr(&self, ast: ExpressionAST) -> DslResult<StructuredExpression> {
        match ast {
            ExpressionAST::Literal(lit) => {
                let value = match lit {
                    LiteralValue::Int(i) => StructuredLiteral::Int(i),
                    LiteralValue::Float(f) => StructuredLiteral::Float(f),
                    LiteralValue::Bool(b) => StructuredLiteral::Bool(b),
                    LiteralValue::String(s) => StructuredLiteral::String(s),
                };
                Ok(StructuredExpression::Literal { value })
            }

            ExpressionAST::Variable(name) => Ok(StructuredExpression::Variable { name }),

            ExpressionAST::FieldAccess { object, field } => {
                let structured_obj = Box::new(self.ast_to_structured_expr(*object)?);
                Ok(StructuredExpression::FieldAccess {
                    object: structured_obj,
                    field,
                })
            }

            ExpressionAST::FunctionCall { name, args } => {
                let structured_args = args
                    .into_iter()
                    .map(|arg| self.ast_to_structured_expr(arg))
                    .collect::<DslResult<Vec<_>>>()?;

                Ok(StructuredExpression::FunctionCall {
                    name,
                    args: structured_args,
                })
            }

            ExpressionAST::BinaryOp { op, left, right } => Ok(StructuredExpression::BinaryOp {
                op,
                left: Box::new(self.ast_to_structured_expr(*left)?),
                right: Box::new(self.ast_to_structured_expr(*right)?),
            }),

            ExpressionAST::UnaryOp { op, operand } => Ok(StructuredExpression::UnaryOp {
                op,
                operand: Box::new(self.ast_to_structured_expr(*operand)?),
            }),
        }
    }

    /// Convert condition AST to structured condition
    fn ast_to_structured_condition(&self, ast: ConditionAST) -> DslResult<StructuredCondition> {
        match ast {
            ConditionAST::And(operands) => {
                let structured_operands = operands
                    .into_iter()
                    .map(|op| self.ast_to_structured_condition(op))
                    .collect::<DslResult<Vec<_>>>()?;

                Ok(StructuredCondition::And {
                    operands: structured_operands,
                })
            }

            ConditionAST::Or(operands) => {
                let structured_operands = operands
                    .into_iter()
                    .map(|op| self.ast_to_structured_condition(op))
                    .collect::<DslResult<Vec<_>>>()?;

                Ok(StructuredCondition::Or {
                    operands: structured_operands,
                })
            }

            ConditionAST::Not(operand) => {
                let structured_operand = Box::new(self.ast_to_structured_condition(*operand)?);
                Ok(StructuredCondition::Not {
                    operand: structured_operand,
                })
            }

            ConditionAST::Comparison { op, left, right } => Ok(StructuredCondition::Comparison {
                op,
                left: self.ast_to_structured_expr(left)?,
                right: self.ast_to_structured_expr(right)?,
            }),

            ConditionAST::Expression(expr) => Ok(StructuredCondition::Expression {
                expr: self.ast_to_structured_expr(expr)?,
            }),
        }
    }
}

impl Default for ActorYAMLConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_set() {
        let compact_yaml = r#"
version: "1.0"
config:
  pp_release_threshold: 0.6
states:
  MOVE:
    - set: {NDT: "distance_to_target()"}
procedures:
  native:
    - distance_to_target
types: {}
"#;

        let converter = ActorYAMLConverter::new();
        let result = converter.convert_str(compact_yaml);
        assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
    }

    #[test]
    fn test_convert_if_statement() {
        let compact_yaml = r#"
version: "1.0"
config:
  pp_release_threshold: 0.6
states:
  MOVE:
    - set: {NDT: "distance_to_target()"}
    - if: "NDT.x < 0.5"
      then:
        - do: switch_pp
procedures:
  native:
    - distance_to_target
    - switch_pp
types: {}
"#;

        let converter = ActorYAMLConverter::new();
        let result = converter.convert_str(compact_yaml);
        assert!(result.is_ok(), "Conversion failed: {:?}", result.err());
    }
}

// Original format conversion (separate impl block)
impl ActorYAMLConverter {
    /// Convert original format spec to structured spec
    pub fn convert_original_to_structured(
        &self,
        original: OriginalBehaviorSpec,
    ) -> DslResult<StructuredBehaviorSpec> {
        // Collect native function names for implicit call detection
        let native_functions: HashMap<String, String> = original.native_procedures
            .iter()
            .map(|(name, _)| (name.clone(), name.clone()))
            .collect();

        // Expand vararg functions into multiple permutations
        let expanded_natives = self.expand_vararg_functions(&original.native_procedures);

        let mut structured = StructuredBehaviorSpec {
            version: "1.0".to_string(),
            config: StructuredConfig {
                pp_release_threshold: 0.6, // TODO: extract from CFG_PP_RELEASE
            },
            states: HashMap::new(),
            procedures: StructuredProcedures {
                native: expanded_natives,
            },
            types: HashMap::new(),
        };

        // Convert states
        for (state_name, statements) in original.states {
            let converted_statements = self.convert_original_statements(&statements, &native_functions)?;
            structured.states.insert(state_name, converted_statements);
        }

        Ok(structured)
    }

    /// Expand vararg functions into multiple permutations
    /// Example: "release_reserved(vararg int pointId)" becomes:
    ///   - "release_reserved(int pointId)"
    ///   - "release_reserved(int pointId, int pointId)"
    ///   - "release_reserved(int pointId, int pointId, int pointId)"
    ///   ... up to MAX_VARARG_PERMUTATIONS
    fn expand_vararg_functions(&self, native_procs: &HashMap<String, NativeProcSpec>) -> Vec<String> {
        const MAX_VARARG_PERMUTATIONS: usize = 5; // Generate up to 5 permutations

        let mut result = Vec::new();

        for (name, _spec) in native_procs {
            // Check if this function has vararg
            if name.contains("vararg") {
                // Parse the signature: "func_name(vararg type param)"
                if let Some((func_name, rest)) = name.split_once('(') {
                    let func_name = func_name.trim();
                    let rest = rest.trim_end_matches(')').trim();

                    // Parse "vararg type param"
                    if rest.starts_with("vararg ") {
                        let param_decl = rest.strip_prefix("vararg ").unwrap().trim();

                        // Generate permutations
                        for count in 1..=MAX_VARARG_PERMUTATIONS {
                            let params: Vec<String> = (0..count)
                                .map(|_| param_decl.to_string())
                                .collect();
                            let signature = format!("{}({})", func_name, params.join(", "));
                            result.push(signature);
                        }
                    } else {
                        // Not a vararg, add as-is
                        result.push(name.clone());
                    }
                } else {
                    // No parentheses, add as-is
                    result.push(name.clone());
                }
            } else {
                // Not a vararg function, add as-is
                result.push(name.clone());
            }
        }

        result
    }

    /// Convert original statements to structured actions
    fn convert_original_statements(
        &self,
        statements: &[OriginalStatement],
        native_functions: &HashMap<String, String>,
    ) -> DslResult<Vec<StructuredAction>> {
        statements
            .iter()
            .map(|stmt| self.convert_original_statement(stmt, native_functions))
            .collect()
    }

    /// Convert a single original statement to structured action
    fn convert_original_statement(
        &self,
        statement: &OriginalStatement,
        native_functions: &HashMap<String, String>,
    ) -> DslResult<StructuredAction> {
        match statement {
            OriginalStatement::Do { action } => {
                // Check if this is a native function call (implicit call)
                let call = if native_functions.contains_key(action) {
                    format!("{}()", action)
                } else {
                    action.clone()
                };
                Ok(StructuredAction::Do { call })
            }

            OriginalStatement::Set { target, value } => {
                // Parse the value expression
                let expr_ast = self.parse_expression_with_implicit_calls(value, native_functions)?;
                let structured_expr = self.ast_to_structured_expr(expr_ast)?;

                Ok(StructuredAction::Set {
                    variable: target.clone(),
                    value: structured_expr,
                })
            }

            OriginalStatement::SetState { state } => {
                Ok(StructuredAction::SetState {
                    state: state.clone(),
                })
            }

            OriginalStatement::If { condition, then_body, else_body } => {
                // Parse condition
                let cond_ast = ConditionParser::parse_from_str(condition)?;
                let structured_cond = self.ast_to_structured_condition(cond_ast)?;

                // Convert bodies
                let then_actions = self.convert_original_statements(then_body, native_functions)?;
                let else_actions = if let Some(else_stmts) = else_body {
                    Some(self.convert_original_statements(else_stmts, native_functions)?)
                } else {
                    None
                };

                Ok(StructuredAction::If {
                    condition: structured_cond,
                    then_body: then_actions,
                    else_body: else_actions,
                })
            }

            OriginalStatement::ElseIf { condition, then_body, else_body } => {
                // ELSE IF is just an IF in the else branch
                let cond_ast = ConditionParser::parse_from_str(condition)?;
                let structured_cond = self.ast_to_structured_condition(cond_ast)?;

                let then_actions = self.convert_original_statements(then_body, native_functions)?;
                let else_actions = if let Some(else_stmts) = else_body {
                    Some(self.convert_original_statements(else_stmts, native_functions)?)
                } else {
                    None
                };

                Ok(StructuredAction::If {
                    condition: structured_cond,
                    then_body: then_actions,
                    else_body: else_actions,
                })
            }

            OriginalStatement::Pass => {
                Ok(StructuredAction::Pass)
            }

            OriginalStatement::Return { value } => {
                // For now, treat RETURN as PASS (states don't have return values)
                // TODO: Handle RETURN in procedure contexts
                Ok(StructuredAction::Pass)
            }

            OriginalStatement::Panic { message } => {
                // Represent PANIC as a DO action
                Ok(StructuredAction::Do {
                    call: format!("panic(\"{}\")", message),
                })
            }

            OriginalStatement::ProcDef { name, body } => {
                // PROC definitions within states are treated as inline procedure calls
                // Convert the body and wrap in a call
                let proc_actions = self.convert_original_statements(body, native_functions)?;

                // For now, inline the procedure body
                // TODO: Extract to separate procedure definition
                // Return a comment action indicating this was a PROC
                Ok(StructuredAction::Do {
                    call: format!("/* PROC:{} - {} statements */", name, proc_actions.len()),
                })
            }
        }
    }

    /// Parse expression with implicit function call detection
    fn parse_expression_with_implicit_calls(
        &self,
        expr_str: &str,
        native_functions: &HashMap<String, String>,
    ) -> DslResult<ExpressionAST> {
        let trimmed = expr_str.trim();

        // Check if this is a bare identifier that's a native function
        if trimmed.chars().all(|c| c.is_alphanumeric() || c == '_') {
            if native_functions.contains_key(trimmed) {
                // Implicit function call: "distance_to_target" → distance_to_target()
                return Ok(ExpressionAST::FunctionCall {
                    name: trimmed.to_string(),
                    args: vec![],
                });
            }
        }

        // Otherwise, parse normally
        ExpressionParser::parse_from_str(expr_str)
    }
}
