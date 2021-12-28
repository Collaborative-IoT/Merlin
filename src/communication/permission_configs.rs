use crate::data_store::db_models::{DBRoomPermissions};

/*
This file contains all possible configurations for a user of a room. Regarding speaking
and moderation permissions.
*/
pub fn regular_speaker(room_id: i32, user_id: i32) -> DBRoomPermissions {
    return DBRoomPermissions {
        id: -1,
        is_mod: false,
        is_speaker: true,
        asked_to_speak: false,
        room_id: room_id,
        user_id: user_id,
    };
}

pub fn modded_speaker(room_id: i32, user_id: i32) -> DBRoomPermissions{
    return DBRoomPermissions {
        id: -1,
        is_mod: true,
        is_speaker: true,
        asked_to_speak: false,
        room_id: room_id,
        user_id: user_id,
    };
}

pub fn modded_non_speaker(room_id: i32, user_id: i32) -> DBRoomPermissions {
    return DBRoomPermissions {
        id: -1,
        is_mod: true,
        is_speaker: false,
        asked_to_speak: false,
        room_id: room_id,
        user_id: user_id,
    };
}

pub fn regular_listener(room_id: i32, user_id: i32) -> DBRoomPermissions {
    return DBRoomPermissions {
        id: -1,
        is_mod: false,
        is_speaker: false,
        asked_to_speak: false,
        room_id: room_id,
        user_id: user_id,
    };
}
