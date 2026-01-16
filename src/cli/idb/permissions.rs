use crate::companion::CompanionResolver;
use crate::grpc::idb::approve_request::Permission as ApprovePermission;
use crate::grpc::idb::revoke_request::Permission as RevokePermission;

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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let parsed_permissions: Result<Vec<_>, _> = permissions
        .iter()
        .map(|p| parse_approve_permission(p))
        .collect();
    let parsed_permissions = parsed_permissions?;

    client
        .approve(&bundle_id, parsed_permissions, scheme)
        .await?;

    Ok(())
}

pub async fn revoke(
    bundle_id: String,
    permissions: Vec<String>,
    scheme: Option<String>,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let parsed_permissions: Result<Vec<_>, _> = permissions
        .iter()
        .map(|p| parse_revoke_permission(p))
        .collect();
    let parsed_permissions = parsed_permissions?;

    client
        .revoke(&bundle_id, parsed_permissions, scheme)
        .await?;

    Ok(())
}
