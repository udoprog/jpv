use serde::{Deserialize, Serialize};

use crate::database::EntryResultKey;
use crate::jmdict;
use crate::jmnedict;
use crate::kanjidic2;
use crate::Weight;

#[derive(Debug, Serialize, Deserialize)]
pub struct Empty;

#[derive(Debug, Serialize, Deserialize)]
pub struct SendClipboard {
    pub ty: Option<String>,
    pub data: Vec<u8>,
}

/// Json payload when sending the clipboard.
#[derive(Debug, Serialize, Deserialize)]
pub struct SendClipboardJson {
    pub primary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientEvent {
    SendClipboardData(SendClipboard),
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
    pub q: Option<&'a str>,
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
