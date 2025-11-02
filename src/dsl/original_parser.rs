/// Parser for original actor.yaml format
///
/// Parses YAML with uppercase keywords (DO:, SET, IF, etc.) into OriginalBehaviorSpec

use crate::dsl::errors::{DslError, DslResult};
use crate::dsl::original_format::*;
use serde_yaml::Value;
use std::collections::HashMap;

impl OriginalBehaviorSpec {
    /// Parse from file
    pub fn parse_from_file(path: &str) -> DslResult<Self> {
        let yaml_str = std::fs::read_to_string(path)
            .map_err(|e| DslError::IoError(e))?;
        Self::parse_from_str(&yaml_str)
    }

    /// Parse from string
    pub fn parse_from_str(yaml: &str) -> DslResult<Self> {
        let value: Value = serde_yaml::from_str(yaml)
            .map_err(|e| DslError::YamlError(e))?;

        let mapping = value.as_mapping()
            .ok_or_else(|| DslError::ConversionError("Root must be a mapping".to_string()))?;

        let mut spec = OriginalBehaviorSpec::default();

        for (key, val) in mapping {
            let key_str = key.as_str()
                .ok_or_else(|| DslError::ConversionError("Keys must be strings".to_string()))?;

            // Skip comments and empty sections
            if key_str.starts_with('#') || key_str.starts_with("##") {
                continue;
            }

            // Config variables: CFG_*
            if key_str.starts_with("CFG_") {
                let config_var = parse_config_var(key_str, val)?;
                spec.config.insert(key_str.to_string(), config_var);
            }
            // States: uppercase names (MOVE, IDLE, etc.)
            else if key_str.chars().all(|c| c.is_uppercase() || c == '_') {
                let statements = parse_statement_list(val)?;
                spec.states.insert(key_str.to_string(), statements);
            }
            // Procedure sections
            else if key_str == "proc" {
                spec.procedures = parse_procedures(val)?;
            }
            else if key_str == "proc-ai" {
                spec.native_procedures = parse_native_procedures(val)?;
            }
            // Global constants and parameters
            else if key_str == "global" {
                let (constants, parameters) = parse_global_section(val)?;
                spec.constants = constants;
                spec.parameters = parameters;
            }
            // Type definitions (heuristic: single-line or has 'fields' key)
            else if is_type_definition(key_str, val) {
                let type_def = parse_type_definition(key_str, val)?;
                spec.types.insert(key_str.to_string(), type_def);
            }
        }

        Ok(spec)
    }
}

/// Parse a config variable like "CFG_PP_RELEASE AT: float"
fn parse_config_var(key: &str, value: &Value) -> DslResult<ConfigVar> {
    // Try to extract attribute and type hint
    // Format: "CFG_NAME AT: type  # comment"
    let value_str = value.as_str()
        .ok_or_else(|| DslError::ConversionError(format!("Config {} must have string value", key)))?;

    // Simple parsing for now - can be enhanced
    Ok(ConfigVar {
        name: key.to_string(),
        attr: Some("AT".to_string()),
        type_hint: value_str.to_string(),
        comment: None,
    })
}

/// Parse global section with constants and parameters
/// Expected format:
/// global:
///   constants:
///     grid_columns: int
///     grid_cell_width: float
///   parameters:
///     delta_time: float
fn parse_global_section(value: &Value) -> DslResult<(HashMap<String, String>, HashMap<String, String>)> {
    let mapping = value.as_mapping()
        .ok_or_else(|| DslError::ConversionError("global: must be a mapping".to_string()))?;

    let mut constants = HashMap::new();
    let mut parameters = HashMap::new();

    for (key, val) in mapping {
        let key_str = key.as_str()
            .ok_or_else(|| DslError::ConversionError("global: keys must be strings".to_string()))?;

        match key_str {
            "constants" => {
                constants = parse_name_type_mapping(val, "constants")?;
            }
            "parameters" => {
                parameters = parse_name_type_mapping(val, "parameters")?;
            }
            _ => {
                return Err(DslError::ConversionError(
                    format!("Unknown global: section key: {}", key_str)
                ));
            }
        }
    }

    Ok((constants, parameters))
}

