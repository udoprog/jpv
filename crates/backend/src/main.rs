use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use axum::body::{boxed, Body};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use clap::Parser;
use lib::database::{Database, Id};
use lib::elements::Entry;
use serde::{Deserialize, Serialize};
use tokio::signal::ctrl_c;
use tokio::signal::windows::ctrl_shutdown;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

static mut DATABASE: Vec<u8> = Vec::new();

#[derive(Parser)]
struct Args {
    /// Bind to the given address. Default is `0.0.0.0:8081`.
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
    let bind = args.bind.as_deref().unwrap_or("0.0.0.0:8081").parse()?;

    let root = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").context("missing CARGO_MANIFEST_DIR")?,
    );
    let database_path = root.join("..").join("..").join("database.bin");
    let data =
        std::fs::read(&database_path).with_context(|| database_path.display().to_string())?;

    // SAFETY: we know this is only initialized once here exclusively.
    let data = unsafe {
        DATABASE = data;
        DATABASE.as_ref()
    };

    tracing::info!("Loading database...");
    let db = lib::database::Database::new(data).context("loading database")?;
    tracing::info!("Database loaded");

    let app = Router::new()
        .route("/search", get(search))
        .layer(Extension(db));

    tracing::info!("Listening on {bind}");

    let server = axum::Server::bind(&bind).serve(app.into_make_service());

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

#[derive(Debug, Deserialize)]
struct SearchRequest {
    q: Option<String>,
}

#[derive(Serialize)]
struct SearchEntry {
    id: Id,
    data: Entry<'static>,
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

    for id in db.lookup(q) {
        let data = db.get(id)?;
        entries.push(SearchEntry { id, data });
    }

    Ok(Json(SearchResponse { entries }))
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        tracing::error!("{}", self.error);
        let mut response = Response::new(boxed(Body::empty()));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response
    }
}
