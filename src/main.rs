mod app;
mod command;
mod console;
mod core;
mod device;
mod helpers;
mod idb;
mod record;
mod session;
mod snapshot;

use clap::Parser;
use command::{Cli, Commands};
use idb::{
    debugserver, file, target, CrashCommands, DsymCommands, DylibCommands, FrameworkCommands,
    IdbCommands, ListCommands, LocationCommands, NotificationCommands, UrlCommands,
    XctraceCommands,
};
use session::resolver::SessionResolver;

/// Apply session UDID to DeviceArgs if not explicitly set
macro_rules! apply_session_udid {
    ($args:expr, $resolved_udid:expr) => {
        if $args.device.udid.is_none() {
            $args.device.udid = $resolved_udid.clone();
        }
    };
}

/// Apply session UDID to an Option<String> udid parameter if not explicitly set
macro_rules! apply_session_udid_option {
    ($udid:expr, $resolved_udid:expr) => {
        if $udid.is_none() {
            $udid = $resolved_udid.clone();
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

        // ==================== IDB Commands ====================
        Commands::Idb { command } => match *command {
            IdbCommands::ListTargets { only, human } => {
                idb::list_targets::run(only, human).await?;
            }
            IdbCommands::Launch {
                bundle_id,
                app_arguments,
                mut udid,
                wait_for_debugger,
                foreground_if_running,
                wait_for,
                pid_file,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::launch::run(
                    bundle_id,
                    app_arguments,
                    udid,
                    wait_for_debugger,
                    foreground_if_running,
                    wait_for,
                    pid_file,
                )
                .await?;
            }
            IdbCommands::Kill => {
                idb::kill::run().await?;
            }
            IdbCommands::Focus { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::focus::run(udid).await?;
            }
            IdbCommands::Log {
                mut udid,
                source,
                log_arguments,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::log::run(udid, source, log_arguments).await?;
            }
            IdbCommands::Install {
                bundle_path,
                mut udid,
                make_debuggable,
                override_mtime,
                compression,
                format,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::install::run(
                    bundle_path,
                    udid,
                    make_debuggable,
                    override_mtime,
                    compression,
                    format,
                )
                .await?;
            }
            IdbCommands::Uninstall {
                bundle_id,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::uninstall::run(bundle_id, udid).await?;
            }
            IdbCommands::Approve {
                bundle_id,
                permissions,
                scheme,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::permissions::approve(bundle_id, permissions, scheme, udid).await?;
            }
            IdbCommands::Crash { command } => match command {
                CrashCommands::List {
                    since,
                    before,
                    bundle_id,
                    name,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::crash::list(since, before, bundle_id, name, udid).await?;
                }
                CrashCommands::Show { name, mut udid } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::crash::show(name, udid).await?;
                }
                CrashCommands::Delete {
                    since,
                    before,
                    bundle_id,
                    name,
                    all,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::crash::delete(since, before, bundle_id, name, all, udid).await?;
                }
            },
            IdbCommands::File { command } => match command {
                file::FileCommands::Ls {
                    paths,
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::ls::run(paths, udid, bundle_id).await?;
                }
                file::FileCommands::Mkdir {
                    path,
                    bundle_id,
                    root,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::mkdir::run(path, bundle_id, root, udid).await?;
                }
                file::FileCommands::Mv {
                    src_paths,
                    dst_path,
                    bundle_id,
                    root,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
                }
                file::FileCommands::Rm {
                    paths,
                    mut udid,
                    bundle_id,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::rm::run(paths, udid, bundle_id).await?;
                }
                file::FileCommands::Pull {
                    src_path,
                    dst_path,
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::pull::run(src_path, dst_path, udid, bundle_id).await?;
                }
                file::FileCommands::Push {
                    src_path,
                    dst_path,
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::push::run(src_path, dst_path, udid, bundle_id).await?;
                }
                file::FileCommands::Tail {
                    path,
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::tail::run(path, udid, bundle_id).await?;
                }
                file::FileCommands::Read {
                    src_path,
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::read::run(src_path, udid, bundle_id).await?;
                }
                file::FileCommands::Write {
                    dst_path,
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::file::write::run(dst_path, udid, bundle_id).await?;
                }
            },
            IdbCommands::Button {
                button,
                duration,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::hid::button::run(button, duration, udid).await?;
            }
            IdbCommands::Key {
                keycode,
                duration,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::hid::key::run(keycode, duration, udid).await?;
            }
            IdbCommands::KeySequence {
                key_sequence,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::hid::key_sequence::run(key_sequence, udid).await?;
            }
            IdbCommands::ListApps { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::list_apps::run(udid).await?;
            }
            IdbCommands::Location { command } => match command {
                LocationCommands::SetLocation {
                    latitude,
                    longitude,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::location::run(latitude, longitude, udid).await?;
                }
            },
            IdbCommands::Notification { command } => match command {
                NotificationCommands::SendNotification {
                    bundle_id,
                    json_payload,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::notification::run(bundle_id, json_payload, udid).await?;
                }
            },
            IdbCommands::Revoke {
                bundle_id,
                permissions,
                scheme,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::permissions::revoke(bundle_id, permissions, scheme, udid).await?;
            }
            IdbCommands::Set {
                name,
                value,
                value_type,
                domain,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::settings::set(name, value, value_type, domain, udid).await?;
            }
            IdbCommands::Get {
                name,
                domain,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::settings::get(name, domain, udid).await?;
            }
            IdbCommands::List { command } => match command {
                ListCommands::Locale { mut udid } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::settings::list_locale(udid).await?;
                }
            },
            IdbCommands::Terminate {
                bundle_id,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::terminate::run(bundle_id, udid).await?;
            }
            IdbCommands::Url { command } => match command {
                UrlCommands::Open { url, mut udid } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::url::run(url, udid).await?;
                }
            },
            IdbCommands::Media { command } => match command {
                idb::media::MediaCommands::AddMedia {
                    file_paths,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::media::add_media(file_paths, udid).await?;
                }
            },
            IdbCommands::Video { command } => match command {
                idb::video::VideoCommands::RecordVideo {
                    output_file,
                    format,
                    fps,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::video::record::run(output_file, format, fps, udid).await?;
                }
                idb::video::VideoCommands::VideoStream {
                    output_file,
                    fps,
                    format,
                    compression_quality,
                    scale_factor,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::video::stream::run(
                        output_file,
                        fps,
                        format,
                        compression_quality,
                        scale_factor,
                        udid,
                    )
                    .await?;
                }
            },
            IdbCommands::PhotosClear { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::photos::clear(udid).await?;
            }
            IdbCommands::AccessibilityDescribeAll { nested, mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::accessibility::describe_all(nested, udid).await?;
            }
            IdbCommands::AccessibilityDescribePoint {
                x,
                y,
                nested,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::accessibility::describe_point(x, y, nested, udid).await?;
            }
            IdbCommands::ContactsUpdate { db_path, mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::contacts::update(db_path, udid).await?;
            }
            IdbCommands::ContactsClear { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::contacts::clear(udid).await?;
            }
            IdbCommands::KeychainClear { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::keychain::clear(udid).await?;
            }
            IdbCommands::SimulateMemoryWarning { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::memory::simulate_warning(udid).await?;
            }
            IdbCommands::XctestInstall {
                test_bundle_path,
                skip_signing,
                compression,
                format,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::xctest_install::run(test_bundle_path, udid, skip_signing, compression, format)
                    .await?;
            }
            IdbCommands::XctestList { mut udid } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::xctest_list::run(udid).await?;
            }
            IdbCommands::XctestListBundle {
                bundle_id,
                app_path,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::xctest_list_bundle::run(bundle_id, app_path, udid).await?;
            }
            IdbCommands::XctestRun {
                test_bundle_id,
                tests_to_run,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::xctest_run::run(test_bundle_id, tests_to_run, udid).await?;
            }
            // Note: Target commands (boot, shutdown, erase, clone, disconnect) require explicit UDID
            // as they are device lifecycle management commands. Session UDID is not applied.
            IdbCommands::Target { command } => match command {
                target::TargetCommands::Boot { udid, headless } => {
                    idb::target::boot::run(udid, headless).await?;
                }
                target::TargetCommands::Shutdown { udid } => {
                    idb::target::shutdown::run(udid).await?;
                }
                target::TargetCommands::Erase { udid } => {
                    idb::target::erase::run(udid).await?;
                }
                target::TargetCommands::Create {
                    name,
                    device_type,
                    os_version,
                } => {
                    idb::target::create::run(name, device_type, os_version).await?;
                }
                target::TargetCommands::Clone { udid } => {
                    idb::target::clone::run(udid).await?;
                }
                target::TargetCommands::Delete { mut udid, all } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::target::delete::run(udid, all).await?;
                }
                target::TargetCommands::Connect {
                    host,
                    port,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::target::connect::run(host, port, udid).await?;
                }
                target::TargetCommands::Disconnect { udid } => {
                    idb::target::disconnect::run(udid).await?;
                }
                target::TargetCommands::Describe {
                    mut udid,
                    diagnostics,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::target::describe::run(udid, diagnostics).await?;
                }
            },
            IdbCommands::Debugserver { command } => match command {
                debugserver::DebugServerCommands::Start {
                    bundle_id,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::debugserver::start::run(bundle_id, udid).await?;
                }
                debugserver::DebugServerCommands::Stop { mut udid } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::debugserver::stop::run(udid).await?;
                }
                debugserver::DebugServerCommands::Status { mut udid } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::debugserver::status::run(udid).await?;
                }
            },
            IdbCommands::Dap {
                bundle,
                port,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::dap::run(bundle, port, udid).await?;
            }
            IdbCommands::Dsym { command } => match command {
                DsymCommands::Install {
                    dsym_path,
                    bundle_id,
                    compression,
                    format,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::dsym::install(dsym_path, bundle_id, compression, format, udid).await?;
                }
            },
            IdbCommands::Dylib { command } => match command {
                DylibCommands::Install {
                    dylib_path,
                    format,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::dylib::install(dylib_path, format, udid).await?;
                }
            },
            IdbCommands::Framework { command } => match command {
                FrameworkCommands::Install {
                    framework_path,
                    format,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::framework::install(framework_path, format, udid).await?;
                }
            },
            IdbCommands::Instruments {
                template,
                app_bundle_id,
                app_args,
                app_env,
                output,
                post_args,
                operation_duration,
                terminate_timeout,
                launch_retry_timeout,
                launch_error_timeout,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::instruments::run(
                    template,
                    app_bundle_id,
                    app_args,
                    app_env,
                    output,
                    post_args,
                    operation_duration,
                    terminate_timeout,
                    launch_retry_timeout,
                    launch_error_timeout,
                    udid,
                )
                .await?;
            }
            IdbCommands::Xctrace { command } => match command {
                XctraceCommands::Record {
                    template,
                    all_processes,
                    attach,
                    launch,
                    launch_args,
                    output,
                    time_limit,
                    package,
                    target_stdin,
                    target_stdout,
                    env,
                    stop_timeout,
                    post_args,
                    mut udid,
                } => {
                    apply_session_udid_option!(udid, resolved_udid);
                    idb::xctrace::record(
                        template,
                        all_processes,
                        attach,
                        launch,
                        launch_args,
                        output,
                        time_limit,
                        package,
                        target_stdin,
                        target_stdout,
                        env,
                        stop_timeout,
                        post_args,
                        udid,
                    )
                    .await?;
                }
            },
            IdbCommands::Shell {
                no_prompt,
                mut udid,
            } => {
                apply_session_udid_option!(udid, resolved_udid);
                idb::shell::run(no_prompt, udid).await?;
            }
        },
    }

    Ok(())
}
