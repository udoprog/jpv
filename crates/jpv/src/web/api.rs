use axum::routing::get;
use axum::Router;

pub(crate) static BIND: &'static str = "127.0.0.1:8081";
pub(crate) static PORT: Option<u16> = Some(8080);

pub(crate) fn open(_: u16) {}

pub(crate) fn router() -> Router {
    Router::new()
        .route("/api/analyze", get(super::analyze))
        .route("/api/search", get(super::search))
        .route("/ws", get(super::ws))
}
