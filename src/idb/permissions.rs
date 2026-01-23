//! Permission management commands for iOS applications
//!
//! # Known Issues
//!
//! The `approve` command has a known issue with specific permission types due to
//! a SQLite schema mismatch in the upstream `idb_companion` daemon:
//!
//! **Affected permissions**: `photos`, `camera`, `contacts`
//! **Error**: "table access has 17 columns but 13 values were supplied"
//! **Status**: Affects both Python idb and Rust agent-mobile
//!
//! **Working permissions**: `location`, `notification`, `url`, `microphone`
//! **Revoke operations**: All permission types work correctly
//!
//! See `docs/KNOWN_ISSUES.md` for detailed information.

use agent_mobile_platform_ios::proto::idb::approve_request::Permission as ApprovePermission;
use agent_mobile_platform_ios::proto::idb::revoke_request::Permission as RevokePermission;

use crate::helpers::{with_client, CommandResult};

fn parse_approve_permission(s: &str) -> Result<ApprovePermission, String> {
    match s.to_lowercase().as_str() {
        "photos" => Ok(ApprovePermission::Photos),
        "camera" => Ok(ApprovePermission::Camera),
        "contacts" => Ok(ApprovePermission::Contacts),
        "url" => Ok(ApprovePermission::Url),
        "location" => Ok(ApprovePermission::Location),
        "notification" => Ok(ApprovePermission::Notification),
        "microphone" => Ok(ApprovePermission::Microphone),
        _ => Err(format!("Invalid permission: {}", s)),
    }
}

fn parse_revoke_permission(s: &str) -> Result<RevokePermission, String> {
    match s.to_lowercase().as_str() {
        "photos" => Ok(RevokePermission::Photos),
        "camera" => Ok(RevokePermission::Camera),
        "contacts" => Ok(RevokePermission::Contacts),
        "url" => Ok(RevokePermission::Url),
        "location" => Ok(RevokePermission::Location),
        "notification" => Ok(RevokePermission::Notification),
        "microphone" => Ok(RevokePermission::Microphone),
        _ => Err(format!("Invalid permission: {}", s)),
    }
}

pub async fn approve(
    bundle_id: String,
    permissions: Vec<String>,
    scheme: Option<String>,
    udid: Option<String>,
) -> CommandResult {
    let parsed_permissions: Result<Vec<_>, _> = permissions
        .iter()
        .map(|p| parse_approve_permission(p))
        .collect();
    let parsed_permissions: Vec<i32> = parsed_permissions?.into_iter().map(|p| p as i32).collect();

    with_client(udid.as_deref(), |mut client| async move {
        client
            .approve(&bundle_id, parsed_permissions, scheme)
            .await?;
        Ok(())
    })
    .await
}

pub async fn revoke(
    bundle_id: String,
    permissions: Vec<String>,
    scheme: Option<String>,
    udid: Option<String>,
) -> CommandResult {
    let parsed_permissions: Result<Vec<_>, _> = permissions
        .iter()
        .map(|p| parse_revoke_permission(p))
        .collect();
    let parsed_permissions: Vec<i32> = parsed_permissions?.into_iter().map(|p| p as i32).collect();

    with_client(udid.as_deref(), |mut client| async move {
        client
            .revoke(&bundle_id, parsed_permissions, scheme)
            .await?;
        Ok(())
    })
    .await
}
