//! Target description operations for IdbClient.

use crate::platform::ios::proto::idb::TargetDescriptionRequest;
use crate::types::{CompanionInfo, TargetDescription, TargetType};

use super::client::IdbClient;

impl IdbClient {
    /// Call describe RPC to get target information
    pub async fn describe(
        &mut self,
        fetch_diagnostics: bool,
    ) -> Result<TargetDescription, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(TargetDescriptionRequest { fetch_diagnostics });

        let response = self.client.describe(request).await?;
        let inner = response.into_inner();

        self.target_from_response(inner)
    }

    pub(crate) fn target_from_response(
        &self,
        response: crate::platform::ios::proto::idb::TargetDescriptionResponse,
    ) -> Result<TargetDescription, Box<dyn std::error::Error + Send + Sync>> {
        let target = response
            .target_description
            .ok_or("Missing target_description in response")?;

        let companion = response.companion;

        let companion_info = companion.map(|c| CompanionInfo {
            udid: c.udid,
            is_local: c.is_local,
            pid: None,
            address: self.address.clone(),
        });

        Ok(TargetDescription {
            name: target.name,
            udid: target.udid,
            state: if target.state.is_empty() {
                None
            } else {
                Some(target.state)
            },
            target_type: TargetType::from_proto_string(&target.target_type),
            os_version: if target.os_version.is_empty() {
                None
            } else {
                Some(target.os_version)
            },
            architecture: if target.architecture.is_empty() {
                None
            } else {
                Some(target.architecture)
            },
            companion_info,
        })
    }
}
