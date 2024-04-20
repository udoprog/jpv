use std::borrow::Cow;
use std::net::SocketAddr;

use anyhow::{bail, Result};
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::ConnectInfo;
use axum::response::IntoResponse;
use axum::Extension;
use lib::api::{self, Request};
use musli::mode::Binary;
use musli::Encode;
use musli_utils::reader::SliceReader;
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_stream::StreamExt;
use tracing::{Instrument, Level};

use crate::background::{Background, Install};
use crate::system;

pub(super) async fn entry(
    ws: WebSocketUpgrade,
    Extension(bg): Extension<Background>,
    Extension(system_events): Extension<system::SystemEvents>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let span = tracing::span!(Level::INFO, "websocket", ?remote);

        let mut server = Server {
            system_events,
            bg: bg.clone(),
            output: Vec::new(),
            body: Vec::new(),
            socket,
        };

        if let Err(error) = server.run().instrument(span).await {
            tracing::error!(?error);
        }
    })
}

struct Server {
    system_events: system::SystemEvents,
    bg: Background,
    output: Vec<u8>,
    body: Vec<u8>,
    socket: WebSocket,
}

impl Server {
    async fn run(&mut self) -> Result<()> {
        tracing::trace!("Accepted");

        const CLOSE_NORMAL: u16 = 1000;
        const CLOSE_PROTOCOL_ERROR: u16 = 1002;
        const CLOSE_TIMEOUT: Duration = Duration::from_secs(30);
        const PING_TIMEOUT: Duration = Duration::from_secs(10);

        let mut last_ping = None::<u32>;
        let mut rng = SmallRng::seed_from_u64(0x404241112);
        let mut close_interval = tokio::time::interval(CLOSE_TIMEOUT);
        close_interval.reset();

        let mut ping_interval = tokio::time::interval(PING_TIMEOUT);
        ping_interval.reset();

        let mut receiver = self.system_events.subscribe();

        self.log_backfill().await?;

        let close_here = loop {
            tokio::select! {
                _ = close_interval.tick() => {
                    break Some((CLOSE_NORMAL, "connection timed out"));
                }
                _ = ping_interval.tick() => {
                    let payload = rng.gen::<u32>();
                    last_ping = Some(payload);
                    let data = payload.to_ne_bytes().into_iter().collect::<Vec<_>>();
                    tracing::trace!(data = ?&data[..], "Sending ping");
                    self.socket.send(Message::Ping(data)).await?;
                    ping_interval.reset();
                }
                event = receiver.recv() => {
                    let Ok(event) = event else {
                        break Some((CLOSE_NORMAL, "system shutting down"));
                    };

                    if let Err(error) = self.system_event(event).await {
                        tracing::error!(?error, "Failed to process system event");
                    };
                }
                message = self.socket.next() => {
                    let Some(message) = message else {
                        break None;
                    };

                    match message? {
                        Message::Text(_) => break Some((CLOSE_PROTOCOL_ERROR, "unsupported message")),
                        Message::Binary(bytes) => {
                            let mut reader = SliceReader::new(&bytes);
                            let (request, result) = self.handle_envelope(&mut reader).await?;

                            if reader.remaining() > 0 {
                                break Some((CLOSE_PROTOCOL_ERROR, "extra data"));
                            }

                            let error = match result {
                                Ok(()) => None,
                                Err(error) => {
                                    tracing::warn!(?error, "Failed to handle request");
                                    self.body.clear();
                                    Some(error.to_string())
                                }
                            };

                            self.write(api::ClientEvent::ClientResponse(api::ClientResponseEnvelope {
                                index: request.index,
                                serial: request.serial,
                                error: error.as_deref(),
                            }))?;

                            self.output.extend_from_slice(&self.body);
                            self.body.clear();
                            self.flush().await?;
                        },
                        Message::Ping(payload) => {
                            self.socket.send(Message::Pong(payload)).await?;
                            continue;
                        },
                        Message::Pong(data) => {
                            tracing::trace!(data = ?&data[..], "Pong");

                            let Some(expected) = last_ping else {
                                continue;
                            };

                            if expected.to_ne_bytes()[..] != data[..] {
                                continue;
                            }

                            close_interval.reset();
                            ping_interval.reset();
                            last_ping = None;
                        },
                        Message::Close(_) => break None,
                    }
                }
            }
        };

        if let Some((code, reason)) = close_here {
            tracing::trace!(code, reason, "Closing websocket with reason");

            self.socket
                .send(Message::Close(Some(CloseFrame {
                    code,
                    reason: Cow::Borrowed(reason),
                })))
                .await?;
        } else {
            tracing::trace!("Closing websocket");
        };

        Ok(())
    }

