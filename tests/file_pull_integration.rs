mod common;

use std::fs;
use std::path::Path;

/// Integration tests for file pull command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
fn test_pull_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let device_path = format!("/tmp/test_pull_source_{}", pid);
    let python_dest = format!("/tmp/test_pull_python_dest_{}", pid);
    let rust_dest = format!("/tmp/test_pull_rust_dest_{}", pid);

    // Create test file on device with known content
    let test_content = b"test content for pull";
    let temp_local = format!("/tmp/test_pull_local_{}", pid);
    fs::write(&temp_local, test_content).expect("Failed to create local test file");
    let _push = common::run_idb_file_command(&["push", &temp_local, &device_path, "--udid", &udid]);

    // Pull with Python idb
    let python_output =
        common::run_idb_file_command(&["pull", &device_path, &python_dest, "--udid", &udid]);

    // Pull with agent-mobile
    let rust_output =
        common::run_agent_mobile_file_command(&["pull", &device_path, &rust_dest, "--udid", &udid]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Compare pulled file contents
    if python_output.status.success() {
        let python_content = fs::read(&python_dest).expect("Failed to read Python pulled file");
        let rust_content = fs::read(&rust_dest).expect("Failed to read Rust pulled file");
        assert_eq!(python_content, rust_content, "Pulled file contents differ");
        assert_eq!(
            python_content, test_content,
            "Pulled content doesn't match original"
        );
    }

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &device_path, "--udid", &udid]);
    let _ = fs::remove_file(&temp_local);
    let _ = fs::remove_file(&python_dest);
    let _ = fs::remove_file(&rust_dest);
}

#[test]
fn test_pull_to_stdout_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let device_path = format!("/tmp/test_pull_stdout_{}", pid);

    // Create test file on device
    let test_content = b"stdout test content";
    let temp_local = format!("/tmp/test_pull_stdout_local_{}", pid);
    fs::write(&temp_local, test_content).expect("Failed to create local test file");
    let _push = common::run_idb_file_command(&["push", &temp_local, &device_path, "--udid", &udid]);

    // Pull to stdout with Python idb
    let python_output = common::run_idb_file_command(&["pull", &device_path, "-", "--udid", &udid]);

    // Pull to stdout with agent-mobile
    let rust_output =
        common::run_agent_mobile_file_command(&["pull", &device_path, "-", "--udid", &udid]);

    // Compare outputs (stdout should contain file content)
    common::compare_file_command_outputs(&python_output, &rust_output);

    if python_output.status.success() {
        assert_eq!(python_output.stdout, test_content);
        assert_eq!(rust_output.stdout, test_content);
    }

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &device_path, "--udid", &udid]);
    let _ = fs::remove_file(&temp_local);
}

#[test]
fn test_pull_with_bundle_id_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);
    let bundle_id = common::get_test_bundle_id();

    let pid = std::process::id();
    let device_path = format!("/tmp/test_pull_bundle_{}", pid);
    let python_dest = format!("/tmp/test_pull_bundle_python_{}", pid);
    let rust_dest = format!("/tmp/test_pull_bundle_rust_{}", pid);

    // Create test file in app container
    let test_content = b"bundle test content";
    let temp_local = format!("/tmp/test_pull_bundle_local_{}", pid);
    fs::write(&temp_local, test_content).expect("Failed to create local test file");
    let _push = common::run_idb_file_command(&[
        "push",
        &temp_local,
        &device_path,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Pull with Python idb
    let python_output = common::run_idb_file_command(&[
        "pull",
        &device_path,
        &python_dest,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Pull with agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "pull",
        &device_path,
        &rust_dest,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Compare pulled file contents
    if python_output.status.success() {
        let python_content = fs::read(&python_dest).expect("Failed to read Python pulled file");
        let rust_content = fs::read(&rust_dest).expect("Failed to read Rust pulled file");
        assert_eq!(python_content, rust_content, "Pulled file contents differ");
    }

    // Cleanup
    let _ = common::run_idb_file_command(&[
        "rm",
        &device_path,
        "--bundle-id",
        &bundle_id,
        "--udid",
        &udid,
    ]);
    let _ = fs::remove_file(&temp_local);
    let _ = fs::remove_file(&python_dest);
    let _ = fs::remove_file(&rust_dest);
}

#[test]
fn test_pull_nonexistent_file_error() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_path = "/tmp/this_file_does_not_exist_12345";
    let dest_path = "/tmp/test_pull_error_dest";

    // Both should fail
    let python_output =
        common::run_idb_file_command(&["pull", nonexistent_path, dest_path, "--udid", &udid]);
    let rust_output = common::run_agent_mobile_file_command(&[
        "pull",
        nonexistent_path,
        dest_path,
        "--udid",
        &udid,
    ]);

    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    common::compare_file_command_outputs(&python_output, &rust_output);

    // Ensure no files were created
    assert!(!Path::new(dest_path).exists());
}

#[test]
fn test_pull_binary_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let device_path = format!("/tmp/test_pull_binary_{}", pid);
    let python_dest = format!("/tmp/test_pull_binary_python_{}", pid);
    let rust_dest = format!("/tmp/test_pull_binary_rust_{}", pid);

    // Create binary test file with various byte values
    let test_content: Vec<u8> = (0..256).map(|i| i as u8).collect();
    let temp_local = format!("/tmp/test_pull_binary_local_{}", pid);
    fs::write(&temp_local, &test_content).expect("Failed to create local test file");
    let _push = common::run_idb_file_command(&["push", &temp_local, &device_path, "--udid", &udid]);

    // Pull with Python idb
    let python_output =
        common::run_idb_file_command(&["pull", &device_path, &python_dest, "--udid", &udid]);

    // Pull with agent-mobile
    let rust_output =
        common::run_agent_mobile_file_command(&["pull", &device_path, &rust_dest, "--udid", &udid]);

    // Compare command outputs
    common::compare_file_command_outputs(&python_output, &rust_output);

    // Compare pulled file contents
    if python_output.status.success() {
        let python_content = fs::read(&python_dest).expect("Failed to read Python pulled file");
        let rust_content = fs::read(&rust_dest).expect("Failed to read Rust pulled file");
        assert_eq!(python_content, rust_content, "Binary file contents differ");
        assert_eq!(
            python_content, test_content,
            "Binary content doesn't match original"
        );
    }

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &device_path, "--udid", &udid]);
    let _ = fs::remove_file(&temp_local);
    let _ = fs::remove_file(&python_dest);
    let _ = fs::remove_file(&rust_dest);
}
