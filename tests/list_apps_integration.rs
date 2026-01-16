use std::process::Command;

/// Integration test comparing agent-mobile list-apps output with Python idb
#[test]
#[ignore] // Run with: cargo test --test list_apps_integration -- --ignored
fn test_list_apps_matches_python_idb() {
    // Get UDID of first booted simulator
    let list_targets_output = Command::new("./target/debug/agent-mobile")
        .args(&["idb", "list-targets"])
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
        .args(&["list-apps", "--udid", udid])
        .output()
        .expect("Failed to run Python idb. Make sure idb is installed.");

    // Run Rust implementation
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(&["idb", "list-apps", "--udid", udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Sort both outputs for comparison (order may vary)
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let mut python_lines: Vec<&str> = python_stdout.lines().collect();

    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
    let mut rust_lines: Vec<&str> = rust_stdout.lines().collect();

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
#[ignore]
fn test_list_apps_without_udid() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(&["idb", "list-apps"])
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
    assert!(stdout.contains(" | "), "Output should be in pipe-delimited format");
}

/// Test that output format matches expected structure
#[test]
#[ignore]
fn test_list_apps_output_format() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(&["idb", "list-apps"])
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
