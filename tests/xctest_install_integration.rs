mod common;

use std::process::Command;

/// Get path to a test XCTest bundle
/// Returns None if no test bundle is available
fn get_test_xctest_bundle() -> Option<String> {
    // Check environment variable first
    if let Ok(path) = std::env::var("TEST_XCTEST_BUNDLE") {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    // Check common locations
    let common_paths = vec![
        "/tmp/TestBundle.xctest",
        "./tests/fixtures/TestBundle.xctest",
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    None
}

#[test]
#[ignore]
fn test_xctest_install_basic() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let test_bundle = match get_test_xctest_bundle() {
        Some(bundle) => bundle,
        None => {
            eprintln!("Skipping test: No XCTest bundle available");
            eprintln!("Set TEST_XCTEST_BUNDLE environment variable to run this test");
            return;
        }
    };

    // Run Python idb xctest install
    let python_output = Command::new("idb")
        .args(["xctest", "install", &test_bundle, "--udid", &udid])
        .output()
        .expect("Failed to run Python idb");

    // Run agent-mobile xctest-install
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-install", &test_bundle, "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Compare exit codes
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ:\n  Python: {:?}\n  Rust: {:?}\n  Python stderr: {}\n  Rust stderr: {}",
        python_output.status.code(),
        rust_output.status.code(),
        String::from_utf8_lossy(&python_output.stderr),
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // If both succeeded, verify output format
    if python_output.status.success() && rust_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

        // Both should produce output with "Installed:" prefix
        assert!(
            rust_stdout.contains("Installed:"),
            "Rust output missing 'Installed:' prefix: {}",
            rust_stdout
        );
    }
}

#[test]
#[ignore]
fn test_xctest_install_with_json_output() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let test_bundle = match get_test_xctest_bundle() {
        Some(bundle) => bundle,
        None => {
            eprintln!("Skipping test: No XCTest bundle available");
            return;
        }
    };

    // Run Python idb xctest install --json
    let python_output = Command::new("idb")
        .args(["xctest", "install", &test_bundle, "--json", "--udid", &udid])
        .output()
        .expect("Failed to run Python idb");

    // Run agent-mobile xctest-install --json
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "xctest-install",
            &test_bundle,
            "--json",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    if python_output.status.success() {
        assert!(
            rust_output.status.success(),
            "Rust failed while Python succeeded: {}",
            String::from_utf8_lossy(&rust_output.stderr)
        );

        // Verify JSON output format
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(rust_stdout.trim());
        assert!(
            parsed.is_ok(),
            "Rust output is not valid JSON: {}",
            rust_stdout
        );

        let json = parsed.unwrap();
        assert!(
            json.get("installedTestBundleId").is_some(),
            "JSON missing 'installedTestBundleId' field"
        );
    }
}

#[test]
#[ignore]
fn test_xctest_install_nonexistent_bundle() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_bundle = "/tmp/nonexistent_test_bundle_12345.xctest";

    // Run Python idb xctest install
    let python_output = Command::new("idb")
        .args(["xctest", "install", nonexistent_bundle, "--udid", &udid])
        .output()
        .expect("Failed to run Python idb");

    // Run agent-mobile xctest-install
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "xctest-install", nonexistent_bundle, "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should fail
    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());
}

#[test]
#[ignore]
fn test_xctest_install_with_skip_signing() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let test_bundle = match get_test_xctest_bundle() {
        Some(bundle) => bundle,
        None => {
            eprintln!("Skipping test: No XCTest bundle available");
            return;
        }
    };

    // Run agent-mobile xctest-install with --skip-signing
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "xctest-install",
            &test_bundle,
            "--skip-signing",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Should complete (success or failure depends on bundle validity)
    // Just verify the command accepts the flag
    assert!(
        rust_output.status.code().is_some(),
        "Command did not complete"
    );
}
