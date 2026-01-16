use crate::companion::CompanionState;
use crate::grpc::IdbClient;
use crate::types::Address;

pub async fn run(
    bundle_id: String,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Resolve companion by UDID
    let state = CompanionState::default();
    let companion = match udid.as_deref() {
        Some(u) => state
            .find_by_udid(u)
            .ok_or_else(|| format_no_companion_error(u))?,
        None => state
            .get_companions()
            .into_iter()
            .next()
            .ok_or_else(format_no_companions_error)?,
    };

    // 2. Connect to companion
    let address = companion.address().ok_or_else(format_no_address_error)?;

    let mut client = match &address {
        Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // 3. Call uninstall RPC
    client.uninstall(&bundle_id).await?;

    // 4. Success (no output for uninstall command, same as Python idb)
    Ok(())
}

/// Format error message when companion is not found for UDID
fn format_no_companion_error(udid: &str) -> String {
    format!("No companion found for UDID: {}", udid)
}

/// Format error message when no companions are available
fn format_no_companions_error() -> String {
    "No companions available. Run 'idb_companion' first.".to_string()
}

/// Format error message when companion has no valid address
fn format_no_address_error() -> String {
    "Companion has no valid address".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_no_companion_error() {
        let error = format_no_companion_error("ABC123-DEF456");
        assert_eq!(error, "No companion found for UDID: ABC123-DEF456");
    }

    #[test]
    fn test_format_no_companions_error() {
        let error = format_no_companions_error();
        assert_eq!(error, "No companions available. Run 'idb_companion' first.");
    }

    #[test]
    fn test_format_no_address_error() {
        let error = format_no_address_error();
        assert_eq!(error, "Companion has no valid address");
    }
}
