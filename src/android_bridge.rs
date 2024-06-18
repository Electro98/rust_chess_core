use std::sync::mpsc::SyncSender;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use futures::channel::mpsc::Receiver;
use futures::executor::block_on;
use futures::SinkExt;
#[allow(unused_imports)]
use log::{debug, info, trace, warn};
use postcard::from_bytes;
use rifgen::rifgen_attr::generate_interface;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::{sleep, timeout};
use tokio_stream::StreamExt;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;
use url::Url;

use crate::game::ui_board;
use crate::online_game::*;
use crate::server::definitions::{ClientMessage, ServerMessage};
use crate::{Cell, Color, DefaultMove, Figure, Game, GameState, MatchInterface, PieceType};

pub struct WrapperGame {
    game: Game,
}

#[generate_interface]
pub enum GameStatus {
    Finished,
    MoveWhite,
    MoveBlack,
    DistantMoveWhite,
    DistantMoveBlack,
}

#[derive(Clone)]
pub struct WrapperMove {
    _move: DefaultMove,
}

#[derive(Clone)]
pub struct WrappedCell {
    cell: Cell,
}

pub type Moves = Vec<WrapperMove>;
pub type Cells = Vec<WrappedCell>;

impl WrapperGame {
    #[generate_interface(constructor)]
    pub fn new() -> WrapperGame {
        Self {
            game: Default::default(),
        }
    }

    #[generate_interface]
    pub fn make_move(&mut self, _move: WrapperMove) -> GameStatus {
        self.game.execute_move(_move._move).into()
    }

