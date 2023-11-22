use std::pin::pin;

use anyhow::{bail, Context, Result};
use async_fuse::Fuse;
use tokio::sync::broadcast::Sender;
use tokio::sync::futures::Notified;
use tokio_dbus::org_freedesktop_dbus::{NameFlag, NameReply};
use tokio_dbus::{BodyBuf, Connection, Flags, Message, MessageKind, ObjectPath, SendBuf};

use crate::command::service::ServiceArgs;
use crate::open_uri;
use crate::system::{Event, SendClipboardData, Setup};

const NAME: &'static str = "se.tedro.JapaneseDictionary";
const PATH: &'static ObjectPath = ObjectPath::new_const(b"/se/tedro/JapaneseDictionary");

pub(crate) async fn send_clipboard(ty: Option<&str>, data: &[u8]) -> Result<()> {
    let mut c = Connection::session_bus().await?;

    let mimetype = ty.unwrap_or("text/plain");

    let (_, send, body) = c.buffers();
    body.write(mimetype)?;
    body.write_slice(data)?;

    let m = send
        .method_call(PATH, "SendClipboardData")
        .with_interface(NAME)
        .with_destination(NAME)
        .with_body(&body)
        .with_flags(Flags::NO_REPLY_EXPECTED);

    send.write_message(m)?;

    c.flush().await?;
    Ok(())
}

pub(crate) async fn shutdown() -> Result<()> {
    let mut c = Connection::session_bus().await?;

    let m = c
        .method_call(PATH, "Shutdown")
        .with_interface(NAME)
        .with_destination(NAME)
        .with_flags(Flags::NO_REPLY_EXPECTED);

    c.write_message(m)?;
    c.flush().await?;
    Ok(())
}

/// Request port from D-Bus service. This will cause the service to activate if
/// it isn't already.
async fn get_port(c: &mut Connection) -> Result<u16> {
    let m = c
        .method_call(PATH, "GetPort")
        .with_interface(NAME)
        .with_destination(NAME);

    c.write_message(m)?;

    c.process().await?;
    let message = c.last_message()?;
    Ok(message.body().load::<u16>()?)
}

pub(crate) async fn setup<'a>(
    service_args: &ServiceArgs,
    port: u16,
    shutdown: Notified<'a>,
    broadcast: Sender<Event>,
) -> Result<Setup<'a>> {
    if service_args.dbus_disable {
        return Ok(Setup::Future(None));
    }

    let mut c = if service_args.dbus_system {
        Connection::system_bus().await?
    } else {
        Connection::session_bus().await?
    };

    // Rely on D-Bus activation to start the background service.
    if service_args.background {
        return Ok(Setup::Port(get_port(&mut c).await?));
    }

    let reply = c.request_name(NAME, NameFlag::DO_NOT_QUEUE).await?;

    match reply {
        NameReply::PRIMARY_OWNER => {}
        NameReply::EXISTS => {
            return Ok(Setup::Port(get_port(&mut c).await?));
        }
        reply => {
            tracing::info!(?reply, "Could not acquire name");
            return Ok(Setup::Busy);
        }
    }

    Ok(Setup::Future(Some(Box::pin(async move {
        let mut shutdown = pin!(Fuse::new(shutdown));

        let mut state = State { port, broadcast };

        loop {
            tokio::select! {
                result = c.process() => {
                    result?;

                    let (recv, send, body) = c.buffers();

                    let message = recv.last_message()?;

                    tracing::trace!(?message);

                    match message.kind() {
                        MessageKind::MethodCall { path, member } => {
                            let (ret, action) = match handle_method_call(&mut state, path, member, &message, body, send) {
                                Ok((m, action)) => (m, action),
                                Err(error) => {
                                    tracing::error!("{}", error);
                                    body.clear();
                                    body.write(error.to_string().as_str())?;
                                    let m = message.error("se.tedro.JapaneseDictionary.Error", send.next_serial()).with_body(body);
                                    (m, None)
                                }
                            };

                            tracing::trace!(?ret);
                            send.write_message(ret)?;

                            if let Some(action) = action {
                                match action {
                                    Action::Shutdown => {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                        _ => {
                        }
                    }
                }
                _ = shutdown.as_mut() => {
                    return Ok(());
                }
            };
        }
    }))))
}

struct State {
    port: u16,
    broadcast: Sender<Event>,
}

enum Action {
    Shutdown,
}

/// Handle a method call.
fn handle_method_call<'a>(
    state: &mut State,
    path: &'a ObjectPath,
    member: &'a str,
    msg: &Message<'a>,
    body: &'a mut BodyBuf,
    send: &mut SendBuf,
) -> Result<(Message<'a>, Option<Action>)> {
    let interface = msg.interface().context("Missing interface")?;

    if path != PATH {
        bail!("Bad path: {}", path);
    };

    let m = match interface {
        "org.freedesktop.Application" => match member {
            "Activate" => {
                let address = format!("http://localhost:{}", state.port);
                open_uri::open(&address);
                (msg.method_return(send.next_serial()), None)
            }
            method => bail!("Unknown method: {method}"),
        },
        "se.tedro.JapaneseDictionary" => match member {
            "GetPort" => {
                body.store(state.port)?;
                (msg.method_return(send.next_serial()).with_body(body), None)
            }
            "SendClipboardData" => {
                let mut body = msg.body();
                let mimetype = body.read::<str>()?;
                let data = body.read::<[u8]>()?;

                tracing::trace!(?mimetype, len = data.len());

                let _ = state
                    .broadcast
                    .send(Event::SendClipboardData(SendClipboardData {
                        mimetype: mimetype.to_owned(),
                        data: data.to_vec(),
                    }));

                (msg.method_return(send.next_serial()), None)
            }
            "Shutdown" => (
                msg.method_return(send.next_serial()),
                Some(Action::Shutdown),
            ),
            method => bail!("Unknown method: {method}"),
        },
        interface => bail!("Unknown interface: {}", interface),
    };

    Ok(m)
}
