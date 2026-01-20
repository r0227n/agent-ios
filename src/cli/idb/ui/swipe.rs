use crate::cli::helpers::CommandResult;
use crate::cli::idb::hid;

pub async fn run(
    x_start: f64,
    y_start: f64,
    x_end: f64,
    y_end: f64,
    duration: Option<f64>,
    delta: Option<f64>,
    udid: Option<String>,
) -> CommandResult {
    hid::swipe::run(x_start, y_start, x_end, y_end, duration, delta, udid).await
}
