use axum::Router;

pub(crate) static BIND: &str = "127.0.0.1:44714";
pub(crate) static PORT: Option<u16> = Some(8080);

pub(crate) fn router() -> Router {
    super::common_routes(Router::new())
}
