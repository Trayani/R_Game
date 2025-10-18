mod common;

use common::{load_test, run_test};
use std::fs;

#[test]
fn json_validation_tests() {
    let test_dir = "./test_data";
    let mut passed = 0;

    if let Ok(entries) = fs::read_dir(test_dir) {
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(test_data) = load_test(&path) {
                    let (all_passed, failed_variant, missing_count, extra_count) = run_test(&test_data);

                    if all_passed {
                        passed += 1;
                    } else {
                        let variant = failed_variant.unwrap_or_else(|| "unknown".to_string());
                        panic!(
                            "Test '{}' failed [{}] (missing: {}, extra: {})",
                            test_data.test_name, variant, missing_count, extra_count
                        );
                    }
                }
            }
        }
    }

    println!("All {} JSON validation tests passed", passed);
}