/// Parse a mapping of name: type entries
/// Used for constants and parameters
fn parse_name_type_mapping(value: &Value, section_name: &str) -> DslResult<HashMap<String, String>> {
    let mapping = value.as_mapping()
        .ok_or_else(|| DslError::ConversionError(
            format!("{}: must be a mapping", section_name)
        ))?;

    let mut result = HashMap::new();
    for (key, val) in mapping {
        let name = key.as_str()
            .ok_or_else(|| DslError::ConversionError(
                format!("{}: keys must be strings", section_name)
            ))?;
        let type_name = val.as_str()
            .ok_or_else(|| DslError::ConversionError(
                format!("{}: '{}' must have a string type", section_name, name)
            ))?;
        result.insert(name.to_string(), type_name.to_string());
    }

    Ok(result)
}

/// Parse a list of statements
fn parse_statement_list(value: &Value) -> DslResult<Vec<OriginalStatement>> {
    // Handle sequence, mapping, or string format
    if let Some(sequence) = value.as_sequence() {
        // Standard list format: [- DO: ..., - SET: ...]
        sequence.iter()
            .map(|item| parse_statement(item))
            .collect()
    } else if let Some(mapping) = value.as_mapping() {
        // Compact mapping format (used in PROC bodies):
        // DO: action
        // SET var: value
        // IF cond: body
        // ELSE: body  <- This belongs to the preceding IF
        let mut statements = Vec::new();
        let mut iter = mapping.iter().peekable();

        while let Some((key, val)) = iter.next() {
            let key_str = key.as_str().unwrap_or("");

            // Check if next entry is ELSE - if so, it belongs to this IF
            let has_else_next = iter.peek()
                .and_then(|(next_key, _)| next_key.as_str())
                .map(|s| s == "ELSE")
                .unwrap_or(false);

            if key_str.starts_with("IF ") && has_else_next {
                // This IF has an ELSE clause following it
                let condition = key_str.strip_prefix("IF ").unwrap().trim().to_string();
                let (then_body, _) = parse_if_body(val)?;

                // Consume the ELSE
                let (_, else_val) = iter.next().unwrap();
                let else_body = Some(parse_statement_list(else_val)?);

                statements.push(OriginalStatement::If { condition, then_body, else_body });
            } else if key_str == "ELSE" {
                // Standalone ELSE without preceding IF - this is an error
                // But it might have been consumed above, so skip
                continue;
            } else {
                // Regular statement
                let mut temp_map = serde_yaml::Mapping::new();
                temp_map.insert(key.clone(), val.clone());
                let temp_value = Value::Mapping(temp_map);
                statements.push(parse_statement(&temp_value)?);
            }
        }
        Ok(statements)
    } else if let Some(string_val) = value.as_str() {
        // Single string value - treat as DO statement (inline action)
        let stmt = parse_inline_statement(string_val)?;
        Ok(vec![stmt])
    } else {
        Err(DslError::ConversionError(format!("Statements must be a list, mapping, or string, got: {:?}", value)))
    }
}

/// Convert YAML value to string representation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => format!("{:?}", value),
    }
}

