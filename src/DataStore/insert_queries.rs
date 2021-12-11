use sea_query::*; //query builder
use chrono::{DateTime, Utc};
use db_models::{
    DBRoom,
    DBRoomPermissions,
    DBFollower,
    DBUser,
    DBUserBlock,
    DBRoomBlock,
    DBScheduledRoom,
    DBScheduledRoomAttendance
};

//Dry violations on purpose, helps read and follow each specific query
pub const insert_user_query = "
INSERT INTO user (
            displayName,
            avatarUrl,
            userName,
            lastOnline,
            githubId,
            discordId,
            githubAccessToken,
            discordAccessToken,
            banned,
            bannedReason,
            bio,
            contributions,
            bannerUrl)
            
            VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13) 
            RETURNING Id;
"
pub const insert_room_query = "
INSERT INTO room(ownerId,chatMode)
VALUES($1,$2) RETURNING Id;
"

pub const insert_room_permission_query = "
INSERT INTO room_permission(
    userId,
    roomId,
    isMod,
    isSpeaker,
    askedToSpeak)
VALUES($1,$2,$3,$4,$5);
";

pub const insert_follower_query = "
INSERT INTO follower(followerId,userId)
VALUES($1,$2);
";

pub const insert_user_block_query = "
INSERT INTO user_block(ownerUserId,blockedUserId)
VALUES($1,$2);
";

pub const insert_room_block_query = "
INSERT INTO room_block(ownerUserId,blockedUserId)
VALUES($1,$2);
";

pub const insert_scheduled_room_query = "
INSERT INTO scheduled_room( 
    roomName,
    numAttending,
    scheduledFor)
VALUES($1,$2,$3) RETURNING Id;
";

pub const insert_scheduled_attendance_query = "
INSERT INTO scheduled_room_attendance(userId,scheduledRoomId,isOwner)
VALUES($1,$2,$3);
";