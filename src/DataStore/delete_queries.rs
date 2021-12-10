use sea_query::*; //query builder
use chrono::{DateTime, Utc};

//Dry violations on purpose, helps read and follow each specific query
pub fn delete_room_query(room_id:i32)->String{
    let query = Query::delete()
        .from_table("room")
        .and_where(Expr::col("Id").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn delete_room_permissions_query(room_id:i32)->String{
    let query = Query::delete()
        .from_table("room_permission")
        .and_where(Expr::col("roomId").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
//ownerRoomId is the room that owns this block.
pub fn delete_room_blocks_query(room_id:i32)->String{
    let query = Query::delete()
        .from_table("room_block")
        .and_where(Expr::col("ownerRoomId").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn delete_scheduled_room_query(room_id:i32)->String{
    let query = Query::delete()
        .from_table("scheduled_room")
        .and_where(Expr::col("Id").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn delete_all_scheduled_room_attendance_query(room_id:i32)->String{
    let query = Query::delete()
        .from_table("scheduled_room_attendance")
        .and_where(Expr::col("Id").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}
pub fn delete_user_room_attendance_query(room_id,user_id)->String{
    let query = Query::delete()
        .from_table("scheduled_room_attendance")
        .and_where(Expr::col("scheduledRoomId").eq(room))
        .and_where(Expr::col("userId"))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}