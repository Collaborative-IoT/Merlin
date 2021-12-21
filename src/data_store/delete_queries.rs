//Dry violations on purpose, helps read and follow each specific query

pub const DELETE_ROOM_QUERY: &str = "
DELETE FROM room
WHERE Id = $1;
";

pub const DELETE_ROOM_PERMISSIONS_QUERY: &str = "
DELETE FROM room_permission
WHERE roomId = $1;
";

//ownerRoomId is the room that owns this block.
pub const DELETE_ROOM_BLOCKS_QUERY: &str = "
DELETE FROM room_block
WHERE ownerRoomId = $1;
";

pub const DELETE_ROOM_BLOCK_QUERY: &str = "
DELETE FROM room_block
WHERE ownerRoomId = $1 and blockedUserId = $2;
";

pub const DELETE_USER_BLOCK_QUERY: &str = "
DELETE FROM user_block
WHERE ownerUserId = $1 and blockedUserId = $2;
";

pub const DELETE_SCHEDULED_ROOM_QUERY: &str = "
DELETE FROM scheduled_room
WHERE Id = $1;
";

pub const DELETE_ALL_SCHEDULED_ROOM_ATTENDANCE_QUERY: &str = "
DELETE FROM scheduled_room_attendance
WHERE scheduledRoomId = $1;
";

pub const DELETE_USER_ROOM_ATTENDANCE_QUERY: &str = "
DELETE FROM scheduled_room_attendance
WHERE scheduledRoomId = $1 and userId = $2;
";

pub const DELETE_FOLLOWER_FOR_USER: &str = "
DELETE FROM follower
WHERE followerId = $1 and userId = $2;
";
