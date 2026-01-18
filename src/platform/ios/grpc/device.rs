//! Device management operations for IdbClient.

use crate::platform::ios::proto::idb::install_request::{self, Destination};
use crate::platform::ios::proto::idb::payload::Source as PayloadSource;
use crate::platform::ios::proto::idb::{InstallRequest, InstallResponse, Payload};
use crate::types::Compression;

use super::client::IdbClient;

impl IdbClient {
    /// Get accessibility information
    pub async fn accessibility_info(
        &mut self,
        point: Option<(f64, f64)>,
        nested: bool,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::platform::ios::proto::idb::accessibility_info_request::Format;
        use crate::platform::ios::proto::idb::Point;

        let request =
            tonic::Request::new(crate::platform::ios::proto::idb::AccessibilityInfoRequest {
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

    /// Update contacts database
    pub async fn contacts_update(
        &mut self,
        db_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request =
            tonic::Request::new(crate::platform::ios::proto::idb::ContactsUpdateRequest {
                payload: Some(Payload {
                    source: Some(PayloadSource::FilePath(db_path.to_string())),
                }),
            });
        let response = self.client.contacts_update(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Clear all contacts
    pub async fn contacts_clear(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request =
            tonic::Request::new(crate::platform::ios::proto::idb::ContactsClearRequest {});
        let response = self.client.contacts_clear(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Clear keychain
    pub async fn clear_keychain(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request =
            tonic::Request::new(crate::platform::ios::proto::idb::ClearKeychainRequest {});
        let response = self.client.clear_keychain(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Simulate memory warning
    pub async fn simulate_memory_warning(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request =
            tonic::Request::new(crate::platform::ios::proto::idb::SimulateMemoryWarningRequest {});
        let response = self.client.simulate_memory_warning(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Bring simulator window to front
    pub async fn focus(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::FocusRequest {});
        let response = self.client.focus(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Approve (grant) app permissions
    pub async fn approve(
        &mut self,
        bundle_id: &str,
        permissions: Vec<i32>,
        scheme: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::ApproveRequest {
            bundle_id: bundle_id.to_string(),
            permissions,
            scheme: scheme.unwrap_or_default(),
        });
        let response = self.client.approve(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Revoke app permissions
    pub async fn revoke(
        &mut self,
        bundle_id: &str,
        permissions: Vec<i32>,
        scheme: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::RevokeRequest {
            bundle_id: bundle_id.to_string(),
            permissions,
            scheme: scheme.unwrap_or_default(),
        });
        let response = self.client.revoke(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Send a push notification to the device
    pub async fn send_notification(
        &mut self,
        bundle_id: &str,
        json_payload: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request =
            tonic::Request::new(crate::platform::ios::proto::idb::SendNotificationRequest {
                bundle_id: bundle_id.to_string(),
                json_payload: json_payload.to_string(),
            });
        let response = self.client.send_notification(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Set device location
    pub async fn set_location(
        &mut self,
        latitude: f64,
        longitude: f64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::SetLocationRequest {
            location: Some(crate::platform::ios::proto::idb::Location {
                latitude,
                longitude,
            }),
        });
        let response = self.client.set_location(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Open a URL on the device
    pub async fn open_url(
        &mut self,
        url: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::platform::ios::proto::idb::OpenUrlRequest {
            url: url.to_string(),
        });
        let response = self.client.open_url(request).await?;
        let _inner = response.into_inner();
        Ok(())
    }

    /// Set a device setting
    pub async fn set_setting(
        &mut self,
        setting: crate::platform::ios::proto::idb::setting_request::Setting,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::platform::ios::proto::idb::SettingRequest;

        let request = tonic::Request::new(SettingRequest {
            setting: Some(setting),
        });

        let _response = self.client.setting(request).await?;
        Ok(())
    }

    /// Get a device setting value
    pub async fn get_setting(
        &mut self,
        setting: crate::platform::ios::proto::idb::Setting,
        name: Option<String>,
        domain: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use crate::platform::ios::proto::idb::GetSettingRequest;

        let request = tonic::Request::new(GetSettingRequest {
            setting: setting as i32,
            name: name.unwrap_or_default(),
            domain: domain.unwrap_or_default(),
        });

        let response = self.client.get_setting(request).await?;
        Ok(response.into_inner().value)
    }

    /// List available settings
    pub async fn list_settings(
        &mut self,
        setting: crate::platform::ios::proto::idb::Setting,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::platform::ios::proto::idb::ListSettingRequest;

        let request = tonic::Request::new(ListSettingRequest {
            setting: setting as i32,
        });

        let response = self.client.list_settings(request).await?;
        Ok(response.into_inner().values)
    }

    /// Install a dSYM bundle
    pub async fn install_dsym(
        &mut self,
        dsym_path: &str,
        bundle_id: Option<String>,
        compression: Option<Compression>,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let mut requests: Vec<InstallRequest> = Vec::new();

        // 1. Destination request (DSYM)
        requests.push(InstallRequest {
            value: Some(install_request::Value::Destination(
                Destination::Dsym as i32,
            )),
        });

        // 2. Payload with file path
        requests.push(InstallRequest {
            value: Some(install_request::Value::Payload(Payload {
                source: Some(PayloadSource::FilePath(dsym_path.to_string())),
            })),
        });

        // 3. Optional: bundle_id (link to app container)
        if let Some(bid) = bundle_id {
            requests.push(InstallRequest {
                value: Some(install_request::Value::NameHint(bid)),
            });
        }

        // 4. Optional: compression
        if let Some(comp) = compression {
            let compression_enum = match comp {
                Compression::Gzip => crate::platform::ios::proto::idb::payload::Compression::Gzip,
                Compression::Zstd => crate::platform::ios::proto::idb::payload::Compression::Zstd,
            };
            requests.push(InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::Compression(compression_enum as i32)),
                })),
            });
        }

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// Install a dylib
    pub async fn install_dylib(
        &mut self,
        dylib_path: &str,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let requests: Vec<InstallRequest> = vec![
            InstallRequest {
                value: Some(install_request::Value::Destination(
                    Destination::Dylib as i32,
                )),
            },
            InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::FilePath(dylib_path.to_string())),
                })),
            },
        ];

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// Install a framework
    pub async fn install_framework(
        &mut self,
        framework_path: &str,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let requests: Vec<InstallRequest> = vec![
            InstallRequest {
                value: Some(install_request::Value::Destination(
                    Destination::Framework as i32,
                )),
            },
            InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::FilePath(framework_path.to_string())),
                })),
            },
        ];

        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }
}
