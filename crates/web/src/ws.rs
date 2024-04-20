use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::mem::take;
use std::rc::Rc;

use anyhow::anyhow;
use gloo::timers::callback::Timeout;
use lib::api;
use musli_utils::reader::SliceReader;
use slab::Slab;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::js_sys::{ArrayBuffer, Uint8Array};
use web_sys::{window, BinaryType, CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::{Callback, Component, Context};

use crate::error::{Error, Result};

const INITIAL_TIMEOUT: u32 = 250;
const MAX_TIMEOUT: u32 = 16000;

pub enum Msg {
    Reconnect,
    Open,
    Close(CloseEvent),
    Message(MessageEvent),
    Error(ErrorEvent),
    ClientRequest((api::OwnedClientRequestEnvelope, Vec<u8>)),
}

#[derive(Debug, Clone, Copy)]
struct Opened {
    at: Option<f64>,
}

pub struct Service<C> {
    shared: Rc<Shared>,
    socket: Option<WebSocket>,
    opened: Option<Opened>,
    state: State,
    buffer: Vec<(api::OwnedClientRequestEnvelope, Vec<u8>)>,
    output: Vec<u8>,
    timeout: u32,
    on_open: Closure<dyn Fn()>,
    on_close: Closure<dyn Fn(CloseEvent)>,
    on_message: Closure<dyn Fn(MessageEvent)>,
    on_error: Closure<dyn Fn(ErrorEvent)>,
    _timeout: Option<Timeout>,
    _ping_timeout: Option<Timeout>,
    _marker: PhantomData<C>,
}

impl<C> Service<C>
where
    C: Component,
    C::Message: From<Msg> + From<Error>,
{
    pub(crate) fn new(ctx: &Context<C>) -> (Self, Handle) {
        let shared = Rc::new(Shared {
            serial: Cell::new(0),
            onmessage: ctx.link().callback(Msg::ClientRequest),
            requests: RefCell::new(Slab::new()),
            broadcasts: RefCell::new(Slab::new()),
            state_changes: RefCell::new(Slab::new()),
        });

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

        let this = Self {
            shared: shared.clone(),
            socket: None,
            opened: None,
            state: State::Closed,
            buffer: Vec::new(),
            output: Vec::new(),
            timeout: INITIAL_TIMEOUT,
            on_open,
            on_close,
            on_message,
            on_error,
            _timeout: None,
            _ping_timeout: None,
            _marker: PhantomData,
        };

        let handle = Handle { shared };

        (this, handle)
    }

    /// Send a client message.
    fn send_message(
        &mut self,
        message: api::OwnedClientRequestEnvelope,
        body: Vec<u8>,
    ) -> Result<()> {
        let Some(socket) = &self.socket else {
            return Err(anyhow!("Socket is not connected").into());
        };

        musli_storage::to_writer(&mut self.output, &message)?;
        self.output.extend(body.as_slice());
        socket.send_with_u8_array(&self.output)?;
        self.output.clear();
        Ok(())
    }

    fn set_open(&mut self) {
        log::trace!("Set open");
        self.opened = Some(Opened { at: now() });
        self.emit_state_change(State::Open);
    }

    fn is_open_for_a_while(&self) -> bool {
        let Some(opened) = self.opened else {
            return false;
        };

        let Some(at) = opened.at else {
            return false;
        };

        let Some(now) = now() else {
            return false;
        };

        (now - at) >= 250.0
    }

    fn set_closed(&mut self, ctx: &Context<C>)
    where
        C::Message: From<Error>,
    {
        log::trace!(
            "Set closed timeout={}, opened={:?}",
            self.timeout,
            self.opened
        );

        if !self.is_open_for_a_while() {
            if self.timeout < MAX_TIMEOUT {
                self.timeout *= 2;
            }
        } else {
            self.timeout = INITIAL_TIMEOUT;
        }

        self.opened = None;
        self.reconnect(ctx);
        self.emit_state_change(State::Closed);
    }

    fn emit_state_change(&mut self, state: State) {
        if self.state != state {
            let callbacks = self.shared.state_changes.borrow();

            for (_, callback) in callbacks.iter() {
                callback.emit(state);
            }

            self.state = state;
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
                log::trace!("Open");
                self.set_open();

                let buffer = take(&mut self.buffer);

                for (message, body) in buffer {
                    if let Err(error) = self.send_message(message, body) {
                        ctx.link().send_message(error);
                    }
                }
            }
            Msg::Close(e) => {
                log::trace!("Close: {} ({})", e.code(), e.reason());
                self.set_closed(ctx);
            }
            Msg::Message(e) => {
                let Ok(array_buffer) = e.data().dyn_into::<ArrayBuffer>() else {
                    return;
                };

                let buffer = Uint8Array::new(&array_buffer).to_vec();
                let mut reader = SliceReader::new(&buffer);

                let event: api::ClientEvent<'_> = match musli_storage::decode(&mut reader) {
                    Ok(event) => event,
                    Err(error) => {
                        log::error!("{}", error);
                        return;
                    }
                };

                log::info!("Got client event: {:?}", event);

                match event {
                    api::ClientEvent::Broadcast(event) => {
                        let broadcasts = self.shared.broadcasts.borrow();

                        let mut it = broadcasts.iter();

                        let last = it.next_back();

                        for (_, callback) in it {
                            callback.emit(borrowme::to_owned(&event.kind));
                        }

                        if let Some((_, callback)) = last {
                            callback.emit(borrowme::to_owned(event.kind));
                        }
                    }
                    api::ClientEvent::ClientResponse(response) => {
                        log::trace!(
                            "Got response: index={}, serial={}",
                            response.index,
                            response.serial
                        );

                        let requests = self.shared.requests.borrow();

                        let Some(pending) = requests.get(response.index) else {
                            return;
                        };

                        if pending.serial == response.serial {
                            if let Some(error) = response.error {
                                pending
                                    .callback
                                    .emit(Err(Error::from(anyhow!("{}", error))));
                            } else {
                                let at = buffer.len() - reader.remaining();
                                pending.callback.emit(Ok((buffer, at)));
                            }
                        }
                    }
                }
            }
            Msg::Error(e) => {
                log::error!("{}", e.message());
                self.set_closed(ctx);
            }
            Msg::ClientRequest((request, body)) => {
                if self.opened.is_none() {
                    self.buffer.push((request, body));
                    return;
                }

                if let Err(error) = self.send_message(request, body) {
                    ctx.link().send_message(error);
                }
            }
        }
    }

    pub(crate) fn reconnect(&mut self, ctx: &Context<C>)
    where
        C::Message: From<Error>,
    {
        if let Some(old) = self.socket.take() {
            if let Err(error) = old.close() {
                ctx.link().send_message(Error::from(error));
            }
        }

        let link = ctx.link().clone();

        self._timeout = Some(Timeout::new(self.timeout, move || {
            link.send_message(Msg::Reconnect);
        }));
    }

    /// Attempt to establish a connection.
    pub(crate) fn connect(&mut self, ctx: &Context<C>) -> Result<()> {
        let window = window().ok_or("no window")?;
        let port = window.location().port()?;
        let url = format!("ws://127.0.0.1:{port}/ws");

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

        ws.set_binary_type(BinaryType::Arraybuffer);
        ws.set_onopen(Some(self.on_open.as_ref().unchecked_ref()));
        ws.set_onclose(Some(self.on_close.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(self.on_message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(self.on_error.as_ref().unchecked_ref()));

        if let Some(old) = self.socket.replace(ws) {
            old.close()?;
        }

        Ok(())
    }
}

fn now() -> Option<f64> {
    Some(window()?.performance()?.now())
}

/// The handle for a pending request. Dropping this handle cancels the request.
#[derive(Default)]
pub struct Request {
    inner: Option<(Rc<Shared>, usize)>,
}

impl Request {
    /// An empty request handler.
    pub fn empty() -> Self {
        Self::default()
    }
}

impl Drop for Request {
    #[inline]
    fn drop(&mut self) {
        if let Some((shared, index)) = self.inner.take() {
            shared.requests.borrow_mut().try_remove(index);
        }
    }
}

/// The handle for a pending request. Dropping this handle cancels the request.
pub struct Listener {
    index: usize,
    shared: Rc<Shared>,
}

impl Drop for Listener {
    #[inline]
    fn drop(&mut self) {
        self.shared.broadcasts.borrow_mut().try_remove(self.index);
    }
}

/// The handle for state change listening. Dropping this handle cancels the request.
pub struct StateListener {
    index: usize,
    shared: Rc<Shared>,
}

impl Drop for StateListener {
    #[inline]
    fn drop(&mut self) {
        self.shared
            .state_changes
            .borrow_mut()
            .try_remove(self.index);
    }
}

struct Pending {
    serial: u32,
    callback: Callback<Result<(Vec<u8>, usize)>>,
}

/// The state of the connection.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum State {
    /// The connection is open.
    Open,
    /// The connection is closed.
    Closed,
}

