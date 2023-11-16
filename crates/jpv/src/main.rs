#![cfg_attr(all(not(feature = "cli"), windows), windows_subsystem = "windows")]

mod command;
mod database;
mod dbus;
mod dirs;
mod open_uri;
mod system;
mod web;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use dirs::Dirs;
#[cfg(windows)]
use tokio::signal::windows::ctrl_shutdown;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Subcommand)]
enum Command {
    /// Run as a service (default).
    Service(command::service::ServiceArgs),
    /// Perform a cli lookup.
    Cli(command::cli::CliArgs),
    /// Send clipboard to the service.
    SendClipboard(command::send_clipboard::SendClipboardArgs),
    /// Build the dictionary database. This must be performed before the cli or service can be used.
    #[cfg(feature = "build")]
    Build(command::build::BuildArgs),
}

#[derive(Parser)]
struct Args {
    /// Bind to the given address. Default is `127.0.0.1:0`.
    #[arg(long, value_name = "address")]
    bind: Option<String>,
    /// Specify paths to indexes to use.
    #[arg(long, value_name = "index")]
    index: Vec<PathBuf>,
    /// Command to run, by default this runs the service.
    #[command(subcommand)]
    command: Option<Command>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::try_parse()?;

    let directive = match &args.command {
        // Logging is not desired for CLI tool by default.
        Some(Command::Cli(..)) => None,
        _ => Some("jpv=info"),
    };

    let mut filter = EnvFilter::builder();

    if let Some(directive) = directive {
        filter = filter.with_default_directive(directive.parse()?);
    }

    let filter = filter.from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let dirs = Dirs::open()?;

    match &args.command {
        None => {
            let service_args = Default::default();
            self::command::service::run(&args, &service_args, &dirs).await?;
        }
        Some(Command::Service(service_args)) => {
            self::command::service::run(&args, service_args, &dirs).await?;
        }
        Some(Command::Cli(cli_args)) => {
            self::command::cli::run(&args, cli_args, &dirs).await?;
        }
        Some(Command::SendClipboard(send_clipboard_args)) => {
            self::command::send_clipboard::run(&send_clipboard_args)?;
        }
        #[cfg(feature = "build")]
        Some(Command::Build(build_args)) => {
            self::command::build::run(&args, build_args, &dirs).await?;
        }
    }

    Ok(())
}
