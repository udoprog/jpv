use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSendClipboardData {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientEvent {
    SendClipboardData(ClientSendClipboardData),
}
