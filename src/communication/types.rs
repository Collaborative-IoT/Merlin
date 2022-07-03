/*
All of the types that will come from or go with a specific request/response.
All requests/responses will start as a BasicRequest/BasicResponse, the true data
of the request/response(if there is any) will be the 'request_containing_data/response_containing_data'
in json serialization.

We try to follow snake case when possible in this document,
but our voice server requires camelcase json fields.
*/
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::option::Option;

//Gathering from client/sending to rabbitmq

#[derive(Deserialize, Serialize)]
pub struct BasicRequest {
    pub request_op_code: String,
    pub request_containing_data: String,
}

#[derive(Deserialize, Serialize, Debug)]
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
pub struct GetFollowListResponse {
    pub user_ids: Vec<FollowInfo>,
    pub for_user: i32,
}

#[derive(Deserialize, Serialize)]
pub struct BasicRoomCreation {
    pub name: String,
    pub desc: String,
    pub public: bool,
}

#[derive(Deserialize, Serialize)]
pub struct DeafAndMuteStatus {
    pub muted: bool,
    pub deaf: bool,
}

#[derive(Deserialize, Serialize)]
pub struct DeafAndMuteStatusUpdate {
    pub muted: bool,
    pub deaf: bool,
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
#[derive(Deserialize, Serialize)]
pub struct CommunicationRoom {
    pub details: RoomDetails,
    pub room_id: i32,
    pub num_of_people_in_room: i32,
    pub voice_server_id: String,
    pub creator_id: i32,
    pub people_preview_data: HashMap<i32, UserPreview>,
    pub auto_speaker_setting: bool,
    pub created_at: String,
    pub chat_mode: String,
}

#[derive(Deserialize, Serialize)]
pub struct AllUsersInRoomResponse {
    pub room_id: i32,
    pub users: Vec<User>,
}

#[derive(Deserialize, Serialize)]
pub struct RoomDetails {
    pub name: String,
    pub chat_throttle: i32,
    pub is_private: bool,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RoomPermissions {
    pub asked_to_speak: bool,
    pub is_speaker: bool,
    pub is_mod: bool,
}

#[derive(Deserialize, Serialize)]

pub struct UserPreview {
    pub display_name: String,
    pub avatar_url: String,
}

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub struct BlockUserFromRoom {
    pub user_id: i32,
    pub room_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct UnblockUserFromRoom {
    pub user_id: i32,
    pub room_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct ScheduledRoomUpdate {
    pub room_id: i32,
    pub name: String,
    pub scheduled_for: String,
    pub description: String,
}

#[derive(Deserialize, Serialize)]
pub struct RoomUpdate {
    pub name: String,
    pub public: bool,
    pub chat_throttle: i32,
    pub description: String,
    pub auto_speaker: bool,
}

#[derive(Deserialize, Serialize)]
pub struct GenericOnlyUserId {
    user_id: i32,
}

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
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

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct VoiceServerDestroyRoom {
    pub roomId: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct VoiceServerCreateRoom {
    pub roomId: String,
}
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct VoiceServerClosePeer {
    pub roomId: String,
    pub peerId: String,
    pub kicked: bool,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug)]
pub struct GenericRoomIdAndPeerId {
    pub roomId: i32,
    pub peerId: i32,
}

#[derive(Deserialize, Serialize)]
pub struct GenericRoomId {
    pub room_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct GenericUserId {
    pub user_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct VoiceServerRequest<T: Serialize> {
    pub op: String,
    pub d: T,
    pub uid: String,
}

#[derive(Deserialize, Serialize)]
pub struct AuthCredentials {
    pub access: String,
    pub refresh: String,
    pub oauth_type: String,
}

//these are optional because
//we may not have to exchange
//refresh for a new set.
#[derive(Deserialize, Serialize)]
pub struct AuthResponse {
    pub new_access: Option<String>,
    pub new_refresh: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct LooseUserPreviewRequest {
    pub user_ids: Vec<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct FollowInfo {
    pub user_id: i32,
    pub username: String,
    pub avatar_url: String,
    pub online: bool,
    pub room_id: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct InitRoomData {
    pub details: RoomDetails,
    pub creator_id: i32,
    pub auto_speaker_setting: bool,
    pub created_at: String,
    pub chat_mode: String,
}

/// What can you join as?
/// in this room?
#[derive(Deserialize, Serialize)]
pub struct JoinTypeInfo {
    pub as_speaker: bool,
    pub as_listener: bool,
    pub room_id: i32,
}

#[derive(Deserialize, Serialize)]
pub struct SingleUserDataResults {
    pub user_id: i32,
    pub data: User,
}

#[derive(Deserialize, Serialize)]
pub struct SingleUserPermissionResults {
    pub user_id: i32,
    pub data: RoomPermissions,
}

#[derive(Deserialize, Serialize)]
pub struct NewModStatus {
    pub new_status: bool,
    pub user_id: i32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Serialize)]
pub struct SpecialLeaveRoomOnDestroy {
    pub roomId: String,
}

#[derive(Deserialize, Serialize)]
pub struct NewIoTServer {
    pub external_id: String,
    pub owner_id: i32,
    pub outside_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct BlockedFromRoom {
    pub users: Vec<User>,
}

#[derive(Deserialize, Serialize)]
pub struct GiveOrRevokeIot {
    pub external_id: String,
    pub user_id: i32,
    pub now_has_permission: bool,
}
#[derive(Deserialize, Serialize)]
pub struct PassiveData {
    pub external_id: String,
    pub passive_data: String,
}
#[derive(Deserialize, Serialize)]
pub struct ExistingIotServer {
    pub owner_id: i32,
    pub external_id: String,
    pub controllers_of_room: Vec<i32>,
    pub passive_data_snap_shot: Option<String>,
    pub outside_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct NewIoTController {
    pub external_id: String,
    pub user_id: i32,
    pub outside_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct RemovedIoTController {
    pub external_id: String,
    pub user_id: i32,
    pub outside_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct RelationModification {
    pub modification_op: String,
    pub data: String,
    pub server_id: String,
}