struct Shared {
    serial: Cell<u32>,
    onmessage: Callback<(api::OwnedClientRequestEnvelope, Vec<u8>)>,
    requests: RefCell<Slab<Pending>>,
    broadcasts: RefCell<Slab<Callback<api::OwnedBroadcastKind>>>,
    state_changes: RefCell<Slab<Callback<State>>>,
}

#[derive(Clone)]
pub(crate) struct Handle {
    shared: Rc<Shared>,
}

impl Handle {
    pub(crate) fn request<T>(&self, request: T, callback: Callback<Result<T::Response>>) -> Request
    where
        T: api::Request,
    {
        let body = match musli_storage::to_vec(&request) {
            Ok(body) => body,
            Err(error) => {
                callback.emit(Err(Error::from(error)));
                return Request::default();
            }
        };

        let mut requests = self.shared.requests.borrow_mut();
        let serial = self.shared.serial.get();
        self.shared.serial.set(serial.wrapping_add(1));

        let pending = Pending {
            serial,
            callback: Callback::from(move |body: Result<(Vec<u8>, usize)>| {
                let (body, at) = match body {
                    Ok(body) => body,
                    Err(error) => {
                        callback.emit(Err(error));
                        return;
                    }
                };

                match musli_storage::from_slice(&body[at..]) {
                    Ok(payload) => {
                        callback.emit(Ok(payload));
                    }
                    Err(error) => {
                        callback.emit(Err(Error::from(error)));
                    }
                }
            }),
        };

        let index = requests.insert(pending);

        self.shared.onmessage.emit((
            api::OwnedClientRequestEnvelope {
                index,
                serial,
                kind: T::KIND.to_string(),
            },
            body,
        ));

        Request {
            inner: Some((self.shared.clone(), index)),
        }
    }

    pub(crate) fn listen<C>(&self, ctx: &Context<C>) -> Listener
    where
        C: Component,
        C::Message: From<api::OwnedBroadcastKind>,
    {
        let mut broadcasts = self.shared.broadcasts.borrow_mut();
        let index = broadcasts.insert(ctx.link().callback(C::Message::from));

        Listener {
            index,
            shared: self.shared.clone(),
        }
    }

    /// Listen for state changes.
    pub(crate) fn state_changes<C>(&self, ctx: &Context<C>) -> StateListener
    where
        C: Component,
        C::Message: From<State>,
    {
        let mut state = self.shared.state_changes.borrow_mut();
        let index = state.insert(ctx.link().callback(C::Message::from));

        StateListener {
            index,
            shared: self.shared.clone(),
        }
    }
}

impl PartialEq for Handle {
    #[inline]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