    async fn send<T>(&mut self, value: T) -> Result<()>
    where
        T: Encode<Binary>,
    {
        self.write(value)?;
        self.flush().await?;
        Ok(())
    }

    fn write<T>(&mut self, value: T) -> Result<()>
    where
        T: Encode<Binary>,
    {
        musli_storage::to_writer(&mut self.output, &value)?;
        Ok(())
    }

    fn write_body<T>(&mut self, value: T) -> Result<()>
    where
        T: Encode<Binary>,
    {
        musli_storage::to_writer(&mut self.body, &value)?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        const MAX_CAPACITY: usize = 1048576;
        self.socket
            .send(Message::Binary(self.output.clone()))
            .await?;
        self.output.clear();
        self.output.shrink_to(MAX_CAPACITY);
        Ok(())
    }

    async fn log_backfill(&mut self) -> Result<()> {
        let log = self.bg.log();

        self.send(api::OwnedClientEvent::Broadcast(api::OwnedBroadcast {
            kind: api::OwnedBroadcastKind::LogBackFill(api::OwnedLogBackFill { log }),
        }))
        .await?;

        Ok(())
    }

    async fn handle_request(
        &mut self,
        reader: &mut SliceReader<'_>,
        request: &api::ClientRequestEnvelope<'_>,
    ) -> Result<()> {
        tracing::trace!("Got request: {:?}", request);

        match request.kind {
            api::GetConfig::KIND => {
                let database = self.bg.database().await;

                let missing_ocr = if self.bg.tesseract().is_none() {
                    Some(api::MissingOcr::for_platform())
                } else {
                    None
                };

                let result = api::GetConfigResult {
                    config: self.bg.config().await,
                    installed: database.installed()?,
                    missing_ocr,
                };

                self.write_body(&result)?;
            }
            api::SearchRequest::KIND => {
                let request = musli_storage::decode(reader)?;
                let response = super::handle_search_request(&self.bg, request).await?;
                self.write_body(&response)?;
            }
            api::AnalyzeRequest::KIND => {
                let request = musli_storage::decode(reader)?;
                let response = super::handle_analyze_request(&self.bg, request).await?;
                self.write_body(&response)?;
            }
            api::InstallAllRequest::KIND => {
                self.bg.install(Install::default());
            }
            api::UpdateConfigRequest::KIND => {
                let request: api::UpdateConfigRequest = musli_storage::decode(reader)?;

                if !request.update_indexes.is_empty() {
                    let install = Install {
                        filter: Some(request.update_indexes),
                        force: true,
                    };

                    self.bg.install(install);
                }

                let config = if let Some(config) = request.config {
                    let Some(config) = self.bg.update_config(config).await else {
                        bail!("Failed to update configuration");
                    };

                    Some(config)
                } else {
                    None
                };

                self.write_body(&api::UpdateConfigResponse { config })?;
            }
            api::GetKanji::KIND => {
                let request: api::GetKanji = musli_storage::decode(reader)?;

                let Some(response) = super::handle_kanji(&self.bg, &request.kanji).await? else {
                    bail!("No such kanji");
                };

                self.write_body(&response)?;
            }
            kind => bail!("Unsupported request kind {kind}"),
        }

        Ok(())
    }

