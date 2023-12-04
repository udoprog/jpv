use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::database::EntryResultKey;
use crate::jmdict;
use crate::jmnedict;
use crate::kanjidic2;
use crate::Weight;

pub trait Request: Serialize {
    /// The kind of the request.
    const KIND: &'static str;

    /// The expected response.
    type Response: 'static + DeserializeOwned;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub q: String,
    pub start: usize,
}

impl Request for AnalyzeRequest {
    const KIND: &'static str = "analyze";
    type Response = OwnedAnalyzeResponse;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub q: String,
}

impl Request for SearchRequest {
    const KIND: &'static str = "search";
    type Response = OwnedSearchResponse;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RebuildRequest;

impl Request for RebuildRequest {
    const KIND: &'static str = "rebuild";
    type Response = Empty;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetConfigRequest;

impl Request for GetConfigRequest {
    const KIND: &'static str = "get-config";
    type Response = Config;
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UpdateConfigRequest(pub Config);

impl Request for UpdateConfigRequest {
    const KIND: &'static str = "update-config";
    type Response = Empty;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Empty;

#[borrowme::borrowme]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendClipboard<'a> {
    #[borrowed_attr(serde(borrow))]
    pub ty: Option<&'a str>,
    #[borrowme(owned = Box<[u8]>, to_owned_with = Box::from)]
    pub data: &'a [u8],
}

/// Json payload when sending the clipboard.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendClipboardJson {
    pub primary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogBackFill<'a> {
    #[borrowed_attr(serde(borrow))]
    pub log: Vec<LogEntry<'a>>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BroadcastKind<'a> {
    #[borrowed_attr(serde(borrow))]
    SendClipboardData(SendClipboard<'a>),
    #[borrowed_attr(serde(borrow))]
    LogBackFill(LogBackFill<'a>),
    #[borrowed_attr(serde(borrow))]
    LogEntry(LogEntry<'a>),
    #[borrowed_attr(serde(borrow))]
    TaskProgress(TaskProgress<'a>),
    #[borrowed_attr(serde(borrow))]
    TaskCompleted(TaskCompleted<'a>),
    Refresh,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Broadcast<'a> {
    #[borrowed_attr(serde(borrow))]
    pub kind: BroadcastKind<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    Search(SearchResponse<'a>),
    #[borrowed_attr(serde(borrow))]
    Analyze(AnalyzeResponse<'a>),
    GetConfig(Config),
    Error(String),
    UpdatedConfig,
    Empty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRequestEnvelope {
    pub index: usize,
    pub serial: u32,
    pub kind: String,
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponseEnvelope {
    pub index: usize,
    pub serial: u32,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub body: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientEvent<'a> {
    #[borrowed_attr(serde(borrow))]
    Broadcast(Broadcast<'a>),
    ClientResponse(ClientResponseEnvelope),
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchPhrase<'a> {
    pub key: EntryResultKey,
    #[borrowed_attr(serde(borrow))]
    pub phrase: jmdict::Entry<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchName<'a> {
    pub key: EntryResultKey,
    #[borrowed_attr(serde(borrow))]
    pub name: jmnedict::Entry<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    pub phrases: Vec<SearchPhrase<'a>>,
    #[borrowed_attr(serde(borrow))]
    pub names: Vec<SearchName<'a>>,
    #[borrowed_attr(serde(borrow))]
    pub characters: Vec<kanjidic2::Character<'a>>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeEntry<'a> {
    pub key: Weight,
    pub string: &'a str,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    pub data: Vec<AnalyzeEntry<'a>>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct EntryResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    pub entry: jmdict::Entry<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct KanjiResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    pub entry: kanjidic2::Character<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry<'a> {
    /// The target being logged.
    pub target: &'a str,
    /// The level of the rebuild.
    pub level: &'a str,
    /// The rext of the rebuild.
    pub text: &'a str,
}

/// A message indicating task progress.
#[borrowme::borrowme]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskProgress<'a> {
    pub name: &'a str,
    pub value: usize,
    pub total: Option<usize>,
    pub step: usize,
    pub steps: usize,
    pub text: &'a str,
}

/// Indicates that a task has been completed.
#[borrowme::borrowme]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCompleted<'a> {
    pub name: &'a str,
}
