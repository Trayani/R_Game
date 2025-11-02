/// Actor YAML Converter - CLI tool
///
/// Converts compact (human-friendly) actor.yaml to structured (machine-friendly) YAML.
///
/// Usage:
///   cargo run --bin actor_converter <input.yaml> [output.yaml]
///
/// If output file is not specified, prints to stdout.

use rustgame3::dsl::{ActorYAMLConverter, Validator};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.yaml> [output.yaml]", args[0]);
        eprintln!();
        eprintln!("Converts compact actor YAML to structured YAML.");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} actor.yaml", args[0]);
        eprintln!("  {} actor.yaml actor_compiled.yaml", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = args.get(2);

    // Read input file
    let compact_yaml = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading {}: {}", input_file, e);
            process::exit(1);
        }
    };

    println!("📖 Reading {}", input_file);

    // Create converter
    let converter = ActorYAMLConverter::new();

    // Convert
    println!("🔄 Converting...");
    let structured_yaml = match converter.convert_str(&compact_yaml) {
        Ok(yaml) => yaml,
        Err(e) => {
            eprintln!("❌ Conversion error: {}", e);
            process::exit(1);
        }
    };

    println!("✅ Conversion successful!");

    // Output
    match output_file {
        Some(path) => {
            match fs::write(path, &structured_yaml) {
                Ok(_) => {
                    println!("📝 Written to {}", path);
                }
                Err(e) => {
                    eprintln!("❌ Error writing to {}: {}", path, e);
                    process::exit(1);
                }
            }
        }
        None => {
            println!("\n{}", structured_yaml);
        }
    }

    println!("🎉 Done!");
}
