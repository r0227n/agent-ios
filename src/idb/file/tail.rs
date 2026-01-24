use crate::helpers::client::{with_client, CommandResult};
use crate::helpers::file_container::file_container;
use crate::helpers::signal::setup_ctrl_c_handler;

pub async fn run(path: String, udid: Option<String>, bundle_id: Option<String>) -> CommandResult {
    let container = file_container(bundle_id);
    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        client.tail(path, container, stop_rx).await?;
        Ok(())
    })
    .await
}