    async fn handle_envelope<'de>(
        &mut self,
        reader: &mut SliceReader<'de>,
    ) -> Result<(api::ClientRequestEnvelope<'de>, Result<()>)> {
        let request: api::ClientRequestEnvelope = musli_storage::decode(&mut *reader)?;
        let result = self.handle_request(reader, &request).await;
        Ok((request, result))
    }

    async fn system_event(&mut self, event: system::Event) -> Result<()> {
        match event {
            system::Event::SendClipboardData(clipboard) => match clipboard.mimetype.as_str() {
                "UTF8_STRING" | "text/plain;charset=utf-8" => {
                    let data = filter_data(&clipboard.data);

                    self.send(api::ClientEvent::Broadcast(api::Broadcast {
                        kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                            ty: Some("text/plain"),
                            data: data.as_ref(),
                        }),
                    }))
                    .await?;
                }
                "STRING" | "text/plain" => {
                    let Some(data) = decode_escaped(&clipboard.data[..]) else {
                        tracing::warn!("failed to decode");
                        return Ok(());
                    };

                    let data = filter_data(&data);

                    self.send(api::ClientEvent::Broadcast(api::Broadcast {
                        kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                            ty: Some("text/plain"),
                            data: data.as_ref(),
                        }),
                    }))
                    .await?;
                }
                ty @ "application/json" => {
                    self.send(api::ClientEvent::Broadcast(api::Broadcast {
                        kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                            ty: Some(ty),
                            data: &clipboard.data,
                        }),
                    }))
                    .await?;
                }
                ty => {
                    let Some(tesseract) = self.bg.tesseract() else {
                        return Ok(());
                    };

                    let Some(event) = handle_mimetype_image(tesseract, ty, &clipboard).await?
                    else {
                        return Ok(());
                    };

                    self.send(event).await?;
                }
            },
            system::Event::SendDynamicImage(image) => {
                let Some(tesseract) = self.bg.tesseract() else {
                    return Ok(());
                };

                let Some(event) = handle_image(tesseract, image).await? else {
                    return Ok(());
                };

                self.send(event).await?;
            }
            system::Event::SendText(text) => {
                let data = filter_data(&text);

                self.send(api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                        ty: Some("text/plain"),
                        data: data.as_ref(),
                    }),
                }))
                .await?;
            }
            system::Event::LogEntry(event) => {
                self.send(api::OwnedClientEvent::Broadcast(api::OwnedBroadcast {
                    kind: api::OwnedBroadcastKind::LogEntry(event),
                }))
                .await?;
            }
            system::Event::TaskProgress(task) => {
                self.send(api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::TaskProgress(api::TaskProgress {
                        name: &task.name,
                        value: task.value,
                        total: task.total,
                        step: task.step,
                        steps: task.steps,
                        text: &task.text,
                    }),
                }))
                .await?;
            }
            system::Event::TaskCompleted(task) => {
                self.send(api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::TaskCompleted(api::TaskCompleted {
                        name: &task.name,
                    }),
                }))
                .await?;
            }
            system::Event::Refresh => {
                self.send(api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::Refresh,
                }))
                .await?;
            }
        }

        Ok(())
    }
}

fn decode_escaped(data: &[u8]) -> Option<String> {
    fn h(b: u8) -> Option<u32> {
        let b = match b {
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            b'0'..=b'9' => b - b'0',
            _ => return None,
        };

        Some(b as u32)
    }

    let mut s = String::new();

    let mut it = data.iter().copied();

    while let Some(b) = it.next() {
        match (b, it.clone().next()) {
            (b'\\', Some(b'u')) => {
                it.next();
                let [a, b, c, d] = [it.next()?, it.next()?, it.next()?, it.next()?];
                let [a, b, c, d] = [h(a)?, h(b)?, h(c)?, h(d)?];
                let c = a << 12 | b << 8 | c << 4 | d;
                s.push(char::from_u32(c)?);
            }
            (b'\\', Some(b'\\')) => {
                it.next();
                s.push('\\');
            }
            (c, _) if c.is_ascii() => {
                s.push(c as char);
            }
            _ => {}
        }
    }

    Some(s)
}

