use crate::cli::helpers::CommandResult;
use crate::cli::idb::accessibility;

pub async fn run(x: f64, y: f64, nested: bool, udid: Option<String>) -> CommandResult {
    accessibility::describe_point(x, y, nested, udid).await
}
