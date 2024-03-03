use std::borrow::Cow;
use std::net::SocketAddr;

use anyhow::{anyhow, Result};
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::ConnectInfo;
use axum::response::IntoResponse;
use axum::Extension;
use futures::sink::SinkExt;
use futures::stream::SplitSink;
use futures::stream::StreamExt;
use lib::api::{self, Request};
use rand::prelude::*;
use rand::rngs::SmallRng;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::{Instrument, Level};

use crate::background::{Background, Install};
use crate::system;

pub(super) async fn entry(
    ws: WebSocketUpgrade,
    Extension(bg): Extension<Background>,
    Extension(system_events): Extension<system::SystemEvents>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let receiver = system_events.subscribe();

    ws.on_upgrade(move |socket| async move {
        let span = tracing::span!(Level::INFO, "websocket", ?remote);

        if let Err(error) = run(receiver, socket, &bg).instrument(span).await {
            tracing::error!(?error);
        }
    })
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

async fn log_backfill(
    sink: &mut SplitSink<WebSocket, Message>,
    log: Vec<api::OwnedLogEntry>,
) -> Result<()> {
    let event = api::OwnedClientEvent::Broadcast(api::OwnedBroadcast {
        kind: api::OwnedBroadcastKind::LogBackFill(api::OwnedLogBackFill { log }),
    });

    let json = serde_json::to_vec(&event)?;
    sink.send(Message::Binary(json)).await?;
    Ok(())
}

async fn system_event(
    bg: &Background,
    sink: &mut SplitSink<WebSocket, Message>,
    event: system::Event,
) -> Result<()> {
    match event {
        system::Event::SendClipboardData(clipboard) => match clipboard.mimetype.as_str() {
            "UTF8_STRING" | "text/plain;charset=utf-8" => {
                let data = filter_data(&clipboard.data);

                let event = api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                        ty: Some("text/plain"),
                        data: data.as_ref(),
                    }),
                });

                let json = serde_json::to_vec(&event)?;
                sink.send(Message::Binary(json)).await?;
            }
            "STRING" | "text/plain" => {
                let Some(data) = decode_escaped(&clipboard.data[..]) else {
                    tracing::warn!("failed to decode");
                    return Ok(());
                };

                let data = filter_data(&data);

                let event = api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                        ty: Some("text/plain"),
                        data: data.as_ref(),
                    }),
                });

                let json = serde_json::to_vec(&event)?;
                sink.send(Message::Binary(json)).await?;
            }
            ty @ "application/json" => {
                let event = api::ClientEvent::Broadcast(api::Broadcast {
                    kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                        ty: Some(ty),
                        data: &clipboard.data,
                    }),
                });

                let json = serde_json::to_vec(&event)?;
                sink.send(Message::Binary(json)).await?;
            }
            ty => {
                let Some(tesseract) = bg.tesseract() else {
                    return Ok(());
                };

                let Some(event) = handle_mimetype_image(tesseract, ty, &clipboard).await? else {
                    return Ok(());
                };

                let json = serde_json::to_vec(&event)?;
                sink.send(Message::Binary(json)).await?;
            }
        },
        system::Event::SendDynamicImage(image) => {
            let Some(tesseract) = bg.tesseract() else {
                return Ok(());
            };

            let Some(event) = handle_image(tesseract, image).await? else {
                return Ok(());
            };

            let json = serde_json::to_vec(&event)?;
            sink.send(Message::Binary(json)).await?;
        }
        system::Event::SendText(text) => {
            let data = filter_data(&text);

            let event = api::ClientEvent::Broadcast(api::Broadcast {
                kind: api::BroadcastKind::SendClipboardData(api::SendClipboard {
                    ty: Some("text/plain"),
                    data: data.as_ref(),
                }),
            });

            let json = serde_json::to_vec(&event)?;
            sink.send(Message::Binary(json)).await?;
        }
        system::Event::LogEntry(event) => {
            let event = api::OwnedClientEvent::Broadcast(api::OwnedBroadcast {
                kind: api::OwnedBroadcastKind::LogEntry(event),
            });

            let json = serde_json::to_vec(&event)?;
            sink.send(Message::Binary(json)).await?;
        }
        system::Event::TaskProgress(task) => {
            let event = api::ClientEvent::Broadcast(api::Broadcast {
                kind: api::BroadcastKind::TaskProgress(api::TaskProgress {
                    name: &task.name,
                    value: task.value,
                    total: task.total,
                    step: task.step,
                    steps: task.steps,
                    text: &task.text,
                }),
            });

            let json = serde_json::to_vec(&event)?;
            sink.send(Message::Binary(json)).await?;
        }
        system::Event::TaskCompleted(task) => {
            let event = api::ClientEvent::Broadcast(api::Broadcast {
                kind: api::BroadcastKind::TaskCompleted(api::TaskCompleted { name: &task.name }),
            });

            let json = serde_json::to_vec(&event)?;

            sink.send(Message::Binary(json)).await?;
        }
        system::Event::Refresh => {
            let event = api::ClientEvent::Broadcast(api::Broadcast {
                kind: api::BroadcastKind::Refresh,
            });

            let json = serde_json::to_vec(&event)?;
            sink.send(Message::Binary(json)).await?;
        }
    }

    Ok(())
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

