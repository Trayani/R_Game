/// Test the original format parser

use rustgame3::dsl::OriginalBehaviorSpec;

fn main() {
    let path = "state_machine/actor.yaml";

    println!("📖 Parsing {}...", path);

    match OriginalBehaviorSpec::parse_from_file(path) {
        Ok(spec) => {
            println!("✅ Successfully parsed!");
            println!();
            println!("States found: {}", spec.states.len());
            for (name, statements) in &spec.states {
                println!("  - {}: {} statements", name, statements.len());
            }
            println!();
            println!("Config vars: {}", spec.config.len());
            for (name, _) in &spec.config {
                println!("  - {}", name);
            }
            println!();
            println!("Procedures: {}", spec.procedures.len());
            for (name, _) in &spec.procedures {
                println!("  - {}", name);
            }
            println!();
            println!("Native procedures: {}", spec.native_procedures.len());
            for (name, _) in &spec.native_procedures {
                println!("  - {}", name);
            }
            println!();
            println!("Types: {}", spec.types.len());
            for (name, _) in &spec.types {
                println!("  - {}", name);
            }
            println!();
            println!("Constants: {}", spec.constants.len());
            for (name, type_name) in &spec.constants {
                println!("  - {}: {}", name, type_name);
            }
            println!();
            println!("Parameters: {}", spec.parameters.len());
            for (name, type_name) in &spec.parameters {
                println!("  - {}: {}", name, type_name);
            }
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
