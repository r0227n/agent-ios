mod app;
mod command;
mod console;
mod core;
mod device;
mod helpers;
mod record;
mod session;
mod snapshot;

use clap::Parser;
use command::{Cli, Commands};
use session::resolver::SessionResolver;

/// Apply session UDID to DeviceArgs if not explicitly set
macro_rules! apply_session_udid {
    ($args:expr, $resolved_udid:expr) => {
        if $args.device.udid.is_none() {
            $args.device.udid = $resolved_udid.clone();
        }
    };
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let session = cli.session.as_deref();

    // Resolve UDID from session (if specified)
    let resolved_udid = if let Some(session_name) = session {
        let resolver = SessionResolver::new();
        resolver.resolve_udid(Some(session_name), None)?
    } else {
        None
    };

    match cli.command {
        // ==================== Core Commands ====================
        Commands::Tap(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::tap::run(args).await?;
        }
        Commands::Check(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::check::run(args, true).await?;
        }
        Commands::Uncheck(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::check::run(args, false).await?;
        }
        Commands::Select(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::select::run(args).await?;
        }
        Commands::LongPress(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::long_press::run(args).await?;
        }
        Commands::Fill(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::fill::run(args).await?;
        }
        Commands::Type(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::type_cmd::run(args).await?;
        }
        Commands::Swipe(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::swipe::run(args).await?;
        }
        Commands::Scroll(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::scroll::run(args).await?;
        }
        Commands::Get(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::get::run(args).await?;
        }
        Commands::Is(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::is_cmd::run(args).await?;
        }
        Commands::Wait(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::wait::run(args).await?;
        }
        Commands::Screenshot(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::screenshot::run(args).await?;
        }
        Commands::Find(mut args) => {
            apply_session_udid!(args, resolved_udid);
            core::find::run(args).await?;
        }
        Commands::Snapshot(mut args) => {
            apply_session_udid!(args, resolved_udid);
            snapshot::run(args).await?;
        }
        Commands::Record(mut args) => {
            apply_session_udid!(args, resolved_udid);
            record::run(args).await?;
        }
        Commands::Console(mut args) => {
            apply_session_udid!(args, resolved_udid);
            console::run(args).await?;
        }

        // ==================== Existing Commands ====================
        Commands::App(args) => {
            app::run(args, resolved_udid.clone()).await?;
        }
        Commands::Device(args) => {
            device::run(args).await?;
        }
        Commands::Session(args) => {
            session::run(args, session).await?;
        }
    }

    Ok(())
}
