# Next Steps: Complete DSL Converter for Original actor.yaml Format

## Current Status ✅
- ✅ Core AST definitions (expressions, conditions, statements)
- ✅ Lexer with logos (tokenization)
- ✅ Expression parser (handles `distance_to_target()`, `NDT.x`, `a + b`)
- ✅ Condition parser (handles `AND(a, b)`, `OR(a, b)`)
- ✅ Converter framework (transforms AST to structured format)
- ✅ Validator (checks function calls, state transitions)
- ✅ CLI tool skeleton

## What Needs to Be Built 🚧

### 1. Original Format Parser (`src/dsl/original_format_parser.rs`)

**Purpose**: Parse YOUR actor.yaml syntax directly (DO:, SET var:, PROC:, etc.)

**Key Features:**
```rust
pub struct OriginalFormatParser {
    yaml_value: serde_yaml::Value,
}

impl OriginalFormatParser {
    pub fn parse_from_file(path: &str) -> DslResult<OriginalBehaviorSpec>;
    pub fn parse_from_str(yaml: &str) -> DslResult<OriginalBehaviorSpec>;

    // Parse different sections
    fn parse_config(&self) -> DslResult<HashMap<String, ConfigValue>>;
    fn parse_states(&self) -> DslResult<HashMap<String, Vec<StatementAST>>>;
    fn parse_types(&self) -> DslResult<HashMap<String, TypeDef>>;
    fn parse_procedures(&self) -> DslResult<HashMap<String, ProcedureDef>>;

    // Parse statements
    fn parse_statement(&self, yaml: &Value) -> DslResult<StatementAST>;
    fn parse_do_statement(&self, value: &str) -> DslResult<StatementAST>;
    fn parse_set_statement(&self, key: &str, value: &str) -> DslResult<StatementAST>;
    fn parse_if_statement(&self, condition: &str, body: &[Value]) -> DslResult<StatementAST>;
    fn parse_proc_block(&self, name: &str, body: &[Value]) -> DslResult<StatementAST>;
}
```

**Syntax to Handle:**

| Your Syntax | Parse To |
|------------|----------|
| `DO: move_to_target` | `StatementAST::Do { action: "move_to_target" }` |
| `SET NDT: distance_to_target` | `StatementAST::Set { variable: "NDT", value: FunctionCall(...) }` |
| `IF condition:` + body | `StatementAST::If { condition: ..., then_body: [...], else_body: None }` |
| `ELSE:` | Part of previous IF statement |
| `ELSE IF condition:` | Nested IF in else_body |
| `RETURN: expr` | `StatementAST::Return { value: expr }` |
| `PASS` | `StatementAST::Pass` |
| `PANIC: "message"` | `StatementAST::Panic { message: "..." }` |
| `PROC:name:` + body | `StatementAST::ProcCall { name: "name", body: [...] }` |

### 2. AST Extensions (`src/dsl/ast.rs`)

Add missing statement types:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementAST {
    Do { action: String },
    Set { variable: String, value: ExpressionAST },
    SetState { state: String },
    If { condition: ConditionAST, then_body: Vec<StatementAST>, else_body: Option<Vec<StatementAST>> },
    Pass,

    // NEW: Add these
    Return { value: ExpressionAST },
    Panic { message: String },
    ProcCall { name: String, body: Vec<StatementAST> },  // For PROC:name: blocks
}
```

### 3. Config Parser

Handle config variables like:
```yaml
CFG_PP_RELEASE AT: float  # range 0..1, default: 0.6
```

Parse to:
```rust
pub struct ConfigValue {
    pub name: String,        // "CFG_PP_RELEASE"
    pub attr: Option<String>, // "AT"
    pub type_hint: String,   // "float"
    pub comment: Option<String>, // "range 0..1, default: 0.6"
}
```

### 4. Type Definition Parser

Handle type definitions like:
```yaml
f2: float x, float y
DirectionType:
  fields:
    cardinality: DirectionTypeCardinality
    south: bool
  const:
    - NORTH(V, false, false)
  alias: DT
```

### 5. Procedure Parser

Handle both `proc:` and `proc-ai:` sections:
```yaml
proc:
  opt_dir:DirectionType:
    - SET south: actor.pp.y < actor.dest.y
    - RETURN: (V, south, false)

proc-ai:
  distance_to_target:
    x:float: abs(actor.dir.target.x - actor.pos.x)
