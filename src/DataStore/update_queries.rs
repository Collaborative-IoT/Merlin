//Dry violations on purpose, helps read and follow each specific query
pub const update_room_owner_query = "
UPDATE room
SET ownerId = $1;
WHERE Id = $2;
"
pub const update_room_mod_status_query = "
UPDATE room_permissions
SET isMod = $1
WHERE roomId = $2 AND userId = $3;
"

pub const update_user_avatar_query = "
UPDATE user
SET avatarUrl = $1
WHERE Id = $2;
";

pub const update_display_name_query = "
UPDATE user
SET displayName = $1
WHERE Id = $2;
";

pub const update_scheduled_room_query = "
UPDATE scheduled_room
SET = numAttending = $1,
      scheduledFor = $2
WHERE Id = $3;   
";

pub const ban_user_query = "
UPDATE user
SET banned = $1,
bannedReason = $2,
WHERE Id = $3;
";