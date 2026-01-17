pub mod button;
pub mod events;
pub mod key;
pub mod key_sequence;
pub mod swipe;
pub mod tap;
pub mod text;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum HidCommands {
    /// Tap on the screen
    Tap {
        /// X coordinate
        x: f64,
        /// Y coordinate
        y: f64,
        /// Press duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Press a button
    Button {
        /// Button type (APPLE_PAY, HOME, LOCK, SIDE_BUTTON, SIRI)
        button: String,
        /// Press duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Press a key
    Key {
        /// Key code
        keycode: u64,
        /// Press duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Press a sequence of keys
    #[command(name = "key-sequence")]
    KeySequence {
        /// Key codes (space-separated)
        key_sequence: Vec<u64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Input text
    Text {
        /// Text to input
        text: String,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Swipe from one point to another
    Swipe {
        /// X coordinate of swipe start
        x_start: f64,
        /// Y coordinate of swipe start
        y_start: f64,
        /// X coordinate of swipe end
        x_end: f64,
        /// Y coordinate of swipe end
        y_end: f64,
        /// Swipe duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Delta in pixels between touch points
        #[arg(long)]
        delta: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
