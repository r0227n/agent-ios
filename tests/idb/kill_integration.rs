use std::fs;
use std::process::Command;

#[test]
fn test_kill_command() {
    // Skip if no state file exists
    if !std::path::Path::new("/tmp/idb/state").exists() {
        eprintln!("Skipping test: no /tmp/idb/state file");
        return;
    }

    // Backup state file
    let backup = fs::read_to_string("/tmp/idb/state").ok();

    // Run Rust kill command
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "kill"])
        .output()
        .expect("Failed to run kill command");

    // Should succeed
    assert!(
        output.status.success(),
        "Kill command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // State file should be deleted
    assert!(
        !std::path::Path::new("/tmp/idb/state").exists(),
        "State file should be deleted after kill"
    );

    // Restore backup if needed
    if let Some(backup_content) = backup {
        fs::write("/tmp/idb/state", backup_content).ok();
    }
}

#[test]
fn test_kill_no_companions() {
    use std::path::Path;

    let state_path = Path::new("/tmp/idb/state");
    let backup_path = Path::new("/tmp/idb/state.backup.kill_test");

    // Backup state file by renaming (atomic operation)
    let had_state_file = if state_path.exists() {
        fs::rename(state_path, backup_path).is_ok()
    } else {
        false
    };

    // Ensure state file is removed
    fs::remove_file(state_path).ok();

    // Run kill command
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "kill"])
        .output()
        .expect("Failed to run kill command");

    // Restore backup before assertions (ensure cleanup on failure)
    if had_state_file && backup_path.exists() {
        fs::rename(backup_path, state_path).ok();
    }

    // Should still succeed with no companions
    assert!(
        output.status.success(),
        "Kill should succeed even with no companions"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No companions to kill"),
        "Should report no companions to kill, got: {}",
        stderr
    );
}
