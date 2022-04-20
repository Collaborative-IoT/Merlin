use std::collections::HashMap;

use super::{owner_queue::OwnerQueue, types::User};

pub fn test_owners_queue() {
    let mut mock_queue = OwnerQueue::new(32);
    adding_and_removing(&mut mock_queue);
}

fn adding_and_removing(queue: &mut OwnerQueue) {
    let mut active_users: HashMap<i32, User> = HashMap::new();
    queue.insert_new_user(22);
    active_users.insert(
        22,
        User {
            current_room_id: 32,
            ..Default::default()
        },
    );
    // removing inactive users
    // 33 should be considered inactive because it isn't in our
    // mock active users map.
    queue.insert_new_user(33);
    assert_eq!(queue.user_queue.back().unwrap(), &33);
    assert_eq!(queue.user_queue.front().unwrap(), &22);
    assert!(queue.user_queue.len() == 2);
    queue.remove_all_invalid_users(&active_users);
    //this user is no longer present
    //but 22 still exist
    assert!(queue.user_queue.back().unwrap() != &33);
    assert_eq!(queue.user_queue.front().unwrap(), &22);

    // make sure we are getting the correct new owner
    // insert few invalid users
    queue.insert_new_user(33);
    queue.insert_new_user(44);
    let new_owner = queue.find_new_owner(&active_users);
    assert_eq!(new_owner.unwrap(), 22);
    // insert a valid user and check
    queue.insert_new_user(55);
    active_users.insert(
        55,
        User {
            current_room_id: 32,
            ..Default::default()
        },
    );
    let new_owner = queue.find_new_owner(&active_users);
    assert_eq!(new_owner.unwrap(), 55);
    // the old invalid users should have gotten cleared out
    // while looking for the new valid user
    assert!(queue.user_queue.len() == 0);
}
