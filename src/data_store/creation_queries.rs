//Dry violations on purpose, helps read and follow each specific query

//initial creation
pub const ROOM_TABLE_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS room(
        Id SERIAL PRIMARY KEY,
        ownerId int NOT NULL,
        chatMode VARCHAR(30)
    );
";
pub const ROOM_PERMISSIONS_TABLE_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS room_permission(
        Id SERIAL PRIMARY KEY,
        userId int NOT NULL,
        roomId int NOT NULL,
        isMod BOOLEAN NOT NULL,
        isSpeaker BOOLEAN NOT NULL,
        askedToSpeak BOOLEAN NOT NULL
    );
";
pub const FOLLOWER_TABLE_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS follower(
        Id SERIAL PRIMARY KEY,
        followerId int NOT NULL,
        userId int NOT NULL
    );
";
pub const USER_TABLE_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS users(
        Id SERIAL PRIMARY KEY,
        displayName VARCHAR(30),
        avatarUrl VARCHAR(255),
        userName VARCHAR(30),
        lastOnline VARCHAR(255),
        githubId VARCHAR(50),
        discordId VARCHAR(50),
        githubAccessToken VARCHAR(255),
        discordAccessToken VARCHAR(255),
        banned BOOLEAN NOT NULL,
        bannedReason VARCHAR(50),
        bio VARCHAR(255),
        contributions int NOT NULL,
        bannerUrl VARCHAR(255)
    );
";
pub const USER_BLOCK_TABLE_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS user_block(
        Id SERIAL PRIMARY KEY,
        ownerUserId int,
        blockedUserId int
    );
";
//ownerRoomId is the room that owns this block.
pub const ROOM_BLOCK_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS room_block(
        Id SERIAL PRIMARY KEY,
        ownerRoomId int,
        blockedUserId int
    );
";
pub const SCHEDULED_ROOM_CREATION: &str = "
    CREATE TABLE IF NOT EXISTS scheduled_room(
        Id SERIAL PRIMARY KEY,
        roomName VARCHAR(255) NOT NULL,
        numAttending int NOT NULL,
        scheduledFor VARCHAR(255)
    );
";
pub const SHEDULED_ROOM_ATTENDANCE: &str = "
    CREATE TABLE IF NOT EXISTS scheduled_room_attendance(
        Id SERIAL PRIMARY KEY,
        userId int NOT NULL,
        scheduledRoomId int,
        isOwner BOOLEAN NOT NULL
    );
";
