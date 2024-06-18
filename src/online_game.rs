use std::sync::mpsc;

use log::{debug, info, warn};
use url::Url;

use crate::{
    engine::Board,
    server::definitions::{ClientMessage, ServerMessage},
    Color, DarkGame,
};

pub type BackgroundThread = std::thread::JoinHandle<()>;

#[rifgen::rifgen_attr::generate_interface]
#[derive(Debug, Clone, Copy)]
pub enum MoveState {
    MyMove,
    MoveValidation,
    OpponentMove,
}

#[derive(Debug)]
pub struct Unconnected;
#[derive(Debug)]
pub struct Connecting {
    background_thread: BackgroundThread,
}
#[derive(Debug)]
pub struct WaitingOpponent {
    background_thread: BackgroundThread,
    pub game: DarkGame,
}
#[derive(Debug)]
pub struct GameInProgress {
    background_thread: BackgroundThread,
    pub game: DarkGame,
    pub state: MoveState,
    pub opponent_connected: bool,
}
#[derive(Debug)]
pub struct Canceled {
    pub reason: String,
}
#[derive(Debug)]
pub struct Finished {
    pub game: DarkGame,
    pub winner: Color,
}

#[derive(Debug)]
pub enum OnlineMatchState {
    InvalidDummy,
    Unconnected(Unconnected),
    Connecting(Connecting),
    WaitingOpponent(WaitingOpponent),
    GameInProgress(GameInProgress),
    Canceled(Canceled),
    Finished(Finished),
}

macro_rules! auto_from {
    ($en:ty, $st:ident) => {
        impl From<$st> for $en {
            fn from(val: $st) -> Self {
                Self::$st(val)
            }
        }
    };
}

auto_from!(OnlineMatchState, Unconnected);
auto_from!(OnlineMatchState, Connecting);
auto_from!(OnlineMatchState, WaitingOpponent);
auto_from!(OnlineMatchState, GameInProgress);
auto_from!(OnlineMatchState, Canceled);
auto_from!(OnlineMatchState, Finished);

impl Unconnected {
    pub fn connect(self, background_thread: BackgroundThread) -> Connecting {
        Connecting { background_thread }
    }
}

impl Default for OnlineMatchState {
    fn default() -> Self {
        Unconnected.into()
    }
}

impl OnlineMatchState {
    fn background_thread(self) -> Option<BackgroundThread> {
        match self {
            OnlineMatchState::InvalidDummy => unreachable!(),
            OnlineMatchState::Unconnected(_) => None,
            OnlineMatchState::Connecting(val) => Some(val.background_thread),
            OnlineMatchState::WaitingOpponent(val) => Some(val.background_thread),
            OnlineMatchState::GameInProgress(val) => Some(val.background_thread),
            OnlineMatchState::Canceled(_) => None,
            OnlineMatchState::Finished(_) => None,
        }
    }

    fn sync_state(
        background_thread: std::thread::JoinHandle<()>,
        board: Board,
        current_player: Color,
        you: Color,
        opponent_connected: bool,
    ) -> OnlineMatchState {
        debug!("Current player: {:?} I'm am {:?}", current_player, you);
        let game = DarkGame::new(board, current_player);
        if current_player == you || opponent_connected {
            GameInProgress {
                background_thread,
                game,
                state: if current_player == you {
                    MoveState::MyMove
                } else {
                    MoveState::OpponentMove
                },
                opponent_connected,
            }
            .into()
        } else {
            WaitingOpponent {
                background_thread,
                game,
            }
            .into()
        }
    }

    pub fn handle_message(self, msg: ServerMessage) -> OnlineMatchState {
        match (self, msg) {
            (OnlineMatchState::Connecting(_), ServerMessage::GameCanceled) => todo!(),
            (OnlineMatchState::WaitingOpponent(_), ServerMessage::OpponentConnected) => todo!(),
            (OnlineMatchState::GameInProgress(_), ServerMessage::GameCanceled) => Canceled {
                reason: "Game canceled by server".to_string(),
            }
            .into(),
            (OnlineMatchState::WaitingOpponent(_), ServerMessage::GameCanceled) => Canceled {
                reason: "Game canceled by server".to_string(),
            }
            .into(),
            (
                state,
                ServerMessage::GameStateSync(board, current_player, you, opponent_connected),
            ) if matches!(
                state,
                OnlineMatchState::GameInProgress(..) | OnlineMatchState::Connecting(..)
            ) =>
            {
                OnlineMatchState::sync_state(
                    state
                        .background_thread()
                        .expect("Bug! State don't have background thread!"),
                    board,
                    current_player,
                    you,
                    opponent_connected,
                )
            }
            (state, ServerMessage::RoomId(id)) => {
                info!(
                    "Unhandled room id message! Assuming debug purpose. Id: {}",
                    id
                );
                state
            }
            (state, msg) => {
                warn!("Invalid combination of message and state!");
                warn!("State: {:?}\nMsg: {:?}", &state, msg);
                state
            }
        }
    }
}
