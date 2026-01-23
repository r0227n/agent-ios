//! XCTest operations for IdbClient.

use std::collections::HashMap;

use crate::proto::idb::install_request::{self, Destination};
use crate::proto::idb::payload::Source as PayloadSource;
use crate::proto::idb::xctest_run_request::{self, Mode};
use crate::proto::idb::{InstallRequest, InstallResponse, Payload, XctestRunRequest};
use agent_mobile_core::types::Compression;

use super::client::IdbClient;

impl IdbClient {
    /// Install an application via bidirectional streaming gRPC
    pub async fn install_xctest(
        &mut self,
        bundle_path: &str,
        skip_signing: bool,
        compression: Option<Compression>,
    ) -> Result<tonic::Streaming<InstallResponse>, Box<dyn std::error::Error + Send + Sync>> {
        // Build the sequence of install requests for XCTest
        let mut requests: Vec<InstallRequest> = Vec::new();

        // 1. Destination request (XCTEST)
        requests.push(InstallRequest {
            value: Some(install_request::Value::Destination(
                Destination::Xctest as i32,
            )),
        });

        // 2. Payload with file path
        requests.push(InstallRequest {
            value: Some(install_request::Value::Payload(Payload {
                source: Some(PayloadSource::FilePath(bundle_path.to_string())),
            })),
        });

        // 3. Optional: skip_signing_bundles
        if skip_signing {
            requests.push(InstallRequest {
                value: Some(install_request::Value::SkipSigningBundles(true)),
            });
        }

        // 4. Optional: compression
        if let Some(comp) = compression {
            let compression_enum = match comp {
                Compression::Gzip => crate::proto::idb::payload::Compression::Gzip,
                Compression::Zstd => crate::proto::idb::payload::Compression::Zstd,
            };
            requests.push(InstallRequest {
                value: Some(install_request::Value::Payload(Payload {
                    source: Some(PayloadSource::Compression(compression_enum as i32)),
                })),
            });
        }

        // Send the request stream
        let request_stream = tokio_stream::iter(requests);
        let response = self.client.install(request_stream).await?;
        Ok(response.into_inner())
    }

    /// List installed XCTest bundles
    pub async fn xctest_list_bundles(
        &mut self,
    ) -> Result<
        Vec<crate::proto::idb::xctest_list_bundles_response::Bundles>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = tonic::Request::new(crate::proto::idb::XctestListBundlesRequest {});
        let response = self.client.xctest_list_bundles(request).await?;
        Ok(response.into_inner().bundles)
    }

    /// List tests inside an installed test bundle
    pub async fn xctest_list_tests(
        &mut self,
        bundle_name: String,
        app_path: Option<String>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let request = tonic::Request::new(crate::proto::idb::XctestListTestsRequest {
            bundle_name,
            app_path: app_path.unwrap_or_default(),
        });
        let response = self.client.xctest_list_tests(request).await?;
        Ok(response.into_inner().names)
    }

    /// Run XCTest tests in logic mode (server-side streaming)
    pub async fn xctest_run_logic(
        &mut self,
        test_bundle_id: String,
        tests_to_run: Vec<String>,
    ) -> Result<
        tonic::Streaming<crate::proto::idb::XctestRunResponse>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let request = tonic::Request::new(XctestRunRequest {
            mode: Some(Mode {
                mode: Some(xctest_run_request::mode::Mode::Logic(
                    xctest_run_request::Logic {},
                )),
            }),
            test_bundle_id,
            tests_to_run,
            tests_to_skip: vec![],
            arguments: vec![],
            environment: HashMap::new(),
            timeout: 0,
            report_activities: false,
            collect_coverage: false,
            report_attachments: false,
            collect_logs: false,
            wait_for_debugger: false,
            code_coverage: None,
            collect_result_bundle: false,
        });

        let response = self.client.xctest_run(request).await?;
        Ok(response.into_inner())
    }
}
