use anyhow::Context;
use lib::api;
use lib::config::Config;
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Headers};
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::error::Error;

/// Get service configuration.
pub(crate) async fn config() -> Result<Config, Error> {
    get("config", []).await
}

/// Update service configuration..
pub(crate) async fn update_config(config: Config) -> Result<api::Empty, Error> {
    post("config", config).await
}

/// Update service configuration..
pub(crate) async fn rebuild() -> Result<api::Empty, Error> {
    post("rebuild", api::Empty).await
}

/// Perform the given search.
pub(crate) async fn search(q: &str, serial: u32) -> Result<api::OwnedSearchResponse, Error> {
    get(
        "search",
        [("q", q), ("serial", serial.to_string().as_str())],
    )
    .await
}

/// Perform the given analysis.
pub(crate) async fn analyze(
    q: &str,
    start: usize,
    serial: u32,
) -> Result<api::OwnedAnalyzeResponse, Error> {
    get(
        "analyze",
        [
            ("q", q),
            ("start", start.to_string().as_str()),
            ("serial", serial.to_string().as_str()),
        ],
    )
    .await
}

async fn get<T, const N: usize>(p: &str, pairs: [(&str, &str); N]) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let window = window().ok_or("no window")?;
    let port = window.location().port()?;

    let url = format!("http://localhost:{port}/api");
    let mut url = Url::parse(&url)?;

    if let Ok(mut path) = url.path_segments_mut() {
        path.push(p);
    }

    {
        let mut p = url.query_pairs_mut();

        for (key, value) in pairs {
            p.append_pair(key, value);
        }
    }

    let request = Request::new_with_str_and_init(url.as_ref(), &opts)?;
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let text = JsFuture::from(resp.text()?).await?;
    let text = text.as_string().context("failed to convert to string")?;
    let response = serde_json::from_str(&text)?;
    Ok(response)
}

/// Send a POST request.
async fn post<B, T>(p: &str, body: B) -> Result<T, Error>
where
    B: Serialize,
    T: DeserializeOwned,
{
    let body = serde_json::to_string(&body)?;
    let body = JsValue::from_str(&body);

    let headers = Headers::new()?;
    headers.append("Content-Type", "application/json")?;

    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&body));
    opts.headers(&headers);

    let window = window().ok_or("no window")?;
    let port = window.location().port()?;

    let url = format!("http://localhost:{port}/api");
    let mut url = Url::parse(&url)?;

    if let Ok(mut path) = url.path_segments_mut() {
        path.push(p);
    }

    let request = Request::new_with_str_and_init(url.as_ref(), &opts)?;
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let text = JsFuture::from(resp.text()?).await?;
    let text = text.as_string().context("failed to convert to string")?;
    let response = serde_json::from_str(&text)?;
    Ok(response)
}
