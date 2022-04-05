use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Map of room_id to users in the room
pub type RoomMeta = HashMap<u32, HashSet<u32>>;

/// find the room_id containing the given user_id
pub async fn find_room_with_user(user_id: u32, rooms: &Arc<Mutex<RoomMeta>>) -> Option<u32> {
    // TODO: avoid this clone
    let rooms = rooms.lock().await.clone().into_iter();
    // There should only ever be one room with the given queried user_id
    for room in rooms {
        if room.1.contains(&user_id) {
            return Some(room.0);
        }
    }

    None
}

/// adds the user_id to the given room_id
/// TODO: replace the option with an error
pub async fn add_user_to_room(user_id: u32, room_id: u32, rooms: &Arc<Mutex<RoomMeta>>) -> Option<()> {
    let mut rooms = rooms.lock().await;
    let room = rooms.get_mut(&room_id).unwrap();
    room.insert(user_id);
    None
}

/// creates a new empty room and return its room_id
pub async fn create_new_room(rooms: &Arc<Mutex<RoomMeta>>) -> Option<u32> {
    let mut rooms = rooms.lock().await;
    // TODO: use a UUID
    let new_room_id = 1u32;
    rooms.insert(new_room_id, HashSet::new());
    Some(new_room_id)
}
