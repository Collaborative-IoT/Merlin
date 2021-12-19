/*
All of the types that will come from or go with a specific request/response.
All requests/responses will start as a BasicRequest/BasicResponse, the true data
of the request/response(if there is any) will be the 'request_containing_data/response_containing_data'
in json serialization.
*/

//Gathering from client
pub struct BasicRequest{
    request_op_code:String,
    request_containing_data:String
}

pub struct BasicResponse{
    response_op_code:String,
    response_containing_data:String
}

pub struct JoinRoomAndGetInfo{
    room_id:i32
}

pub struct GetFollowList{
    user_id:i32,
}

pub struct GetUserProfile{
    user_id:i32
}

//basic types 
pub struct Room{
    details:RoomDetails,
    room_id: i32,
    num_of_people_in_room:i32,
    voice_server_id:i32,
    creator_id:i32,
    people_preview_data: Vec<UserPreview>,
    auto_speaker_setting:bool,
    created_at:String,
    chat_mode: String,
}

pub struct RoomDetails{
    name: bool,
    chat_throttle: i32,
    is_private:bool,
    description:String
}

pub struct RoomPermissions{
    asked_to_speak:bool,
    is_speaker:bool,
    is_mod:bool
}

pub struct MessageToken{
    type_of_token:String,
    value:String
}

pub struct UserPreview{
    user_id:i32,
    display_name:String,
    num_followers:i32,
    avatar_url:String
}

pub struct UserProfileEdit{
    display_name:String,
    username:String,
    bio:String,
    avatar_url:String,
    banner_url:String
}

pub struct RoomSettingsEditOrCreation{
    name:String,
    privacy:String,
    description:String,
}

pub struct ScheduledRoomUpdate{
    room_id:i32,
    details:RoomSettingsEditOrCreation
}

pub struct RoomUpdate{
    name: String,
    privacy: String,
    chat_throttle: i32,
    description: String,
    auto_speaker: bool
}

pub struct GenericOnlyBool{
    value:bool
}

pub struct GenericOnlyUserId{
    user_id:i32
}

pub struct Mute{
    muted:bool
}

pub struct ScheduledRoomGather{
    range:String,
    user_id:i32
}

pub struct User{
    you_are_following:bool,
    username:String,
    they_blocked_you:bool,
    num_following:i32,
    num_followers:i32,
    last_online:String,
    user_id:i32,
    follows_you:bool,
    contributions:i32,
    display_name:String,
    bio:String,
    avatar_url:String,
    banner_url: String,
    i_blocked_them:bool
}

pub struct BaseUser{
    username: String,
    last_online: String,
    user_id: i32,
    bio: String,
    display_name: String,
    avatar_url: String,
    banner_url: String,
    num_following: i32,
    num_followers: i32,
    current_room: Room,
    contributions: i32,
}

pub struct MessageBroadcastRequestDetails{
    tokens:Vec<MessageToken>,
    whispered_to:Vec<String>
}

pub struct Message{
    id: String,//uuid
    user_id: i32,
    avatar_url:String,
    color: String,
    display_name:String,
    tokens: Vec<MessageToken>,
    username: String,
    deleted:bool,
    deleter_id:String,
    sent_at:String,
    is_whisper:bool
}