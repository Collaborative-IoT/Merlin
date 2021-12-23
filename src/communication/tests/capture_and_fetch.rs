use crate::communication::communication_types::User;
use crate::communication::data_capturer;
use crate::communication::data_capturer::CaptureResult;
use crate::communication::data_fetcher;
use crate::data_store::db_models::{
    DBFollower, DBRoom, DBRoomBlock, DBScheduledRoom, DBScheduledRoomAttendance, DBUser,
    DBUserBlock,
};
use crate::data_store::sql_execution_handler::ExecutionHandler;
use chrono::Utc;
use std::collections::HashSet;
use tokio_postgres::{Error, NoTls};

pub async fn test_capture_user(execution_handler: &mut ExecutionHandler) -> (i32, i32) {
    println!("testing capture user and gather");
    let new_user: DBUser = generate_user_struct();
    let new_second_user: DBUser = generate_different_user_struct();
    let first_capture_user_id: i32 =
        data_capturer::capture_new_user(execution_handler, &new_user).await;
    //try to capture the same one twice
    //to test against duplicates
    assert!(first_capture_user_id != -1); // -1 means issue or duplicate
    let second_capture_user_id: i32 =
        data_capturer::capture_new_user(execution_handler, &new_user).await;
    assert_eq!(second_capture_user_id, -1); // should be -1 due to the duplication
                                            //insert second user that should succeed
    let second_real_capture_user_id: i32 =
        data_capturer::capture_new_user(execution_handler, &new_second_user).await;
    assert!(second_real_capture_user_id != -1);
    return (first_capture_user_id, second_real_capture_user_id);
}

pub async fn test_follow_capture_and_gather(
    execution_handler: &mut ExecutionHandler,
    user_ids: (&i32, &i32),
) {
    println!("testing follow and capture gather");
    let new_follow: DBFollower = DBFollower {
        id: -1,
        follower_id: user_ids.1.clone(),
        user_id: user_ids.0.clone(),
    };
    let follow_capture_result: CaptureResult =
        data_capturer::capture_new_follower(execution_handler, &new_follow).await;
    assert_eq!(follow_capture_result.encountered_error, false);
    assert_eq!(follow_capture_result.desc, "Action Successful");

    //gather the following user
    //as the user who is being followed to test
    //if the user struct is being filled out correctly
    //since the user struct contains fields like "follows_you"
    let mock_user_ids_to_gather: Vec<i32> = vec![user_ids.1.clone()];
    let users_gather_result: (bool, Vec<User>) = data_fetcher::get_users_for_user(
        user_ids.0.clone(),
        mock_user_ids_to_gather,
        execution_handler,
    )
    .await;
    assert_eq!(users_gather_result.0, false);
    assert_eq!(users_gather_result.1.len(), 1);
    //we captured this exact user with its values in previous tests
    let old_user_that_was_captured = generate_different_user_struct();
    compare_user_and_db_user(&users_gather_result.1[0], &old_user_that_was_captured);
    assert_eq!(users_gather_result.1[0].follows_you, true);
    assert_eq!(users_gather_result.1[0].i_blocked_them, false);
    assert_eq!(users_gather_result.1[0].you_are_following, false);
}

pub async fn test_user_block_capture_and_gather(
    execution_handler: &mut ExecutionHandler,
    user_ids: (&i32, &i32),
) {
    println!("testing user block capture and gather");
    let user_block: DBUserBlock = DBUserBlock {
        id: -1 as i32,
        owner_user_id: user_ids.0.to_owned(),
        blocked_user_id: user_ids.1.to_owned(),
    };
    let capture_result: CaptureResult =
        data_capturer::capture_new_user_block(execution_handler, &user_block).await;
    assert_eq!(capture_result.encountered_error, false);
    assert_eq!(capture_result.desc, "Action Successful");

    //test against duplicates
    let second_capture_result: CaptureResult =
        data_capturer::capture_new_user_block(execution_handler, &user_block).await;
    assert_eq!(second_capture_result.encountered_error, true);
    assert_eq!(second_capture_result.desc, "Issue with execution");

    //check user struct gather properties
    //to make sure the user struct is being
    //filled out properly, since the second user should
    //be blocked from the POV of the first user
    let mock_user_ids_to_gather: Vec<i32> = vec![user_ids.1.clone()];
    let users_gather_result: (bool, Vec<User>) = data_fetcher::get_users_for_user(
        user_ids.0.clone(),
        mock_user_ids_to_gather,
        execution_handler,
    )
    .await;
    assert_eq!(users_gather_result.1.len(), 1);
    assert_eq!(users_gather_result.1[0].i_blocked_them, true);
}

