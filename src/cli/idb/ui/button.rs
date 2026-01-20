use crate::cli::helpers::CommandResult;
use crate::cli::idb::hid;

pub async fn run(button: String, duration: Option<f64>, udid: Option<String>) -> CommandResult {
    hid::button::run(button, duration, udid).await
}
