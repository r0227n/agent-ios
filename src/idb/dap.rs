//! DAP (Debug Adapter Protocol) command
//!
//! This command spawns a debug server using the VSCode DAP protocol,
//! bridging stdin/stdout to the remote DAP server.

use crate::helpers::{setup_ctrl_c_handler, with_client, CommandResult};

pub async fn run(bundle: String, port: Option<u16>, udid: Option<String>) -> CommandResult {
    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        client.dap(bundle, port, stop_rx).await?;
        Ok(())
    })
    .await
}
