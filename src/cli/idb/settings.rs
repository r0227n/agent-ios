use crate::companion::CompanionResolver;
use crate::grpc::idb::setting_request::{self, Setting as SettingOneof};
use crate::grpc::idb::Setting;

/// Set a device setting
pub async fn set(
    name: String,
    value: String,
    value_type: Option<String>,
    domain: Option<String>,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let setting = setting_request::StringSetting {
        setting: Setting::Any as i32,
        value,
        name,
        domain: domain.unwrap_or_default(),
        value_type: value_type.unwrap_or_else(|| "string".to_string()),
    };

    client
        .set_setting(SettingOneof::StringSetting(setting))
        .await?;

    Ok(())
}

/// Get a device setting value
pub async fn get(
    name: String,
    domain: Option<String>,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let value = client.get_setting(Setting::Any, Some(name), domain).await?;

    println!("{}", value);
    Ok(())
}

/// List available locales
pub async fn list_locale(
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    let locales = client.list_settings(Setting::Locale).await?;

    for locale in locales {
        println!("{}", locale);
    }

    Ok(())
}
