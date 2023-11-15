#[cfg(feature = "bundle")]
#[path = "bundle.rs"]
mod r#impl;

#[cfg(not(feature = "bundle"))]
#[path = "api.rs"]
mod r#impl;

pub(crate) use self::r#impl::{BIND, PORT};

use std::cmp::Reverse;
use std::fmt;
use std::future::Future;
use std::net::TcpListener;

use anyhow::{Error, Result};
use axum::body::{boxed, Body};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use futures::sink::SinkExt;
use futures::stream::SplitSink;
use futures::stream::StreamExt;
use lib::api;
use lib::database::{Database, EntryResultKey};
use lib::jmdict;
use lib::kanjidic2;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::system;

pub(crate) fn setup(
    local_port: u16,
    listener: TcpListener,
    db: Database<'static>,
    system_events: system::SystemEvents,
) -> Result<impl Future<Output = Result<()>>> {
    let server = match axum::Server::from_tcp(listener) {
        Ok(server) => server,
        Err(error) => {
            return Err(error.into());
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(format!("http://localhost:{}", local_port).parse::<HeaderValue>()?)
        .allow_origin(format!("http://127.0.0.1:{}", local_port).parse::<HeaderValue>()?)
        .allow_methods([Method::GET]);

    let app = self::r#impl::router()
        .layer(Extension(db))
        .layer(Extension(system_events))
        .layer(cors);

    let service = server.serve(app.into_make_service());

    Ok(async move {
        service.await?;
        Ok(())
    })
}

fn common_routes(router: Router) -> Router {
    router
        .route("/api/analyze", get(analyze))
        .route("/api/search", get(search))
        .route("/api/entry/:sequence", get(entry))
        .route("/api/kanji/:literal", get(kanji))
        .route("/ws", get(ws))
}

type RequestResult<T> = std::result::Result<T, RequestError>;

struct RequestError {
    error: anyhow::Error,
    status: Option<StatusCode>,
}

impl RequestError {
    fn not_found<M>(msg: M) -> Self
    where
        M: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self {
            error: anyhow::Error::msg(msg),
            status: Some(StatusCode::NOT_FOUND),
        }
    }
}

impl From<anyhow::Error> for RequestError {
    #[inline]
    fn from(error: anyhow::Error) -> Self {
        Self {
            error,
            status: None,
        }
    }
}

#[derive(Deserialize)]
struct EntryQuery {
    #[serde(default)]
    serial: Option<u32>,
}

#[derive(Serialize)]
struct EntryResponse {
    entry: jmdict::Entry<'static>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serial: Option<u32>,
}

#[derive(Deserialize)]
struct KanjiQuery {
    #[serde(default)]
    serial: Option<u32>,
}

#[derive(Serialize)]
struct KanjiResponse {
    entry: kanjidic2::Character<'static>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serial: Option<u32>,
}

#[derive(Deserialize)]
struct SearchRequest {
    q: Option<String>,
    #[serde(default)]
    serial: Option<u32>,
}

#[derive(Serialize)]
struct SearchEntry {
    key: EntryResultKey,
    entry: jmdict::Entry<'static>,
}

#[derive(Serialize)]
struct SearchResponse {
    entries: Vec<SearchEntry>,
    characters: Vec<kanjidic2::Character<'static>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serial: Option<u32>,
}

async fn entry(
    Path(sequence): Path<u32>,
    Query(query): Query<EntryQuery>,
    Extension(db): Extension<Database<'static>>,
) -> RequestResult<Json<EntryResponse>> {
    let Some(entry) = db.sequence_to_entry(sequence)? else {
        return Err(RequestError::not_found(format!(
            "Missing entry by id `{}`",
            sequence
        )));
    };

    Ok(Json(EntryResponse {
        entry,
        serial: query.serial,
    }))
}

async fn kanji(
    Path(literal): Path<String>,
    Query(query): Query<KanjiQuery>,
    Extension(db): Extension<Database<'static>>,
) -> RequestResult<Json<KanjiResponse>> {
    let Some(entry) = db.literal_to_kanji(&literal)? else {
        return Err(RequestError::not_found(format!(
            "Missing kanji by literal `{literal}`",
        )));
    };

    Ok(Json(KanjiResponse {
        entry,
        serial: query.serial,
    }))
}

async fn search(
    Query(request): Query<SearchRequest>,
    Extension(db): Extension<Database<'static>>,
) -> RequestResult<Json<SearchResponse>> {
    let Some(q) = request.q.as_deref() else {
        return Err(Error::msg("Missing `q`").into());
    };

    let mut entries = Vec::new();

    let search = db.search(q)?;

    for (key, entry) in search.entries {
        entries.push(SearchEntry { key, entry });
    }

    Ok(Json(SearchResponse {
        entries,
        characters: search.characters,
        serial: request.serial,
    }))
}

async fn ws(
    ws: WebSocketUpgrade,
    Extension(system_events): Extension<system::SystemEvents>,
) -> impl IntoResponse {
    let mut system_events = system_events.0.subscribe();

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

    async fn handle_event(
        sink: &mut SplitSink<WebSocket, Message>,
        event: system::Event,
    ) -> Result<()> {
        match event {
            system::Event::SendClipboardData(clipboard) => match clipboard.mimetype.as_bytes() {
                b"UTF8_STRING" | b"text/plain;charset=utf-8" => {
                    let event = api::ClientEvent::SendClipboardData(api::ClientSendClipboardData {
                        data: String::from_utf8_lossy(&clipboard.data).into_owned(),
                    });

                    let json = serde_json::to_vec(&event)?;
                    sink.send(Message::Binary(json)).await?;
                }
                b"STRING" | b"text/plain" => {
                    let Some(data) = decode_escaped(&clipboard.data[..]) else {
                        tracing::warn!("failed to decode");
                        return Ok(());
                    };

                    let event =
                        api::ClientEvent::SendClipboardData(api::ClientSendClipboardData { data });

                    let json = serde_json::to_vec(&event)?;
                    sink.send(Message::Binary(json)).await?;
                }
                _ => {}
            },
        }

        Ok(())
    }

    ws.on_upgrade(move |socket| async move {
        let (mut sender, mut receiver) = socket.split();

        loop {
            tokio::select! {
                event = system_events.recv() => {
                    let Ok(event) = event else {
                        break;
                    };

                    if let Err(error) = handle_event(&mut sender, event).await {
                        tracing::error!("{}", error);
                        break;
                    }
                }
                message = receiver.next() => {
                    let Some(message) = message else {
                        break;
                    };

                    tracing::info!(?message);
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct AnalyzeRequest {
    q: String,
    start: usize,
    #[serde(default)]
    serial: Option<u32>,
}

#[derive(Serialize)]
struct AnalyzeEntry {
    key: jmdict::EntryKey,
    string: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    data: Vec<AnalyzeEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serial: Option<u32>,
}

async fn analyze(
    Query(request): Query<AnalyzeRequest>,
    Extension(db): Extension<Database<'static>>,
) -> RequestResult<Json<AnalyzeResponse>> {
    let mut entries = Vec::new();

    for (key, string) in db.analyze(&request.q, request.start)? {
        entries.push(AnalyzeEntry { key, string });
    }

    entries
        .sort_by(|a, b| (Reverse(a.string.len()), &a.key).cmp(&(Reverse(b.string.len()), &b.key)));
    Ok(Json(AnalyzeResponse {
        data: entries,
        serial: request.serial,
    }))
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.error);
        let bytes = format!("{}", self.error).into_bytes();
        let mut response = Response::new(boxed(Body::from(bytes)));
        *response.status_mut() = self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        response
    }
}