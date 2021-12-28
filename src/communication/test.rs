use crate::communication::tests::capture_and_fetch;

/*
Tests aren't executed in a isolated pattern
due to the nature of the tests. These tests
are state driven and require the database instance
that we are testing on to have specific data
at each individual test. Some tests build on
other tests that come before them.
*/

pub async fn test() {
    test_capture_and_fetch().await;
}

async fn test_capture_and_fetch() {
    let execution_handler_result = capture_and_fetch::setup_execution_handler().await;
    let mut execution_handler = execution_handler_result.unwrap();
    let user_ids: (i32, i32) = capture_and_fetch::test_capture_user(&mut execution_handler).await;
    capture_and_fetch::test_follow_capture_and_gather(
        &mut execution_handler,
        (&user_ids.0, &user_ids.1),
    )
    .await;
    capture_and_fetch::test_user_block_capture_and_gather(
        &mut execution_handler,
        (&user_ids.0, &user_ids.1),
    )
    .await;
    let room_id = capture_and_fetch::test_room_capture_and_gather(&mut execution_handler).await;
    capture_and_fetch::test_room_owner_update_capture_and_gather(&mut execution_handler, &room_id)
        .await;
    let sch_room_id =
        capture_and_fetch::test_scheduled_room_capture_and_gather(&mut execution_handler).await;
    capture_and_fetch::test_scheduled_room_update_capture(&mut execution_handler, &sch_room_id)
        .await;
    let blocked_user_id =
        capture_and_fetch::test_room_block_and_gather(&mut execution_handler, &room_id).await;
    capture_and_fetch::test_room_permission_capture_and_gather(&mut execution_handler).await;
    capture_and_fetch::test_room_permission_update(&mut execution_handler).await;
    capture_and_fetch::test_user_follow_removal(&mut execution_handler, (&user_ids.0, &user_ids.1))
        .await;
    capture_and_fetch::test_user_block_removal(&mut execution_handler, (&user_ids.0, &user_ids.1))
        .await;
    capture_and_fetch::test_room_block_removal(&mut execution_handler, &room_id, blocked_user_id)
        .await;
    capture_and_fetch::test_room_removal(&mut &mut execution_handler, &room_id).await;
}
