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
use tokio_postgres::{Client,Error,row::Row};
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
//DRY VIOLATIONS on purpose!, helps follow the data to the point of
//execution.
impl ExecutionHandler{
    pub fn new(client_val:Client)->Self{
        Self{
            client:client_val,
            num_of_errors:0,
            in_error_state:false
        }
    }

    //creation
    pub async fn create_table_if_needed(&mut self,query:&str)-> Result<(),Error>{
        self.client.query(query,&[]).await?;
        return Ok(());
    }

    pub async fn create_all_tables_if_needed(&mut self)->Result<(),Error>{
        self.create_table_if_needed(creation_queries::ROOM_TABLE_CREATION).await?;
        self.create_table_if_needed(creation_queries::ROOM_PERMISSIONS_TABLE_CREATION).await?;
        self.create_table_if_needed(creation_queries::FOLLOWER_TABLE_CREATION).await?;
        self.create_table_if_needed(creation_queries::USER_TABLE_CREATION).await?;
        self.create_table_if_needed(creation_queries::USER_BLOCK_TABLE_CREATION).await?;
        self.create_table_if_needed(creation_queries::ROOM_BLOCK_CREATION).await?;
        self.create_table_if_needed(creation_queries::SCHEDULED_ROOM_CREATION).await?;
        self.create_table_if_needed(creation_queries::SHEDULED_ROOM_ATTENDANCE).await?;
        return Ok(());
    }

    // insertion
    pub async fn insert_user(&mut self,user:&DBUser)->Result<i32,Error>{
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
        return Ok(());
    }

    pub async fn insert_follower(&mut self,follower:DBFollower)->Result<(),Error>{
        let query = insert_queries::INSERT_FOLLOWER_QUERY;
        self.client.query(query,&[
            &follower.follower_id,
            &follower.user_id]).await?;
        return Ok(());
    }

    pub async fn insert_user_block(&mut self, user_block:DBUserBlock)->Result<(),Error>{
        let query = insert_queries::INSERT_USER_BLOCK_QUERY;
        self.client.query(query,&[
            &user_block.owner_user_id,
            &user_block.blocked_user_id]).await?;
        return Ok(());
    }

    pub async fn insert_room_block(&mut self, room_block:DBRoomBlock)->Result<(),Error>{
        let query = insert_queries::INSERT_ROOM_BLOCK_QUERY;
        self.client.query(query,&[
            &room_block.owner_room_id,
            &room_block.blocked_user_id]).await?;
        return Ok(());
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
        return Ok(());
    }

    //deletion
    pub async fn delete_room(&mut self, room_id:&i32)->Result<u64,Error>{
        let query = delete_queries::DELETE_ROOM_QUERY;
        let num_modified = self.client.execute(query,&[room_id]).await?;
        return Ok(num_modified);
    }

    pub async fn delete_all_room_permissions(&mut self, room_id:&i32)->Result<u64,Error>{
        let query = delete_queries::DELETE_ROOM_PERMISSIONS_QUERY;
        let num_modified = self.client.execute(query,&[room_id]).await?;
        return Ok(num_modified);
    }

    pub async fn delete_room_blocks(&mut self, room_id:&i32)->Result<u64,Error>{
        let query = delete_queries::DELETE_ROOM_BLOCKS_QUERY;
        let num_modified =self.client.execute(query,&[room_id]).await?;
        return Ok(num_modified);
    }

    pub async fn delete_scheduled_room(&mut self, room_id:&i32)->Result<u64,Error>{
        let query = delete_queries::DELETE_SCHEDULED_ROOM_QUERY;
        let num_modified =self.client.execute(query,&[room_id]).await?;
        return Ok(num_modified);
    }

    pub async fn delete_all_scheduled_room_attendance(&mut self, room_id:&i32)->Result<u64,Error>{
        let query = delete_queries::DELETE_ALL_SCHEDULED_ROOM_ATTENDANCE_QUERY;
        let num_modified = self.client.execute(query,&[room_id]).await?;
        return Ok(num_modified);
    }

    pub async fn delete_user_room_attendance(&mut self, user_id:&i32,room_id:&i32)->Result<u64,Error>{
        let query = delete_queries::DELETE_USER_ROOM_ATTENDANCE_QUERY;
        let num_modified = self.client.execute(query,&[room_id,user_id]).await?;
        return Ok(num_modified);
    }

