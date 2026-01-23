//! Core Commands - AI Agent 向け高レベル CLI
//!
//! ref 識別子 (@e1, @e2) を使った要素操作を提供します。
//!
//! ## コマンド一覧
//! - `tap` - 要素タップ
//! - `long-press` - 長押しジェスチャー
//! - `fill` - テキストフィールド入力 (クリア + 入力)
//! - `type` - テキスト追記入力
//! - `swipe` - スワイプジェスチャー
//! - `scroll` - スクロール
//! - `get` - 要素情報取得
//! - `is` - 状態確認
//! - `wait` - 要素待機
//! - `screenshot` - スクリーンショット
//! - `find` - semantic locators による要素検索

pub mod fill;
pub mod find;
pub mod get;
pub mod is_cmd;
pub mod long_press;
pub mod ref_resolver;
pub mod screenshot;
pub mod scroll;
pub mod swipe;
pub mod tap;
pub mod type_cmd;
pub mod wait;

// Re-export Args for CLI integration
pub use fill::FillArgs;
pub use find::FindArgs;
pub use get::GetArgs;
pub use is_cmd::IsArgs;
pub use long_press::LongPressArgs;
pub use screenshot::ScreenshotArgs;
pub use scroll::ScrollArgs;
pub use swipe::SwipeArgs;
pub use tap::TapArgs;
pub use type_cmd::TypeArgs;
pub use wait::WaitArgs;
