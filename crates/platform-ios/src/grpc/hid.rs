//! HID (Human Interface Device) operations for IdbClient.

use super::client::IdbClient;

impl IdbClient {
    /// Send HID events to the device (client-side streaming)
    pub async fn hid(
        &mut self,
        events: Vec<crate::proto::idb::HidEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request_stream = tokio_stream::iter(events);
        let response = self.client.hid(request_stream).await?;
        let _inner = response.into_inner();
        Ok(())
    }
}
