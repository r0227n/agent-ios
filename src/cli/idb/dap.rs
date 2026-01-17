//! DAP (Debug Adapter Protocol) command
//!
//! This command spawns a debug server using the VSCode DAP protocol,
//! bridging stdin/stdout to the remote DAP server.

use crate::cli::helpers::{setup_ctrl_c_handler, with_client, CommandResult};

pub async fn run(dap_pkg_path: String, udid: Option<String>) -> CommandResult {
    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        client.dap(dap_pkg_path, stop_rx).await?;
        Ok(())
    })
    .await
}
