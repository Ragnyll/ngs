use std::collections::{HashMap, HashSet};

/// Map of room_id to users in the room
type RoomMeta = HashMap<u32, HashSet<u32>>;

/// find the room_id containing the given user_id
pub fn find_room_with_user(user_id: u32, rooms: RoomMeta ) -> Option<u32> {
    // There should only ever be one room with the given queried user_id
    for room in rooms {
        if room.1.contains(&user_id) {
            return Some(room.0)
        }
    }

    None
}
