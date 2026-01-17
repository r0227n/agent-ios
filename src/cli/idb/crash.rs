use crate::companion::CompanionResolver;

pub async fn list(
    since: Option<u64>,
    before: Option<u64>,
    bundle_id: Option<String>,
    name: Option<String>,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let crashes = client.crash_list(since, before, bundle_id, name).await?;

    for crash in crashes {
        println!(
            "{} | {} | {} | {} | {} | {} | {}",
            crash.name,
            crash.bundle_id,
            crash.process_name,
            crash.parent_process_name,
            crash.process_identifier,
            crash.parent_process_identifier,
            crash.timestamp
        );
    }

    Ok(())
}

pub async fn show(
    name: String,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let result = client.crash_show(&name).await?;

    if let Some(info) = result.info {
        println!("Name: {}", info.name);
        println!("Bundle ID: {}", info.bundle_id);
        println!("Process: {}", info.process_name);
        println!("Parent Process: {}", info.parent_process_name);
        println!("PID: {}", info.process_identifier);
        println!("Parent PID: {}", info.parent_process_identifier);
        println!("Timestamp: {}", info.timestamp);
        println!("\nContents:\n{}", result.contents);
    }

    Ok(())
}

pub async fn delete(
    since: Option<u64>,
    before: Option<u64>,
    bundle_id: Option<String>,
    name: Option<String>,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let deleted = client.crash_delete(since, before, bundle_id, name).await?;

    println!("Deleted {} crash log(s)", deleted.len());

    Ok(())
}
