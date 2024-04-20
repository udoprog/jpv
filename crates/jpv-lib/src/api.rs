use std::collections::HashSet;

use musli::de::DecodeOwned;
use musli::mode::Binary;
use musli::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::database::EntryResultKey;
use crate::jmdict;
use crate::jmnedict;
use crate::kanjidic2;
use crate::Weight;

pub trait Request: Encode<Binary> {
    /// The kind of the request.
    const KIND: &'static str;
    /// The expected response.
    type Response: 'static + DecodeOwned<Binary>;
}

#[derive(Debug, Encode, Decode, Deserialize)]
pub struct AnalyzeRequest {
    pub q: String,
    pub start: usize,
}

impl Request for AnalyzeRequest {
    const KIND: &'static str = "analyze";
    type Response = OwnedAnalyzeResponse;
}

#[derive(Debug, Encode, Decode, Deserialize)]
pub struct SearchRequest {
    pub q: String,
}

impl Request for SearchRequest {
    const KIND: &'static str = "search";
    type Response = OwnedSearchResponse;
}

#[derive(Debug, Encode, Decode)]
pub struct InstallAllRequest;

impl Request for InstallAllRequest {
    const KIND: &'static str = "install-all";
    type Response = Empty;
}

#[derive(Debug, Encode, Decode)]
pub struct GetState;

impl Request for GetState {
    const KIND: &'static str = "get-state";
    type Response = GetStateResult;
}

#[derive(Debug, Encode, Decode)]
pub struct GetStateResult {
    /// Installed dictionaries.
    pub installed: HashSet<String>,
}

#[derive(Debug, Encode, Decode)]
pub struct GetConfig;

impl Request for GetConfig {
    const KIND: &'static str = "get-config";
    type Response = GetConfigResult;
}

#[derive(Debug, Encode, Decode)]
pub struct GetKanji {
    pub kanji: String,
}

impl Request for GetKanji {
    const KIND: &'static str = "get-kanji";
    type Response = OwnedKanjiResponse;
}

/// Missing OCR support.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub struct InstallUrl {
    /// Title of the URL.
    pub text: String,
    /// Hover title.
    pub title: String,
    /// The URL where to install it from.
    pub url: String,
}

/// Missing OCR support.
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub struct MissingOcr {
    /// The URL where to install it from.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub install_url: Option<InstallUrl>,
}

impl MissingOcr {
    #[cfg(unix)]
    pub fn for_platform() -> Self {
        Self { install_url: None }
    }

    #[cfg(windows)]
    pub fn for_platform() -> Self {
        Self {
            install_url: Some(InstallUrl {
                text: "Install Tesseract-OCR".to_string(),
                title: "Download and install Tesseract-OCR from UB-Mannheim.\nDon't forget to add Japanese as additional script!".to_string(),
                url: "https://github.com/UB-Mannheim/tesseract/wiki".to_string(),
            }),
        }
    }
}

#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct GetConfigResult {
    /// System configuration.
    pub config: Config,
    /// Installed dictionaries.
    #[musli(default, skip_encoding_if = HashSet::is_empty)]
    pub installed: HashSet<String>,
    /// Indicates that OCR support is missing, and some indications of how to install it.
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub missing_ocr: Option<MissingOcr>,
}

#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct UpdateConfigRequest {
    /// Configuration update to save.
    pub config: Option<Config>,
    /// Collection of indexes to update.
    pub update_indexes: Vec<String>,
}

impl Request for UpdateConfigRequest {
    const KIND: &'static str = "update-config";
    type Response = UpdateConfigResponse;
}

#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct UpdateConfigResponse {
    /// Indicates that the configuration has been updated with the given value.
    pub config: Option<Config>,
}

#[derive(Debug, Clone, Copy, Serialize, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct Empty;

#[borrowme::borrowme]
#[derive(Debug, Clone, Encode, Decode)]
pub struct SendClipboard<'a> {
    #[musli(mode = Text, default, name = "type", skip_encoding_if = Option::is_none)]
    pub ty: Option<&'a str>,
    #[borrowme(owned = Box<[u8]>, to_owned_with = Box::from)]
    pub data: &'a [u8],
}

/// Json payload when sending the clipboard.
#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct SendClipboardJson {
    pub primary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub secondary: Option<String>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Encode, Decode)]
pub struct LogBackFill<'a> {
    pub log: Vec<LogEntry<'a>>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Encode, Decode)]
pub enum BroadcastKind<'a> {
    SendClipboardData(SendClipboard<'a>),
    LogBackFill(LogBackFill<'a>),
    LogEntry(LogEntry<'a>),
    TaskProgress(TaskProgress<'a>),
    TaskCompleted(TaskCompleted<'a>),
    Refresh,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Encode, Decode)]
pub struct Broadcast<'a> {
    pub kind: BroadcastKind<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub enum ClientResponse<'a> {
    Search(SearchResponse<'a>),
    Analyze(AnalyzeResponse<'a>),
    GetConfig(Config),
    Error(String),
    UpdatedConfig,
    Empty,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
pub struct ClientRequestEnvelope<'de> {
    pub index: usize,
    pub serial: u32,
    pub kind: &'de str,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Encode, Decode)]
pub struct ClientResponseEnvelope<'de> {
    pub index: usize,
    pub serial: u32,
    #[musli(default, skip_encoding_if = Option::is_none)]
    pub error: Option<&'de str>,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
pub enum ClientEvent<'a> {
    Broadcast(Broadcast<'a>),
    ClientResponse(ClientResponseEnvelope<'a>),
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct SearchPhrase<'a> {
    pub key: EntryResultKey,
    pub phrase: jmdict::Entry<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct SearchName<'a> {
    pub key: EntryResultKey,
    pub name: jmnedict::Entry<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct SearchResponse<'a> {
    pub phrases: Vec<SearchPhrase<'a>>,
    pub names: Vec<SearchName<'a>>,
    pub characters: Vec<kanjidic2::Character<'a>>,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
pub struct AnalyzeEntry<'a> {
    pub key: Weight,
    pub string: &'a str,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
pub struct AnalyzeResponse<'a> {
    pub data: Vec<AnalyzeEntry<'a>>,
}

#[borrowme::borrowme]
#[derive(Debug, Encode, Decode)]
#[musli(mode = Text, name_all = "kebab-case")]
pub struct EntryResponse<'a> {
    pub entry: jmdict::Entry<'a>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, Encode, Decode)]
pub struct KanjiResponse<'a> {
    pub kanji: kanjidic2::Character<'a>,
    #[musli(default, skip_encoding_if = Vec::is_empty)]
    pub radicals: Vec<&'a str>,
}

#[borrowme::borrowme]
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct LogEntry<'a> {
    /// Timestamp of the log entry in milliseconds since the unix epoch.
    pub timestamp: u64,
    /// The target being logged.
    pub target: &'a str,
    /// The level of the rebuild.
    pub level: &'a str,
    /// The rext of the rebuild.
    pub text: &'a str,
}

/// A message indicating task progress.
#[borrowme::borrowme]
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
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
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct TaskCompleted<'a> {
    pub name: &'a str,
}
