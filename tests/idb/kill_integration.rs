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
    // Temporarily move state file if it exists
    let backup = if std::path::Path::new("/tmp/idb/state").exists() {
        Some(fs::read_to_string("/tmp/idb/state").unwrap())
    } else {
        None
    };

    // Remove state file
    fs::remove_file("/tmp/idb/state").ok();

    // Run kill command
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "kill"])
        .output()
        .expect("Failed to run kill command");

    // Should still succeed with no companions
    assert!(
        output.status.success(),
        "Kill should succeed even with no companions"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No companions to kill"),
        "Should report no companions to kill"
    );

    // Restore backup if needed
    if let Some(backup_content) = backup {
        fs::write("/tmp/idb/state", backup_content).ok();
    }
}