/// Parse a single statement
fn parse_statement(value: &Value) -> DslResult<OriginalStatement> {
    // Statements are mappings with one key
    let mapping = value.as_mapping()
        .ok_or_else(|| DslError::ConversionError("Statement must be a mapping".to_string()))?;

    for (key, val) in mapping {
        let key_str = key.as_str()
            .ok_or_else(|| DslError::ConversionError("Statement key must be string".to_string()))?;

        // DO: action
        if key_str == "DO" {
            let action = val.as_str()
                .ok_or_else(|| DslError::ConversionError("DO value must be string".to_string()))?
                .to_string();
            return Ok(OriginalStatement::Do { action });
        }

        // SET variable: expression
        if key_str.starts_with("SET ") {
            let target = key_str.strip_prefix("SET ").unwrap().trim().to_string();

            // Special case: SET state: IDLE
            if target == "state" {
                let state = val.as_str()
                    .ok_or_else(|| DslError::ConversionError("State name must be string".to_string()))?
                    .to_string();
                return Ok(OriginalStatement::SetState { state });
            }

            // Convert value to string (handles numbers, bools, strings)
            let value_str = value_to_string(val);

            return Ok(OriginalStatement::Set { target, value: value_str });
        }

        // IF condition:
        if key_str.starts_with("IF ") {
            let condition = key_str.strip_prefix("IF ").unwrap().trim().to_string();
            let (then_body, else_body) = parse_if_body(val)?;
            return Ok(OriginalStatement::If { condition, then_body, else_body });
        }

        // ELSE IF condition:
        if key_str.starts_with("ELSE IF ") {
            let condition = key_str.strip_prefix("ELSE IF ").unwrap().trim().to_string();
            let (then_body, else_body) = parse_if_body(val)?;
            return Ok(OriginalStatement::ElseIf { condition, then_body, else_body });
        }

        // PROC:name: or PROC:name (both formats allowed)
        if key_str.starts_with("PROC:") {
            let name = key_str.strip_prefix("PROC:")
                .map(|s| s.trim_end_matches(':'))
                .unwrap()
                .to_string();

            let body = parse_statement_list(val)?;
            return Ok(OriginalStatement::ProcDef { name, body });
        }

        // PASS
        if key_str == "PASS" {
            return Ok(OriginalStatement::Pass);
        }

        // RETURN or RETURN:
        if key_str == "RETURN" || key_str.starts_with("RETURN:") || key_str.starts_with("RETURN ") {
            let value_opt = if val.is_null() {
                None
            } else {
                Some(val.as_str()
                    .ok_or_else(|| DslError::ConversionError("RETURN value must be string".to_string()))?
                    .to_string())
            };
            return Ok(OriginalStatement::Return { value: value_opt });
        }

        // PANIC
        if key_str == "PANIC" || key_str.starts_with("PANIC:") {
            let message = val.as_str()
                .ok_or_else(|| DslError::ConversionError("PANIC message must be string".to_string()))?
                .to_string();
            return Ok(OriginalStatement::Panic { message });
        }

        // Unknown statement
        return Err(DslError::ConversionError(format!("Unknown statement type: {}", key_str)));
    }

    Err(DslError::ConversionError("Empty statement".to_string()))
}

/// Parse IF body (handles ELSE and ELSE IF)
fn parse_if_body(value: &Value) -> DslResult<(Vec<OriginalStatement>, Option<Vec<OriginalStatement>>)> {
    // Check if inline (single statement)
    if value.is_string() {
        let stmt_str = value.as_str().unwrap();
        let stmt = parse_inline_statement(stmt_str)?;
        return Ok((vec![stmt], None));
    }

    // Handle mapping format (compact IF body)
    if let Some(mapping) = value.as_mapping() {
        let mut then_body = Vec::new();
        let mut else_body = None;

        for (key, val) in mapping {
            let key_str = key.as_str().unwrap_or("");

            // Check for ELSE clause
            if key_str == "ELSE" {
                else_body = Some(parse_statement_list(val)?);
                continue;
            }

            // Check for ELSE IF clause
            if key_str.starts_with("ELSE IF ") {
                // Create a temporary mapping for the ELSE IF statement
                let mut temp_map = serde_yaml::Mapping::new();
                temp_map.insert(key.clone(), val.clone());
                let else_if_stmt = parse_statement(&Value::Mapping(temp_map))?;
                else_body = Some(vec![else_if_stmt]);
                continue;
            }

            // Regular statement in then_body
            let mut temp_map = serde_yaml::Mapping::new();
            temp_map.insert(key.clone(), val.clone());
            then_body.push(parse_statement(&Value::Mapping(temp_map))?);
        }

        return Ok((then_body, else_body));
    }

    // Parse as sequence (list format)
    let sequence = value.as_sequence()
        .ok_or_else(|| DslError::ConversionError("IF body must be list, mapping, or string".to_string()))?;

    let mut then_body = Vec::new();
    let mut else_body = None;
    let mut in_else = false;

    for item in sequence {
        // Check if this is ELSE or ELSE IF
        if let Some(mapping) = item.as_mapping() {
            for (key, val) in mapping {
                let key_str = key.as_str().unwrap_or("");

                if key_str == "ELSE" {
                    in_else = true;
                    else_body = Some(parse_statement_list(val)?);
                    continue;
                }

                if key_str.starts_with("ELSE IF ") {
                    in_else = true;
                    // ELSE IF is treated as a statement in the else branch
                    let else_if_stmt = parse_statement(item)?;
                    else_body = Some(vec![else_if_stmt]);
                    continue;
                }
            }
        }

        if !in_else {
            then_body.push(parse_statement(item)?);
        }
    }

    Ok((then_body, else_body))
}

