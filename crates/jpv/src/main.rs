#![cfg_attr(all(not(feature = "cli"), windows), windows_subsystem = "windows")]

use std::net::SocketAddr;

use anyhow::{Context, Error, Result};
use axum::body::{boxed, Body};
use axum::extract::Query;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use clap::Parser;
use lib::database::{Database, EntryResultKey};
use lib::elements::{Entry, EntryKey};
use serde::{Deserialize, Serialize};
use tokio::signal::ctrl_c;
use tokio::signal::windows::ctrl_shutdown;
use tower_http::cors::CorsLayer;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
struct Args {
    /// Bind to the given address. Default is `127.0.0.1:8081`.
    #[arg(long)]
    bind: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = EnvFilter::builder().from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .finish()
        .try_init()?;

    let args = Args::try_parse()?;
    let bind: SocketAddr = args.bind.as_deref().unwrap_or(self::bundle::BIND).parse()?;

    // SAFETY: we know this is only initialized once here exclusively.
    let data = unsafe { self::bundle::database()? };

    tracing::info!("Loading database...");
    let db = lib::database::Database::new(data).context("loading database")?;
    tracing::info!("Database loaded");

    let cors = CorsLayer::new()
        .allow_origin(format!("http://localhost:8080").parse::<HeaderValue>().unwrap())
        .allow_origin(format!("http://127.0.0.1:8080").parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET]);

    let app = self::bundle::router().layer(Extension(db)).layer(cors);

    tracing::info!("Listening on {bind}");

    let server = axum::Server::bind(&bind).serve(app.into_make_service());
    self::bundle::open();

    let ctrl_c = ctrl_c();
    let mut shutdown = ctrl_shutdown()?;

    tokio::select! {
        result = server => {
            result?;
        }
        _ = ctrl_c => {
            tracing::info!("Shutting down...");
        }
        _ = shutdown.recv() => {
            tracing::info!("Shutting down...");
        }
    }

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
}

#[derive(Serialize)]
struct SearchEntry {
    key: EntryResultKey,
    entry: Entry<'static>,
}

#[derive(Serialize)]
struct SearchResponse {
    entries: Vec<SearchEntry>,
}

async fn search(
    Query(request): Query<SearchRequest>,
    Extension(db): Extension<Database<'static>>,
) -> RequestResult<Json<SearchResponse>> {
    let Some(q) = request.q.as_deref() else {
        return Err(Error::msg("Missing `q`").into());
    };

    let mut entries = Vec::new();

    for (key, entry) in db.search(q)? {
        entries.push(SearchEntry { key, entry });
    }

    entries.sort_by(|a, b| a.key.key.cmp(&b.key.key));
    Ok(Json(SearchResponse { entries }))
}

#[derive(Deserialize)]
struct AnalyzeRequest {
    q: String,
    start: usize,
}

#[derive(Serialize)]
struct AnalyzeEntry {
    key: EntryKey,
    string: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    data: Vec<AnalyzeEntry>,
}

async fn analyze(
    Query(request): Query<AnalyzeRequest>,
    Extension(db): Extension<Database<'static>>,
) -> RequestResult<Json<AnalyzeResponse>> {
    let mut entries = Vec::new();

    for (key, string) in db.analyze(&request.q, request.start) {
        entries.push(AnalyzeEntry { key, string });
    }

    Ok(Json(AnalyzeResponse { data: entries }))
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.error);
        let mut response = Response::new(boxed(Body::empty()));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response
    }
}

#[cfg(not(feature = "bundle"))]
mod bundle {
    use std::path::PathBuf;

    use anyhow::{Result, Context};
    use axum::Router;
    use axum::routing::get;

    pub(super) static BIND: &'static str = "127.0.0.1:8081";

    static mut DATABASE: Vec<u8> = Vec::new();

    pub(super) unsafe fn database() -> Result<&'static [u8]> {
        let root = PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").context("missing CARGO_MANIFEST_DIR")?,
        );

        let path = root.join("..").join("..").join("database.bin");

        tracing::info!("Reading from {}", path.display());

        let data =
            std::fs::read(&path).with_context(|| path.display().to_string())?;

        DATABASE = data;
        Ok(DATABASE.as_ref())
    }

    pub(super) fn open() {
    }

    pub(super) fn router() -> Router {
        Router::new()
            .route("/analyze", get(super::analyze))
            .route("/search", get(super::search))
    }
}

#[cfg(feature = "bundle")]
mod bundle {
    use std::borrow::Cow;

    use anyhow::Result;
    use axum::Router;
    use axum::http::{header, StatusCode, Uri};
    use axum::response::{IntoResponse, Response};
    use axum::routing::get;
    use rust_embed::RustEmbed;

    pub(super) static BIND: &'static str = "127.0.0.1:8080";

    static DATABASE: &'static [u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../database.bin"));

    pub(super) unsafe fn database() -> Result<&'static [u8]> {
        Ok(&DATABASE)
    }

    pub(super) fn open() {
        let _ = webbrowser::open("http://localhost:8080");
    }

    pub(super) fn router() -> Router {
        Router::new()
            .route("/", get(index_handler))
            .route("/api/analyze", get(super::analyze))
            .route("/api/search", get(super::search))
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
