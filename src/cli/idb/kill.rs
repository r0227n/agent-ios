use crate::companion::CompanionState;

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
            eprintln!("Killing companion for {} (PID: {})", companion.udid, pid);

            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                match kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                    Ok(_) => eprintln!("Successfully killed PID {}", pid),
                    Err(e) => eprintln!("Failed to kill PID {}: {}", pid, e),
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
