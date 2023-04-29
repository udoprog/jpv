use anyhow::Context;
use thiserror::Error;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
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

/// Perform the given search.
pub(crate) async fn search(q: &str) -> Result<String, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let mut url = Url::parse("http://localhost:8080/api/search")?;
    url.query_pairs_mut().append_pair("q", q);

    let request = Request::new_with_str_and_init(&url.to_string(), &opts)?;
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let text = JsFuture::from(resp.text()?).await?;
    let text = text.as_string().context("failed to convert to string")?;
    let _response: serde_json::Value = serde_json::from_str(&text)?;
    Ok(text)
}
