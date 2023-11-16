use serde::{Deserialize, Serialize};

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
