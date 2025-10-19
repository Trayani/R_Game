mod common;

use common::{parse_standard_test, flip_standard_horizontal, flip_standard_vertical, flip_standard_both};
use rustgame3::raycast;
use std::fs;

#[test]
fn standard_format_tests() {
    let test_dir = "./test_data/standard";
    let mut passed = 0;

    if let Ok(entries) = fs::read_dir(test_dir) {
        let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();

            // Skip markdown files and directories
            if path.extension().and_then(|s| s.to_str()) == Some("md") || path.is_dir() {
                continue;
            }

            let test_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            match parse_standard_test(&path) {
                Ok((grid, start_x, start_y, expected_visible)) => {
                    // Test all 4 variants
                    let variants = vec![
                        ("original", (grid.clone(), start_x, start_y, expected_visible.clone())),
                        ("h_flip", flip_standard_horizontal(&grid, start_x, start_y, &expected_visible)),
                        ("v_flip", flip_standard_vertical(&grid, start_x, start_y, &expected_visible)),
                        ("hv_flip", flip_standard_both(&grid, start_x, start_y, &expected_visible)),
                    ];

                    for (variant_name, (variant_grid, variant_x, variant_y, variant_expected)) in variants {
                        let actual_visible = raycast(&variant_grid, variant_x, variant_y, false);
                        let missing: Vec<_> = variant_expected.difference(&actual_visible).copied().collect();
                        let extra: Vec<_> = actual_visible.difference(&variant_expected).copied().collect();

                        if !missing.is_empty() || !extra.is_empty() {
                            panic!(
                                "Test '{}' [{}] failed (missing: {}, extra: {})",
                                test_name, variant_name, missing.len(), extra.len()
                            );
                        }
                    }

                    passed += 1;
                }
                Err(e) => {
                    panic!("Test '{}' failed to parse: {}", test_name, e);
                }
            }
        }
    }

    println!("All {} standard format tests passed", passed);
}
