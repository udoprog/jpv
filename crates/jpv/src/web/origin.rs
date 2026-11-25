//! Checks that a request comes from the service's own pages.
//!
//! Any page the user has open in a browser can send requests to the service,
//! so requests which change something (the WebSocket and every non-GET
//! endpoint) are only accepted when:
//!
//! * The `Host` header names the service by a loopback name or an IP address,
//!   on one of the ports it is served from. A DNS-rebinding page reaches the
//!   service under its own domain name, which this refuses.
//! * The `Origin` header, if present, is a page served by the service: a
//!   loopback name (`localhost` and `127.0.0.1` are interchangeable) or the
//!   same host as `Host`, on one of the ports it is served from.
//!
//! Requests without an `Origin` come from non-browser clients, which aren't
//! subject to the browser's cross-origin rules to begin with.

use std::net::IpAddr;

use axum::extract::{Request, State};
use axum::http::uri::{Authority, Scheme};
use axum::http::{header, HeaderMap, Method, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

/// The origins the service accepts requests which change something from.
#[derive(Debug, Clone)]
pub(super) struct Origins {
    /// Ports the service is served from: the one it listens on and, during
    /// development, the one of the server proxying to it.
    ports: Vec<u16>,
}

impl Origins {
    pub(super) fn new(ports: impl IntoIterator<Item = u16>) -> Self {
        let mut ports = ports.into_iter().collect::<Vec<_>>();
        ports.sort();
        ports.dedup();
        Self { ports }
    }

    /// Whether a request with the given headers comes from the service's own
    /// pages, or from a client which isn't a browser.
    pub(super) fn allows(&self, headers: &HeaderMap) -> bool {
        let origin = headers.get(header::ORIGIN);
        let host = headers.get(header::HOST);

        let host = match host {
            Some(host) => {
                let Some(host) = host.to_str().ok().and_then(|h| h.parse::<Authority>().ok())
                else {
                    return false;
                };

                if !is_ip_or_localhost(host.host()) || !self.has_port(host.port_u16().unwrap_or(80))
                {
                    return false;
                }

                Some(host)
            }
            None => None,
        };

        let Some(origin) = origin else {
            return true;
        };

        let Some(host) = host else {
            return false;
        };

        let Some(origin) = origin.to_str().ok().and_then(|o| o.parse::<Uri>().ok()) else {
            return false;
        };

        let default_port = match origin.scheme() {
            Some(scheme) if *scheme == Scheme::HTTP => 80,
            Some(scheme) if *scheme == Scheme::HTTPS => 443,
            _ => return false,
        };

        // An origin is only a scheme and an authority.
        if origin.path_and_query().is_some_and(|p| p.as_str() != "/") {
            return false;
        }

        let Some(authority) = origin.authority() else {
            return false;
        };

        let name = authority.host();

        let known = is_loopback(name) || name.eq_ignore_ascii_case(host.host());
        known && self.has_port(authority.port_u16().unwrap_or(default_port))
    }

    fn has_port(&self, port: u16) -> bool {
        self.ports.contains(&port)
    }
}

/// Middleware refusing requests which change something, that is any but
/// `GET`, `HEAD` and `OPTIONS`, unless [`Origins::allows`] them.
pub(super) async fn guard(
    State(origins): State<Origins>,
    request: Request,
    next: Next,
) -> Response {
    let safe = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );

    if !safe && !origins.allows(request.headers()) {
        return forbidden();
    }

    next.run(request).await
}

pub(super) fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        "Only allowed from the service's own pages",
    )
        .into_response()
}

fn parse_ip(name: &str) -> Option<IpAddr> {
    let name = name
        .strip_prefix('[')
        .and_then(|n| n.strip_suffix(']'))
        .unwrap_or(name);
    name.parse().ok()
}

fn is_localhost(name: &str) -> bool {
    name.eq_ignore_ascii_case("localhost")
}

fn is_ip_or_localhost(name: &str) -> bool {
    is_localhost(name) || parse_ip(name).is_some()
}

fn is_loopback(name: &str) -> bool {
    is_localhost(name) || parse_ip(name).is_some_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use axum::http::{header, HeaderMap, HeaderValue};

    use super::Origins;

    fn allows(host: Option<&'static str>, origin: Option<&'static str>) -> bool {
        let mut headers = HeaderMap::new();

        if let Some(host) = host {
            headers.insert(header::HOST, HeaderValue::from_static(host));
        }

        if let Some(origin) = origin {
            headers.insert(header::ORIGIN, HeaderValue::from_static(origin));
        }

        Origins::new([44714, 8080]).allows(&headers)
    }

    #[test]
    fn absent_origin() {
        assert!(allows(None, None));
        assert!(allows(Some("127.0.0.1:44714"), None));
        assert!(allows(Some("localhost:44714"), None));
        // A rebound name refuses even non-browser clients.
        assert!(!allows(Some("evil.example:44714"), None));
    }

    #[test]
    fn same_origin() {
        assert!(allows(
            Some("127.0.0.1:44714"),
            Some("http://127.0.0.1:44714")
        ));
        assert!(allows(
            Some("localhost:44714"),
            Some("http://localhost:44714")
        ));
        assert!(allows(Some("[::1]:44714"), Some("http://[::1]:44714")));
        assert!(allows(
            Some("LOCALHOST:44714"),
            Some("http://localhost:44714")
        ));
        // Served on all interfaces and reached by a LAN address.
        assert!(allows(
            Some("192.168.1.5:44714"),
            Some("http://192.168.1.5:44714")
        ));
    }

    #[test]
    fn localhost_alias() {
        // The UI is opened on localhost and connects its WebSocket to
        // 127.0.0.1.
        assert!(allows(
            Some("127.0.0.1:44714"),
            Some("http://localhost:44714")
        ));
        assert!(allows(
            Some("localhost:44714"),
            Some("http://127.0.0.1:44714")
        ));
        assert!(allows(Some("127.0.0.1:44714"), Some("http://[::1]:44714")));
        // The development server proxying to the service.
        assert!(allows(
            Some("127.0.0.1:44714"),
            Some("http://localhost:8080")
        ));
        assert!(allows(
            Some("127.0.0.1:8080"),
            Some("http://localhost:8080")
        ));
    }

    #[test]
    fn foreign_origin() {
        assert!(!allows(
            Some("127.0.0.1:44714"),
            Some("https://example.com")
        ));
        assert!(!allows(
            Some("127.0.0.1:44714"),
            Some("http://example.com:44714")
        ));
        assert!(!allows(
            Some("127.0.0.1:44714"),
            Some("http://127.0.0.1:3000")
        ));
        assert!(!allows(Some("127.0.0.1:44714"), Some("http://localhost")));
        assert!(!allows(Some("127.0.0.1:44714"), Some("null")));
        assert!(!allows(Some("127.0.0.1:44714"), Some("file://")));
        assert!(!allows(
            Some("127.0.0.1:44714"),
            Some("http://192.168.1.5:44714")
        ));
        assert!(!allows(None, Some("http://127.0.0.1:44714")));
    }

    #[test]
    fn dns_rebinding() {
        // A page on evil.example whose name now resolves to 127.0.0.1.
        assert!(!allows(
            Some("evil.example:44714"),
            Some("http://evil.example:44714")
        ));
        assert!(!allows(
            Some("evil.example:44714"),
            Some("http://localhost:44714")
        ));
    }

    #[test]
    fn foreign_host_port() {
        assert!(!allows(
            Some("127.0.0.1:3000"),
            Some("http://127.0.0.1:3000")
        ));
        assert!(!allows(Some("127.0.0.1"), Some("http://127.0.0.1")));
    }
}
