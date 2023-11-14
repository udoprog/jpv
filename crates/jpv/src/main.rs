#![cfg_attr(all(not(feature = "cli"), windows), windows_subsystem = "windows")]

mod database;
mod dbus;
mod system;
mod web;

use std::net::SocketAddr;
use std::net::TcpListener;
use std::pin::pin;

use anyhow::{Context, Result};
use async_fuse::Fuse;
use clap::Parser;
use tokio::signal::ctrl_c;
#[cfg(windows)]
use tokio::signal::windows::ctrl_shutdown;
use tokio::sync::Notify;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    /// Bind to the given address. Default is `127.0.0.1`.
    #[arg(long)]
    bind: Option<String>,
    /// Do not bind to D-Bus.
    #[cfg(feature = "dbus")]
    #[arg(long)]
    dbus_disable: bool,
    /// Bind to session bus.
    #[cfg(feature = "dbus")]
    #[arg(long)]
    dbus_session: bool,
    /// Bind to system bus.
    #[cfg(feature = "dbus")]
    #[arg(long)]
    dbus_system: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = Args::try_parse()?;
    let addr: SocketAddr = args.bind.as_deref().unwrap_or(self::web::BIND).parse()?;
    let listener = TcpListener::bind(addr)?;
    let local_addr = listener.local_addr()?;
    let local_port = self::web::PORT.unwrap_or(local_addr.port());

    let shutdown = Notify::new();

    let (sender, _) = tokio::sync::broadcast::channel(16);
    let system_events = system::SystemEvents(sender.clone());

    let mut dbus = match dbus::setup(&args, local_port, shutdown.notified(), sender)? {
        system::Setup::Future(dbus) => dbus,
        system::Setup::Port(port) => {
            self::web::open(port);
            return Ok(());
        }
        system::Setup::Busy => {
            return Ok(());
        }
    };

    // SAFETY: we know this is only initialized once here exclusively.
    let data = unsafe { self::database::open()? };

    tracing::info!("Loading database...");
    let db = lib::database::Database::new(data).context("loading database")?;
    tracing::info!("Database loaded");

    let mut server = pin!(web::setup(local_port, listener, db, system_events)?);
    tracing::info!("Listening on http://{local_addr}");

    let mut ctrl_c = pin!(Fuse::new(ctrl_c()));

    loop {
        tokio::select! {
            result = server.as_mut() => {
                result?;
            }
            result = dbus.as_mut() => {
                result?;
                tracing::info!("System integration shut down");
                break;
            }
            _ = ctrl_c.as_mut() => {
                tracing::info!("Shutting down...");
                shutdown.notify_one();
            }
        }
    }

    tracing::info!("Bye bye");
    Ok(())
}