    //update
    pub async fn update_room_owner_query(&mut self, room_id:&i32,new_owner_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_ROOM_OWNER_QUERY;
        let num_modified = self.client.execute(query,&[&new_owner_id,room_id]).await?;
        return Ok(num_modified);
    }
    //sets user to mod or not mod for a room
    pub async fn update_room_mod_status(&mut self, room_id:&i32,user_id:&i32,is_mod:bool)->Result<u64,Error>{
        let query = update_queries::UPDATE_ROOM_MOD_STATUS_QUERY;
        let num_modified = self.client.execute(query,&[&is_mod,room_id,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_user_avatar(&mut self, avatar_url:String, user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_USER_AVATAR_QUERY;
        let num_modified = self.client.execute(query,&[&avatar_url,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_display_name(&mut self, display_name:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_DISPLAY_NAME_QUERY;
        let num_modified = self.client.execute(query,&[&display_name,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_scheduled_room(
        &mut self, 
        num_attending:&i32,
        scheduled_for:String,
        room_id:&i32)->Result<u64,Error>{
            let query = update_queries::UPDATE_SCHEDULED_ROOM_QUERY;
            let num_modified = self.client.execute(query,&[num_attending,&scheduled_for,room_id]).await?;
            return Ok(num_modified);
        }
    
    pub async fn update_ban_status_of_user(
        &mut self, 
        banned:bool,
        banned_reason:String,
        user_id:&i32)->Result<u64,Error>{
            let query = update_queries::BAN_USER_QUERY;
            let num_modified = self.client.execute(query,&[&banned,&banned_reason,user_id]).await?;
            return Ok(num_modified);
    }

    pub async fn update_user_bio(&mut self,bio:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_BIO_QUERY;
        let num_modified = self.client.execute(query,&[&bio,&user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_github_access_token(&mut self,new_token:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_GITHUB_ACCESS_TOKEN_QUERY;
        let num_modified = self.client.execute(query,&[&new_token,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_discord_access_token(&mut self,new_token:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_DISCORD_ACCESS_TOKEN_QUERY;
        let num_modified = self.client.execute(query,&[&new_token,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_contributions(&mut self,new_contributions:&i32,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_CONTRIBUTIONS_QUERY;
        let num_modified = self.client.execute(query,&[&new_contributions,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_banner_url(&mut self, new_banner_url:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_BANNER_URL_QUERY;
        let num_modified = self.client.execute(query,&[&new_banner_url,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_last_online(&mut self, new_last_online:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_LAST_ONLINE_QUERY;
        let num_modified = self.client.execute(query,&[&new_last_online,user_id]).await?;
        return Ok(num_modified);
    }

    pub async fn update_user_name(&mut self, new_user_name:String,user_id:&i32)->Result<u64,Error>{
        let query = update_queries::UPDATE_USER_NAME_QUERY;
        let num_modified = self.client.execute(query,&[&new_user_name,user_id]).await?;
        return Ok(num_modified);
    }

    //select
    pub async fn select_all_rooms(&mut self)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_ROOM_QUERY;
        let result:Vec<Row> = self.client.query(query,&[]).await?;
        return Ok(result);
    }

    pub async fn select_all_scheduled_rooms(&mut self)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_SCHEDULED_ROOMS_QUERY;
        let result:Vec<Row> = self.client.query(query,&[]).await?;
        return Ok(result);
    }

    pub async fn select_all_attendance_for_scheduled_room(&mut self, room_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_SCHEDULED_ROOM_ATTENDANCE_FOR_ROOM_QUERY;
        let result:Vec<Row> = self.client.query(query,&[room_id]).await?;
        return Ok(result);
    }

    pub async fn select_all_room_attendance_for_user(&mut self,user_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_ATTENDANCE_FOR_USER_QUERY;
        let result:Vec<Row> = self.client.query(query,&[user_id]).await?;
        return Ok(result);
    }

    pub async fn select_all_followers_for_user(&mut self, user_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_FOLLOWERS_FOR_USER_QUERY;
        let result:Vec<Row> = self.client.query(query,&[user_id]).await?;
        return Ok(result);
    }

    pub async fn select_all_following_for_user(&mut self,user_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_FOLLOWING_FOR_USER_QUERY;
        let result:Vec<Row> = self.client.query(query,&[user_id]).await?;
        return Ok(result);
    }

    pub async fn select_all_blocked_for_user(&mut self, user_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_BLOCKED_FOR_USER_QUERY;
        let result:Vec<Row> = self.client.query(query,&[user_id]).await?;
        return Ok(result);
    }

    pub async fn select_all_blockers_for_user(&mut self, user_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_BLOCKERS_FOR_USER_QUERY;
        let result:Vec<Row> = self.client.query(query,&[user_id]).await?;
        return Ok(result);
    }

    pub async fn select_all_blocked_users_for_room(&mut self,room_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_BLOCKED_USERS_FOR_ROOM_QUERY;
        let result:Vec<Row> = self.client.query(query,&[room_id]).await?;
        return Ok(result);
    }

    //SELECTS permissions for one room for one user
    pub async fn select_all_room_permissions_for_user(&mut self,user_id:&i32,room_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_ALL_ROOM_PERMISSIONS_FOR_USER;
        let result:Vec<Row> = self.client.query(query,&[user_id,room_id]).await?;
        return Ok(result);
    }

    pub async fn select_user_by_id(&mut self, user_id:&i32)->Result<Vec<Row>,Error>{
        let query = select_queries::SELECT_USER_BY_ID;
        let result:Vec<Row> = self.client.query(query,&[user_id]).await?;
        return Ok(result);
    }
}