use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Map of room_id to users in the room
pub type RoomMeta = HashMap<u32, HashSet<u32>>;

/// find the room_id containing the given user_id
pub async fn find_room_with_user(user_id: u32, rooms: Arc<Mutex<RoomMeta>>) -> Option<u32> {
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
