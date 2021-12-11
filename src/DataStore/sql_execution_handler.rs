use tokio_postgres::Client;
use crate::DataStore::db_models::{
    DBRoom,
    DBRoomPermissions,
    DBFollower,
    DBUser,
    DBUserBlock,
    DBRoomBlock,
    DBScheduledRoom,
    DBScheduledRoomAttendance
};
use tokio_postgres::{NoTls, Error};
use crate::DataStore::select_queries;
use crate::DataStore::insert_queries;
use crate::DataStore::delete_queries;
use crate::DataStore::update_queries;
use crate::DataStore::creation_queries;

pub struct ExecutionHandler{
    client:Client,
    num_of_errors:i32,
    in_error_state:bool
}

//Handles the main sql execution by making usage of the DB types.
//dry violations on purpose, helps follow the data to the point of
//execution.
impl ExecutionHandler{
    pub fn new(client_val:Client)->Self{
        Self{
            client:client_val,
            num_of_errors:0,
            in_error_state:false
        }
    }

    pub async fn insert_user(&mut self,user:DBUser)->Result<i32,Error>{
        let query = insert_queries::INSERT_USER_QUERY;
        let rows = self.client.query(query,&[
                &user.display_name,
                &user.avatar_url,
                &user.user_name,
                &user.last_online,
                &user.github_id,
                &user.discord_id,
                &user.github_access_token,
                &user.discord_access_token,
                &user.banned,
                &user.banned_reason,
                &user.bio,
                &user.contributions,
                &user.banner_url
        ]).await?;
        let user_id:i32 = rows[0].get(0);
        return Ok(user_id);
    }

    pub async fn insert_room(&mut self,room:DBRoom)->Result<i32,Error>{
        let query = insert_queries::INSERT_ROOM_QUERY;
        let rows = self.client.query(query,&[&room.owner_id,&room.chat_mode]).await?;
        let room_id:i32 = rows[0].get(0);
        return Ok(room_id);
    }

    pub async fn insert_room_permission(&mut self, permissions:DBRoomPermissions)->Result<(),Error>{
        let query = insert_queries::INSERT_ROOM_PERMISSION_QUERY;
        self.client.query(query,&[
            &permissions.user_id,
            &permissions.room_id,
            &permissions.is_mod,
            &permissions.is_speaker,
            &permissions.asked_to_speak]).await?;
        Ok(())
    }

    pub async fn insert_follower(&mut self,follower:DBFollower)->Result<(),Error>{
        let query = insert_queries::INSERT_FOLLOWER_QUERY;
        self.client.query(query,&[
            &follower.follower_id,
            &follower.user_id]).await?;
        Ok(())
    }

    pub async fn insert_user_block(&mut self, user_block:DBUserBlock)->Result<(),Error>{
        let query = insert_queries::INSERT_USER_BLOCK_QUERY;
        self.client.query(query,&[
            &user_block.owner_user_id,
            &user_block.blocked_user_id]).await?;
        Ok(())
    }

    pub async fn insert_room_block(&mut self, room_block:DBRoomBlock)->Result<(),Error>{
        let query = insert_queries::INSERT_ROOM_BLOCK_QUERY;
        self.client.query(query,&[
            &room_block.owner_room_id,
            &room_block.blocked_user_id]).await?;
        Ok(())
    }

    pub async fn insert_scheduled_room(&mut self, scheduled_room:DBScheduledRoom)->Result<i32,Error>{
        let query = insert_queries::INSERT_SCHEDULED_ROOM_QUERY;
        let rows = self.client.query(query,&[
            &scheduled_room.room_name,
            &scheduled_room.num_attending,
            &scheduled_room.scheduled_for]).await?;
        let room_id:i32 = rows[0].get(0);
        return Ok(room_id);
    }

    pub async fn insert_scheduled_room_attendance(&mut self, scheduled_room_attendance:DBScheduledRoomAttendance)->Result<(),Error>{
        let query = insert_queries::INSERT_SCHEDULED_ATTENDANCE_QUERY;
        self.client.query(query,&[
            &scheduled_room_attendance.user_id,
            &scheduled_room_attendance.scheduled_room_id,
            &scheduled_room_attendance.is_owner
        ]).await?;
        Ok(())
    }
}