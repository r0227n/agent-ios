use agent_mobile_platform_ios::companion::CompanionState;

/// Format message for killing a companion
fn format_kill_message(udid: &str, pid: u32) -> String {
    format!("Killing companion for {} (PID: {})", udid, pid)
}

/// Format success message after killing a process
fn format_kill_success(pid: u32) -> String {
    format!("Successfully killed PID {}", pid)
}

/// Format error message when kill fails
fn format_kill_error<E: std::fmt::Display>(pid: u32, error: E) -> String {
    format!("Failed to kill PID {}: {}", pid, error)
}

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = CompanionState::default();
    let companions = state.get_companions();

    if companions.is_empty() {
        eprintln!("No companions to kill");
        return Ok(());
    }

    // Kill each companion process
    for companion in &companions {
        if let Some(pid) = companion.pid {
            eprintln!("{}", format_kill_message(&companion.udid, pid));

            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                match kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                    Ok(_) => eprintln!("{}", format_kill_success(pid)),
                    Err(e) => eprintln!("{}", format_kill_error(pid, e)),
                }
            }

            #[cfg(not(unix))]
            {
                eprintln!("Process killing not supported on this platform");
            }
        }
    }

    // Clear the state file
    state.clear()?;
    eprintln!("Cleared companion state");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_kill_message() {
        let msg = format_kill_message("ABC123-DEF456", 12345_u32);
        assert_eq!(msg, "Killing companion for ABC123-DEF456 (PID: 12345)");
    }

    #[test]
    fn test_format_kill_success() {
        let msg = format_kill_success(12345_u32);
        assert_eq!(msg, "Successfully killed PID 12345");
    }

    #[test]
    fn test_format_kill_error() {
        let msg = format_kill_error(12345_u32, "Operation not permitted");
        assert_eq!(msg, "Failed to kill PID 12345: Operation not permitted");
    }
}
