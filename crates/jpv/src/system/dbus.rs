use std::ffi::CString;
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
use tokio::sync::futures::Notified;

use crate::System;

const NAME: &'static str = "se.tedro.JapaneseDictionary";
const PATH: &'static str = "/se/tedro/JapaneseDictionary";

pub(crate) fn setup<'a>(port: u16, shutdown: Notified<'a>) -> Result<System<'a>> {
    let stop = Arc::new(AtomicBool::new(false));

    let c = Connection::new_session()?;

    let reply = c.request_name(NAME, false, false, true)?;

    match reply {
        RequestNameReply::PrimaryOwner => {}
        RequestNameReply::Exists => {
            let proxy = c.with_proxy(NAME, PATH, Duration::from_millis(5000));
            let (port,): (u16,) = proxy.method_call(NAME, "GetPort", ())?;
            return Ok(System::Port(port));
        }
        reply => {
            tracing::info!(?reply, "Could not acquire name");
            return Ok(System::Busy);
        }
    }

    let task: tokio::task::JoinHandle<Result<()>> = tokio::task::spawn_blocking({
        let stop = stop.clone();

        move || {
            tracing::trace!(?reply);

            fn to_c_str(n: &str) -> CString {
                CString::new(n.as_bytes()).unwrap()
            }

            let mut state = State { port };

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

    Ok(System::Future(Box::pin(async move {
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
    })))
}

struct State {
    port: u16,
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
            tracing::info!(?mimetype, ?data);
            msg.method_return()
        }
        _ => bail!("Unknown method"),
    };

    Ok(m)
}
