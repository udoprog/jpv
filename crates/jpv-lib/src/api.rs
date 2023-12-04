use serde::{Deserialize, Serialize};

use crate::database::EntryResultKey;
use crate::jmdict;
use crate::jmnedict;
use crate::kanjidic2;
use crate::Weight;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientRequestKind {
    Search(OwnedSearchRequest),
    Analyze(OwnedAnalyzeRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRequest {
    pub index: usize,
    pub serial: u32,
    pub kind: ClientRequestKind,
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
pub enum ClientResponseKind<'a> {
    #[borrowed_attr(serde(borrow))]
    Search(SearchResponse<'a>),
    #[borrowed_attr(serde(borrow))]
    Analyze(AnalyzeResponse<'a>),
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientResponse<'a> {
    pub index: usize,
    pub serial: u32,
    #[borrowed_attr(serde(borrow))]
    pub kind: ClientResponseKind<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientEvent<'a> {
    #[borrowed_attr(serde(borrow))]
    Broadcast(Broadcast<'a>),
    #[borrowed_attr(serde(borrow))]
    ClientResponse(ClientResponse<'a>),
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
pub struct SearchRequest<'a> {
    #[borrowed_attr(serde(borrow))]
    pub q: &'a str,
    #[serde(default)]
    pub serial: Option<u32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest<'a> {
    pub q: &'a str,
    pub start: usize,
    #[serde(default)]
    pub serial: Option<u32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryQuery {
    #[serde(default)]
    pub serial: Option<u32>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct EntryResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    pub entry: jmdict::Entry<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KanjiQuery {
    #[serde(default)]
    pub serial: Option<u32>,
}

#[borrowme::borrowme]
#[derive(Debug, Serialize, Deserialize)]
pub struct KanjiResponse<'a> {
    #[borrowed_attr(serde(borrow))]
    pub entry: kanjidic2::Character<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
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
