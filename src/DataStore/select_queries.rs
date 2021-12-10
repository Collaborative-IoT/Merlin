use sea_query::*; //query builder
use chrono::{DateTime, Utc};

//Dry violations on purpose, helps read and follow each specific query
pub fn select_all_rooms_query()->String{
    let query = Query::select()
        .from("room")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_scheduled_rooms_query()->String{
    let query = Query::select()
        .from("scheduled_room")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_scheduled_room_attendance_for_room_query(scheduled_room_id:i32)->String{
    let query = Query::select()
        .from("scheduled_room_attendance")
        .and_where(Expr::col("Id").eq(scheduled_room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_attendances_for_user_query(user_id:i32)->String{
    let query = Query::select()
        .from("scheduled_room_attendance")
        .and_where(Expr::col("userId").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_followers_for_user_query(user_id:i32)->String{
    let query = Query::select()
        .from("follower")
        .and_where(Expr::col("userId").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_following_for_user_query(user_id:i32)->String{
    let query = Query::select()
        .from("follower")
        .and_where(Expr::col("followerId").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_blocked_for_user_query(user_id:i32)->String{
    let query = Query::select()
        .from("user_block")
        .and_where(Expr::col("ownerUserId").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_blockers_for_user_query(user_id:i32)->String{
    let query = Query::select()
        .from("user_block")
        .and_where(Expr::col("blockedUserId").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_all_blocked_users_for_room_query(room_id:i32)->String{
    let query = Query::select() 
        .from("room_block")
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn select_room_permissions_for_user_query(user_id:i32,room_id:i32)->String{
    let query = Query::select()
        .from("room_permission")
        .and_where(Expr::col("userId").eq(user_id))
        .and_where(Expr::col("roomId").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}