async fn handle_mimetype_image(
    tesseract: &Mutex<tesseract::Tesseract>,
    ty: &str,
    c: &system::SendClipboardData,
) -> Result<Option<api::OwnedClientEvent>> {
    use image::ImageFormat;

    let format = match ty {
        "image/png" => ImageFormat::Png,
        "image/tiff" => ImageFormat::Tiff,
        "image/webp" => ImageFormat::WebP,
        "image/jpeg" | "image/jpg" => ImageFormat::Jpeg,
        _ => return Ok(None),
    };

    tracing::trace!(len = c.data.len(), "Decoding image");

    let image = match image::load_from_memory_with_format(&c.data[..], format) {
        Ok(image) => image,
        Err(error) => {
            tracing::warn!(?error, "Failed to load clipboard image");
            return Ok(None);
        }
    };

    handle_image(tesseract, image).await
}

async fn handle_image(
    tesseract: &Mutex<tesseract::Tesseract>,
    image: image::DynamicImage,
) -> Result<Option<api::OwnedClientEvent>> {
    let data = image.as_bytes();
    let width = usize::try_from(image.width())?;
    let height = usize::try_from(image.height())?;
    let bytes_per_pixel = usize::from(image.color().bytes_per_pixel());

    tracing::trace!(len = data.len(), width, height, bytes_per_pixel);

    let text = match tesseract
        .lock()
        .await
        .image_to_text(data, width, height, bytes_per_pixel)
    {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!(?error, "Image recognition failed");
            return Ok(None);
        }
    };

    let trimmed = trim_whitespace(&text[..]);

    tracing::trace!(text = &text[..], ?trimmed, "Recognized");

    Ok(Some(api::OwnedClientEvent::Broadcast(
        api::OwnedBroadcast {
            kind: api::OwnedBroadcastKind::SendClipboardData(api::OwnedSendClipboard {
                ty: Some("text/plain".to_owned()),
                data: filter_data(trimmed.as_ref()).into(),
            }),
        },
    )))
}

fn trim_whitespace(input: &str) -> Cow<'_, str> {
    let mut output = String::new();
    let mut c = input.char_indices();

    'ws: {
        for (n, c) in c.by_ref() {
            if c.is_whitespace() {
                output.push_str(&input[..n]);
                break 'ws;
            }
        }

        return Cow::Borrowed(input);
    };

    for (_, c) in c {
        if !c.is_whitespace() {
            output.push(c);
        }
    }

    Cow::Owned(output)
}

fn filter_data<T>(data: &T) -> Cow<'_, [u8]>
where
    T: ?Sized + AsRef<[u8]>,
{
    let mut data = data.as_ref();

    fn filter(b: u8) -> bool {
        b.is_ascii_control() || b.is_ascii_whitespace()
    }

    while let [a, rest @ ..] = data {
        if filter(*a) {
            data = rest;
            continue;
        }

        break;
    }

    while let [rest @ .., a] = data {
        if filter(*a) {
            data = rest;
            continue;
        }

        break;
    }

    let mut output = Vec::new();
    let mut it = data.iter().enumerate();

    'ws: {
        for (n, b) in it.by_ref() {
            if filter(*b) {
                output.extend_from_slice(&data[..n]);
                break 'ws;
            }
        }

        return Cow::Borrowed(data);
    }

    for (_, b) in it {
        if filter(*b) {
            continue;
        }

        output.push(*b);
    }

    Cow::Owned(output)
}
