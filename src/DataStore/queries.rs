//initial creation
pub const ROOM_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS room(
        Id int PRIMARY KEY,
        ownerId int
    );
";
pub const ROOM_PERMISSIONS_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS room_permissions(
        Id int PRIMARY KEY,
        roomId int,
        isMod BOOLEAN NOT NULL,
        isSpeaker BOOLEAN NOT NULL,
        askedToSpeak BOOLEAN NOT NULL,
    );
";
pub const FOLLOWER_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS follower(
        Id int PRIMARY KEY,
        followerId int,
        userId int
    );
";
pub const USER_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS user(
        Id int PRIMARY KEY,
        displayName VARCHAR(30),
        avatarUrl VARCHAR(30),
    );
";
pub const USER_BLOCK_TABLE_CREATION = "
    CREATE TABLE IF NOT EXISTS user_block(
        Id int PRIMARY KEY,
        ownerUserId int,
        blockedUserId int
    );
";
pub const ROOM_BLOCK_CREATION = "
    CREATE TABLE IF NOT EXISTS room_block(
        Id int PRIMARY KEY,
        ownerRoomId int,
        blockedUserId int
    );
";
pub const SCHEDULED_ROOM_CREATION = "
    CREATE TABLE IF NOT EXISTS scheduled_room(
        Id int PRIMARY KEY,
        roomName int,
        numAttending int,
        scheduledFor TIMESTAMP,
    );
";
pub const SHEDULED_ROOM_ATTENDANCE = "
    CREATE TABLE IF NOT EXISTS scheduled_room_attendance(
        Id int PRIMARY KEY,
        userId int,
        scheduledRoomId int,
        isOwner BOOLEAN NOT NULL
    );
";

//insertion
pub const INSERT_ROOM = "
    INSERT INTO room (Id,ownerId) VALUES($1,$2)
";
pub const INSERT_ROOM_PERMISSION = "
    INSERT INTO room_permissions (Id,roomId,isMod,isSpeaker,askedToSpeak)
    VALUES ($1,$2,$3,$4,$5)
";
pub const INSERT_FOLLOWER = "
    INSERT INTO follower (Id,followerId,userId) 
    VALUES($1,$2,$3)
";
pub const INSERT_USER = "
    INSERT INTO user (Id,displayName,avatarUrl)
    VALUES($1,$2,$3)
";
pub const INSERT_USER_BLOCK = "
    INSERT INTO user_block (Id,ownerUserId,blockedUserId)
    VALUES($1,$2,$3)
";
pub const INSERT_ROOM_BLOCK = "
    INSERT INTO room_block (Id,ownerRoomId,blockedUserId)
    VALUES($1,$2,$3)
";
pub const INSERT_SCHEDULED_ROOM = "
    INSERT INTO scheduled_room (Id,roomName,numAttending,scheduledFor)
    VALUES($1,$2,$3,$4)
";
pub const INSERT_SCHEDULED_ATTENDANCE = "
    INSERT INTO scheduled_room_attendance (Id,userId,scheduledRoomId,isOwner)
    ($1,$2,$3,$4)
";

//update 
pub const UPDATE_ROOM_OWNER = "
    UPDATE room
    SET ownerId = $1
    WHERE Id = $2
";
pub const UPDATE_ROOM_MOD_STATUS = "
    UPDATE room_permissions 
    SET isMod = $1
    WHERE Id = $2
";
pub const UPDATE_USER_AVATAR = "
    UPDATE user
    SET avatarUrl = $1
    WHERE Id = $2
";
pub const UPDATE_USER_DISPLAY_NAME = "
    UPDATE user 
    SET displayName = $1
    WHERE Id = $2
";
pub const UPDATE_SCHEDULED_ROOM = "
    UPDATE scheduled_room
    SET roomName = $1,
        numAttending = $2,
        scheduledFor = $3
    WHERE Id = $4
";
pub const UPDATE_SCHEDULED_ROOM_OWNER = "
    UPDATE scheduled_room_attendance
    SET isOwner = $1
    WHERE Id = $2
";

//gather
pub const SELECT_USERS = "
    SELECT * FROM user
";
