//! Device management operations for IdbClient.

use super::client::IdbClient;

impl IdbClient {
    /// Get accessibility information
    pub async fn accessibility_info(
        &mut self,
        point: Option<(f64, f64)>,
        nested: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::proto::idb::accessibility_info_request::Format;
        use crate::proto::idb::Point;

        let request = tonic::Request::new(crate::proto::idb::AccessibilityInfoRequest {
            point: point.map(|(x, y)| Point { x, y }),
            format: if nested {
                Format::Nested as i32
            } else {
                Format::Legacy as i32
            },
        });
        let response = self.client.accessibility_info(request).await?;
        Ok(response.into_inner().json)
    }
}
