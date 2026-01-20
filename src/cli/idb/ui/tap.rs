use crate::cli::helpers::CommandResult;
use crate::cli::idb::hid;

pub async fn run(x: f64, y: f64, duration: Option<f64>, udid: Option<String>) -> CommandResult {
    hid::tap::run(x, y, duration, udid).await
}
