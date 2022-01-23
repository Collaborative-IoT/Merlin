/*
All of the types that will come from or go with a specific request/response.
All requests/responses will start as a BasicRequest/BasicResponse, the true data
of the request/response(if there is any) will be the 'request_containing_data/response_containing_data'
in json serialization.
*/
use serde::{Deserialize, Serialize};

use std::option::Option;

//Gathering from client/sending to rabbitmq

#[derive(Deserialize, Serialize)]
pub struct BasicRequest {
    pub request_op_code: String,
    pub request_containing_data: String,
}

#[derive(Deserialize, Serialize)]
pub struct BasicResponse {
    pub response_op_code: String,
    pub response_containing_data: String,
}

#[derive(Deserialize, Serialize)]
pub struct JoinRoomAndGetInfo {
    pub room_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct GetFollowList {
    pub user_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct UserRemovedFromRoom {
    pub user_id: i32,
    pub type_of_ban: String,
    pub requester: i32,
    pub room_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct GetUserProfile {
    pub user_id: i32,
}

//basic types
pub struct Room {
    details: RoomDetails,
    room_id: i32,
    num_of_people_in_room: i32,
    voice_server_id: i32,
    creator_id: i32,
    people_preview_data: Vec<UserPreview>,
    auto_speaker_setting: bool,
    created_at: String,
    chat_mode: String,
}

pub struct RoomDetails {
    name: bool,
    chat_throttle: i32,
    is_private: bool,
    description: String,
}

pub struct RoomPermissions {
    pub asked_to_speak: bool,
    pub is_speaker: bool,
    pub is_mod: bool,
}

pub struct MessageToken {
    type_of_token: String,
    value: String,
}

pub struct UserPreview {
    pub display_name: String,
    pub avatar_url: String,
}

pub struct UserProfileEdit {
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
}

pub struct RoomSettingsEditOrCreation {
    pub name: String,
    pub scheduled_for: String,
    pub description: String,
}

pub struct ScheduledRoomUpdate {
    pub room_id: i32,
    pub name: String,
    pub scheduled_for: String,
    pub description: String,
}

pub struct RoomUpdate {
    name: String,
    privacy: String,
    chat_throttle: i32,
    description: String,
    auto_speaker: bool,
}

pub struct GenericOnlyBool {
    value: bool,
}

pub struct GenericOnlyUserId {
    user_id: i32,
}

pub struct Mute {
    muted: bool,
}

pub struct ScheduledRoomGather {
    range: String,
    user_id: i32,
}

pub struct User {
    pub you_are_following: bool,
    pub username: String,
    pub they_blocked_you: bool,
    pub num_following: i32,
    pub num_followers: i32,
    pub last_online: String,
    pub user_id: i32,
    pub follows_you: bool,
    pub contributions: i32,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: String,
    pub banner_url: String,
    pub i_blocked_them: bool,
}

pub struct BaseUser {
    pub username: String,
    pub last_online: String,
    pub user_id: i32,
    pub bio: String,
    pub display_name: String,
    pub avatar_url: String,
    pub banner_url: String,
    pub num_following: i32,
    pub num_followers: i32,
    pub contributions: i32,
}

pub struct MessageBroadcastRequestDetails {
    tokens: Vec<MessageToken>,
    whispered_to: Vec<String>,
}

pub struct Message {
    id: String, //uuid
    user_id: i32,
    avatar_url: String,
    color: String,
    display_name: String,
    tokens: Vec<MessageToken>,
    username: String,
    deleted: bool,
    deleter_id: String,
    sent_at: String,
    is_whisper: bool,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerRemoveSpeaker {
    pub roomId: String,
    pub peerId: String,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerDestroyRoom {
    pub roomId: String,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerCreateRoom {
    pub roomId: String,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerAddSpeaker {
    pub roomId: String,
    pub peerId: String,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerClosePeer {
    pub roomId: String,
    pub peerId: String,
    pub kicked: bool,
}

#[derive(Deserialize, Serialize)]
pub struct GenericRoomIdAndPeerId {
    pub roomId: String,
    pub peerId: String,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerRequest<T: Serialize> {
    pub op: String,
    pub d: T,
    pub uid: String,
}
