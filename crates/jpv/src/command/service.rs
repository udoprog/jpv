use std::net::SocketAddr;
use std::net::TcpListener;
use std::pin::pin;

use anyhow::{anyhow, Context, Result};
use async_fuse::Fuse;
use clap::Parser;
use tokio::signal::ctrl_c;
#[cfg(windows)]
use tokio::signal::windows::ctrl_shutdown;
use tokio::sync::Notify;

use crate::dirs::Dirs;
use crate::open_uri;
use crate::Args;
use crate::{database, dbus, system, web};

#[derive(Default, Parser)]
pub(crate) struct ServiceArgs {
    /// Run the dictionary as a background service. This will prevent a browser window from being opened to the service once it's started.
    #[arg(long)]
    pub(crate) background: bool,
    /// Do not open the URI of the dictionary when started.
    #[arg(long)]
    pub(crate) no_open: bool,
    /// Disable D-Bus binding.
    #[cfg(feature = "dbus")]
    #[arg(long)]
    pub(crate) dbus_disable: bool,
    /// Bind to the D-Bus system bus.
    #[cfg(feature = "dbus")]
    #[arg(long)]
    pub(crate) dbus_system: bool,
}

pub(crate) async fn run(args: &Args, service_args: &ServiceArgs, dirs: &Dirs) -> Result<()> {
    let addr: SocketAddr = args.bind.as_deref().unwrap_or(self::web::BIND).parse()?;
    let listener = TcpListener::bind(addr)?;
    let local_addr = listener.local_addr()?;
    let local_port = web::PORT.unwrap_or(local_addr.port());

    let shutdown = Notify::new();

    let (sender, _) = tokio::sync::broadcast::channel(16);
    let system_events = system::SystemEvents(sender.clone());

    let mut dbus = match dbus::setup(service_args, local_port, shutdown.notified(), sender)
        .context("Setting up D-Bus")?
    {
        system::Setup::Future(dbus) => match dbus {
            Some(dbus) => Fuse::new(dbus),
            None => Fuse::empty(),
        },
        system::Setup::Port(port) => {
            tracing::info!("Listening on http://localhost:{port}");

            if !service_args.no_open {
                let address = format!("http://localhost:{port}");
                open_uri::open(&address);
            }

            return Ok(());
        }
        system::Setup::Busy => {
            return Ok(());
        }
    };

    // SAFETY: we know this is only initialized once here exclusively.
    let (data, location) = unsafe { database::open(args, dirs)? };

    let db = lib::database::Database::open(data)
        .with_context(|| anyhow!("Loading dictionary from {location}"))?;

    let mut server = pin!(web::setup(local_port, listener, db, system_events)?);
    tracing::info!("Listening on http://{local_addr}");

    if !service_args.no_open {
        let address = format!("http://localhost:{local_port}");
        open_uri::open(&address);
    }

    let mut ctrl_c = pin!(Fuse::new(ctrl_c()));

    loop {
        tokio::select! {
            result = server.as_mut() => {
                result?;
            }
            result = dbus.as_pin_mut() => {
                result?;
                tracing::info!("D-Bus integration shut down");
                break;
            }
            _ = ctrl_c.as_mut() => {
                tracing::info!("Shutting down...");
                shutdown.notify_one();

                if dbus.is_empty() {
                    break;
                }
            }
        }
    }

    tracing::info!("Bye bye");
    Ok(())
}
