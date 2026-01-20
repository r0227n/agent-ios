use crate::cli::helpers::CommandResult;
use crate::cli::idb::hid;

pub async fn run(text: String, udid: Option<String>) -> CommandResult {
    hid::text::run(text, udid).await
}
