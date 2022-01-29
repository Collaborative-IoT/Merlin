use crate::data_store::db_models::DBRoomPermissions;

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

pub fn modded_speaker(room_id: i32, user_id: i32) -> DBRoomPermissions {
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

pub fn create_non_preset(
    room_id: i32,
    user_id: i32,
    asked: bool,
    is_speaker: bool,
    is_mod: bool,
) -> DBRoomPermissions {
    return DBRoomPermissions {
        id: -1,
        is_mod: is_mod,
        is_speaker: is_speaker,
        asked_to_speak: asked,
        room_id: room_id,
        user_id: user_id,
    };
}
