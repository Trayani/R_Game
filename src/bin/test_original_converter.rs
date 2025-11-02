/// Test the original format converter (original → structured)

use rustgame3::dsl::{OriginalBehaviorSpec, ActorYAMLConverter};

fn main() {
    let path = "state_machine/actor.yaml";

    println!("📖 Parsing {}...", path);

    // Parse original format
    let original = match OriginalBehaviorSpec::parse_from_file(path) {
        Ok(spec) => {
            println!("✅ Successfully parsed original format!");
            spec
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            std::process::exit(1);
        }
    };

    println!();
    println!("🔄 Converting to structured format...");

    // Convert to structured format
    let converter = ActorYAMLConverter::new();
    match converter.convert_original_to_structured(original) {
        Ok(structured) => {
            println!("✅ Successfully converted!");
            println!();
            println!("Structured format:");
            println!("  States: {}", structured.states.len());
            for (name, actions) in &structured.states {
                println!("    - {}: {} actions", name, actions.len());
            }
            println!("  Native procedures: {}", structured.procedures.native.len());

            // Try to serialize to JSON
            println!();
            println!("📝 Serializing to JSON...");
            match serde_json::to_string_pretty(&structured) {
                Ok(json) => {
                    println!("✅ JSON output:");
                    println!("{}", json);
                }
                Err(e) => {
                    eprintln!("❌ JSON serialization error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Conversion error: {}", e);
            std::process::exit(1);
        }
    }
}
