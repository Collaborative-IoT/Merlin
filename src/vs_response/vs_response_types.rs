use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct VoiceServerResponse {
    pub op: String,
    pub d: serde_json::Value,
    pub uid: String,
}