    #[generate_interface]
    pub fn possible_moves(&self, rank: usize, file: usize) -> Moves {
        debug!("Counting moves for r: {} f: {}", rank, file);
        let moves = self.game.possible_moves(rank, file);
        debug!(
            "Moves count: {}",
            moves.as_ref().map(|m| m.len()).unwrap_or(0)
        );
        moves
            .map(|moves| {
                moves
                    .into_iter()
                    .map(|inner| WrapperMove { _move: inner })
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    #[generate_interface]
    pub fn board(&self) -> Cells {
        ui_board(&self.game.vision_board(self.current_player()))
            .into_iter()
            .flatten()
            .map(WrappedCell::new)
            .collect()
    }

    #[generate_interface]
    pub fn current_player(&self) -> Color {
        self.game.current_player()
    }

    #[generate_interface]
    pub fn checked(&self) -> bool {
        self.game.checked()
    }

    #[generate_interface]
    pub fn game_ended(&self) -> bool {
        self.game.game_ended()
    }
}

impl WrapperMove {
    #[generate_interface]
    pub fn to_rank(&self) -> u32 {
        self._move.to.0
    }

    #[generate_interface]
    pub fn to_file(&self) -> u32 {
        self._move.to.1
    }

    #[generate_interface]
    pub fn from_rank(&self) -> u32 {
        self._move.from.0
    }

    #[generate_interface]
    pub fn from_file(&self) -> u32 {
        self._move.from.1
    }
}

impl From<GameState> for GameStatus {
    fn from(value: GameState) -> Self {
        match value {
            GameState::PlayerMove(color) => match color {
                Color::Black => Self::MoveBlack,
                Color::White => Self::MoveWhite,
            },
            GameState::DistantMove(color) => match color {
                Color::Black => Self::DistantMoveBlack,
                Color::White => Self::DistantMoveWhite,
            },
            GameState::Finished => Self::Finished,
        }
    }
}

impl WrappedCell {
    fn new(cell: Cell) -> Self {
        Self { cell }
    }

    #[generate_interface]
    pub fn hidden(&self) -> bool {
        matches!(self.cell, Cell::Unknown)
    }

    #[generate_interface]
    pub fn has_figure(&self) -> bool {
        matches!(self.cell, Cell::Figure(..))
    }

    fn figure(&self) -> Option<&Figure> {
        match &self.cell {
            Cell::Figure(fig) => Some(fig),
            _ => None,
        }
    }

    #[generate_interface]
    pub fn kind(&self) -> PieceType {
        self.figure().unwrap().kind.clone()
    }
    #[generate_interface]
    pub fn color(&self) -> Color {
        self.figure().unwrap().color
    }
    #[generate_interface]
    pub fn impose_check(&self) -> bool {
        self.figure().unwrap().impose_check
    }
    #[generate_interface]
    pub fn can_move(&self) -> bool {
        self.figure().unwrap().can_move
    }
}

#[generate_interface]
pub enum GlobalMatchState {
    Error,
    Unconnected,
    Connecting,
    WaitingOpponent,
    GameInProgress,
    Canceled,
    Finished,
}

impl From<&OnlineMatchState> for GlobalMatchState {
    fn from(value: &OnlineMatchState) -> Self {
        match value {
            OnlineMatchState::InvalidDummy => Self::Error,
            OnlineMatchState::Unconnected(_) => Self::Unconnected,
            OnlineMatchState::Connecting(_) => Self::Connecting,
            OnlineMatchState::WaitingOpponent(_) => Self::WaitingOpponent,
            OnlineMatchState::GameInProgress(_) => Self::GameInProgress,
            OnlineMatchState::Canceled(_) => Self::Canceled,
            OnlineMatchState::Finished(_) => Self::Finished,
        }
    }
}

#[generate_interface]
pub trait StateObserver {
    fn on_state_changed(&self, state: GlobalMatchState);
}

#[derive(Default)]
pub struct OnlineGame {
    state: Arc<Mutex<OnlineMatchState>>,
    socket: Option<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    obserser: Option<Box<dyn StateObserver>>,
    runtime: Option<Runtime>,
}

impl OnlineGame {
    #[generate_interface(constructor)]
    pub fn new() -> OnlineGame {
        Default::default()
    }

    #[generate_interface]
    pub fn install_observer(&mut self, observer: Box<dyn StateObserver>) {
        observer.on_state_changed(GlobalMatchState::Unconnected);
        self.obserser = Some(observer);
    }

    fn notify_observer(&self, state: &OnlineMatchState) {
        if let Some(observer) = &self.obserser {
            observer.on_state_changed(state.into())
        }
    }

    #[generate_interface]
    pub fn connect(&mut self, url: String) {
        trace!("Received url: {}", url);
        let url = Url::parse(&url);
        let state = &mut *self.state.lock().unwrap();
        if !matches!(state, OnlineMatchState::Unconnected(..)) {
            warn!("Trying to connect not in unconnected state!");
            return;
        }
        if url.is_err() {
            warn!("Failed to parse url!");
            return;
        }
        let old_value = std::mem::replace(state, OnlineMatchState::InvalidDummy);
        *state = if let OnlineMatchState::Unconnected(unconnected) = old_value {
            self.runtime = Some(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            );
            self.runtime.as_mut().unwrap().block_on(async {
                let (socket, _) = tokio_tungstenite::connect_async(url.unwrap())
                    .await
                    .or_else(|err| {
                        debug!("Failed to connect! Err: {}", err);
                        Err(())
                    })
                    .unwrap();
                trace!("Connected to server!");
                let socket = Arc::new(Mutex::new(socket));
                self.socket = Some(socket.clone());
                unconnected
                    .connect(Self::background_updater(self.state.clone(), socket))
                    .into()
            })
        } else {
            unreachable!();
        };
        trace!("Trying to notify observer!");
        self.notify_observer(state);
    }

    #[generate_interface]
    pub fn send_move(&mut self, _move: WrapperMove) {
        let state = &mut *self.state.lock().unwrap();
        self.runtime.as_mut().unwrap().block_on(async {
            let socket = &mut *self.socket.as_mut().unwrap().lock().unwrap();
            if socket
                .send(ClientMessage::MakeMove(_move._move).into())
                .await
                .is_err()
            {
                return;
            }
            let message = socket.try_next().await.unwrap().unwrap();
            let msg: ServerMessage = if let Message::Binary(bin) = message {
                from_bytes(&bin)
            } else {
                debug!("Server message: {}", message);
                return;
            }
            .unwrap();
            let old_state = std::mem::replace(state, OnlineMatchState::InvalidDummy);
            *state = old_state.handle_message(msg);
        });
        self.notify_observer(state);
    }

    fn background_updater(
        state: Arc<Mutex<OnlineMatchState>>,
        socket: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    ) -> BackgroundThread {
        let thread = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let result = update_in_background(state.clone(), socket).await;
                // lock and update state
                if let Err(err) = result {
                    let final_state = match err {
                        tungstenite::Error::ConnectionClosed => {
                            debug!("Connection closed");
                            Unconnected.into()
                        }
                        tungstenite::Error::AlreadyClosed => {
                            info!("Error in code, connection is already closed, but read/write again");
                            Unconnected.into()
                        }
                        err => {
                            warn!("Found error in connection: {}", err);
                            Canceled {
                                reason: "Error in connection".to_string(),
                            }
                            .into()
                        }
                    };
                    let mut locked = state.lock().unwrap();
                    *locked = final_state;
                }
            });
        });
        debug!("Successfully started new thread!");
        thread
    }

    #[generate_interface]
    pub fn check_state(&self) {
        let state = &*self.state.lock().unwrap();
        self.notify_observer(state)
    }

    #[generate_interface]
    pub fn player(&self) -> Color {
        let state = self.state.lock().unwrap();
        match &*state {
            OnlineMatchState::GameInProgress(game) => match game.state {
                MoveState::MyMove => game.game.current_player(),
                MoveState::MoveValidation => game.game.current_player(),
                MoveState::OpponentMove => game.game.current_player().opposite(),
            },
            _ => todo!(),
        }
    }

    #[generate_interface]
    pub fn current_player(&self) -> Color {
        let state = self.state.lock().unwrap();
        match &*state {
            OnlineMatchState::WaitingOpponent(game) => game.game.current_player(),
            OnlineMatchState::GameInProgress(game) => game.game.current_player(),
            _ => todo!(),
        }
    }

    #[generate_interface]
    pub fn sub_state(&self) -> MoveState {
        let state = self.state.lock().unwrap();
        match &*state {
            OnlineMatchState::GameInProgress(game) => game.state,
            _ => todo!(),
        }
    }

    #[generate_interface]
    pub fn possible_moves(&self, rank: usize, file: usize) -> Moves {
        let state = self.state.lock().unwrap();
        if let OnlineMatchState::GameInProgress(game) = &*state {
            debug!("Counting moves for r: {} f: {}", rank, file);
            let moves = game.game.possible_moves(rank, file);
            debug!(
                "Moves count: {}",
                moves.as_ref().map(|m| m.len()).unwrap_or(0)
            );
            moves
                .map(|moves| {
                    moves
                        .into_iter()
                        .map(|inner| WrapperMove { _move: inner })
                        .collect()
                })
                .unwrap_or_else(Vec::new)
        } else {
            todo!()
        }
    }

    #[generate_interface]
    pub fn board(&self) -> Cells {
        let state = self.state.lock().unwrap();
        if let OnlineMatchState::GameInProgress(game) = &*state {
            game.game
                .current_board()
                .into_iter()
                .flatten()
                .map(WrappedCell::new)
                .collect()
        } else {
            todo!()
        }
    }
}

async fn update_in_background(
    state: Arc<Mutex<OnlineMatchState>>,
    socket: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
) -> Result<(), tungstenite::Error> {
    loop {
        let _ = sleep(Duration::from_millis(200));
        let state = &mut *if let Ok(locked_state) = state.try_lock() {
            locked_state
        } else {
            continue;
        };
        let socket = &mut *socket.lock().unwrap();
        let msg = match timeout(Duration::from_millis(10), socket.try_next()).await {
            Ok(msg) => msg?.unwrap(),
            Err(_) => continue,
        };
        debug!("Server message: {}", msg);
        let msg: ServerMessage = if let Message::Binary(bin) = msg {
            from_bytes(&bin)
        } else {
            debug!("Server message: {}", msg);
            continue;
        }
        .unwrap();
        let old_state = std::mem::replace(state, OnlineMatchState::InvalidDummy);
        *state = old_state.handle_message(msg);
    }
}
