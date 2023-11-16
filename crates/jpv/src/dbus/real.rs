use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::pin::pin;
use std::str::from_utf8;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_fuse::Fuse;
use dbus::blocking::stdintf::org_freedesktop_dbus::RequestNameReply;
use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::Message;
use tokio::sync::broadcast::Sender;
use tokio::sync::futures::Notified;

use crate::command::service::ServiceArgs;
use crate::system::{Event, SendClipboardData, Setup};

const NAME: &'static str = "se.tedro.JapaneseDictionary";
const PATH: &'static str = "/se/tedro/JapaneseDictionary";
const TIMEOUT: Duration = Duration::from_millis(5000);

pub(crate) fn send_clipboard(ty: Option<&str>, data: &OsStr) -> Result<()> {
    let c = Connection::new_session()?;
    let proxy = c.with_proxy(NAME, PATH, TIMEOUT);
    let mimetype = ty.unwrap_or("text/plain");
    proxy.method_call(NAME, "SendClipboardData", (mimetype, data.as_bytes()))?;
    Ok(())
}

pub(crate) fn shutdown() -> Result<()> {
    let c = Connection::new_session()?;
    let proxy = c.with_proxy(NAME, PATH, TIMEOUT);
    proxy.method_call(NAME, "Shutdown", ())?;
    Ok(())
}

pub(crate) fn setup<'a>(
    service_args: &ServiceArgs,
    port: u16,
    shutdown: Notified<'a>,
    broadcast: Sender<Event>,
) -> Result<Setup<'a>> {
    if service_args.dbus_disable {
        return Ok(Setup::Future(None));
    }

    let stop = Arc::new(AtomicBool::new(false));

    let c = if service_args.dbus_system {
        Connection::new_system()?
    } else {
        Connection::new_session()?
    };

    // Rely on D-Bus activation to start the background service.
    if service_args.background {
        return Ok(Setup::Port(get_port(&c)?));
    }

    let reply = c.request_name(NAME, false, false, true)?;

    match reply {
        RequestNameReply::PrimaryOwner => {}
        RequestNameReply::Exists => {
            return Ok(Setup::Port(get_port(&c)?));
        }
        reply => {
            tracing::info!(?reply, "Could not acquire name");
            return Ok(Setup::Busy);
        }
    }

    let task: tokio::task::JoinHandle<Result<()>> = tokio::task::spawn_blocking({
        let stop = stop.clone();

        move || {
            tracing::trace!(?reply);

            fn to_c_str(n: &str) -> CString {
                CString::new(n.as_bytes()).unwrap()
            }

            let mut state = State {
                port,
                broadcast,
                stop: stop.clone(),
            };

            c.start_receive(
                MatchRule::new(),
                Box::new(move |msg, conn| {
                    tracing::trace!(?msg);

                    match msg.msg_type() {
                        dbus::MessageType::MethodCall => {
                            match handle_method_call(&mut state, &msg) {
                                Ok(m) => {
                                    let _ = conn.channel().send(m);
                                }
                                Err(error) => {
                                    let error = error.to_string();

                                    let _ = conn.channel().send(msg.error(
                                        &"se.tedro.JapaneseDictionary.Error".into(),
                                        &to_c_str(error.as_str()),
                                    ));
                                }
                            };
                        }
                        _ => {}
                    }

                    true
                }),
            );

            let sleep = Duration::from_millis(250);

            while !stop.load(Ordering::Acquire) {
                c.process(sleep)?;
            }

            Ok(())
        }
    });

    Ok(Setup::Future(Some(Box::pin(async move {
        let mut task = pin!(task);
        let mut shutdown = pin!(Fuse::new(shutdown));

        loop {
            tokio::select! {
                _ = shutdown.as_mut() => {
                    stop.store(true, Ordering::Release);
                    continue;
                }
                result = task.as_mut() => {
                    result??;
                    return Ok(());
                }
            };
        }
    }))))
}

/// Request port from D-Bus service. This will cause the service to activate if
/// it isn't already.
fn get_port(c: &Connection) -> Result<u16, anyhow::Error> {
    let proxy = c.with_proxy(NAME, PATH, TIMEOUT);
    let (port,): (u16,) = proxy.method_call(NAME, "GetPort", ())?;
    Ok(port)
}

struct State {
    port: u16,
    broadcast: Sender<Event>,
    stop: Arc<AtomicBool>,
}

/// Handle a method call.
fn handle_method_call(state: &mut State, msg: &Message) -> Result<Message> {
    let path = msg.path().context("Missing destination")?;
    let member = msg.member().context("Missing member")?;

    let PATH = from_utf8(path.as_bytes()).context("Bad path")? else {
        bail!("Unknown path")
    };

    let m = match from_utf8(member.as_bytes()).context("Bad method")? {
        "GetPort" => msg.return_with_args((state.port,)),
        "SendClipboardData" => {
            let (mimetype, data): (String, Vec<u8>) = msg.read2()?;
            tracing::trace!(?mimetype, len = data.len());

            let _ = state
                .broadcast
                .send(Event::SendClipboardData(SendClipboardData {
                    mimetype,
                    data,
                }));

            msg.method_return()
        }
        "Shutdown" => {
            state.stop.store(true, Ordering::Release);
            msg.method_return()
        }
        _ => bail!("Unknown method"),
    };

    Ok(m)
}
