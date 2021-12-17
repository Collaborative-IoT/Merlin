
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

pub BasicResponse{
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

//requests with no containing data, the server knows
//credentials needed for these requests.
pub struct GetBlockedFromRoomUsers;
pub struct GetMyFollowing;
pub struct GetTopPublicRooms;
pub struct GetCurrentRoomUsers;

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

pub struct UserPreview{
    user_id:i32,
    display_name:String,
    num_followers:i32,
    avatar_url:String
}

pub struct User{
    you_are_following:bool,
    username:String,
    online: bool,
    num_following:i32,
    num_followers:i32,
    last_online:String,
    user_id:i32,
    follows_you:bool,
    contributions:i32,
    display_name:String,
    current_room_id:i32,
    current_room:Room,
    bio:String,
    avatar_url:String,
    banner_url String,
    whisper_privacy_setting:String,
    i_blocked_them:bool
}

pub struct BaseUser{
    username: String,
    online: bool,
    last_online: String,
    user_id: i32,
    bio: String,
    display_name: String,
    avatar_url: String,
    banner_url: String
    num_following: i32,
    num_followers: i32,
    current_room: Room,
    contributions: i32,
}

pub struct Message{
    id: UUID;
    userId: UUID;
    avatarUrl: UUID;
    color: string;
    displayName: string;
    tokens: MessageToken[];
    username: string;
    deleted?: boolean;
    deleterId?: UUID;
    sentAt: string;
    isWhisper?: boolean;
}