/// Parse inline statement (e.g., "PASS")
fn parse_inline_statement(stmt_str: &str) -> DslResult<OriginalStatement> {
    match stmt_str.trim() {
        "PASS" => Ok(OriginalStatement::Pass),
        other => {
            // Could be a DO statement without DO keyword
            // Or an expression - for now treat as DO
            Ok(OriginalStatement::Do { action: other.to_string() })
        }
    }
}

/// Parse procedures from proc: section
fn parse_procedures(value: &Value) -> DslResult<HashMap<String, ProcedureDef>> {
    let mapping = value.as_mapping()
        .ok_or_else(|| DslError::ConversionError("proc: must be a mapping".to_string()))?;

    let mut procedures = HashMap::new();

    for (key, val) in mapping {
        let key_str = key.as_str().unwrap();

        // Parse function signature: "name(params):return_type:"
        let (name, params, return_type) = parse_function_signature(key_str)?;

        // Parse body
        let body = if val.is_sequence() {
            parse_statement_list(val)?
        } else {
            vec![]
        };

        procedures.insert(name.clone(), ProcedureDef {
            name,
            params,
            return_type,
            body,
        });
    }

    Ok(procedures)
}

/// Parse native procedures from proc-ai: section
fn parse_native_procedures(value: &Value) -> DslResult<HashMap<String, NativeProcSpec>> {
    let mapping = value.as_mapping()
        .ok_or_else(|| DslError::ConversionError("proc-ai: must be a mapping".to_string()))?;

    let mut procedures = HashMap::new();

    for (key, val) in mapping {
        let key_str = key.as_str().unwrap();

        // Parse function signature
        let (name, params, return_type) = parse_function_signature(key_str)?;

        // Doc is the YAML value (can be complex)
        let doc = format!("{:?}", val);  // Simplified for now

        procedures.insert(name.clone(), NativeProcSpec {
            name,
            params,
            return_type,
            doc,
        });
    }

    Ok(procedures)
}

/// Parse function signature like "opt_dir:DirectionType:" or "to_pos(int pointId):f2:"
/// or "optimal_direction(DT d):Direction:"
fn parse_function_signature(sig: &str) -> DslResult<(String, Vec<(String, String)>, Option<String>)> {
    // Split by ':' to separate name/params from return type
    let parts: Vec<&str> = sig.split(':').collect();
    let name_and_params = parts[0].trim();

    // Extract function name and params
    let (func_name, params) = if let Some(paren_start) = name_and_params.find('(') {
        // Has parameters: "func_name(type1 param1, type2 param2)"
        let func_name = name_and_params[..paren_start].trim().to_string();

        // Find closing paren
        let paren_end = name_and_params.rfind(')').unwrap_or(name_and_params.len());
        let params_str = &name_and_params[paren_start + 1..paren_end];

        // Parse parameters: "type1 param1, type2 param2"
        let mut params = Vec::new();
        for param in params_str.split(',') {
            let param = param.trim();
            if param.is_empty() {
                continue;
            }

            // Split "type name" by whitespace
            let words: Vec<&str> = param.split_whitespace().collect();
            if words.len() >= 2 {
                let param_type = words[..words.len()-1].join(" ");  // All but last = type
                let param_name = words[words.len()-1].to_string();  // Last = name
                params.push((param_name, param_type));
            } else if words.len() == 1 {
                // Just a type, no name (e.g., "int")
                params.push(("_".to_string(), words[0].to_string()));
            }
        }

        (func_name, params)
    } else {
        // No parameters: "func_name"
        (name_and_params.to_string(), vec![])
    };

    // Extract return type if present
    let return_type = if parts.len() > 1 && !parts[1].is_empty() {
        Some(parts[1].trim().to_string())
    } else {
        None
    };

    Ok((func_name, params, return_type))
}

/// Check if this is a type definition
fn is_type_definition(key: &str, value: &Value) -> bool {
    // Type definitions either:
    // 1. Have a mapping with 'fields' key
    // 2. Are single-line format like "f2: float x, float y"
    if let Some(mapping) = value.as_mapping() {
        mapping.contains_key(&Value::String("fields".to_string()))
    } else if value.is_string() {
        // Single-line type def
        key.chars().all(|c| c.is_alphanumeric() || c == '_')
    } else {
        false
    }
}

/// Parse type definition
fn parse_type_definition(name: &str, value: &Value) -> DslResult<TypeDef> {
    // Simple parsing for now
    Ok(TypeDef {
        name: name.to_string(),
        fields: vec![],  // TODO: parse fields
        constants: vec![],
        alias: None,
    })
}
