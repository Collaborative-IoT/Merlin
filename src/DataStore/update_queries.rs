//Dry violations on purpose, helps read and follow each specific query
pub const UPDATE_ROOM_OWNER_QUERY: &str = "
UPDATE room
SET ownerId = $1
WHERE Id = $2;
";

pub const UPDATE_ROOM_MOD_STATUS_QUERY: &str = "
UPDATE room_permissions
SET isMod = $1
WHERE roomId = $2 AND userId = $3;
";

pub const UPDATE_ENTIRE_ROOM_PERMISSIONS:&str = "
UPDATE room_permissions
SET isMod = $1,
    isSpeaker = $2,
    askedToSpeak = $3
WHERE roomId = $4 AND userId = $5;
";

pub const UPDATE_USER_AVATAR_QUERY: &str = "
UPDATE users
SET avatarUrl = $1
WHERE Id = $2;
";

pub const UPDATE_DISPLAY_NAME_QUERY: &str = "
UPDATE users
SET displayName = $1
WHERE Id = $2;
";

pub const UPDATE_SCHEDULED_ROOM_QUERY: &str = "
UPDATE scheduled_room
SET numAttending = $1,
      scheduledFor = $2
WHERE Id = $3;   
";

pub const UPDATE_GITHUB_ACCESS_TOKEN_QUERY: &str = "
UPDATE users
SET githubAccessToken = $1
WHERE Id = $2;
";

pub const UPDATE_DISCORD_ACCESS_TOKEN_QUERY: &str = "
UPDATE users
SET discordAccessToken = $1
WHERE Id = $2;
";

pub const UPDATE_CONTRIBUTIONS_QUERY: &str = "
UPDATE users
SET contributions = $1
WHERE Id = $2;
";

pub const UPDATE_BANNER_URL_QUERY: &str = "
UPDATE users
SET bannerUrl = $1
WHERE Id = $2;
";

pub const UPDATE_BIO_QUERY: &str = "
UPDATE users
SET bio = $1
WHERE Id = $2;
";

pub const UPDATE_LAST_ONLINE_QUERY: &str = "
UPDATE users
SET lastOnline = $1
WHERE Id = $2;
";

pub const UPDATE_USER_NAME_QUERY: &str = "
UPDATE users
SET userName = $1
WHERE Id = $2;
";

pub const BAN_USER_QUERY: &str = "
UPDATE users
SET banned = $1,
bannedReason = $2
WHERE Id = $3;
";

pub const UPDATE_ENTIRE_USER: &str = "
UPDATE users
SET displayName = $1,
    avatarUrl = $2,
    userName = $3,
    lastOnline = $4,
    githubId = $5,
    discordId = $6,
    githubAccessToken = $7,
    discordAccessToken = $8,
    banned = $9,
    bannedReason = $10,
    bio = $11,
    contributions = $12,
    bannerUrl = $13
WHERE Id = $14;
";