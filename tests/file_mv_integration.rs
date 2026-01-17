mod common;

use std::process::Command;

/// Integration tests for file mv command
/// Compares Python idb output with agent-mobile output to ensure compatibility

#[test]
#[ignore] // Run with: cargo test --test file_mv_integration -- --ignored
fn test_mv_basic_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let src_path = format!("/tmp/test_mv_src_{}", pid);
    let dst_path = format!("/tmp/test_mv_dst_{}", pid);

    // Create test file using Python idb
    let _create = common::run_idb_file_command(&["push", "/dev/null", &src_path, "--udid", &udid]);

    // Move file with Python idb
    let python_output =
        common::run_idb_file_command(&["mv", &src_path, &dst_path, "--udid", &udid]);

    // Recreate test file for Rust test
    let _create = common::run_idb_file_command(&["push", "/dev/null", &src_path, "--udid", &udid]);

    // Move file with agent-mobile
    let rust_output =
        common::run_agent_mobile_file_command(&["mv", &src_path, &dst_path, "--udid", &udid]);

    common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &dst_path, "--udid", &udid]);
}

#[test]
#[ignore]
fn test_mv_multiple_files_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let src1 = format!("/tmp/test_mv_multi_src1_{}", pid);
    let src2 = format!("/tmp/test_mv_multi_src2_{}", pid);
    let dst_dir = format!("/tmp/test_mv_multi_dst_{}", pid);

    // Create test files and directory
    let _create1 = common::run_idb_file_command(&["push", "/dev/null", &src1, "--udid", &udid]);
    let _create2 = common::run_idb_file_command(&["push", "/dev/null", &src2, "--udid", &udid]);
    let _mkdir = common::run_idb_file_command(&["mkdir", &dst_dir, "--udid", &udid]);

    // Move files with Python idb
    let python_output = common::run_idb_file_command(&[
        "mv", &src1, &src2, &dst_dir, "--udid", &udid,
    ]);

    // Recreate test files for Rust test
    let _create1 = common::run_idb_file_command(&["push", "/dev/null", &src1, "--udid", &udid]);
    let _create2 = common::run_idb_file_command(&["push", "/dev/null", &src2, "--udid", &udid]);

    // Move files with agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "mv", &src1, &src2, &dst_dir, "--udid", &udid,
    ]);

    common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &dst_dir, "--udid", &udid]);
}

#[test]
#[ignore]
fn test_mv_with_root_flag_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let src_path = format!("/tmp/test_mv_root_src_{}", pid);
    let dst_path = format!("/tmp/test_mv_root_dst_{}", pid);

    // Create test file
    let _create = common::run_idb_file_command(&[
        "push",
        "/dev/null",
        &src_path,
        "--root",
        "--udid",
        &udid,
    ]);

    // Move with Python idb
    let python_output = common::run_idb_file_command(&[
        "mv", &src_path, &dst_path, "--root", "--udid", &udid,
    ]);

    // Recreate for Rust test
    let _create = common::run_idb_file_command(&[
        "push",
        "/dev/null",
        &src_path,
        "--root",
        "--udid",
        &udid,
    ]);

    // Move with agent-mobile
    let rust_output = common::run_agent_mobile_file_command(&[
        "mv", &src_path, &dst_path, "--root", "--udid", &udid,
    ]);

    common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &dst_path, "--root", "--udid", &udid]);
}

#[test]
#[ignore]
fn test_mv_source_not_found_error() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_src = "/tmp/this_file_does_not_exist_12345";
    let dst_path = "/tmp/test_mv_dst";

    // Both should fail
    let python_output =
        common::run_idb_file_command(&["mv", nonexistent_src, dst_path, "--udid", &udid]);
    let rust_output =
        common::run_agent_mobile_file_command(&["mv", nonexistent_src, dst_path, "--udid", &udid]);

    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());

    common::compare_file_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_mv_rename_file_equivalence() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let pid = std::process::id();
    let original_name = format!("/tmp/test_mv_rename_original_{}", pid);
    let new_name = format!("/tmp/test_mv_rename_new_{}", pid);

    // Create test file
    let _create =
        common::run_idb_file_command(&["push", "/dev/null", &original_name, "--udid", &udid]);

    // Rename with Python idb
    let python_output =
        common::run_idb_file_command(&["mv", &original_name, &new_name, "--udid", &udid]);

    // Recreate for Rust test
    let _create =
        common::run_idb_file_command(&["push", "/dev/null", &original_name, "--udid", &udid]);

    // Rename with agent-mobile
    let rust_output =
        common::run_agent_mobile_file_command(&["mv", &original_name, &new_name, "--udid", &udid]);

    common::compare_file_command_outputs(&python_output, &rust_output);

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", &new_name, "--udid", &udid]);
}
