mod common;

use std::fs;

/// Integration tests for file push command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
#[ignore] // Run with: cargo test --test file_push_integration -- --ignored
fn test_push_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let local_file = format!("/tmp/test_push_local_{}", pid);
    let device_path_python = format!("/tmp/test_push_device_python_{}", pid);
    let device_path_rust = format!("/tmp/test_push_device_rust_{}", pid);

    // Create local test file
    let test_content = b"test content for push";
    fs::write(&local_file, test_content).expect("Failed to create local test file");

    // Push with Python idb
    let python_output =
        common::run_idb_file_command(&["push", &local_file, &device_path_python, "--udid", &udid]);

    // Push with agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "push",
        &local_file,
        &device_path_rust,
        "--udid",
        &udid,
    ]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Verify files were pushed correctly by pulling them back
    if python_output.status.success() {
        let pull_python_dest = format!("/tmp/test_push_verify_python_{}", pid);
        let pull_rust_dest = format!("/tmp/test_push_verify_rust_{}", pid);

        let _pull_python = common::run_idb_file_command(&[
            "pull",
            &device_path_python,
            &pull_python_dest,
            "--udid",
            &udid,
        ]);
        let _pull_rust = common::run_idb_file_command(&[
            "pull",
            &device_path_rust,
            &pull_rust_dest,
            "--udid",
            &udid,
        ]);

        let python_content =
            fs::read(&pull_python_dest).expect("Failed to read Python pushed file");
        let rust_content = fs::read(&pull_rust_dest).expect("Failed to read Rust pushed file");

        assert_eq!(
            python_content, test_content,
            "Python pushed content differs"
        );
        assert_eq!(rust_content, test_content, "Rust pushed content differs");

        // Cleanup
        let _ = fs::remove_file(&pull_python_dest);
        let _ = fs::remove_file(&pull_rust_dest);
        let _ = common::run_idb_file_command(&["rm", &device_path_python, "--udid", &udid]);
        let _ = common::run_idb_file_command(&["rm", &device_path_rust, "--udid", &udid]);
    }

    let _ = fs::remove_file(&local_file);
}

#[test]
#[ignore]
fn test_push_with_bundle_id_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);
    let bundle_id = common::get_test_bundle_id();

    let pid = std::process::id();
    let local_file = format!("/tmp/test_push_bundle_local_{}", pid);
    let device_path_python = format!("/tmp/test_push_bundle_python_{}", pid);
    let device_path_rust = format!("/tmp/test_push_bundle_rust_{}", pid);

    // Create local test file
    let test_content = b"bundle test content";
    fs::write(&local_file, test_content).expect("Failed to create local test file");

    // Push with Python idb
    let python_output = common::run_idb_file_command(&[
        "push",
        &local_file,
        &device_path_python,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Push with agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "push",
        &local_file,
        &device_path_rust,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    if python_output.status.success() {
        let _ = common::run_idb_file_command(&[
            "rm",
            &device_path_python,
            "--bundle-id",
            &bundle_id,
            "--udid",
            &udid,
        ]);
        let _ = common::run_idb_file_command(&[
            "rm",
            &device_path_rust,
            "--bundle-id",
            &bundle_id,
            "--udid",
            &udid,
        ]);
    }

    let _ = fs::remove_file(&local_file);
}

#[test]
#[ignore]
fn test_push_binary_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let local_file = format!("/tmp/test_push_binary_local_{}", pid);
    let device_path_python = format!("/tmp/test_push_binary_python_{}", pid);
    let device_path_rust = format!("/tmp/test_push_binary_rust_{}", pid);

    // Create binary test file with various byte values
    let test_content: Vec<u8> = (0..256).map(|i| i as u8).collect();
    fs::write(&local_file, &test_content).expect("Failed to create local test file");

    // Push with Python idb
    let python_output =
        common::run_idb_file_command(&["push", &local_file, &device_path_python, "--udid", &udid]);

    // Push with agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "push",
        &local_file,
        &device_path_rust,
        "--udid",
        &udid,
    ]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Verify binary files were pushed correctly
    if python_output.status.success() {
        let pull_python_dest = format!("/tmp/test_push_binary_verify_python_{}", pid);
        let pull_rust_dest = format!("/tmp/test_push_binary_verify_rust_{}", pid);

        let _pull_python = common::run_idb_file_command(&[
            "pull",
            &device_path_python,
            &pull_python_dest,
            "--udid",
            &udid,
        ]);
        let _pull_rust = common::run_idb_file_command(&[
            "pull",
            &device_path_rust,
            &pull_rust_dest,
            "--udid",
            &udid,
        ]);

        let python_content =
            fs::read(&pull_python_dest).expect("Failed to read Python pushed file");
        let rust_content = fs::read(&pull_rust_dest).expect("Failed to read Rust pushed file");

        assert_eq!(
            python_content, test_content,
            "Python binary content differs"
        );
        assert_eq!(rust_content, test_content, "Rust binary content differs");

        // Cleanup
        let _ = fs::remove_file(&pull_python_dest);
        let _ = fs::remove_file(&pull_rust_dest);
        let _ = common::run_idb_file_command(&["rm", &device_path_python, "--udid", &udid]);
        let _ = common::run_idb_file_command(&["rm", &device_path_rust, "--udid", &udid]);
    }

    let _ = fs::remove_file(&local_file);
}

