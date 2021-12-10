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
pub fn insert_user_query(user:DBUser)->String{
    let query = Query::insert()
        .into_table("user")
        .columns(vec![
            "displayName",
            "avatarUrl",
            "userName",
            "lastOnline",
            "githubId",
            "discordId",
            "githubAccessToken",
            "discordAccessToken",
            "banned",
            "bannedReason",
            "bio",
            "contributions",
            "bannerUrl"])
        .values_panic(vec![
            user.display_name,
            user.avatar_url,
            user.user_name,
            user.last_online,
            user.github_id,
            user.discord_id,
            user.github_access_token,
            user.discord_access_token,
            user.banned,
            user.banned_reason,
            user.bio,
            user.contributions,
            user.bannerUrl
        ])
        .returning_col("Id")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_room_query(room:DBRoom)->String{
    let query = Query::insert()
        .into_table("room")
        .columns(vec!["ownerId","chatMode"])
        .values_panic(vec![room.owner_id,room.chat_mode])
        .returning_col("Id")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_room_permission_query(permissions:DBRoomPermissions)->String{
    let query = Query::insert()
        .into_table("room_permission")
        .columns(vec![
            "userId",
            "roomId",
            "isMod",
            "isSpeaker",
            "askedToSpeak"])
        .values_panic(vec![
            permissions.user_id,
            permissions.room_id,
            permissions.is_mod,
            permissions.is_speaker,
            permissions.asked_to_speak])
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_follower_query(follower:DBFollower)->String{
    let query = Query::insert()
        .into_table("follower")
        .columns(vec![
            "followerId",
            "userId"])
        .values_panic(vec![follower.follower_id,follower.user_id])
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_user_block_query(user_block:DBUserBlock)->String{
    let query = Query::insert()
        .into_table("user_block")
        .columns(vec![
            "ownerUserId",
            "blockedUserId"])
        .values_panic(vec![user_block.owner_user_id,user_block.blocked_user_id])
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_room_block_query(room_block:DBRoomBlock)->String{
    let query = Query::insert()
        .into_table("room_block")
        .columns(vec![
            "ownerUserId",
            "blockedUserId"])
        .values_panic(vec![room_block.owner_room_id,room_block.blocked_user_id])
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_scheduled_room_query(scheduled_room:DBScheduledRoom)->String{
    let query = Query::insert()
        .into_table("scheduled_room")
        .columns(vec![
            "roomName",
            "numAttending",
            "scheduledFor"])
        .values_panic(vec![
            scheduled_room.room_name,
            scheduled_room.num_attending,
            scheduled_room.scheduled_room])
        .returning_col("Id")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn insert_scheduled_attendance_query(attendance:DBScheduledRoomAttendance)->String{
    let query = Query::insert()
        .into_table("scheduled_room_attendance")
        .columns(vec![
            "userId",
            "scheduledRoomId",
            "isOwner"])
        .values_panic(vec![
            attendance.user_id,
            attendance.scheduled_room_id,
            attendance.is_owner])
        .returning_col("Id")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}