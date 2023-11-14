use std::marker::PhantomData;

use gloo::timers::callback::Timeout;
use lib::api;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::js_sys::{ArrayBuffer, Uint8Array};
use web_sys::{window, BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::{Component, Context};

use crate::error::{Error, Result};

pub enum Msg {
    Reconnect,
    Open,
    Close(CloseEvent),
    Message(MessageEvent),
    Error(ErrorEvent),
}

#[derive(Default)]
pub struct Service<C> {
    socket: Option<WebSocket>,
    _on_open: Option<Closure<dyn Fn()>>,
    _on_close: Option<Closure<dyn Fn(CloseEvent)>>,
    _on_message: Option<Closure<dyn Fn(MessageEvent)>>,
    _on_error: Option<Closure<dyn Fn(ErrorEvent)>>,
    _timeout: Option<Timeout>,
    _marker: PhantomData<C>,
}

impl<C> Service<C>
where
    C: Component,
    C::Message: From<Msg> + From<Error> + From<api::ClientEvent>,
{
    pub(crate) fn new() -> Self {
        Self {
            socket: None,
            _on_open: None,
            _on_close: None,
            _on_message: None,
            _on_error: None,
            _timeout: None,
            _marker: PhantomData,
        }
    }

    pub(crate) fn update(&mut self, ctx: &Context<C>, message: Msg) {
        match message {
            Msg::Reconnect => {
                if let Err(error) = self.connect(ctx) {
                    ctx.link().send_message(error);
                }
            }
            Msg::Open => {
                log::info!("open");
            }
            Msg::Close(e) => {
                log::info!("close: {:?}", e);

                if let Err(error) = self.reconnect(ctx) {
                    ctx.link().send_message(error);
                }
            }
            Msg::Message(e) => {
                let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
                    return;
                };

                let array_buffer = Uint8Array::new(&array_buffer).to_vec();

                match serde_json::from_slice::<api::ClientEvent>(&array_buffer) {
                    Ok(event) => {
                        ctx.link().send_message(event);
                    }
                    Err(error) => {
                        log::info!("error: {:?}", error);
                    }
                }
            }
            Msg::Error(e) => {
                log::info!("error: {:?}", e);

                if let Err(error) = self.reconnect(ctx) {
                    ctx.link().send_message(error);
                }
            }
        }
    }

    pub(crate) fn reconnect(&mut self, ctx: &Context<C>) -> Result<()> {
        if let Some(old) = self.socket.take() {
            old.close()?;
        }

        self._on_open = None;
        self._on_close = None;
        self._on_message = None;
        self._on_error = None;

        let link = ctx.link().clone();

        self._timeout = Some(Timeout::new(1000, move || {
            link.send_message(Msg::Reconnect);
        }));

        Ok(())
    }

    /// Attempt to establish a connection.
    pub(crate) fn connect(&mut self, ctx: &Context<C>) -> Result<()> {
        let window = window().ok_or("no window")?;
        let port = window.location().port()?;
        let url = format!("ws://localhost:{port}/ws");

        let ws = match WebSocket::new(&url) {
            Ok(ws) => ws,
            Err(error) => {
                let link = ctx.link().clone();

                self._timeout = Some(Timeout::new(1000, move || {
                    link.send_message(Msg::Reconnect);
                }));

                return Err(error.into());
            }
        };

        let on_open = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn()> = Box::new(move || {
                link.send_message(Msg::Open);
            });

            Closure::wrap(cb)
        };

        let on_close = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn(CloseEvent)> = Box::new(move |e: CloseEvent| {
                link.send_message(Msg::Close(e));
            });

            Closure::wrap(cb)
        };

        let on_message = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn(MessageEvent)> = Box::new(move |e: MessageEvent| {
                link.send_message(Msg::Message(e));
            });

            Closure::wrap(cb)
        };

        let on_error = {
            let link = ctx.link().clone();

            let cb: Box<dyn Fn(ErrorEvent)> = Box::new(move |e: ErrorEvent| {
                link.send_message(Msg::Error(e));
            });

            Closure::wrap(cb)
        };

        ws.set_binary_type(BinaryType::Arraybuffer);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        if let Some(old) = self.socket.replace(ws) {
            old.close()?;
        }

        self._on_open = Some(on_open);
        self._on_close = Some(on_close);
        self._on_message = Some(on_message);
        self._on_error = Some(on_error);
        Ok(())
    }
}
