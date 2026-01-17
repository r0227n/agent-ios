use crate::companion::CompanionResolver;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::FileContainer;
use tokio::sync::watch;

pub async fn run(
    path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let container = bundle_id.map(|bundle_id| FileContainer {
        kind: FileContainerKind::Application as i32,
        bundle_id,
    });

    // Setup Ctrl-C handler
    let (stop_tx, stop_rx) = watch::channel(false);
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = stop_tx.send(true);
    });

    client.tail(path, container, stop_rx).await?;

    Ok(())
}
