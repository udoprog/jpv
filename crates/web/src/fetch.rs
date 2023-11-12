use anyhow::Context;
use lib::database::EntryResultKey;
use lib::jmdict;
use lib::kanjidic2;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("{0}")]
    JsError(Box<str>),
    #[error("{0}")]
    ParseError(#[from] url::ParseError),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("{0}")]
    Error(#[from] anyhow::Error),
}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self::JsError(format!("{:?}", value).into())
    }
}

impl From<&'static str> for FetchError {
    fn from(value: &'static str) -> Self {
        Self::JsError(value.into())
    }
}

impl From<String> for FetchError {
    fn from(value: String) -> Self {
        Self::JsError(value.into())
    }
}

#[derive(Deserialize)]
pub struct SearchEntry {
    pub key: EntryResultKey,
    pub entry: jmdict::OwnedEntry,
}

#[derive(Deserialize)]
pub struct SearchResponse {
    pub entries: Vec<SearchEntry>,
    pub characters: Vec<kanjidic2::OwnedCharacter>,
}

/// Perform the given search.
pub(crate) async fn search(q: &str) -> Result<SearchResponse, FetchError> {
    request("search", [("q", q)]).await
}

#[derive(Deserialize)]
pub struct AnalyzeEntry {
    pub key: jmdict::EntryKey,
    pub string: String,
}

#[derive(Deserialize)]
pub struct AnalyzeResponse {
    pub data: Vec<AnalyzeEntry>,
}

/// Perform the given analysis.
pub(crate) async fn analyze(q: &str, start: usize) -> Result<AnalyzeResponse, FetchError> {
    request("analyze", [("q", q), ("start", start.to_string().as_str())]).await
}

async fn request<T, const N: usize>(p: &str, pairs: [(&str, &str); N]) -> Result<T, FetchError>
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

    let request = Request::new_with_str_and_init(&url.to_string(), &opts)?;
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let text = JsFuture::from(resp.text()?).await?;
    let text = text.as_string().context("failed to convert to string")?;
    let response = serde_json::from_str(&text)?;
    Ok(response)
}
