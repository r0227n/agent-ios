use agent_mobile_platform_ios::proto::idb::payload::Source as PayloadSource;

use crate::helpers::file_container::file_container;
use crate::helpers::client::{with_client, CommandResult};
use agent_mobile_core::OutputWriter;
use std::io::Write;

pub async fn run(
    src_path: String,
    dst_path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> CommandResult {
    let container = file_container(bundle_id);

    with_client(udid.as_deref(), |mut client| async move {
        let mut stream = client.pull(src_path, container).await?;

        let mut writer = OutputWriter::from_path(&dst_path)?;

        while let Some(response) = stream.message().await? {
            if let Some(payload) = response.payload {
                match payload.source {
                    Some(PayloadSource::Data(data)) => {
                        writer.write_all(&data)?;
                    }
                    Some(PayloadSource::FilePath(path)) => {
                        let data = std::fs::read(&path)?;
                        writer.write_all(&data)?;
                    }
                    _ => {}
                }
            }
        }
        writer.flush()?;

        Ok(())
    })
    .await
}
