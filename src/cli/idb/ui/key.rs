use crate::cli::helpers::CommandResult;
use crate::cli::idb::hid;

pub async fn run(keycode: u64, duration: Option<f64>, udid: Option<String>) -> CommandResult {
    hid::key::run(keycode, duration, udid).await
}
