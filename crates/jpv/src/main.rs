#![cfg_attr(all(not(feature = "cli"), windows), windows_subsystem = "windows")]

use std::cmp::Reverse;
use std::future::Future;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::pin::pin;
use std::pin::Pin;

use anyhow::{Context, Error, Result};
use async_fuse::Fuse;
use axum::body::{boxed, Body};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Query;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use clap::Parser;
use futures::sink::SinkExt;
use futures::stream::SplitSink;
use futures::stream::StreamExt;
use lib::api;
use lib::database::{Database, EntryResultKey};
use lib::jmdict;
use lib::kanjidic2;
use serde::{Deserialize, Serialize};
use tokio::signal::ctrl_c;
#[cfg(windows)]
use tokio::signal::windows::ctrl_shutdown;
use tokio::sync::broadcast::Sender;
use tokio::sync::Notify;
use tower_http::cors::CorsLayer;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    /// Bind to the given address. Default is `127.0.0.1`.
    #[arg(long)]
    bind: Option<String>,
}

enum System<'a> {
    Future(Pin<Box<dyn Future<Output = Result<()>> + 'a>>),
    Port(u16),
    Busy,
}

#[derive(Clone)]
struct SystemSendClipboardData {
    mimetype: String,
    data: Vec<u8>,
}

#[derive(Clone)]
enum SystemEvent {
    SendClipboardData(SystemSendClipboardData),
}

#[derive(Clone)]
struct SystemEvents(Sender<SystemEvent>);

#[cfg(all(unix, feature = "dbus"))]
#[path = "system/dbus.rs"]
mod system;

#[cfg(all(unix, not(feature = "dbus")))]
#[path = "system/unix.rs"]
mod system;

#[cfg(not(unix))]
#[path = "system/generic.rs"]
mod system;

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = Args::try_parse()?;
    let addr: SocketAddr = args.bind.as_deref().unwrap_or(self::bundle::BIND).parse()?;
    let listener = TcpListener::bind(addr)?;
    let local_addr = listener.local_addr()?;
    let local_port = self::bundle::PORT.unwrap_or(local_addr.port());

    let shutdown = Notify::new();

    let (sender, _) = tokio::sync::broadcast::channel(16);
    let system_events = SystemEvents(sender.clone());

    let mut system = match system::setup(local_port, shutdown.notified(), sender)? {
        System::Future(system) => system,
        System::Port(port) => {
            self::bundle::open(port);
            return Ok(());
        }
        System::Busy => {
            return Ok(());
        }
    };

    let server = match axum::Server::from_tcp(listener) {
        Ok(server) => server,
        Err(error) => {
            return Err(error.into());
        }
    };

    // SAFETY: we know this is only initialized once here exclusively.
    let data = unsafe { self::database::open()? };

    tracing::info!("Loading database...");
    let db = lib::database::Database::new(data).context("loading database")?;
    tracing::info!("Database loaded");

    let cors = CorsLayer::new()
        .allow_origin(format!("http://localhost:{}", local_port).parse::<HeaderValue>()?)
        .allow_origin(format!("http://127.0.0.1:{}", local_port).parse::<HeaderValue>()?)
        .allow_methods([Method::GET]);

    let app = self::bundle::router()
        .layer(Extension(db))
        .layer(Extension(system_events))
        .layer(cors);

    let mut server = pin!(server.serve(app.into_make_service()));
    tracing::info!("Listening on http://{local_addr}");

    let mut ctrl_c = pin!(Fuse::new(ctrl_c()));

    loop {
        tokio::select! {
            result = server.as_mut() => {
                result?;
            }
            result = system.as_mut() => {
                result?;
                tracing::info!("System integration shut down");
                break;
            }
            _ = ctrl_c.as_mut() => {
                tracing::info!("Shutting down...");
                shutdown.notify_one();
            }
        }
    }

    tracing::info!("Bye bye");
    Ok(())
}

type RequestResult<T> = std::result::Result<T, RequestError>;

struct RequestError {
    error: anyhow::Error,
}

impl From<anyhow::Error> for RequestError {
    #[inline]
    fn from(error: anyhow::Error) -> Self {
        Self { error }
    }
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
    Extension(system_events): Extension<SystemEvents>,
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
        event: SystemEvent,
    ) -> Result<()> {
        match event {
            SystemEvent::SendClipboardData(clipboard) => match clipboard.mimetype.as_bytes() {
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
        let mut response = Response::new(boxed(Body::empty()));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response
    }
}

#[cfg(feature = "bundle-database")]
mod database {
    use anyhow::Result;

    static DATABASE: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../database.bin"));

    pub(super) unsafe fn open() -> Result<&'static [u8]> {
        Ok(&DATABASE)
    }
}

