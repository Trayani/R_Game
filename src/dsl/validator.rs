/// Validator - static analysis and validation of behavior specifications

use crate::dsl::errors::{DslError, DslResult};
use crate::dsl::structured_format::*;
use std::collections::HashSet;

pub struct Validator {
    known_functions: HashSet<String>,
}

impl Validator {
    pub fn new(known_functions: Vec<String>) -> Self {
        Validator {
            known_functions: known_functions.into_iter().collect(),
        }
    }

    pub fn validate(&self, spec: &StructuredBehaviorSpec) -> DslResult<()> {
        // Validate function calls
        self.validate_function_calls(spec)?;

        // Validate state transitions
        self.validate_state_transitions(spec)?;

        Ok(())
    }

    fn validate_function_calls(&self, spec: &StructuredBehaviorSpec) -> DslResult<()> {
        for (state_name, actions) in &spec.states {
            for action in actions {
                self.validate_action_functions(action, state_name)?;
            }
        }
        Ok(())
    }

    fn validate_action_functions(
        &self,
        action: &StructuredAction,
        context: &str,
    ) -> DslResult<()> {
        match action {
            StructuredAction::Do { call } => {
                if !self.known_functions.contains(call) {
                    return Err(DslError::UndefinedFunction {
                        name: call.clone(),
                        context: context.to_string(),
                    });
                }
            }
            StructuredAction::Set { value, .. } => {
                self.validate_expression_functions(value, context)?;
            }
            StructuredAction::If {
                condition,
                then_body,
                else_body,
            } => {
                self.validate_condition_functions(condition, context)?;
                for action in then_body {
                    self.validate_action_functions(action, context)?;
                }
                if let Some(else_actions) = else_body {
                    for action in else_actions {
                        self.validate_action_functions(action, context)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_expression_functions(
        &self,
        expr: &StructuredExpression,
        context: &str,
    ) -> DslResult<()> {
        match expr {
            StructuredExpression::FunctionCall { name, args } => {
                if !self.known_functions.contains(name) {
                    return Err(DslError::UndefinedFunction {
                        name: name.clone(),
                        context: context.to_string(),
                    });
                }
                for arg in args {
                    self.validate_expression_functions(arg, context)?;
                }
            }
            StructuredExpression::FieldAccess { object, .. } => {
                self.validate_expression_functions(object, context)?;
            }
            StructuredExpression::BinaryOp { left, right, .. } => {
                self.validate_expression_functions(left, context)?;
                self.validate_expression_functions(right, context)?;
            }
            StructuredExpression::UnaryOp { operand, .. } => {
                self.validate_expression_functions(operand, context)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_condition_functions(
        &self,
        condition: &StructuredCondition,
        context: &str,
    ) -> DslResult<()> {
        match condition {
            StructuredCondition::And { operands } | StructuredCondition::Or { operands } => {
                for operand in operands {
                    self.validate_condition_functions(operand, context)?;
                }
            }
            StructuredCondition::Not { operand } => {
                self.validate_condition_functions(operand, context)?;
            }
            StructuredCondition::Comparison { left, right, .. } => {
                self.validate_expression_functions(left, context)?;
                self.validate_expression_functions(right, context)?;
            }
            StructuredCondition::Expression { expr } => {
                self.validate_expression_functions(expr, context)?;
            }
        }
        Ok(())
    }

    fn validate_state_transitions(&self, spec: &StructuredBehaviorSpec) -> DslResult<()> {
        let valid_states: HashSet<_> = spec.states.keys().cloned().collect();

        for (state_name, actions) in &spec.states {
            for action in actions {
                self.validate_state_transition_in_action(action, state_name, &valid_states)?;
            }
        }

        Ok(())
    }

    fn validate_state_transition_in_action(
        &self,
        action: &StructuredAction,
        current_state: &str,
        valid_states: &HashSet<String>,
    ) -> DslResult<()> {
        match action {
            StructuredAction::SetState { state } => {
                if !valid_states.contains(state) {
                    return Err(DslError::InvalidStateTransition {
                        from: current_state.to_string(),
                        to: state.clone(),
                    });
                }
            }
            StructuredAction::If {
                then_body,
                else_body,
                ..
            } => {
                for action in then_body {
                    self.validate_state_transition_in_action(action, current_state, valid_states)?;
                }
                if let Some(else_actions) = else_body {
                    for action in else_actions {
                        self.validate_state_transition_in_action(
                            action,
                            current_state,
                            valid_states,
                        )?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
