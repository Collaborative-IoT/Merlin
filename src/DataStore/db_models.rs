
//There is no ORM, these are just structs used for passing required fields
//for insertion and gather

pub struct DBRoom{
    id:i32,
    owner_id:i32,
    chat_mode:String
}
pub struct DBRoomPermissions{
    id:i32,
    user_id:i32,
    room_id:i32,
    is_mod:bool,
    is_speaker:bool,
    asked_to_speak:bool,
}
pub struct DBFollower{
    id:i32,
    follower_id:i32,
    user_id:i32
}
pub struct DBUser{
    id:i32,
    display_name:String,
    avatar_url:Stringm,
    user_name:String,
    last_online:DateTime<Utc>,
    github_id:String,
    discord_id:String,
    github_access_token:String,
    discord_access_token:String,
    banned:bool,
    banned_reason:String,
    bio:String,
    contributions:i32,
    banner_url:String
}
pub struct DBUserBlock{
    id:i32,
    owner_user_id:i32,
    blocked_user_id:i32
}
pub struct DBRoomBlock{
    id:i32,
    owner_room_id:i32,
    blocked_user_id:i32
}
pub struct DBScheduledRoom{
    id:i32,
    room_name:i32,
    num_attending:i32,
    scheduled_for:DateTime<Utc>,
}
pub struct DBScheduledRoomAttendance{
    id:i32,
    user_id:i32,
    scheduled_room_id:i32,
    is_owner:bool
}