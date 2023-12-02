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

use anyhow::{Error, Result};
use axum::body::{boxed, Body};
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Extension, Json, Router};
use lib::api;
use lib::database::Database;
use tower_http::cors::CorsLayer;

use crate::system;

pub(crate) fn setup(
    local_port: u16,
    listener: TcpListener,
    db: Database,
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

    let service = server.serve(app.into_make_service_with_connect_info::<SocketAddr>());

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
    Query(query): Query<api::EntryQuery>,
    Extension(db): Extension<Database>,
) -> RequestResult<Json<api::OwnedEntryResponse>> {
    let Some(entry) = db.sequence_to_entry(sequence)? else {
        return Err(RequestError::not_found(format!(
            "Missing entry by id `{}`",
            sequence
        )));
    };

    Ok(Json(api::OwnedEntryResponse {
        entry: lib::to_owned(entry),
        serial: query.serial,
    }))
}

async fn kanji(
    Path(literal): Path<String>,
    Query(query): Query<api::KanjiQuery>,
    Extension(db): Extension<Database>,
) -> RequestResult<Json<api::OwnedKanjiResponse>> {
    let Some(entry) = db.literal_to_kanji(&literal)? else {
        return Err(RequestError::not_found(format!(
            "Missing kanji by literal `{literal}`",
        )));
    };

    Ok(Json(api::OwnedKanjiResponse {
        entry: lib::to_owned(entry),
        serial: query.serial,
    }))
}

async fn search(
    Query(request): Query<api::OwnedSearchRequest>,
    Extension(db): Extension<Database>,
) -> RequestResult<Json<api::OwnedSearchResponse>> {
    let Some(q) = request.q.as_deref() else {
        return Err(Error::msg("Missing `q`").into());
    };

    let search = db.search(q)?;

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

    Ok(Json(api::OwnedSearchResponse {
        phrases,
        names,
        characters: lib::to_owned(search.characters),
        serial: request.serial,
    }))
}

async fn analyze(
    Query(request): Query<api::OwnedAnalyzeRequest>,
    Extension(db): Extension<Database>,
) -> RequestResult<Json<api::OwnedAnalyzeResponse>> {
    let mut data = Vec::new();

    for (key, string) in db.analyze(&request.q, request.start)? {
        data.push(api::OwnedAnalyzeEntry {
            key,
            string: string.to_owned(),
        });
    }

    data.sort_by(|a, b| (Reverse(a.string.len()), &a.key).cmp(&(Reverse(b.string.len()), &b.key)));

    Ok(Json(api::OwnedAnalyzeResponse {
        data,
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
