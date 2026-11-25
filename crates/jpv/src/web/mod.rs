#[cfg(feature = "bundle")]
#[path = "bundle.rs"]
mod r#impl;

#[cfg(not(feature = "bundle"))]
#[path = "api.rs"]
mod r#impl;

mod files;
mod origin;
mod ws;

mod json;

use self::json::Json;
pub(crate) use self::r#impl::{BIND, PORT};

use std::cmp::Reverse;
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware, Extension, Router};
use lib::api;
use lib::config::Config;
use musli::Encode;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};

use self::origin::Origins;

use crate::background::{Background, Install};
use crate::system;

pub(crate) fn setup(
    listener: TcpListener,
    background: Background,
    system_events: system::SystemEvents,
) -> Result<impl Future<Output = Result<()>>> {
    let origins = Origins::new([listener.local_addr()?.port()].into_iter().chain(PORT));

    let app = self::r#impl::router()
        .layer(middleware::from_fn_with_state(
            origins.clone(),
            origin::guard,
        ))
        .layer(Extension(origins))
        .layer(Extension(background))
        .layer(Extension(system_events));

    let service = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    );

    Ok(async move {
        service.await?;
        Ok(())
    })
}

fn common_routes(router: Router) -> Router {
    router
        .route("/api/version", get(version).layer(public()))
        .route("/api/config", get(config).post(update_config))
        .route("/api/rebuild", post(rebuild))
        .route("/api/analyze", get(analyze).layer(public()))
        .route("/api/search", get(search).layer(public()))
        .route("/api/entry/{sequence}", get(entry).layer(public()))
        .route("/api/kanji/{literal}", get(kanji).layer(public()))
        .route("/ws", get(ws::entry))
}

/// Lets any site read a read-only endpoint, such as the browser extension
/// does from the pages it runs in.
fn public() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::GET, Method::HEAD])
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
    let db = bg.database().await;

    let Some(entry) = db.sequence_to_entry(sequence)? else {
        return Err(RequestError::not_found(format!(
            "Missing entry by id `{sequence}`"
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
    let Some(kanji) = handle_kanji(&bg, &literal).await? else {
        return Err(RequestError::not_found(format!(
            "Missing kanji by literal `{literal}`",
        )));
    };

    Ok(Json(kanji))
}

async fn handle_kanji(bg: &Background, literal: &str) -> Result<Option<api::OwnedKanjiResponse>> {
    let db = bg.database().await;

    let Some(entry) = db.literal_to_kanji(literal)? else {
        return Ok(None);
    };

    let radicals = db.literal_to_radicals(literal)?;

    Ok(Some(api::OwnedKanjiResponse {
        kanji: lib::to_owned(entry),
        radicals: radicals
            .map(|e| lib::to_owned(e.radicals))
            .unwrap_or_default(),
    }))
}

async fn search(
    Query(request): Query<api::SearchRequest>,
    Extension(bg): Extension<Background>,
) -> RequestResult<Json<api::OwnedSearchResponse>> {
    Ok(Json(handle_search_request(&bg, request).await?))
}

async fn handle_search_request(
    bg: &Background,
    request: api::SearchRequest,
) -> Result<api::OwnedSearchResponse> {
    let db = bg.database().await;
    let search = db.search(&request.q)?;

    let mut phrases = Vec::new();
    let mut names = Vec::new();

    let lang = request.lang.as_deref().unwrap_or("eng");

    for (key, mut phrase) in search.phrases {
        phrase.retain_lang(lang);

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

#[derive(Encode)]
#[musli(mode = Text, name_all = "kebab-case")]
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
    Ok(Json(bg.config().await))
}

/// Read the current service configuration.
async fn update_config(
    Extension(bg): Extension<Background>,
    axum::Json(config): axum::Json<Config>,
) -> RequestResult<Json<api::Empty>> {
    if bg.update_config_from_web(config).await.is_none() {
        return Err(RequestError::internal("Failed to update configuration"));
    }

    Ok(Json(api::Empty))
}

/// Trigger a rebuild of the database.
async fn rebuild(Extension(bg): Extension<Background>) -> RequestResult<Json<api::Empty>> {
    bg.install(Install::default());
    Ok(Json(api::Empty))
}

/// Perform text analysis.
async fn analyze(
    Query(request): Query<api::AnalyzeRequest>,
    Extension(bg): Extension<Background>,
) -> RequestResult<Json<api::OwnedAnalyzeResponse>> {
    Ok(Json(handle_analyze_request(&bg, request).await?))
}

async fn handle_analyze_request(
    bg: &Background,
    request: api::AnalyzeRequest,
) -> Result<api::OwnedAnalyzeResponse> {
    let mut data = Vec::new();

    let db = bg.database().await;

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
        let mut response = Response::new(Body::from(bytes));
        *response.status_mut() = self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        response
    }
}
