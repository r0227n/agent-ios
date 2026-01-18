mod common;

use std::process::Command;

#[test]
fn test_file_write_and_read() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let test_content = "Hello from agent-mobile test!";
    let test_path = "/tmp/agent_mobile_test_file.txt";

    // Test 1: Write file using stdin
    let python_write = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | idb file write {} --udid {}",
            test_content, test_path, udid
        ))
        .output()
        .expect("Failed to run Python idb file write");

    let rust_write = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo '{}' | ./target/debug/agent-mobile idb file write {} --udid {}",
            test_content, test_path, udid
        ))
        .output()
        .expect("Failed to run agent-mobile file write");

    assert_eq!(
        python_write.status.code(),
        rust_write.status.code(),
        "Write command exit codes differ"
    );

    // Test 2: Read file to stdout
    let python_read = common::run_idb_file_command(&["read", test_path, "--udid", &udid]);
    let rust_read = common::run_agent_mobile_file_command(&["read", test_path, "--udid", &udid]);

    assert_eq!(
        python_read.status.code(),
        rust_read.status.code(),
        "Read command exit codes differ"
    );

    // Compare stdout content
    let python_output = String::from_utf8_lossy(&python_read.stdout);
    let rust_output = String::from_utf8_lossy(&rust_read.stdout);

    assert_eq!(
        python_output.trim(),
        rust_output.trim(),
        "Read outputs differ"
    );

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", test_path, "--udid", &udid]);
}

#[test]
fn test_file_read_nonexistent() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_path = format!("/tmp/nonexistent_file_{}.txt", std::process::id());

    let python_output = common::run_idb_file_command(&["read", &nonexistent_path, "--udid", &udid]);
    let rust_output =
        common::run_agent_mobile_file_command(&["read", &nonexistent_path, "--udid", &udid]);

    // Both should fail
    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());
}

#[test]
fn test_file_write_binary_data() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let test_path = "/tmp/agent_mobile_test_binary.bin";
    let binary_data = vec![0u8, 1, 2, 3, 255, 254, 253];

    // Create binary file
    let temp_file = std::env::temp_dir().join("test_binary.bin");
    std::fs::write(&temp_file, &binary_data).expect("Failed to create temp file");

    // Write binary file
    let rust_write = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cat {} | ./target/debug/agent-mobile idb file write {} --udid {}",
            temp_file.display(),
            test_path,
            udid
        ))
        .output()
        .expect("Failed to run agent-mobile file write");

    assert!(rust_write.status.success(), "Binary write failed");

    // Read back
    let rust_read = common::run_agent_mobile_file_command(&["read", test_path, "--udid", &udid]);
    assert!(rust_read.status.success(), "Binary read failed");

    // Verify binary content
    assert_eq!(rust_read.stdout, binary_data, "Binary data mismatch");

    // Cleanup
    let _ = common::run_idb_file_command(&["rm", test_path, "--udid", &udid]);
    let _ = std::fs::remove_file(temp_file);
}