#[test]
#[ignore]
fn test_push_nonexistent_source_error() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_file = "/tmp/this_file_does_not_exist_12345";
    let device_path = "/tmp/test_push_error_dest";

    // Both should fail
    let python_output =
        common::run_idb_file_command(&["push", nonexistent_file, device_path, "--udid", &udid]);
    let rust_output = common::run_agent_mobile_file_command(&[
        "push",
        nonexistent_file,
        device_path,
        "--udid",
        &udid,
    ]);

    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_push_overwrite_existing_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let local_file1 = format!("/tmp/test_push_overwrite_local1_{}", pid);
    let local_file2 = format!("/tmp/test_push_overwrite_local2_{}", pid);
    let device_path_python = format!("/tmp/test_push_overwrite_python_{}", pid);
    let device_path_rust = format!("/tmp/test_push_overwrite_rust_{}", pid);

    // Create local test files with different content
    let content1 = b"original content";
    let content2 = b"overwritten content";
    fs::write(&local_file1, content1).expect("Failed to create first local test file");
    fs::write(&local_file2, content2).expect("Failed to create second local test file");

    // Push first file with Python idb
    let _push1_python =
        common::run_idb_file_command(&["push", &local_file1, &device_path_python, "--udid", &udid]);

    // Push first file with agent-mobile
    let _push1_rust = common::run_agent_mobile_file_command(&[
        "push",
        &local_file1,
        &device_path_rust,
        "--udid",
        &udid,
    ]);

    // Overwrite with second file using Python idb
    let python_output =
        common::run_idb_file_command(&["push", &local_file2, &device_path_python, "--udid", &udid]);

    // Overwrite with second file using agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "push",
        &local_file2,
        &device_path_rust,
        "--udid",
        &udid,
    ]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Verify files were overwritten correctly
    if python_output.status.success() {
        let pull_python_dest = format!("/tmp/test_push_overwrite_verify_python_{}", pid);
        let pull_rust_dest = format!("/tmp/test_push_overwrite_verify_rust_{}", pid);

        let _pull_python = common::run_idb_file_command(&[
            "pull",
            &device_path_python,
            &pull_python_dest,
            "--udid",
            &udid,
        ]);
        let _pull_rust = common::run_idb_file_command(&[
            "pull",
            &device_path_rust,
            &pull_rust_dest,
            "--udid",
            &udid,
        ]);

        let python_content =
            fs::read(&pull_python_dest).expect("Failed to read Python pushed file");
        let rust_content = fs::read(&pull_rust_dest).expect("Failed to read Rust pushed file");

        assert_eq!(python_content, content2, "Python overwrite failed");
        assert_eq!(rust_content, content2, "Rust overwrite failed");

        // Cleanup
        let _ = fs::remove_file(&pull_python_dest);
        let _ = fs::remove_file(&pull_rust_dest);
        let _ = common::run_idb_file_command(&["rm", &device_path_python, "--udid", &udid]);
        let _ = common::run_idb_file_command(&["rm", &device_path_rust, "--udid", &udid]);
    }

    let _ = fs::remove_file(&local_file1);
    let _ = fs::remove_file(&local_file2);
}
