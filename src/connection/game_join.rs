#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// When a peer joins the server the first message they send must be a GameJoinRequest.
///
/// If the Game join request is not valid they cannot join a game and will be booted from the
/// server.
#[derive(Debug, Deserialize)]
pub struct GameJoinRequest {
    /// The user joing the game's userid
    pub user_id: String,

    /// The opponent being requested to play against.
    ///
    /// If the opponent_requested is None then the user_id is requesting a random opponent.
    pub opponent_request: Option<String>,
}

/// NGS will respond to the clients Join Requet with a GameJoinResponse, letting them know that
/// they have succesfully queued for a match or if their request is invalid or expired.
#[derive(Debug, Serialize)]
pub struct GameJoinResponse<'a> {
    /// The status of the user in the queue
    status: GameJoinStatus<'a>,

    /// The opponent to play against. Will be `None` if an opponent has not yet been found
    opponent: Option<&'a str>,
}

impl<'a> GameJoinResponse<'a> {
    /// Creates a new GameJoinResponse
    fn new(status: GameJoinStatus<'a>, opponent: Option<&'a str>) -> Self {
        Self { status, opponent }
    }
}

/// The various statuses representing a users status in the game join queue.
#[derive(Debug, Serialize)]
enum GameJoinStatus<'a> {
    /// The user has been accepted by the server, but is waiting for an opponent
    Waiting { msg: &'a str },
    /// The user has been accepted by the server and an opponent is ready
    Ready { msg: &'a str },
    /// The user's request to join the server has been timed out
    TimedOut { msg: &'a str },
    /// The user's legitimate request to join the server has been denied
    Denied { msg: &'a str },
}

/// Creates a response telling the user they have succesfully joined the server, but are awaiting
/// their opponent.
pub fn respond_wait<'a>() -> GameJoinResponse<'a> {
    GameJoinResponse::new(
        GameJoinStatus::Waiting {
            msg: "you have been accepted by the game server and are awaiting an opponent",
        },
        None,
    )
}
