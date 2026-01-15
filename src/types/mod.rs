pub mod output;
pub mod target;

pub use output::{human_format_target, json_format_target};
pub use target::{Address, CompanionInfo, TargetDescription, TargetType};
