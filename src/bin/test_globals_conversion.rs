/// Test constants and parameters conversion

use rustgame3::dsl::{OriginalBehaviorSpec, ActorYAMLConverter};

fn main() {
    let yaml = r#"
global:
  constants:
    CONST_VALUE: int
  parameters:
    param_value: float

IDLE:
  - SET x: CONST_VALUE
  - SET y: param_value

proc:
  test_proc:int:
    - SET result: CONST_VALUE
    - RETURN: result

proc-ai:
  native_func(): some function
"#;

    println!("📖 Parsing test YAML...");
    let original = match OriginalBehaviorSpec::parse_from_str(yaml) {
        Ok(spec) => {
            println!("✅ Parse successful!");
            println!("   Constants: {}", spec.constants.len());
            for (name, type_name) in &spec.constants {
                println!("     - {}: {}", name, type_name);
            }
            println!("   Parameters: {}", spec.parameters.len());
            for (name, type_name) in &spec.parameters {
                println!("     - {}: {}", name, type_name);
            }
            spec
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            std::process::exit(1);
        }
    };

    println!();
    println!("🔄 Converting to structured format...");
    let converter = ActorYAMLConverter::new();
    match converter.convert_original_to_structured(original) {
        Ok(structured) => {
            println!("✅ Conversion successful!");
            println!("   Constants: {}", structured.constants.len());
            for (name, def) in &structured.constants {
                println!("     - {}: {}", name, def.type_name);
            }
            println!("   Parameters: {}", structured.parameters.len());
            for (name, def) in &structured.parameters {
                println!("     - {}: {}", name, def.type_name);
            }

            println!();
            println!("📝 Checking implicit call handling...");
            println!("   IDLE state actions:");
            if let Some(actions) = structured.states.get("IDLE") {
                for (i, action) in actions.iter().enumerate() {
                    println!("     {}. {:?}", i+1, action);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Conversion error: {}", e);
            std::process::exit(1);
        }
    }
}
