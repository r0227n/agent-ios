use crate::cli::helpers::CommandResult;
use crate::cli::idb::accessibility;

pub async fn run(nested: bool, udid: Option<String>) -> CommandResult {
    accessibility::describe_all(nested, udid).await
}