async fn run(
    mut system_events: Receiver<system::Event>,
    socket: WebSocket,
    bg: &Background,
) -> Result<()> {
    tracing::trace!("Accepted");

    const CLOSE_NORMAL: u16 = 1000;
    const CLOSE_PROTOCOL_ERROR: u16 = 1002;
    const CLOSE_TIMEOUT: Duration = Duration::from_secs(30);
    const PING_TIMEOUT: Duration = Duration::from_secs(10);

    let (mut sender, mut receiver) = socket.split();

    let mut last_ping = None::<u32>;
    let mut rng = SmallRng::seed_from_u64(0x404241112);
    let mut close_interval = tokio::time::interval(CLOSE_TIMEOUT);
    close_interval.reset();

    let mut ping_interval = tokio::time::interval(PING_TIMEOUT);
    ping_interval.reset();

    let log = bg.log();

    log_backfill(&mut sender, log).await?;

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
                sender.send(Message::Ping(data)).await?;
                ping_interval.reset();
            }
            event = system_events.recv() => {
                let Ok(event) = event else {
                    break Some((CLOSE_NORMAL, "system shutting down"));
                };

                if let Err(error) = system_event(bg, &mut sender, event).await {
                    tracing::error!(?error, "Failed to process system event");
                };
            }
            message = receiver.next() => {
                let Some(message) = message else {
                    break None;
                };

                match message? {
                    Message::Text(_) => break Some((CLOSE_PROTOCOL_ERROR, "unsupported message")),
                    Message::Binary(bytes) => {
                        let request = match serde_json::from_slice::<api::ClientRequestEnvelope>(&bytes[..]) {
                            Ok(event) => event,
                            Err(error) => {
                                tracing::warn!(?error, "Failed to decode message");
                                continue;
                            }
                        };

                        tracing::trace!("Got request: {:?}", request);

                        let result: Result<serde_json::Value> = match request.kind.as_str() {
                            api::SearchRequest::KIND => {
                                let request = serde_json::from_value(request.body)?;
                                let response = super::handle_search_request(bg, request)?;
                                Ok(serde_json::to_value(&response)?)
                            },
                            api::AnalyzeRequest::KIND => {
                                let request = serde_json::from_value(request.body)?;
                                let response = super::handle_analyze_request(bg, request)?;
                                Ok(serde_json::to_value(&response)?)
                            },
                            api::InstallAllRequest::KIND => {
                                bg.install(Install::default());
                                Ok(serde_json::Value::Null)
                            }
                            api::GetConfig::KIND => {
                                let database = bg.database();

                                let missing_ocr = if bg.tesseract().is_none() {
                                    Some(api::MissingOcr::for_platform())
                                } else {
                                    None
                                };

                                let result = api::GetConfigResult {
                                    config: bg.config(),
                                    installed: database.installed()?,
                                    missing_ocr,
                                };

                                Ok(serde_json::to_value(&result)?)
                            }
                            api::UpdateConfigRequest::KIND => {
                                let request: api::UpdateConfigRequest = serde_json::from_value(request.body)?;

                                'out: {
                                    if !request.update_indexes.is_empty() {
                                        let mut install = Install::default();
                                        install.filter = Some(request.update_indexes);
                                        install.force = true;
                                        bg.install(install);
                                    }

                                    let config = if let Some(config) = request.config {
                                        let Some(config) = bg.update_config(config).await else {
                                            break 'out Err(anyhow!("Failed to update configuration"));
                                        };

                                        Some(config)
                                    } else {
                                        None
                                    };

                                    Ok(serde_json::to_value(&api::UpdateConfigResponse {
                                        config
                                    })?)
                                }
                            }
                            api::GetKanji::KIND => {
                                let request: api::GetKanji = serde_json::from_value(request.body)?;
                                let response = super::handle_kanji(bg, &request.kanji)?;
                                Ok(serde_json::to_value(&response)?)
                            }
                            _ => {
                                Err(anyhow!("Unsupported request"))
                            }
                        };

                        let (body, error) = match result {
                            Ok(value) => (value, None),
                            Err(error) => (serde_json::Value::Null, Some(error.to_string())),
                        };

                        let payload = serde_json::to_vec(&api::OwnedClientEvent::ClientResponse(api::ClientResponseEnvelope {
                            index: request.index,
                            serial: request.serial,
                            body,
                            error,
                        }))?;

                        sender.send(Message::Binary(payload)).await?;
                    },
                    Message::Ping(payload) => {
                        sender.send(Message::Pong(payload)).await?;
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

        sender
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
