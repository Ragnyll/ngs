use std::collections::HashMap;
use std::sync::Arc;
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

#[cfg(test)]
pub mod test {
    use std::sync::Arc;
    use std::collections::HashMap;
    use super::RoomMeta;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_find_room_with_user() {
        let room_meta: Arc<Mutex<RoomMeta>> = Arc::new(Mutex::new(HashMap::new()));

        // find room with user does not find user if it's not found in any room
        assert!(super::find_room_with_user(1u32, &room_meta).await.is_none());

        // add the user to the room then make sure they can be found
        let expected_room_id = Uuid::new_v4();
        room_meta.lock().await.insert(1u32, expected_room_id);

        assert_eq!(super::find_room_with_user(1u32, &room_meta).await.unwrap(), expected_room_id);
    }
}
