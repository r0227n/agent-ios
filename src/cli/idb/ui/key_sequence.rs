use crate::cli::helpers::CommandResult;
use crate::cli::idb::hid;

pub async fn run(key_sequence: Vec<u64>, udid: Option<String>) -> CommandResult {
    hid::key_sequence::run(key_sequence, udid).await
}
