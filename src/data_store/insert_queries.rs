//Dry violations on purpose, helps read and follow each specific query
pub const INSERT_USER_QUERY: &str = "
INSERT INTO users (
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

pub const INSERT_ROOM_QUERY: &str = "
INSERT INTO room(ownerId,chatMode)
VALUES($1,$2) RETURNING Id;
";

pub const INSERT_ROOM_PERMISSION_QUERY: &str = "
INSERT INTO room_permission(
    userId,
    roomId,
    isMod,
    isSpeaker,
    askedToSpeak)
VALUES($1,$2,$3,$4,$5);
";

pub const INSERT_FOLLOWER_QUERY: &str = "
INSERT INTO follower(followerId,userId)
VALUES($1,$2);
";

pub const INSERT_USER_BLOCK_QUERY: &str = "
INSERT INTO user_block(ownerUserId,blockedUserId)
VALUES($1,$2);
";

pub const INSERT_ROOM_BLOCK_QUERY: &str = "
INSERT INTO room_block(ownerRoomId,blockedUserId)
VALUES($1,$2);
";

pub const INSERT_SCHEDULED_ROOM_QUERY: &str = "
INSERT INTO scheduled_room( 
    roomName,
    numAttending,
    scheduledFor,
    description)
VALUES($1,$2,$3,$4) RETURNING Id;
";

pub const INSERT_SCHEDULED_ATTENDANCE_QUERY: &str = "
INSERT INTO scheduled_room_attendance(userId,scheduledRoomId,isOwner)
VALUES($1,$2,$3);
";
