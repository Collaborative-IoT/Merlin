use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AuthResponse {
    pub outside_name: String,
    pub passed_auth: bool,
    pub server_id: Option<String>,
}

/// Communication with the integration server
#[derive(Deserialize, Serialize)]
pub struct GeneralMessage {
    pub category: String,
    pub data: String,
    pub server_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HouseOfIoTCredentials {
    //the connection str is usually just the location
    //of the server
    pub connection_str: String,
    pub name_and_type: String,
    pub password: String,
    pub admin_password: String,
    pub outside_name: String,
    pub user_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct DisconnectMsg {
    pub server_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HOIActionDataIncoming {
    pub server_id: String,
    pub bot_name: String,
    pub action: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HOIActionDataOutgoing {
    pub bot_name: String,
    pub action: String,
}
