use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Map of user_id to room_id in the room
pub type RoomMeta = HashMap<u32, Uuid>;

/// find the room_id containing the given user_id
pub async fn find_room_with_user(user_id: u32, rooms: &Arc<Mutex<RoomMeta>>) -> Option<Uuid> {
    log::trace!("Searching rooms {:?} for user_id {}", rooms, user_id);
    if let Some(room_id) = rooms.lock().await.get(&user_id) {
        log::trace!("Found user {} in room {}", user_id, room_id);
        return Some(*room_id);
    }

    // the queried user was not found in any room
    log::trace!("User {} was not found in any room", user_id);
    None
}

/// adds the user_id to the given room_id
pub async fn assign_user_to_room(user_id: u32, room_id: Uuid, user_rooms: &Arc<Mutex<RoomMeta>>) {
    user_rooms.lock().await.insert(user_id, room_id);
    log::trace!("Assigned user {} to room {:?}", user_id, room_id);
}

/// Removes the user from the specified room.
///
/// If the user was not present in the room it will
pub async fn remove_user_from_room(user_id: u32, room_id: u32, user_rooms: &Arc<Mutex<RoomMeta>>) {
    user_rooms.lock().await.remove(&user_id);
    log::trace!("User {} was unassiged from the room {}", user_id, room_id);
}

#[derive(Error, Debug)]
pub enum RoomError {}
