use crate::companion::CompanionResolver;
use crate::grpc::idb::file_container::Kind as FileContainerKind;
use crate::grpc::idb::payload::Source as PayloadSource;
use crate::grpc::idb::FileContainer;
use std::io::Write;

pub async fn run(
    src_path: String,
    dst_path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let container = bundle_id.map(|bundle_id| FileContainer {
        kind: FileContainerKind::Application as i32,
        bundle_id,
    });

    let mut stream = client.pull(src_path, container).await?;

    if dst_path == "-" {
        // Output to stdout
        while let Some(response) = stream.message().await? {
            if let Some(payload) = response.payload {
                match payload.source {
                    Some(PayloadSource::Data(data)) => {
                        std::io::stdout().write_all(&data)?;
                    }
                    Some(PayloadSource::FilePath(path)) => {
                        let data = std::fs::read(&path)?;
                        std::io::stdout().write_all(&data)?;
                    }
                    _ => {}
                }
            }
        }
        std::io::stdout().flush()?;
    } else {
        // Write to file
        let mut file = std::fs::File::create(&dst_path)?;
        while let Some(response) = stream.message().await? {
            if let Some(payload) = response.payload {
                match payload.source {
                    Some(PayloadSource::Data(data)) => {
                        file.write_all(&data)?;
                    }
                    Some(PayloadSource::FilePath(path)) => {
                        let data = std::fs::read(&path)?;
                        file.write_all(&data)?;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
