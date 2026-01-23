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

    match cli.command {
        // ==================== Core Commands ====================
        Commands::Tap(args) => {
            core::tap::run(args).await?;
        }
        Commands::Check(args) => {
            core::check::run(args, true).await?;
        }
        Commands::Uncheck(args) => {
            core::check::run(args, false).await?;
        }
        Commands::Select(args) => {
            core::select::run(args).await?;
        }
        Commands::LongPress(args) => {
            core::long_press::run(args).await?;
        }
        Commands::Fill(args) => {
            core::fill::run(args).await?;
        }
        Commands::Type(args) => {
            core::type_cmd::run(args).await?;
        }
        Commands::Swipe(args) => {
            core::swipe::run(args).await?;
        }
        Commands::Scroll(args) => {
            core::scroll::run(args).await?;
        }
        Commands::Get(args) => {
            core::get::run(args).await?;
        }
        Commands::Is(args) => {
            core::is_cmd::run(args).await?;
        }
        Commands::Wait(args) => {
            core::wait::run(args).await?;
        }
        Commands::Screenshot(args) => {
            core::screenshot::run(args).await?;
        }
        Commands::Find(args) => {
            core::find::run(args).await?;
        }
        Commands::Snapshot(args) => {
            snapshot::run(args).await?;
        }
        Commands::Record(args) => {
            record::run(args).await?;
        }
        Commands::Console(args) => {
            console::run(args).await?;
        }

        // ==================== Existing Commands ====================
        Commands::App(args) => {
            app::run(args).await?;
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
                udid,
                wait_for_debugger,
                foreground_if_running,
                wait_for,
                pid_file,
            } => {
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
            IdbCommands::Focus { udid } => {
                idb::focus::run(udid).await?;
            }
            IdbCommands::Log {
                udid,
                source,
                log_arguments,
            } => {
                idb::log::run(udid, source, log_arguments).await?;
            }
            IdbCommands::Install {
                bundle_path,
                udid,
                make_debuggable,
                override_mtime,
                compression,
                format,
            } => {
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
            IdbCommands::Uninstall { bundle_id, udid } => {
                idb::uninstall::run(bundle_id, udid).await?;
            }
            IdbCommands::Approve {
                bundle_id,
                permissions,
                scheme,
                udid,
            } => {
                idb::permissions::approve(bundle_id, permissions, scheme, udid).await?;
            }
            IdbCommands::Crash { command } => match command {
                CrashCommands::List {
                    since,
                    before,
                    bundle_id,
                    name,
                    udid,
                } => {
                    idb::crash::list(since, before, bundle_id, name, udid).await?;
                }
                CrashCommands::Show { name, udid } => {
                    idb::crash::show(name, udid).await?;
                }
                CrashCommands::Delete {
                    since,
                    before,
                    bundle_id,
                    name,
                    all,
                    udid,
                } => {
                    idb::crash::delete(since, before, bundle_id, name, all, udid).await?;
                }
            },
            IdbCommands::File { command } => match command {
                file::FileCommands::Ls {
                    paths,
                    bundle_id,
                    udid,
                } => {
                    idb::file::ls::run(paths, udid, bundle_id).await?;
                }
                file::FileCommands::Mkdir {
                    path,
                    bundle_id,
                    root,
                    udid,
                } => {
                    idb::file::mkdir::run(path, bundle_id, root, udid).await?;
                }
                file::FileCommands::Mv {
                    src_paths,
                    dst_path,
                    bundle_id,
                    root,
                    udid,
                } => {
                    idb::file::mv::run(src_paths, dst_path, bundle_id, root, udid).await?;
                }
                file::FileCommands::Rm {
                    paths,
                    udid,
                    bundle_id,
                } => {
                    idb::file::rm::run(paths, udid, bundle_id).await?;
                }
                file::FileCommands::Pull {
                    src_path,
                    dst_path,
                    bundle_id,
                    udid,
                } => {
                    idb::file::pull::run(src_path, dst_path, udid, bundle_id).await?;
                }
                file::FileCommands::Push {
                    src_path,
                    dst_path,
                    bundle_id,
                    udid,
                } => {
                    idb::file::push::run(src_path, dst_path, udid, bundle_id).await?;
                }
                file::FileCommands::Tail {
                    path,
                    bundle_id,
                    udid,
                } => {
                    idb::file::tail::run(path, udid, bundle_id).await?;
                }
                file::FileCommands::Read {
                    src_path,
                    bundle_id,
                    udid,
                } => {
                    idb::file::read::run(src_path, udid, bundle_id).await?;
                }
                file::FileCommands::Write {
                    dst_path,
                    bundle_id,
                    udid,
                } => {
                    idb::file::write::run(dst_path, udid, bundle_id).await?;
                }
            },
            IdbCommands::Tap {
                x,
                y,
                duration,
                udid,
            } => {
                idb::hid::tap::run(x, y, duration, udid).await?;
            }
            IdbCommands::Button {
                button,
                duration,
                udid,
            } => {
                idb::hid::button::run(button, duration, udid).await?;
            }
            IdbCommands::Key {
                keycode,
                duration,
                udid,
            } => {
                idb::hid::key::run(keycode, duration, udid).await?;
            }
            IdbCommands::KeySequence { key_sequence, udid } => {
                idb::hid::key_sequence::run(key_sequence, udid).await?;
            }
            IdbCommands::Text { text, udid } => {
                idb::hid::text::run(text, udid).await?;
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
                idb::hid::swipe::run(x_start, y_start, x_end, y_end, duration, delta, udid).await?;
            }
            IdbCommands::ListApps { udid } => {
                idb::list_apps::run(udid).await?;
            }
            IdbCommands::Location { command } => match command {
                LocationCommands::SetLocation {
                    latitude,
                    longitude,
                    udid,
                } => {
                    idb::location::run(latitude, longitude, udid).await?;
                }
            },
            IdbCommands::Notification { command } => match command {
                NotificationCommands::SendNotification {
                    bundle_id,
                    json_payload,
                    udid,
                } => {
                    idb::notification::run(bundle_id, json_payload, udid).await?;
                }
            },
            IdbCommands::Revoke {
                bundle_id,
                permissions,
                scheme,
                udid,
            } => {
                idb::permissions::revoke(bundle_id, permissions, scheme, udid).await?;
            }
            IdbCommands::Set {
                name,
                value,
                value_type,
                domain,
                udid,
            } => {
                idb::settings::set(name, value, value_type, domain, udid).await?;
            }
            IdbCommands::Get { name, domain, udid } => {
                idb::settings::get(name, domain, udid).await?;
            }
            IdbCommands::List { command } => match command {
                ListCommands::Locale { udid } => {
                    idb::settings::list_locale(udid).await?;
                }
            },
            IdbCommands::Terminate { bundle_id, udid } => {
                idb::terminate::run(bundle_id, udid).await?;
            }
            IdbCommands::Url { command } => match command {
                UrlCommands::Open { url, udid } => {
                    idb::url::run(url, udid).await?;
                }
            },
            IdbCommands::Media { command } => match command {
                idb::media::MediaCommands::AddMedia { file_paths, udid } => {
                    idb::media::add_media(file_paths, udid).await?;
                }
            },
            IdbCommands::Video { command } => match command {
                idb::video::VideoCommands::RecordVideo {
                    output_file,
                    format,
                    fps,
                    udid,
                } => {
                    idb::video::record::run(output_file, format, fps, udid).await?;
                }
                idb::video::VideoCommands::VideoStream {
                    output_file,
                    fps,
                    format,
                    compression_quality,
                    scale_factor,
                    udid,
                } => {
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
            IdbCommands::PhotosClear { udid } => {
                idb::photos::clear(udid).await?;
            }
            IdbCommands::AccessibilityDescribeAll { nested, udid } => {
                idb::accessibility::describe_all(nested, udid).await?;
            }
            IdbCommands::AccessibilityDescribePoint { x, y, nested, udid } => {
                idb::accessibility::describe_point(x, y, nested, udid).await?;
            }
            IdbCommands::ContactsUpdate { db_path, udid } => {
                idb::contacts::update(db_path, udid).await?;
            }
            IdbCommands::ContactsClear { udid } => {
                idb::contacts::clear(udid).await?;
            }
            IdbCommands::KeychainClear { udid } => {
                idb::keychain::clear(udid).await?;
            }
            IdbCommands::SimulateMemoryWarning { udid } => {
                idb::memory::simulate_warning(udid).await?;
            }
            IdbCommands::XctestInstall {
                test_bundle_path,
                skip_signing,
                compression,
                format,
                udid,
            } => {
                idb::xctest_install::run(test_bundle_path, udid, skip_signing, compression, format)
                    .await?;
            }
            IdbCommands::XctestList { udid } => {
                idb::xctest_list::run(udid).await?;
            }
            IdbCommands::XctestListBundle {
                bundle_id,
                app_path,
                udid,
            } => {
                idb::xctest_list_bundle::run(bundle_id, app_path, udid).await?;
            }
            IdbCommands::XctestRun {
                test_bundle_id,
                tests_to_run,
                udid,
            } => {
                idb::xctest_run::run(test_bundle_id, tests_to_run, udid).await?;
            }
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
                target::TargetCommands::Delete { udid, all } => {
                    idb::target::delete::run(udid, all).await?;
                }
                target::TargetCommands::Connect { host, port, udid } => {
                    idb::target::connect::run(host, port, udid).await?;
                }
                target::TargetCommands::Disconnect { udid } => {
                    idb::target::disconnect::run(udid).await?;
                }
                target::TargetCommands::Describe { udid, diagnostics } => {
                    idb::target::describe::run(udid, diagnostics).await?;
                }
            },
            IdbCommands::Debugserver { command } => match command {
                debugserver::DebugServerCommands::Start { bundle_id, udid } => {
                    idb::debugserver::start::run(bundle_id, udid).await?;
                }
                debugserver::DebugServerCommands::Stop { udid } => {
                    idb::debugserver::stop::run(udid).await?;
                }
                debugserver::DebugServerCommands::Status { udid } => {
                    idb::debugserver::status::run(udid).await?;
                }
            },
            IdbCommands::Dap { bundle, port, udid } => {
                idb::dap::run(bundle, port, udid).await?;
            }
            IdbCommands::Dsym { command } => match command {
                DsymCommands::Install {
                    dsym_path,
                    bundle_id,
                    compression,
                    format,
                    udid,
                } => {
                    idb::dsym::install(dsym_path, bundle_id, compression, format, udid).await?;
                }
            },
            IdbCommands::Dylib { command } => match command {
                DylibCommands::Install {
                    dylib_path,
                    format,
                    udid,
                } => {
                    idb::dylib::install(dylib_path, format, udid).await?;
                }
            },
            IdbCommands::Framework { command } => match command {
                FrameworkCommands::Install {
                    framework_path,
                    format,
                    udid,
                } => {
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
                udid,
            } => {
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
                    udid,
                } => {
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
            IdbCommands::Shell { no_prompt, udid } => {
                idb::shell::run(no_prompt, udid).await?;
            }
        },
    }

    Ok(())
}
