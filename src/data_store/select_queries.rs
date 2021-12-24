//Dry violations on purpose, helps read and follow each specific query
pub const SELECT_ALL_ROOM_QUERY: &str = "
SELECT * FROM room;
";

pub const SELECT_ROOM_BY_ID: &str = "
SELECT * FROM room
WHERE Id = $1;
";

pub const SELECT_SCHEDULED_ROOM_BY_ID: &str = "
SELECT * FROM scheduled_room
WHERE Id = $1;
";

pub const SELECT_ALL_SCHEDULED_ROOMS_QUERY: &str = "
SELECT * FROM scheduled_room;
";

pub const SELECT_ALL_SCHEDULED_ROOM_ATTENDANCE_FOR_ROOM_QUERY: &str = "
SELECT * FROM scheduled_room_attendance 
WHERE scheduledRoomId = $1;
";

pub const SELECT_ALL_OWNED_SCHEDULED_ROOMS_FOR_USER: &str = "
SELECT * FROM scheduled_room_attendance
WHERE userId = $1 AND isOwner = true;
";

pub const SELECT_ALL_ATTENDANCE_FOR_USER_QUERY: &str = "
SELECT * FROM scheduled_room_attendance 
WHERE userId = 1$;
";

pub const SELECT_ALL_FOLLOWERS_FOR_USER_QUERY: &str = "
SELECT * FROM follower
WHERE userId = $1;
";

pub const SELECT_ALL_FOLLOWING_FOR_USER_QUERY: &str = "
SELECT * FROM follower 
WHERE followerId = $1;
";

pub const SELECT_SINGLE_FOLLOWING_FOR_USER_QUERY: &str = "
SELECT * FROM follower
WHERE followerId = $1 AND userId = $2;
";

pub const SELECT_SINGLE_ROOM_ATTENDANCE_FOR_USER_QUERY: &str = "
SELECT * FROM scheduled_room_attendance 
WHERE userId = $1 AND scheduledRoomId = $2;
";

pub const SELECT_SINGLE_USER_BLOCK_FOR_USER_QUERY: &str = "
SELECT * FROM user_block
WHERE ownerUserId = $1 AND blockedUserId = $2;
";

pub const SELECT_SINGLE_ROOM_BLOCK_FOR_USER_QUERY: &str = "
SELECT * FROM room_block 
WHERE ownerRoomId = $1 AND blockedUserId = $2;
";

pub const SELECT_ALL_BLOCKED_FOR_USER_QUERY: &str = "
SELECT * FROM user_block
WHERE ownerUserId = $1;
";

pub const SELECT_ALL_BLOCKERS_FOR_USER_QUERY: &str = "
SELECT * FROM user_block
WHERE blockedUserId = $1;
";
pub const SELECT_ALL_BLOCKED_USERS_FOR_ROOM_QUERY: &str = "
SELECT * FROM room_block
WHERE ownerRoomId = $1;
";

pub const SELECT_ALL_ROOM_PERMISSIONS_FOR_USER: &str = "
SELECT * FROM room_permission
WHERE userId = $1 and roomId = $2;
";

pub const SELECT_USER_BY_ID: &str = "
SELECT * FROM users
WHERE Id = $1;
";

pub const SELECT_ALL_PERMISSIONS_FOR_ROOM: &str = "
SELECT * FROM room_permission
WHERE roomId = $1;
";

pub const SELECT_USER_PREVIEW_FOR_USER: &str = "
SELECT Id, displayName, avatarUrl FROM users
WHERE Id = $1;
";

//no account can have both a github and discord id,
//the one that doesn't exist is -1
pub const SELECT_USER_BY_CREATION_IDENTIFIERS: &str = "
SELECT * FROM users
    WHERE githubId = $1 AND discordId = $2;
";
