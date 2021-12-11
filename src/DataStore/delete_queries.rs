use sea_query::*; //query builder
use chrono::{DateTime, Utc};

//Dry violations on purpose, helps read and follow each specific query

pub const delete_room_query = "
DELETE FROM room
WHERE Id = $1;
";

pub const delete_room_permissions_query="
DELETE FROM room_permission
WHERE roomId = $1;
";

//ownerRoomId is the room that owns this block.
pub const delete_room_blocks_query = "
DELETE FROM room_block
WHERE ownerRoomId = $1;
";

pub const delete_scheduled_room_query ="
DELETE * FROM scheduled_room
WHERE Id = $1;
";

pub const delete_all_scheduled_room_attendance_query = "
DELETE FROM scheduled_room_attendance
WHERE Id = $1;
";

pub const delete_user_room_attendance_query = "
DELETE * FROM scheduled_room_attendance
WHERE scheduledRoomId = $1 and userId = $2;
";