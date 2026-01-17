mod common;

#[test]
#[ignore]
fn test_photos_clear_basic() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // Run Python idb photos clear
    let python_output = common::run_idb_command(&["photos", "clear", "--udid", &udid]);

    // Run agent-mobile photos-clear
    let rust_output = common::run_agent_mobile_command(&["idb", "photos-clear", "--udid", &udid]);

    // Compare outputs
    common::compare_command_outputs(&python_output, &rust_output);
}

#[test]
#[ignore]
fn test_photos_clear_without_udid_single_target() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // If only one target is booted, should work without --udid
    let python_output = common::run_idb_command(&["photos", "clear"]);
    let rust_output = common::run_agent_mobile_command(&["idb", "photos-clear"]);

    // Both should have same exit code
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ"
    );
}
