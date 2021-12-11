use chrono::{DateTime, Utc};

//Dry violations on purpose, helps read and follow each specific query
pub const insert_user_query: &str = "
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
";

pub const insert_room_query: &str = "
INSERT INTO room(ownerId,chatMode)
VALUES($1,$2) RETURNING Id;
";

pub const insert_room_permission_query: &str = "
INSERT INTO room_permission(
    userId,
    roomId,
    isMod,
    isSpeaker,
    askedToSpeak)
VALUES($1,$2,$3,$4,$5);
";

pub const insert_follower_query: &str = "
INSERT INTO follower(followerId,userId)
VALUES($1,$2);
";

pub const insert_user_block_query: &str = "
INSERT INTO user_block(ownerUserId,blockedUserId)
VALUES($1,$2);
";

pub const insert_room_block_query: &str = "
INSERT INTO room_block(ownerUserId,blockedUserId)
VALUES($1,$2);
";

pub const insert_scheduled_room_query: &str = "
INSERT INTO scheduled_room( 
    roomName,
    numAttending,
    scheduledFor)
VALUES($1,$2,$3) RETURNING Id;
";

pub const insert_scheduled_attendance_query: &str = "
INSERT INTO scheduled_room_attendance(userId,scheduledRoomId,isOwner)
VALUES($1,$2,$3);
";