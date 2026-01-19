use std::io::Write;
use std::process::{Command, Stdio};

/// Test shell CLI help output
#[test]
fn test_shell_cli_help() {
    crate::common::build_agent_mobile();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "shell", "--help"])
        .output()
        .expect("Failed to run shell --help");

    assert!(
        output.status.success(),
        "shell --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have --no-prompt option
    assert!(
        stdout.contains("--no-prompt"),
        "Expected --no-prompt option in help output"
    );
    // Should have --udid option
    assert!(
        stdout.contains("--udid"),
        "Expected --udid option in help output"
    );
}

/// Test shell help compatibility with Python idb
#[test]
fn test_shell_help_python_compatibility() {
    crate::common::build_agent_mobile();

    // Python idb
    let python_output = Command::new("idb")
        .args(["shell", "--help"])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "shell", "--help"])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should succeed
    assert!(python_output.status.success(), "Python idb help failed");
    assert!(rust_output.status.success(), "agent-mobile help failed");

    // Both should mention --no-prompt option
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);
    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

    assert!(
        python_stdout.contains("no-prompt") || python_stdout.contains("no_prompt"),
        "Python help should mention no-prompt"
    );
    assert!(
        rust_stdout.contains("no-prompt"),
        "Rust help should mention no-prompt"
    );
}

/// Test shell exits on 'exit' command
#[test]
fn test_shell_exit_command() {
    crate::common::build_agent_mobile();

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "shell", "--no-prompt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn shell");

    // Send exit command
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"exit\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    assert!(
        output.status.success(),
        "shell should exit cleanly with exit command"
    );
}

/// Test shell runs help command
#[test]
fn test_shell_help_command() {
    crate::common::build_agent_mobile();

    let mut child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "shell", "--no-prompt"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn shell");

    // Send help and exit commands
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"help\nexit\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    assert!(output.status.success(), "shell should handle help command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SUCCESS=1"),
        "help command should output SUCCESS=1"
    );
}
