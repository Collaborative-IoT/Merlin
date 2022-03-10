use crate::state::state::ServerState;
use crate::vs_response::handler;
use crate::vs_response::types::VoiceServerResponse;
pub async fn route_msg(msg: String, state: &mut ServerState) {
    let temp_value: serde_json::Value = serde_json::from_str(&msg).unwrap();
    println!("{}", temp_value);
    if temp_value["uid"] != serde_json::Value::Null {
        println!("tt");
        let op = temp_value["op"].as_str().unwrap();
        match op {
            "you_left_room" => {
                println!("tt2");
                let response: VoiceServerResponse = serde_json::from_str(&msg).unwrap();
                handler::notify_user_and_room(response, state, "user_left_room".to_owned()).await;
            }
            //we don't need to let them know which one
            //because they will get the permissions and know
            //if this new user is a listener/speaker.
            //
            //Everytime a user joins, the frontend
            //requests permissions.
            "you-joined-as-speaker" | "you-joined-as-peer" => {
                println!("tt3");
                let response: VoiceServerResponse = serde_json::from_str(&msg).unwrap();
                handler::notify_user_and_room(response, state, "new_user_joined".to_owned()).await;
            }

            "you-are-now-a-speaker" => {
                println!("tt4");
                let response: VoiceServerResponse = serde_json::from_str(&msg).unwrap();
                handler::notify_user_and_room(response, state, "new_speaker".to_owned()).await;
            }

            //private updates for users only, like
            //getting recv tracks and connecting send
            //transports etc.
            _ => {
                println!("tt5");
                let response: VoiceServerResponse = serde_json::from_str(&msg).unwrap();
                handler::notify_user_only(response, state).await;
            }
        };
    }
    //when the response is suppose to go to the entire room
    //with no filters as to who see what. Some responses
    //have filters meaning only one user sees like
    //credentials etc.
    else {
        println!("tt6");
        handler::notify_entire_room(temp_value, state).await;
    }
}
