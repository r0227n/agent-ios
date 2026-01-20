/// Integration tests for mkdir command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
fn test_mkdir_creates_directory() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    // Create a test directory in root container
    let test_path = format!("/tmp/test_mkdir_{}", std::process::id());

    let output = crate::common::run_agent_mobile_file_command(&[
        "mkdir", &test_path, "--root", "--udid", &udid,
    ]);

    // Should not error
    if !output.status.success() {
        panic!("mkdir failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    // No output on success (matching Python idb behavior)
    assert!(
        output.stdout.is_empty(),
        "Expected no output on success, got: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Cleanup
    let _ = crate::common::run_idb_file_command(&["rm", &test_path, "--root", "--udid", &udid]);
}

#[test]
fn test_mkdir_without_udid() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let test_path = format!("/tmp/test_mkdir_no_udid_{}", std::process::id());

    let output = crate::common::run_agent_mobile_file_command(&["mkdir", &test_path, "--root"]);

    // Should not error if at least one simulator is booted
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("No companions available")
                || stderr.contains("Multiple companions available"),
            "Expected companion error, got: {}",
            stderr
        );
        return; // エラーが期待通りなら早期リターン
    }

    // Cleanup
    let _ = crate::common::run_idb_file_command(&["rm", &test_path, "--root", "--udid", &udid]);
}

#[test]
fn test_mkdir_invalid_bundle_id() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let test_path = "/tmp/test_invalid";

    let output = crate::common::run_agent_mobile_file_command(&[
        "mkdir",
        test_path,
        "--bundle-id",
        "com.nonexistent.app",
        "--udid",
        &udid,
    ]);

    // Should error
    assert!(
        !output.status.success(),
        "Expected error for invalid bundle ID"
    );
}

#[test]
fn test_mkdir_compatibility_with_python_idb() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let test_path1 = format!("/tmp/test_python_mkdir1_{}", std::process::id());
    let test_path2 = format!("/tmp/test_python_mkdir2_{}", std::process::id());

    // Run Python idb
    let python_output =
        crate::common::run_idb_file_command(&["mkdir", &test_path1, "--root", "--udid", &udid]);

    // Run Rust implementation
    let rust_output = crate::common::run_agent_mobile_file_command(&[
        "mkdir",
        &test_path2,
        "--root",
        "--udid",
        &udid,
    ]);

    // Both should succeed with no output
    assert_eq!(python_output.status.success(), rust_output.status.success());
    assert_eq!(
        python_output.stdout.is_empty(),
        rust_output.stdout.is_empty()
    );

    // Compare outputs
    crate::common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ = crate::common::run_idb_file_command(&["rm", &test_path1, "--root", "--udid", &udid]);
    let _ = crate::common::run_idb_file_command(&["rm", &test_path2, "--root", "--udid", &udid]);
}

#[test]
fn test_mkdir_with_bundle_id_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);
    let bundle_id = crate::common::get_test_bundle_id();

    let test_path_python = format!("/tmp/test_mkdir_bundle_python_{}", std::process::id());
    let test_path_rust = format!("/tmp/test_mkdir_bundle_rust_{}", std::process::id());

    // Run Python idb
    let python_output = crate::common::run_idb_file_command(&[
        "mkdir",
        &test_path_python,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Run Rust implementation
    let rust_output = crate::common::run_agent_mobile_file_command(&[
        "mkdir",
        &test_path_rust,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Compare outputs
    crate::common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ = crate::common::run_idb_file_command(&[
        "rm",
        &test_path_python,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let _ = crate::common::run_idb_file_command(&[
        "rm",
        &test_path_rust,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
}

#[test]
fn test_mkdir_nested_directory_equivalence() {
    crate::common::build_agent_mobile();
    let udid = crate::common::get_available_udid();
    crate::common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let parent_dir_python = format!("/tmp/test_mkdir_nested_python_{}", pid);
    let parent_dir_rust = format!("/tmp/test_mkdir_nested_rust_{}", pid);
    let nested_path_python = format!("{}/nested/deep/path", parent_dir_python);
    let nested_path_rust = format!("{}/nested/deep/path", parent_dir_rust);

    // Create parent directories first
    let _python_parent = crate::common::run_idb_file_command(&[
        "mkdir",
        &parent_dir_python,
        "--root",
        "--udid",
        &udid,
    ]);
    let _rust_parent = crate::common::run_agent_mobile_file_command(&[
        "mkdir",
        &parent_dir_rust,
        "--root",
        "--udid",
        &udid,
    ]);

    // Create nested path with Python idb
    let python_output = crate::common::run_idb_file_command(&[
        "mkdir",
        &nested_path_python,
        "--root",
        "--udid",
        &udid,
    ]);

    // Create nested path with agent-mobile
    let rust_output = crate::common::run_agent_mobile_file_command(&[
        "mkdir",
        &nested_path_rust,
        "--root",
        "--udid",
        &udid,
    ]);

    // Compare outputs
    crate::common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ =
        crate::common::run_idb_file_command(&["rm", &parent_dir_python, "--root", "--udid", &udid]);
    let _ =
        crate::common::run_idb_file_command(&["rm", &parent_dir_rust, "--root", "--udid", &udid]);
}
