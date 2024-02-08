use std::error::Error;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::pin::pin;

use anyhow::{Context, Result};
use async_fuse::Fuse;
use clap::Parser;
use lib::config::Config;
use lib::data;
use lib::Dirs;
use tokio::signal::ctrl_c;
use tokio::sync::Notify;

use crate::background::Background;
use crate::dbus;
use crate::open_uri;
use crate::system;
use crate::tasks::Tasks;
use crate::web;
use crate::windows;
use crate::Args;

#[cfg(windows)]
async fn shutdown_signal() -> Result<()> {
    use tokio::signal::windows::ctrl_shutdown;
    let mut ctrl_shutdown = ctrl_shutdown()?;
    ctrl_shutdown.recv().await;
    Ok(())
}

#[cfg(not(windows))]
async fn shutdown_signal() -> Result<()> {
    std::future::pending::<()>().await;
    Ok(())
}

#[derive(Default, Parser)]
pub(crate) struct ServiceArgs {
    /// Run the dictionary as a background service. This will prevent a browser window from being opened to the service once it's started.
    #[arg(long)]
    pub(crate) background: bool,
    /// Do not open the URI of the dictionary when started.
    #[arg(long)]
    pub(crate) no_open: bool,
    /// Disable D-Bus binding.
    #[cfg(all(unix, feature = "dbus"))]
    #[arg(long)]
    pub(crate) dbus_disable: bool,
    /// Bind to the D-Bus system bus.
    #[cfg(all(unix, feature = "dbus"))]
    #[arg(long)]
    pub(crate) dbus_system: bool,
    /// Bind to the given address. Default is `127.0.0.1:44714`.
    #[arg(long, value_name = "address")]
    bind: Option<String>,
}

pub(crate) async fn run(
    args: &Args,
    service_args: &ServiceArgs,
    dirs: Dirs,
    config: Config,
    system_events: system::SystemEvents,
    log: crate::log::Capture,
) -> Result<()> {
    let addr: SocketAddr = service_args
        .bind
        .as_deref()
        .unwrap_or(self::web::BIND)
        .parse()?;

    let shutdown = Notify::new();

    let mut dbus = match dbus::setup(service_args)
        .await
        .context("Setting up D-Bus")?
    {
        system::Setup::Start(dbus) => dbus,
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

    let mut windows = match windows::setup()? {
        system::Setup::Start(windows) => windows,
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

    let listener = TcpListener::bind(addr)?;
    let local_addr = listener.local_addr()?;
    let local_port = web::PORT.unwrap_or(local_addr.port());

    let mut windows = match &mut windows {
        Some(windows) => Fuse::new(windows.start(local_port, shutdown.notified(), &system_events)),
        None => Fuse::empty(),
    };

    let mut dbus = match &mut dbus {
        Some(dbus) => Fuse::new(dbus.start(local_port, shutdown.notified(), &system_events)),
        None => Fuse::empty(),
    };

    // SAFETY: we know this is only initialized once here exclusively.
    let indexes = data::open_from_args(&args.index[..], &dirs)?;
    let db = lib::database::Database::open(indexes, &config)?;

    let (channel, mut receiver) = tokio::sync::mpsc::unbounded_channel();

    let tesseract = match tesseract::open("jpn") {
        Ok(tesseract) => {
            if let Some(path) = tesseract.path() {
                tracing::info!("Tesseract OCR support enabled from {}", path.display());
            } else {
                tracing::info!("Tesseract OCR support enabled from system");
            }

            Some(tesseract)
        }
        Err(error) => {
            tracing::warn!("Failed to load Tesseract-OCR: {error}");

            let mut error = error.source();

            while let Some(source) = error {
                tracing::warn!("Caused by: {source}");
                error = source.source();
            }

            None
        }
    };

    let background = Background::new(
        dirs,
        channel,
        config,
        db,
        system_events.clone(),
        tesseract,
        log,
    )?;

    let mut server = pin!(web::setup(
        listener,
        background.clone(),
        system_events.clone()
    )?);
    tracing::info!("Listening on http://{local_addr}");

    if !service_args.no_open {
        let address = format!("http://localhost:{local_port}");
        open_uri::open(&address);
    }

    let mut tasks = Tasks::new();

    let mut shutdown_signal = pin!(Fuse::new(async {
        tokio::select! {
            result = shutdown_signal() => {
                result?;
            }
            result = ctrl_c() => {
                result?;
            }
        }

        Ok::<_, anyhow::Error>(())
    }));

    let mut needs_shutdown_signal = dbus.is_empty() && windows.is_empty();

    while needs_shutdown_signal || !dbus.is_empty() || !windows.is_empty() {
        tokio::select! {
            result = server.as_mut() => {
                result?;
                tracing::info!("Server shut down");
            }
            result = dbus.as_pin_mut() => {
                result?;
                tracing::info!("D-Bus integration shut down");
                shutdown.notify_waiters();
            }
            result = windows.as_pin_mut() => {
                result?;
                tracing::info!("Windows integration shut down");
                shutdown.notify_waiters();
            }
            Some(event) = receiver.recv() => {
                background.handle_event(event, args, &mut tasks).await.context("Handling background event")?;
            }
            result = tasks.wait() => {
                let completed = result?;
                background.complete_task(completed);
            }
            _ = shutdown_signal.as_mut() => {
                tracing::info!("Shutting down...");
                shutdown.notify_waiters();
                needs_shutdown_signal = false;
            }
        }
    }

    // Causes any background processes to shut down.
    tasks.finish().await;
    tracing::info!("Bye!");
    Ok(())
}
