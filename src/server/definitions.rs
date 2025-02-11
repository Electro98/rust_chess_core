use std::collections::HashMap;
use std::sync::Arc;

use crate::{core::engine::Board, Color, DefaultExternalMove, Game};
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use warp::filters::ws::Message;

pub use uuid::Uuid;

#[allow(unused_imports)]
pub use log::{debug as debg, error as err, info as inf, trace as trc, warn as wrn};

pub type GameId = String;

pub struct Client {
    pub id: Uuid,
    pub sender: UnboundedSender<Result<Message, warp::Error>>,
    pub game_id: GameId,
}

pub struct OnlineGame {
    pub id: GameId,
    pub game: Game,
    pub white: Option<Client>,
    pub black: Option<Client>,
}

pub type Rooms = Arc<RwLock<HashMap<GameId, OnlineGame>>>;

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    OpponentConnected,
    OpponentDisconected,
    GameCanceled,
    GameFinished(Color),
    GameStateSync(Board, Color, Color, bool),
    RoomId(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    MakeMove(DefaultExternalMove),
}

impl OnlineGame {
    pub fn get_player(&self, player: Color) -> Option<&Client> {
        match player {
            Color::Black => self.black.as_ref(),
            Color::White => self.white.as_ref(),
        }
    }

    pub fn get_player_mut(&mut self, player: Color) -> &mut Option<Client> {
        match player {
            Color::Black => &mut self.black,
            Color::White => &mut self.white,
        }
    }
}

impl From<ServerMessage> for warp::ws::Message {
    fn from(value: ServerMessage) -> Self {
        Self::binary(to_allocvec(&value).unwrap())
    }
}
impl From<ClientMessage> for tungstenite::Message {
    fn from(value: ClientMessage) -> Self {
        Self::binary(to_allocvec(&value).unwrap())
    }
}

#[derive(Debug)]
pub enum ParsingMessageError {
    NonBinaryError,
    PostcardError(postcard::Error),
}

impl TryFrom<tungstenite::Message> for ServerMessage {
    type Error = ParsingMessageError;
    fn try_from(value: tungstenite::Message) -> Result<Self, Self::Error> {
        if value.is_binary() {
            match from_bytes(&value.into_data()) {
                Ok(result) => Ok(result),
                Err(err) => Err(ParsingMessageError::PostcardError(err)),
            }
        } else {
            Err(ParsingMessageError::NonBinaryError)
        }
    }
}

impl TryFrom<warp::ws::Message> for ClientMessage {
    type Error = ParsingMessageError;
    fn try_from(value: warp::ws::Message) -> Result<Self, Self::Error> {
        if value.is_binary() {
            match from_bytes(&value.into_bytes()) {
                Ok(result) => Ok(result),
                Err(err) => Err(ParsingMessageError::PostcardError(err)),
            }
        } else {
            Err(ParsingMessageError::NonBinaryError)
        }
    }
}
