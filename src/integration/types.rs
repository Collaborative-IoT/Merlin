use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AuthResponse {
    pub outside_name: String,
    pub passed_auth: bool,
    pub server_id: Option<String>,
}
