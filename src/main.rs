mod cli;
mod companion;
mod grpc;
mod platform;
mod simctl;
mod types;

use clap::Parser;
use cli::idb::{
    debugserver, file, target, CrashCommands, DsymCommands, DylibCommands, FrameworkCommands,
    IdbCommands, ListCommands, LocationCommands, NotificationCommands, UrlCommands,
    XctraceCommands,
};
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Idb { command } => match *command {
            IdbCommands::ListTargets { only, human } => {
                cli::idb::list_targets::run(only, human).await?;
            }
            IdbCommands::Launch {
                bundle_id,
                app_arguments,
                udid,
                wait_for_debugger,
                foreground_if_running,
                wait_for,
                pid_file,
            } => {
                cli::idb::launch::run(
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
                cli::idb::kill::run().await?;
            }
            IdbCommands::Screenshot { dest_path, udid } => {
                cli::idb::screenshot::run(dest_path, udid).await?;
            }
            IdbCommands::Focus { udid } => {
                cli::idb::focus::run(udid).await?;
            }
            IdbCommands::Log {
                udid,
                source,
                log_arguments,
            } => {
                cli::idb::log::run(udid, source, log_arguments).await?;
            }
            IdbCommands::Install {
                bundle_path,
                udid,
                make_debuggable,
                override_mtime,
                compression,
                format,
            } => {
                cli::idb::install::run(
                    bundle_path,
                    udid,
                    make_debuggable,
                    override_mtime,
                    compression,
                    format,
                )
                .await?;
            }
            IdbCommands::Uninstall { bundle_id, udid } => {
                cli::idb::uninstall::run(bundle_id, udid).await?;
            }
            IdbCommands::Approve {
                bundle_id,
                permissions,
                scheme,
                udid,
            } => {
                cli::idb::permissions::approve(bundle_id, permissions, scheme, udid).await?;
            }
            IdbCommands::Crash { command } => match command {
                CrashCommands::List {
                    since,
                    before,
                    bundle_id,
                    name,
                    udid,
                } => {
                    cli::idb::crash::list(since, before, bundle_id, name, udid).await?;
                }
                CrashCommands::Show { name, udid } => {
                    cli::idb::crash::show(name, udid).await?;
                }
                CrashCommands::Delete {
                    since,
                    before,
                    bundle_id,
                    name,
                    all,
                    udid,
                } => {
                    cli::idb::crash::delete(since, before, bundle_id, name, all, udid).await?;
                }
            },
            IdbCommands::File { command } => match command {
                file::FileCommands::Ls {
                    paths,
                    bundle_id,
                    udid,
                } => {
                    cli::idb::file::ls::run(paths, udid, bundle_id).await?;
                }
                file::FileCommands::Mkdir {
                    path,
                    bundle_id,
                    root,
                    udid,
                } => {
                    cli::idb::file::mkdir::run(path, bundle_id, root, udid).await?;
                }
                file::FileCommands::Mv {
                    src_paths,
                    dst_path,
                    bundle_id,
                    root,
                    udid,
                } => {
                    cli::idb::file::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
                }
                file::FileCommands::Rm {
                    paths,
                    udid,
                    bundle_id,
                } => {
                    cli::idb::file::rm::run(paths, udid, bundle_id).await?;
                }
                file::FileCommands::Pull {
                    src_path,
                    dst_path,
                    bundle_id,
                    udid,
                } => {
                    cli::idb::file::pull::run(src_path, dst_path, udid, bundle_id).await?;
                }
                file::FileCommands::Push {
                    src_path,
                    dst_path,
                    bundle_id,
                    udid,
                } => {
                    cli::idb::file::push::run(src_path, dst_path, udid, bundle_id).await?;
                }
                file::FileCommands::Tail {
                    path,
                    bundle_id,
                    udid,
                } => {
                    cli::idb::file::tail::run(path, udid, bundle_id).await?;
                }
                file::FileCommands::Read {
                    src_path,
                    bundle_id,
                    udid,
                } => {
                    cli::idb::file::read::run(src_path, udid, bundle_id).await?;
                }
                file::FileCommands::Write {
                    dst_path,
                    bundle_id,
                    udid,
                } => {
                    cli::idb::file::write::run(dst_path, udid, bundle_id).await?;
                }
            },
            IdbCommands::Tap {
                x,
                y,
                duration,
                udid,
            } => {
                cli::idb::hid::tap::run(x, y, duration, udid).await?;
            }
            IdbCommands::Button {
                button,
                duration,
                udid,
            } => {
                cli::idb::hid::button::run(button, duration, udid).await?;
            }
            IdbCommands::Key {
                keycode,
                duration,
                udid,
            } => {
                cli::idb::hid::key::run(keycode, duration, udid).await?;
            }
            IdbCommands::KeySequence { key_sequence, udid } => {
                cli::idb::hid::key_sequence::run(key_sequence, udid).await?;
            }
            IdbCommands::Text { text, udid } => {
                cli::idb::hid::text::run(text, udid).await?;
            }
            IdbCommands::Swipe {
                x_start,
                y_start,
                x_end,
                y_end,
                duration,
                delta,
                udid,
            } => {
                cli::idb::hid::swipe::run(x_start, y_start, x_end, y_end, duration, delta, udid)
                    .await?;
            }
            IdbCommands::ListApps { udid } => {
                cli::idb::list_apps::run(udid).await?;
            }
            IdbCommands::Location { command } => match command {
                LocationCommands::SetLocation {
                    latitude,
                    longitude,
                    udid,
                } => {
                    cli::idb::location::run(latitude, longitude, udid).await?;
                }
            },
            IdbCommands::Notification { command } => match command {
                NotificationCommands::SendNotification {
                    bundle_id,
                    json_payload,
                    udid,
                } => {
                    cli::idb::notification::run(bundle_id, json_payload, udid).await?;
                }
            },
            IdbCommands::Revoke {
                bundle_id,
                permissions,
                scheme,
                udid,
            } => {
                cli::idb::permissions::revoke(bundle_id, permissions, scheme, udid).await?;
            }
            IdbCommands::Set {
                name,
                value,
                value_type,
                domain,
                udid,
            } => {
                cli::idb::settings::set(name, value, value_type, domain, udid).await?;
            }
            IdbCommands::Get { name, domain, udid } => {
                cli::idb::settings::get(name, domain, udid).await?;
            }
            IdbCommands::List { command } => match command {
                ListCommands::Locale { udid } => {
                    cli::idb::settings::list_locale(udid).await?;
                }
            },
            IdbCommands::Terminate { bundle_id, udid } => {
                cli::idb::terminate::run(bundle_id, udid).await?;
            }
            IdbCommands::Url { command } => match command {
                UrlCommands::Open { url, udid } => {
                    cli::idb::url::run(url, udid).await?;
                }
            },
            IdbCommands::Media { command } => match command {
                cli::idb::media::MediaCommands::AddMedia { file_paths, udid } => {
                    cli::idb::media::add_media(file_paths, udid).await?;
                }
            },
            IdbCommands::Video { command } => match command {
                cli::idb::video::VideoCommands::RecordVideo {
                    output_file,
                    format,
                    fps,
                    udid,
                } => {
                    cli::idb::video::record::run(output_file, format, fps, udid).await?;
                }
                cli::idb::video::VideoCommands::VideoStream {
                    output_file,
                    fps,
                    format,
                    compression_quality,
                    scale_factor,
                    udid,
                } => {
                    cli::idb::video::stream::run(
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
            IdbCommands::PhotosClear { udid } => {
                cli::idb::photos::clear(udid).await?;
            }
            IdbCommands::AccessibilityDescribeAll { nested, udid } => {
                cli::idb::accessibility::describe_all(nested, udid).await?;
            }
            IdbCommands::AccessibilityDescribePoint { x, y, nested, udid } => {
                cli::idb::accessibility::describe_point(x, y, nested, udid).await?;
            }
            IdbCommands::ContactsUpdate { db_path, udid } => {
                cli::idb::contacts::update(db_path, udid).await?;
            }
            IdbCommands::ContactsClear { udid } => {
                cli::idb::contacts::clear(udid).await?;
            }
            IdbCommands::KeychainClear { udid } => {
                cli::idb::keychain::clear(udid).await?;
            }
            IdbCommands::SimulateMemoryWarning { udid } => {
                cli::idb::memory::simulate_warning(udid).await?;
            }
            IdbCommands::XctestInstall {
                test_bundle_path,
                skip_signing,
                compression,
                format,
                udid,
            } => {
                cli::idb::xctest_install::run(
                    test_bundle_path,
                    udid,
                    skip_signing,
                    compression,
                    format,
                )
                .await?;
            }
            IdbCommands::XctestList { udid } => {
                cli::idb::xctest_list::run(udid).await?;
            }
            IdbCommands::XctestListBundle {
                bundle_id,
                app_path,
                udid,
            } => {
                cli::idb::xctest_list_bundle::run(bundle_id, app_path, udid).await?;
            }
            IdbCommands::XctestRun {
                test_bundle_id,
                tests_to_run,
                udid,
            } => {
                cli::idb::xctest_run::run(test_bundle_id, tests_to_run, udid).await?;
            }
            IdbCommands::Target { command } => match command {
                target::TargetCommands::Boot { udid, headless } => {
                    cli::idb::target::boot::run(udid, headless).await?;
                }
                target::TargetCommands::Shutdown { udid } => {
                    cli::idb::target::shutdown::run(udid).await?;
                }
                target::TargetCommands::Erase { udid } => {
                    cli::idb::target::erase::run(udid).await?;
                }
                target::TargetCommands::Create {
                    name,
                    device_type,
                    os_version,
                } => {
                    cli::idb::target::create::run(name, device_type, os_version).await?;
                }
                target::TargetCommands::Clone { udid } => {
                    cli::idb::target::clone::run(udid).await?;
                }
                target::TargetCommands::Delete { udid, all } => {
                    cli::idb::target::delete::run(udid, all).await?;
                }
                target::TargetCommands::Connect { host, port, udid } => {
                    cli::idb::target::connect::run(host, port, udid).await?;
                }
                target::TargetCommands::Disconnect { udid } => {
                    cli::idb::target::disconnect::run(udid).await?;
                }
                target::TargetCommands::Describe { udid, diagnostics } => {
                    cli::idb::target::describe::run(udid, diagnostics).await?;
                }
            },
            IdbCommands::Debugserver { command } => match command {
                debugserver::DebugServerCommands::Start { bundle_id, udid } => {
                    cli::idb::debugserver::start::run(bundle_id, udid).await?;
                }
                debugserver::DebugServerCommands::Stop { udid } => {
                    cli::idb::debugserver::stop::run(udid).await?;
                }
                debugserver::DebugServerCommands::Status { udid } => {
                    cli::idb::debugserver::status::run(udid).await?;
                }
            },
            IdbCommands::Dap { bundle, port, udid } => {
                cli::idb::dap::run(bundle, port, udid).await?;
            }
            IdbCommands::Dsym { command } => match command {
                DsymCommands::Install {
                    dsym_path,
                    bundle_id,
                    compression,
                    format,
                    udid,
                } => {
                    cli::idb::dsym::install(dsym_path, bundle_id, compression, format, udid)
                        .await?;
                }
            },
            IdbCommands::Dylib { command } => match command {
                DylibCommands::Install {
                    dylib_path,
                    format,
                    udid,
                } => {
                    cli::idb::dylib::install(dylib_path, format, udid).await?;
                }
            },
            IdbCommands::Framework { command } => match command {
                FrameworkCommands::Install {
                    framework_path,
                    format,
                    udid,
                } => {
                    cli::idb::framework::install(framework_path, format, udid).await?;
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
                udid,
            } => {
                cli::idb::instruments::run(
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
                    udid,
                } => {
                    cli::idb::xctrace::record(
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
            IdbCommands::Shell { no_prompt, udid } => {
                cli::idb::shell::run(no_prompt, udid).await?;
            }
        },
        Commands::Hid(args) => {
            cli::hid::run(args).await?;
        }
        Commands::Element(args) => {
            cli::element::run(args).await?;
        }
        Commands::App(args) => {
            cli::app::run(args).await?;
        }
        Commands::Device(args) => {
            cli::device::run(args).await?;
        }
        Commands::Snapshot(args) => {
            cli::snapshot::run(args).await?;
        }
    }

    Ok(())
}