pub async fn test_room_capture_and_gather(execution_handler: &mut ExecutionHandler) -> i32 {
    let mock_room: DBRoom = DBRoom {
        id: -1,
        owner_id: -222, //we only need the id for the further tests
        chat_mode: "fast".to_owned(),
    };
    let room_id: i32 = data_capturer::capture_new_room(execution_handler, &mock_room).await;
    assert!(room_id != -1);

    let gather_results: (bool, i32, String) =
        data_fetcher::get_room_owner_and_settings(execution_handler, &room_id).await;
    //no error
    assert_eq!(gather_results.0, false);
    //correct owner id and chatmode
    assert_eq!(gather_results.1, -222);
    assert_eq!(gather_results.2, "fast");
    return room_id;
}

pub async fn test_room_block_and_gather(execution_handler: &mut ExecutionHandler, room_id: &i32) {
    let room_block: DBRoomBlock = DBRoomBlock {
        id: -1,
        owner_room_id: room_id.clone(),
        blocked_user_id: -333,
    };
    let second_room_block: DBRoomBlock = DBRoomBlock {
        id: -1,
        owner_room_id: room_id.clone(),
        blocked_user_id: -444,
    };
    let first_capture_result: CaptureResult =
        data_capturer::capture_new_room_block(execution_handler, &room_block).await;
    let second_capture_result: CaptureResult =
        data_capturer::capture_new_room_block(execution_handler, &second_room_block).await;
    assert_eq!(first_capture_result.encountered_error, false);
    assert_eq!(first_capture_result.desc, "Action Successful");
    assert_eq!(second_capture_result.encountered_error, false);
    assert_eq!(second_capture_result.desc, "Action Successful");
    //test against duplicates
    let duplicate_capture_result: CaptureResult =
        data_capturer::capture_new_room_block(execution_handler, &room_block).await;
    assert_eq!(duplicate_capture_result.encountered_error, true);
    assert_eq!(duplicate_capture_result.desc, "Issue with execution");
    //test gathering new captured
    let fetch_result: (bool, HashSet<i32>) =
        data_fetcher::get_blocked_user_ids_for_room(execution_handler, room_id).await;
    let user_ids = fetch_result.1;
    assert_eq!(fetch_result.0, false);
    assert_eq!(user_ids.contains(&room_block.blocked_user_id), true);
    assert_eq!(user_ids.contains(&second_room_block.blocked_user_id), true);
}

fn compare_user_and_db_user(communication_user: &User, db_user: &DBUser) {
    assert_eq!(db_user.display_name, communication_user.display_name);
    assert_eq!(db_user.avatar_url, communication_user.avatar_url);
    assert_eq!(db_user.banner_url, communication_user.banner_url);
    assert_eq!(db_user.last_online, communication_user.last_online);
    assert_eq!(db_user.bio, communication_user.bio);
    assert_eq!(db_user.user_name, communication_user.username);
    assert_eq!(db_user.contributions, communication_user.contributions);
}

async fn generate_room_struct() -> DBRoom {
    return DBRoom {
        id: -1, //doesn't matter in insertion
        owner_id: 33,
        chat_mode: "slow".to_owned(),
    };
}

fn generate_user_struct() -> DBUser {
    let user: DBUser = DBUser {
        id: 0, //doesn't matter in insertion
        display_name: "test12".to_string(),
        avatar_url: "tes2t.com/avatar".to_string(),
        user_name: "ultima2ete_tester".to_string(),
        last_online: Utc::now().to_string(),
        github_id: "dw1".to_string(),
        discord_id: "2dwed".to_string(),
        github_access_token: "2323wed2".to_string(),
        discord_access_token: "29wedwed320".to_string(),
        banned: true,
        banned_reason: "ban evadding".to_string(),
        bio: "teeeest".to_string(),
        contributions: 40,
        banner_url: "test.com/dwtest_banner".to_string(),
    };
    return user;
}

fn generate_different_user_struct() -> DBUser {
    let user: DBUser = DBUser {
        id: 0, //doesn't matter in insertion
        display_name: "teseeeet12".to_string(),
        avatar_url: "test.cxexeeom/avatar2".to_string(),
        user_name: "ultimatxeeexe_tester2".to_string(),
        last_online: Utc::now().to_string(),
        github_id: "124555".to_string(),
        discord_id: "2234452".to_string(),
        github_access_token: "23diudi2322".to_string(),
        discord_access_token: "2ejnedjn93202".to_string(),
        banned: true,
        banned_reason: "ban evaejkeouding2".to_string(),
        bio: "teldmdst2".to_string(),
        contributions: 40,
        banner_url: "test.doijeoocom/test_banner2".to_string(),
    };
    return user;
}

pub async fn setup_execution_handler() -> Result<ExecutionHandler, Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres port=5432 password=password",
        NoTls,
    )
    .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("connection error: {}", e);
        }
    });
    let handler = ExecutionHandler::new(client);
    return Ok(handler);
}
