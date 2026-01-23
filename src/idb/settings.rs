use agent_mobile_platform_ios::proto::idb::setting_request::{self, Setting as SettingOneof};
use agent_mobile_platform_ios::proto::idb::Setting;

use crate::helpers::client::{with_client, CommandResult};

/// Set a device setting
pub async fn set(
    name: String,
    value: String,
    value_type: Option<String>,
    domain: Option<String>,
    udid: Option<String>,
) -> CommandResult {
    let setting = setting_request::StringSetting {
        setting: Setting::Any as i32,
        value,
        name,
        domain: domain.unwrap_or_default(),
        value_type: value_type.unwrap_or_else(|| "string".to_string()),
    };

    with_client(udid.as_deref(), |mut client| async move {
        client
            .set_setting(SettingOneof::StringSetting(setting))
            .await?;
        Ok(())
    })
    .await
}

/// Get a device setting value
pub async fn get(name: String, domain: Option<String>, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let value = client.get_setting(Setting::Any, Some(name), domain).await?;
        println!("{}", value);
        Ok(())
    })
    .await
}

/// List available locales
pub async fn list_locale(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let locales = client.list_settings(Setting::Locale).await?;
        for locale in locales {
            println!("{}", locale);
        }
        Ok(())
    })
    .await
}
