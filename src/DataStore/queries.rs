pub const ROOM_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS room(
        Id SERIAL PRIMARY KEY,
        OwnerId  VARCHAR(30)
    );
";
pub const ROOM_PERMISSIONS_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS room_permissions(
        Id SERIAL PRIMARY KEY,
        roomId VARCHAR(30),
        isMod BOOLEAN NOT NULL,
        isSpeaker BOOLEAN NOT NULL,
        askedToSpeak BOOLEAN NOT NULL,
    );
";
pub const FOLLOWER_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS follower(
        Id SERIAL PRIMARY KEY,
        followerId VARCHAR(30),
        userId VARCHAR(30)
    );
";
pub const USER_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS user(
        Id SERIAL PRIMARY KEY,
        displayName VARCHAR(30),
        avatarUrl VARCHAR(30),
    );
";
pub const USER_BLOCK_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS user_block(
        Id SERIAL PRIMARY KEY,
        owneruserId VARCHAR(30),
        blockedUserId VARCHAR(30)
    );
";
pub const ROOM_BLOCK_CREATION = "
    CREATE TABLE IF NOT EXISTS room_block(
        Id SERIAL PRIMARY KEY,
        ownerRoomId VARCHAR(30),
        blockedUserId VARCHAR(30)
    );
";
pub const SCHEDULED_ROOM_CREATION = "
    CREATE TABLE IF NOT EXISTS scheduled_room(
        Id SERIAL PRIMARY KEY,
        name VARCHAR(30),
        numAttending int,
        scheduledFor TIMESTAMP,
    );
";
pub const SHEDULED_ROOM_ATTENDANCE = "
    CREATE TABLE IF NOT EXISTS scheduled_room_attendance(
        Id SERIAL PRIMARY KEY,
        userId VARCHAR(30),
        scheduledRoomId VARCHAR(30),
        isOwner BOOLEAN NOT NULL
    );
";