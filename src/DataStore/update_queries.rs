use sea_query::*; //query builder
use chrono::{DateTime, Utc};

pub fn update_room_owner_query(new_owner:i32,room_id:i32)-> String{
    let query = Query::update()
        .table("room")
        .values(vec![("ownerId",new_owner)])
        .and_where(Expr::col("Id").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}

pub fn update_room_mod_status_query(user_id:i32, is_mod:bool)-> String{
    let query = Query::update()
        .table("room_permissions")
        .values(vec![("isMod",is_mod)])
        .and_where(Expr::col("userId").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}

pub fn update_user_avatar_query(user_id:i32,avatar_url:&str)-> String{
    let query = Query::update()
        .table("user")
        .values(vec![("avatarUrl",avatar_url)])
        .and_where(Expr::col("Id").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}

pub fn update_display_name_query(user_id:i32,new_name:&str)-> String{
    let query = Query::update()
        .table("user")
        .values(vec![("displayName",new_name)])
        .and_where(Expr::col("Id").eq(user_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}

pub update_scheduled_room_query(num_attending:i32,scheduled_for:DateTime<Utc>, room_id:i32){
    let query = Query::update()
        .table("scheduled_room")
        .values(vec![("numAttending",num_attending),("scheduledFor",scheduled_for)])
        .and_where(Expr::col("Id").eq(room_id))
        .to_owned();
    return query.to_string(PostgresQueryBuilder);
}