use std::process::Command;

fn get_idb_output() -> String {
    let output = Command::new("idb")
        .args(["list-targets"])
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH");

    assert!(
        output.status.success(),
        "Python idb list-targets failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn get_agent_mobile_output() -> String {
    // Use --human flag to match idb's default human-readable output
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-targets", "--human"])
        .output()
        .expect("Failed to execute agent-mobile command");

    assert!(
        output.status.success(),
        "agent-mobile list-targets failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn normalize_output(output: &str) -> Vec<String> {
    let mut lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    lines.sort();
    lines
}

#[test]
fn test_list_targets_output_matches_idb() {
    let idb_output = get_idb_output();
    let agent_output = get_agent_mobile_output();

    let idb_lines = normalize_output(&idb_output);
    let agent_lines = normalize_output(&agent_output);

    assert_eq!(
        idb_lines, agent_lines,
        "Output of 'idb list-targets' should match 'agent-mobile idb list-targets'"
    );
}

fn get_idb_json_output() -> String {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .expect("Failed to execute Python idb - ensure idb is installed and in PATH");

    assert!(
        output.status.success(),
        "Python idb list-targets --json failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn get_agent_mobile_json_output() -> String {
    // agent-mobile defaults to JSON output (no flag needed)
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "list-targets"])
        .output()
        .expect("Failed to execute agent-mobile command");

    assert!(
        output.status.success(),
        "agent-mobile list-targets failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn parse_json_lines(output: &str) -> Vec<serde_json::Value> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[test]
fn test_list_targets_json_output_semantically_matches_idb() {
    let idb_output = get_idb_json_output();
    let agent_output = get_agent_mobile_json_output();

    let mut idb_json: Vec<serde_json::Value> = parse_json_lines(&idb_output);
    let mut agent_json: Vec<serde_json::Value> = parse_json_lines(&agent_output);

    // Sort by udid for consistent comparison
    idb_json.sort_by(|a, b| {
        a.get("udid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("udid").and_then(|v| v.as_str()).unwrap_or(""))
    });
    agent_json.sort_by(|a, b| {
        a.get("udid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("udid").and_then(|v| v.as_str()).unwrap_or(""))
    });

    assert_eq!(
        idb_json.len(),
        agent_json.len(),
        "Number of targets should match"
    );

    for (idb_item, agent_item) in idb_json.iter().zip(agent_json.iter()) {
        // Compare key fields
        assert_eq!(
            idb_item.get("name"),
            agent_item.get("name"),
            "name field should match"
        );
        assert_eq!(
            idb_item.get("udid"),
            agent_item.get("udid"),
            "udid field should match"
        );
        assert_eq!(
            idb_item.get("state"),
            agent_item.get("state"),
            "state field should match"
        );
        assert_eq!(
            idb_item.get("type"),
            agent_item.get("type"),
            "type field should match"
        );
        assert_eq!(
            idb_item.get("os_version"),
            agent_item.get("os_version"),
            "os_version field should match"
        );
        assert_eq!(
            idb_item.get("architecture"),
            agent_item.get("architecture"),
            "architecture field should match"
        );
    }
}
