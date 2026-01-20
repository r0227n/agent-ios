use std::process::Command;

/// Normalize a list-apps output line for comparison.
/// - Sorts the architectures field (Python idb uses set[str] with non-deterministic order)
/// - Excludes process_state (index 4) and pid (index 6) as they can change between command runs
fn normalize_list_apps_line(line: &str) -> String {
    let parts: Vec<&str> = line.split(" | ").collect();
    if parts.len() != 7 {
        return line.to_string();
    }

    // The 4th field (index 3) contains architectures like "arm64, x86_64"
    let mut archs: Vec<&str> = parts[3].split(", ").collect();
    archs.sort();
    let sorted_archs = archs.join(", ");

    // Exclude process_state (index 4) and pid (index 6) as they are dynamic
    // Only compare: bundle_id | name | install_type | arch | debuggable
    format!(
        "{} | {} | {} | {} | {}",
        parts[0], parts[1], parts[2], sorted_archs, parts[5]
    )
}

/// Integration test comparing agent-mobile list-apps output with Python idb
#[test]
fn test_list_apps_matches_python_idb() {
    // Get UDID of first booted simulator
    let list_targets_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-targets"])
        .output()
        .expect("Failed to run list-targets");

    let targets_str = String::from_utf8_lossy(&list_targets_output.stdout);
    let udid = targets_str
        .lines()
        .find(|line| line.contains("\"state\":\"Booted\""))
        .and_then(|line| {
            line.split("\"udid\":\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
        })
        .expect("No booted simulator found. Please boot a simulator first.");

    // Run Python idb
    let python_output = Command::new("idb")
        .args(["list-apps", "--udid", udid])
        .output()
        .expect("Failed to run Python idb. Make sure idb is installed.");

    // Run Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-apps", "--udid", udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Normalize and sort both outputs for comparison
    // Normalization sorts architectures within each line (Python uses set[str] which has non-deterministic order)
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let mut python_lines: Vec<String> = python_stdout
        .lines()
        .map(normalize_list_apps_line)
        .collect();

    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
    let mut rust_lines: Vec<String> = rust_stdout.lines().map(normalize_list_apps_line).collect();

    python_lines.sort();
    rust_lines.sort();

    // Compare outputs
    assert_eq!(
        python_lines, rust_lines,
        "list-apps output differs from Python idb"
    );
}

/// Test that list-apps works without UDID when only one target is available
#[test]
fn test_list_apps_without_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-apps"])
        .output()
        .expect("Failed to run agent-mobile");

    // Should not error if at least one simulator is booted
    if !output.status.success() {
        panic!(
            "list-apps failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Output should contain app information
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(" | "),
        "Output should be in pipe-delimited format"
    );
}

/// Test that output format matches expected structure
#[test]
fn test_list_apps_output_format() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-apps"])
        .output()
        .expect("Failed to run agent-mobile");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Each line should have the format:
    // bundle_id | name | install_type | arch | process_state | debuggable | pid=X
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(" | ").collect();
        assert_eq!(
            parts.len(),
            7,
            "Expected 7 pipe-separated fields in output: {}",
            line
        );

        // Last field should be pid=...
        assert!(
            parts[6].starts_with("pid="),
            "Last field should start with 'pid=': {}",
            parts[6]
        );
    }
}