```

### 6. JSON Output (`src/dsl/json_output.rs`)

**Change from YAML to JSON output:**

```rust
impl StructuredBehaviorSpec {
    pub fn save_to_json(&self, path: &str) -> DslResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_json(path: &str) -> DslResult<Self> {
        let json = std::fs::read_to_string(path)?;
        let spec = serde_json::from_str(&json)?;
        Ok(spec)
    }
}
```

**Benefits:**
- Faster parsing (no unsafe-libyaml)
- Smaller files
- Better error messages
- Native Rust support

### 7. Enhanced CLI Tool

Update `src/bin/actor_converter.rs`:

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    let input_file = &args[1];
    let output_file = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
        // Auto-generate: actor.yaml → actor.json
        input_file.replace(".yaml", ".json")
    });

    // Parse original format
    let original = OriginalFormatParser::parse_from_file(input_file)?;

    // Convert to AST
    let ast = original.to_ast()?;

    // Convert AST to structured format
    let converter = ActorYAMLConverter::new();
    let structured = converter.ast_to_structured(ast)?;

    // Validate
    let validator = Validator::new(structured.get_native_functions());
    validator.validate(&structured)?;

    // Output JSON
    structured.save_to_json(&output_file)?;

    println!("✅ Converted {} → {}", input_file, output_file);
}
```

## Implementation Order

1. **Phase 1** (2-3 hours): Original format parser
   - Parse states (MOVE, IDLE)
   - Parse statements (DO, SET, IF, ELSE)
   - Handle nested blocks (indentation)

2. **Phase 2** (1 hour): Extended AST
   - Add RETURN, PANIC, ProcCall statements
   - Handle PROC blocks
   - Config variables

3. **Phase 3** (1 hour): Type/Procedure parsing
   - Parse type definitions
   - Parse proc: and proc-ai: blocks
   - Handle function signatures

4. **Phase 4** (30 min): JSON output
   - Replace YAML serialization with JSON
   - Update CLI tool

5. **Phase 5** (1 hour): Testing
   - Test with full actor.yaml
   - Verify all statements parse correctly
   - Check JSON output structure

## Testing Strategy

Create test files in `tests/dsl/`:

```rust
#[test]
fn test_parse_move_state() {
    let yaml = r#"
MOVE:
  - DO: move_to_target
  - SET NDT: distance_to_target
"#;
    let result = OriginalFormatParser::parse_from_str(yaml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_if_else() {
    let yaml = r#"
MOVE:
  - IF condition:
      DO: action1
    ELSE:
      DO: action2
"#;
    let result = OriginalFormatParser::parse_from_str(yaml);
    assert!(result.is_ok());
}

#[test]
fn test_parse_proc_block() {
    let yaml = r#"
MOVE:
  - PROC:switch_pp:
      DO: release_reserved(actor.ppId)
      SET NEXT: actor.reserved_point
"#;
    let result = OriginalFormatParser::parse_from_str(yaml);
    assert!(result.is_ok());
}
```

## Key Challenges

### Challenge 1: YAML Indentation

Your format uses indentation for nesting:
```yaml
- IF condition:
    - DO: action1  # Indented = inside IF body
- DO: action2      # Not indented = outside IF
```

**Solution**: Use serde_yaml::Value and manually traverse the structure, checking indentation levels.

### Challenge 2: Inline IF

```yaml
- IF actor.dest.x < 0: PASS  # Single-line IF
```

**Solution**: Check if value after `:` is a string (inline statement) or array (body block).

### Challenge 3: ELSE IF

```yaml
IF condition1:
  body1
ELSE IF condition2:
  body2
ELSE:
  body3
```

**Solution**: Parse ELSE IF as nested IF in else_body of parent IF.

### Challenge 4: PROC Blocks

```yaml
- PROC:switch_pp:
    DO: action
```

**Solution**: Treat PROC as a special statement that contains a nested body.

## Documentation

Create `docs/dsl_syntax.md`:
- Complete syntax reference
- Examples for each statement type
- How to add new keywords
- How to extend the parser

## Questions to Resolve

1. **PROC execution**: Should PROC blocks be inlined where they're called, or kept as separate procedures?
2. **Type checking**: Should we validate types at conversion time or defer to runtime?
3. **Config variables**: How should CFG_ variables be passed to the runtime?
4. **proc vs proc-ai**: Should they be handled differently?

## Success Criteria

- ✅ Parse your full actor.yaml without errors
- ✅ Output valid JSON with complete AST
- ✅ All statements preserve semantics
- ✅ Validator catches undefined functions
- ✅ Clear error messages with line numbers
- ✅ CLI tool is easy to use
