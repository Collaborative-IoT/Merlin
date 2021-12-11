
//Dry violations on purpose, helps read and follow each specific query
pub const select_all_rooms_query: &str = "
SELECT * FROM room;
";

pub const select_all_scheduled_rooms_query: &str = "
SELECT * FROM scheduled_room;
";

pub const select_all_scheduled_room_attendance_for_room_query: &str= "
SELECT * FROM scheduled_room_attendance 
WHERE Id = $1;
";

pub const select_all_attendances_for_user_query: &str= "
SELECT * FROM scheduled_room_attendance 
WHERE userId = 1$;
";

pub const select_all_followers_for_user_query: &str= "
SELECT * FROM follower
WHERE userId = $1;
";

pub const select_all_following_for_user_query: &str="
SELECT * FROM follower 
WHERE followerId = $1;
";

pub const select_all_blocked_for_user_query: &str = "
SELECT * FROM user_block
WHERE ownerUserId = $1;
";

pub const select_all_blockers_for_user_query: &str ="
SELECT * FROM user_block
WHERE blockedUserId = $1;
";
pub const select_all_blocked_users_for_room_query: &str = "
SELECT * FROM room_block
WHERE ownerRoomId = $1;
";

pub const select_all_room_permissions_for_user: &str = "
SELECT * FROM room_permission
WHERE userId = $1 and roomId = $2;
";