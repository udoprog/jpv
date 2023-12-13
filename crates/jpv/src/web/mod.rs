#[cfg(feature = "bundle")]
#[path = "bundle.rs"]
mod r#impl;

#[cfg(not(feature = "bundle"))]
#[path = "api.rs"]
mod r#impl;

mod ws;

pub(crate) use self::r#impl::{BIND, PORT};

use std::cmp::Reverse;
use std::fmt;
use std::future::Future;
use std::net::{SocketAddr, TcpListener};

use anyhow::Result;
use axum::body::{boxed, Body};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use lib::api;
use lib::config::Config;
use serde::Serialize;
use tower_http::cors::{AllowMethods, AllowOrigin, CorsLayer};

use crate::background::Background;
use crate::system;

pub(crate) fn setup(
    listener: TcpListener,
    background: Background,
    system_events: system::SystemEvents,
) -> Result<impl Future<Output = Result<()>>> {
    let server = match axum::Server::from_tcp(listener) {
        Ok(server) => server,
        Err(error) => {
            return Err(error.into());
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any());

    let app = self::r#impl::router()
        .layer(Extension(background))
        .layer(Extension(system_events))
        .layer(cors);

    let service = server.serve(app.into_make_service_with_connect_info::<SocketAddr>());

    Ok(async move {
        service.await?;
        Ok(())
    })
}

fn common_routes(router: Router) -> Router {
    router
        .route("/api/version", get(version))
        .route("/api/config", get(config).post(update_config))
        .route("/api/rebuild", post(rebuild))
        .route("/api/analyze", get(analyze))
        .route("/api/search", get(search))
        .route("/api/entry/:sequence", get(entry))
        .route("/api/kanji/:literal", get(kanji))
        .route("/ws", get(ws::entry))
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

    fn internal<M>(msg: M) -> Self
    where
        M: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self {
            error: anyhow::Error::msg(msg),
            status: Some(StatusCode::INTERNAL_SERVER_ERROR),
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

async fn entry(
    Path(sequence): Path<u32>,
    Extension(bg): Extension<Background>,
) -> RequestResult<Json<api::OwnedEntryResponse>> {
    let db = bg.database();

    let Some(entry) = db.sequence_to_entry(sequence)? else {
        return Err(RequestError::not_found(format!(
            "Missing entry by id `{}`",
            sequence
        )));
    };

    Ok(Json(api::OwnedEntryResponse {
        entry: lib::to_owned(entry),
    }))
}

async fn kanji(
    Path(literal): Path<String>,
    Extension(bg): Extension<Background>,
) -> RequestResult<Json<api::OwnedKanjiResponse>> {
    let db = bg.database();

    let Some(entry) = db.literal_to_kanji(&literal)? else {
        return Err(RequestError::not_found(format!(
            "Missing kanji by literal `{literal}`",
        )));
    };

    Ok(Json(api::OwnedKanjiResponse {
        entry: lib::to_owned(entry),
    }))
}

async fn search(
    Query(request): Query<api::SearchRequest>,
    Extension(bg): Extension<Background>,
) -> RequestResult<Json<api::OwnedSearchResponse>> {
    Ok(Json(handle_search_request(&bg, request)?))
}

fn handle_search_request(
    bg: &Background,
    request: api::SearchRequest,
) -> Result<api::OwnedSearchResponse> {
    let db = bg.database();
    let search = db.search(&request.q)?;

    let mut phrases = Vec::new();
    let mut names = Vec::new();

    for (key, phrase) in search.phrases {
        phrases.push(api::OwnedSearchPhrase {
            key,
            phrase: lib::to_owned(phrase),
        });
    }

    for (key, name) in search.names {
        names.push(api::OwnedSearchName {
            key,
            name: lib::to_owned(name),
        });
    }

    Ok(api::OwnedSearchResponse {
        phrases,
        names,
        characters: lib::to_owned(search.characters),
    })
}

#[derive(Serialize)]
struct VersionResponse {
    version: &'static str,
}

/// Get the current service version.
async fn version() -> RequestResult<Json<VersionResponse>> {
    Ok(Json(VersionResponse {
        version: crate::VERSION,
    }))
}

/// Read the current service configuration.
async fn config(Extension(bg): Extension<Background>) -> RequestResult<Json<Config>> {
    Ok(Json(bg.config()))
}

/// Read the current service configuration.
async fn update_config(
    Extension(bg): Extension<Background>,
    Json(config): Json<Config>,
) -> RequestResult<Json<api::Empty>> {
    if !bg.update_config(config).await {
        return Err(RequestError::internal("Failed to update configuration"));
    }

    Ok(Json(api::Empty))
}

/// Trigger a rebuild of the database.
async fn rebuild(Extension(bg): Extension<Background>) -> RequestResult<Json<api::Empty>> {
    bg.rebuild().await;
    Ok(Json(api::Empty))
}

/// Perform text analysis.
async fn analyze(
    Query(request): Query<api::AnalyzeRequest>,
    Extension(bg): Extension<Background>,
) -> RequestResult<Json<api::OwnedAnalyzeResponse>> {
    Ok(Json(handle_analyze_request(&bg, request)?))
}

fn handle_analyze_request(
    bg: &Background,
    request: api::AnalyzeRequest,
) -> Result<api::OwnedAnalyzeResponse> {
    let mut data = Vec::new();

    let db = bg.database();

    for (key, string) in db.analyze(&request.q, request.start)? {
        data.push(api::OwnedAnalyzeEntry {
            key,
            string: string.to_owned(),
        });
    }

    data.sort_by(|a, b| (Reverse(a.string.len()), &a.key).cmp(&(Reverse(b.string.len()), &b.key)));

    Ok(api::OwnedAnalyzeResponse { data })
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