#[cfg(not(feature = "bundle-database"))]
mod database {
    use std::fs::File;
    use std::io;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};

    #[cfg(not(unix))]
    static mut DATABASE: musli_zerocopy::AlignedBuf = AlignedBuf::new();

    #[cfg(not(unix))]
    pub(super) unsafe fn open() -> Result<&'static [u8]> {
        use musli_zerocopy::AlignedBuf;
        use std::io::Read;

        let root = PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").context("missing CARGO_MANIFEST_DIR")?,
        );

        let path = manifest_dir.join("..").join("..").join("database.bin");

        tracing::info!("Reading from {}", path.display());

        fn read(path: &Path, output: &mut AlignedBuf) -> io::Result<()> {
            let mut f = File::open(path)?;

            let mut chunk = [0; 1024];

            loop {
                let n = f.read(&mut chunk[..])?;

                if n == 0 {
                    break;
                }

                output.extend_from_slice(&chunk[..n]);
            }

            Ok(())
        }

        read(&path, &mut DATABASE).with_context(|| path.display().to_string())?;
        Ok(DATABASE.as_slice())
    }

    #[cfg(unix)]
    static mut DATABASE: Option<memmap::Mmap> = None;

    #[cfg(unix)]
    pub(super) unsafe fn open() -> Result<&'static [u8]> {
        use core::mem::ManuallyDrop;

        use memmap::MmapOptions;

        let path = match std::env::var_os("CARGO_MANIFEST_DIR") {
            Some(manifest_dir) => {
                let mut path = PathBuf::from(manifest_dir);
                path.push("..");
                path.push("..");
                path.push("database.bin");
                path
            }
            None => PathBuf::from("/usr/share/jpv/database.bin"),
        };

        tracing::info!("Reading from {}", path.display());

        fn read(path: &Path) -> io::Result<&'static [u8]> {
            let f = ManuallyDrop::new(File::open(path)?);

            let mmap = unsafe { MmapOptions::new().map(&f)? };

            unsafe {
                DATABASE = Some(mmap);

                match &DATABASE {
                    Some(mmap) => Ok(&mmap[..]),
                    None => unreachable!(),
                }
            }
        }

        let slice = read(&path).with_context(|| path.display().to_string())?;
        Ok(slice)
    }
}

#[cfg(not(feature = "bundle"))]
mod bundle {
    use axum::routing::get;
    use axum::Router;

    pub(super) static BIND: &'static str = "127.0.0.1:8081";
    pub(super) static PORT: Option<u16> = Some(8080);

    pub(super) fn open(_: u16) {}

    pub(super) fn router() -> Router {
        Router::new()
            .route("/api/analyze", get(super::analyze))
            .route("/api/search", get(super::search))
            .route("/ws", get(super::ws))
    }
}

#[cfg(feature = "bundle")]
mod bundle {
    use std::borrow::Cow;

    use axum::http::{header, StatusCode, Uri};
    use axum::response::{IntoResponse, Response};
    use axum::routing::get;
    use axum::Router;
    use rust_embed::RustEmbed;

    pub(super) static BIND: &'static str = "127.0.0.1:0";
    pub(super) static PORT: Option<u16> = None;

    pub(super) fn open(port: u16) {
        let address = format!("http://localhost:{port}");
        let _ = webbrowser::open(&address);
    }

    pub(super) fn router() -> Router {
        Router::new()
            .route("/", get(index_handler))
            .route("/api/analyze", get(super::analyze))
            .route("/api/search", get(super::search))
            .route("/ws", get(super::ws))
            .route("/*file", get(static_handler))
            .fallback(index_handler)
    }

    async fn index_handler() -> impl IntoResponse {
        StaticFile(Cow::Borrowed("index.html"))
    }

    async fn static_handler(uri: Uri) -> impl IntoResponse {
        StaticFile(Cow::Owned(uri.path().trim_start_matches('/').to_string()))
    }

    #[derive(RustEmbed)]
    #[folder = "../web/dist"]
    struct Asset;

    pub struct StaticFile(Cow<'static, str>);

    impl IntoResponse for StaticFile {
        fn into_response(self) -> Response {
            match Asset::get(self.0.as_ref()) {
                Some(content) => {
                    let mime = mime_guess::from_path(self.0.as_ref()).first_or_octet_stream();
                    ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                }
                None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
            }
        }
    }
}
