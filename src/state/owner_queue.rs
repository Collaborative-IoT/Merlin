use std::collections::{HashMap, LinkedList};

use super::types::User;

/// The owner queue that represents who is
/// next in line to gather the owner role
/// of a room, in the event that the
/// owner leaves the room without
/// selecting a new owner.
pub struct OwnerQueue {
    pub user_queue: LinkedList<i32>,
    pub room_id: i32,
}

impl OwnerQueue {
    pub fn new(room_id: i32) -> Self {
        Self {
            user_queue: LinkedList::new(),
            room_id: room_id,
        }
    }

    /// Inserts new user into the
    /// queue of waiting users for
    /// the owner position.
    pub fn insert_new_user(&mut self, user_id: i32) {
        self.user_queue.push_back(user_id);
    }

    pub fn remove_all_invalid_users(&mut self, active_users: &HashMap<i32, User>) {
        let mut new_normal_queue: LinkedList<i32> = LinkedList::new();

        for item in self.user_queue.iter() {
            if let Some(user) = active_users.get(item) {
                if user.current_room_id == self.room_id {
                    new_normal_queue.push_back(item.clone());
                }
            }
        }
        self.user_queue = new_normal_queue;
    }

    pub fn find_new_owner(&mut self, active_users: &HashMap<i32, User>) -> Option<i32> {
        // Pop through all users until we find the next eligible.
        // An eligible user is someone who is in our room still.
        loop {
            if self.user_queue.len() == 0 {
                return None;
            }
            let next = self.user_queue.pop_front().unwrap();

            // If this user exists and they are in our current room.
            if let Some(user) = active_users.get(&next) {
                if user.current_room_id == self.room_id {
                    return Some(next);
                }
            }
        }
    }
}
