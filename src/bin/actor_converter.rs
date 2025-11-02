/// Actor YAML Converter - CLI tool
///
/// Converts original actor.yaml (human-friendly) to structured JSON (machine-friendly).
///
/// Usage:
///   cargo run --bin actor_converter <input.yaml> [output.json]
///
/// If output file is not specified, defaults to <input>.json
/// Example: actor.yaml → actor.json

use rustgame3::dsl::{OriginalBehaviorSpec, ActorYAMLConverter};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.yaml> [output.json]", args[0]);
        eprintln!();
        eprintln!("Converts original actor YAML to structured JSON.");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} actor.yaml", args[0]);
        eprintln!("  {} actor.yaml actor.json", args[0]);
        eprintln!();
        eprintln!("If no output file is specified, uses <input>.json");
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = if let Some(output) = args.get(2) {
        output.clone()
    } else {
        // Auto-generate: actor.yaml → actor.json
        input_file.replace(".yaml", ".json")
    };

    println!("📖 Parsing {}...", input_file);

    // Parse original format
    let original = match OriginalBehaviorSpec::parse_from_file(input_file) {
        Ok(spec) => {
            println!("✅ Successfully parsed original format!");
            println!("   States: {}", spec.states.len());
            println!("   Config variables: {}", spec.config.len());
            println!("   Procedures: {}", spec.procedures.len());
            println!("   Native procedures: {}", spec.native_procedures.len());
            println!("   Types: {}", spec.types.len());
            spec
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };

    println!();
    println!("🔄 Converting to structured format...");

    // Convert to structured format
    let converter = ActorYAMLConverter::new();
    let structured = match converter.convert_original_to_structured(original) {
        Ok(spec) => {
            println!("✅ Conversion successful!");
            println!("   States: {}", spec.states.len());
            for (name, actions) in &spec.states {
                println!("     - {}: {} actions", name, actions.len());
            }
            spec
        }
        Err(e) => {
            eprintln!("❌ Conversion error: {}", e);
            process::exit(1);
        }
    };

    println!();
    println!("📝 Writing JSON to {}...", output_file);

    // Serialize to JSON
    let json = match serde_json::to_string_pretty(&structured) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("❌ JSON serialization error: {}", e);
            process::exit(1);
        }
    };

    // Write output file
    match fs::write(&output_file, &json) {
        Ok(_) => {
            println!("✅ Written to {}", output_file);
        }
        Err(e) => {
            eprintln!("❌ Error writing to {}: {}", output_file, e);
            process::exit(1);
        }
    }

    println!();
    println!("🎉 Done!");
    println!();
    println!("Summary:");
    println!("  Input:  {} (original YAML format)", input_file);
    println!("  Output: {} (structured JSON format)", output_file);
}
