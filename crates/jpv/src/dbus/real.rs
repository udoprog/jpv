use std::future::Future;
use std::pin::{pin, Pin};

use anyhow::{bail, Context, Result};
use async_fuse::Fuse;
use tokio::sync::futures::Notified;
use tokio_dbus::org_freedesktop_dbus::{NameFlag, NameReply};
use tokio_dbus::{
    ty, BodyBuf, Buffers, Connection, Flags, Message, MessageKind, ObjectPath, SendBuf, Serial,
};

use crate::command::service::ServiceArgs;
use crate::open_uri;
use crate::system::{Event, SendClipboardData, Setup, Start, SystemEvents};

const NAME: &str = "se.tedro.JapaneseDictionary";
const PATH: &ObjectPath = ObjectPath::new_const(b"/se/tedro/JapaneseDictionary");

async fn connect(system: bool) -> Result<(Connection, Buffers)> {
    let mut c = if system {
        Connection::system_bus()?
    } else {
        Connection::session_bus()?
    };

    let mut buf = Buffers::new();
    c.connect(&mut buf).await?;

    let serial = buf.hello()?;
    wait_reply(&mut c, &mut buf, serial).await?;

    Ok((c, buf))
}

async fn wait_reply(c: &mut Connection, buf: &mut Buffers, serial: Serial) -> Result<()> {
    loop {
        c.wait(buf).await?;

        let message = buf.recv.last_message()?;

        match message.kind() {
            MessageKind::MethodReturn { reply_serial } if reply_serial == serial => {
                return Ok(());
            }
            MessageKind::Error {
                error_name,
                reply_serial,
            } if reply_serial == serial => {
                bail!("D-Bus error: {error_name}");
            }
            _ => {}
        }
    }
}

pub(crate) async fn send_clipboard(ty: Option<&str>, data: &[u8]) -> Result<()> {
    let (mut c, mut buf) = connect(false).await?;

    let mimetype = ty.unwrap_or("text/plain");

    buf.body.clear();
    buf.body.arguments((mimetype, data))?;

    let m = buf
        .send
        .method_call(PATH, "SendClipboardData")
        .with_interface(NAME)
        .with_destination(NAME)
        .with_body(&buf.body)
        .with_flags(Flags::NO_REPLY_EXPECTED);

    buf.send.write_message(m)?;

    c.flush(&mut buf).await?;
    Ok(())
}

pub(crate) async fn shutdown() -> Result<()> {
    let (mut c, mut buf) = connect(false).await?;

    let m = buf
        .send
        .method_call(PATH, "Shutdown")
        .with_interface(NAME)
        .with_destination(NAME)
        .with_flags(Flags::NO_REPLY_EXPECTED);

    buf.send.write_message(m)?;
    c.flush(&mut buf).await?;
    Ok(())
}

/// Request port from D-Bus service. This will cause the service to activate if
/// it isn't already.
async fn get_port(c: &mut Connection, buf: &mut Buffers) -> Result<u16> {
    let m = buf
        .send
        .method_call(PATH, "GetPort")
        .with_interface(NAME)
        .with_destination(NAME);

    let serial = buf.send.write_message(m)?;
    wait_reply(c, buf, serial).await?;

    let message = buf.recv.last_message()?;
    Ok(message.body().load::<u16>()?)
}

pub(crate) async fn setup(service_args: &ServiceArgs) -> Result<Setup> {
    if service_args.dbus_disable {
        return Ok(Setup::Start(None));
    }

    tracing::debug!("Connecting to D-Bus");

    let (mut c, mut buf) = connect(service_args.dbus_system).await?;

    // Rely on D-Bus activation to start the background service.
    if service_args.background {
        return Ok(Setup::Port(get_port(&mut c, &mut buf).await?));
    }

    tracing::debug!("Requesting name");

    let serial = buf.request_name(NAME, NameFlag::DO_NOT_QUEUE)?;
    wait_reply(&mut c, &mut buf, serial).await?;
    let reply = buf.recv.last_message()?.body().load::<NameReply>()?;

    match reply {
        NameReply::PRIMARY_OWNER => {}
        NameReply::EXISTS => {
            return Ok(Setup::Port(get_port(&mut c, &mut buf).await?));
        }
        reply => {
            tracing::info!(?reply, "Could not acquire name");
            return Ok(Setup::Busy);
        }
    }

    Ok(Setup::Start(Some(Box::new(DbusStart { c, buf }))))
}

struct DbusStart {
    c: Connection,
    buf: Buffers,
}

impl Start for DbusStart {
    fn start<'a>(
        &'a mut self,
        port: u16,
        shutdown: Notified<'a>,
        system_events: &'a SystemEvents,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let mut shutdown = pin!(Fuse::new(shutdown));
            let mut state = State {
                port,
                system_events,
            };

            loop {
                tokio::select! {
                    result = self.c.wait(&mut self.buf) => {
                        result?;

                        let message = self.buf.recv.last_message()?;

                        tracing::trace!(?message);

                        if let MessageKind::MethodCall { path, member } = message.kind() {
                            self.buf.body.clear();

                            let (ret, action) = match handle_method_call(&mut state, path, member, &message, &mut self.buf.body, &mut self.buf.send) {
                                Ok((m, action)) => (m, action),
                                Err(error) => {
                                    tracing::error!("{}", error);
                                    self.buf.body.clear();
                                    self.buf.body.store(error.to_string())?;
                                    let m = message.error("se.tedro.JapaneseDictionary.Error", self.buf.send.next_serial()).with_body(&self.buf.body);
                                    (m, None)
                                }
                            };

                            tracing::trace!(?ret);
                            self.buf.send.write_message(ret)?;

                            if let Some(action) = action {
                                match action {
                                    Action::Shutdown => {
                                        self.c.flush(&mut self.buf).await?;
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                    _ = shutdown.as_mut() => {
                        return Ok(());
                    }
                };
            }
        })
    }
}

struct State<'a> {
    port: u16,
    system_events: &'a SystemEvents,
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

                state
                    .system_events
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
        "org.freedesktop.DBus.Properties" => match member {
            "GetAll" => {
                let _ = msg.body().read::<str>()?;
                body.store_array::<(ty::Str, ty::Variant)>()?.finish();
                (msg.method_return(send.next_serial()).with_body(body), None)
            }
            _ => {
                bail!("Unsupported: {interface}.{member}")
            }
        },
        _ => {
            bail!("Unsupported: {interface}.{member}")
        }
    };

    Ok(m)
}
