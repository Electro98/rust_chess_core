use std::{
    borrow::BorrowMut,
    env::current_exe,
    sync::{
        mpsc::{self, SendError},
        Arc, Mutex, MutexGuard,
    },
    time::Duration,
};

use chess_engine::{
    engine::Move as ImplMove, Cell as FieldCell, Color, DefaultMove, Game, GameState,
    MatchInterface,
};
use chess_engine::{server::definitions::ClientMessage, Figure};
use eframe::egui::{self, Vec2};
use gui::{background_color, piece_image};
use postcard::from_bytes;
use tungstenite::{stream::NoDelay, Message};
use url::Url;

mod gui;

#[allow(unused_imports)]
pub use log::{debug as debg, error as err, info as inf, trace as trc, warn as wrn};

#[derive(Debug)]
pub struct Unconnected;
#[derive(Debug)]
pub struct Connecting {
    client_thread: std::thread::JoinHandle<()>,
}
#[derive(Debug)]
pub struct WaitingOpponent {
    client_thread: std::thread::JoinHandle<()>,
    game: Game,
}
#[derive(Debug)]
pub struct PlayerMove {
    client_thread: std::thread::JoinHandle<()>,
    game: Game,
}
#[derive(Debug)]
pub struct MoveValidation {
    client_thread: std::thread::JoinHandle<()>,
    game: Game,
}
#[derive(Debug)]
pub struct OpponentMove {
    client_thread: std::thread::JoinHandle<()>,
    game: Game,
}
#[derive(Debug)]
pub struct Canceled {
    pub reason: String,
}
#[derive(Debug)]
pub struct Finished {
    game: Game,
    winner: Color,
}

impl Unconnected {
    fn start_connection(
        self,
        url: Url,
        online_match: Arc<Mutex<OnlineMatchState>>,
        tx: mpsc::Receiver<ClientMessage>,
    ) -> Connecting {
        Connecting {
            client_thread: std::thread::spawn(move || {
                let result = game_client(url, &online_match, tx);
                if let Err(err) = result {
                    let final_state = match err {
                        tungstenite::Error::ConnectionClosed => {
                            debg!("Connection closed");
                            Unconnected.into()
                        }
                        tungstenite::Error::AlreadyClosed => {
                            inf!(
                                "Error in code, connection is already closed, but read/write again"
                            );
                            Unconnected.into()
                        }
                        err => {
                            wrn!("Found error in connection: {}", err);
                            Canceled {
                                reason: "Error in connection".to_string(),
                            }
                            .into()
                        }
                    };
                    let mut locked = online_match.lock().unwrap();
                    *locked = final_state;
                }
            }),
        }
    }
}

impl Connecting {
    fn first_msg(self, msg: Vec<u8>) -> OnlineMatchState {
        let msg: ClientMessage = match from_bytes(msg.as_slice()) {
            Ok(msg) => msg,
            Err(err) => {
                err!("Failed to parse first message from server: {}", err);
                return self.into();
            }
        };
        match msg {
            ClientMessage::GameStateSync(board, current_player, you, opponent_connected) => {
                let game = Game::with_player(board, current_player);
                match (current_player == you, opponent_connected) {
                    (false, true) => OpponentMove {
                        client_thread: self.client_thread,
                        game,
                    }
                    .into(),
                    (false, false) => WaitingOpponent {
                        client_thread: self.client_thread,
                        game,
                    }
                    .into(),
                    _ => PlayerMove {
                        client_thread: self.client_thread,
                        game,
                    }
                    .into(),
                }
            }
            _ => todo!("Got unexpected first message!"),
        }
    }
}

impl Into<OnlineMatchState> for Unconnected {
    fn into(self) -> OnlineMatchState {
        OnlineMatchState::Unconnected(self)
    }
}
impl Into<OnlineMatchState> for Connecting {
    fn into(self) -> OnlineMatchState {
        OnlineMatchState::Connecting(self)
    }
}
impl Into<OnlineMatchState> for WaitingOpponent {
    fn into(self) -> OnlineMatchState {
        OnlineMatchState::WaitingOpponent(self)
    }
}
impl Into<OnlineMatchState> for PlayerMove {
    fn into(self) -> OnlineMatchState {
        OnlineMatchState::PlayerMove(self)
    }
}
impl Into<OnlineMatchState> for OpponentMove {
    fn into(self) -> OnlineMatchState {
        OnlineMatchState::OpponentMove(self)
    }
}
impl Into<OnlineMatchState> for Canceled {
    fn into(self) -> OnlineMatchState {
        OnlineMatchState::Canceled(self)
    }
}

#[derive(Debug)]
enum OnlineMatchState {
    #[allow(dead_code)]
    InvalidDummy,
    Unconnected(Unconnected),
    Connecting(Connecting),
    WaitingOpponent(WaitingOpponent),
    PlayerMove(PlayerMove),
    OpponentMove(OpponentMove),
    MoveValidation(MoveValidation),
    Canceled(Canceled),
    Finished(Finished),
}

impl Default for OnlineMatchState {
    fn default() -> Self {
        Unconnected.into()
    }
}

struct OnlineClient {
    pub online_match: Arc<Mutex<OnlineMatchState>>,
    tx: Option<mpsc::Sender<ClientMessage>>,
}

impl Default for OnlineClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OnlineClient {
    fn new() -> Self {
        Self {
            online_match: Default::default(),
            tx: None,
        }
    }
    fn connect(&mut self, url: Url) {
        let (tx, rx) = mpsc::channel();
        self.tx = Some(tx);
        let online_match = &mut *self.online_match.lock().unwrap();
        let old_value = std::mem::replace(online_match, OnlineMatchState::InvalidDummy);
        *online_match = if let OnlineMatchState::Unconnected(internals) = old_value {
            internals
                .start_connection(url, self.online_match.clone(), rx)
                .into()
        } else {
            old_value
        };
    }
    fn send_move(&self, _move: DefaultMove) -> Result<bool, SendError<ClientMessage>> {
        let state = &*self.online_match.lock().unwrap();
        match state {
            OnlineMatchState::PlayerMove(_) => self
                .tx
                .as_ref()
                .expect("Sender is not initialized, but waiting for player to move?!")
                .send(ClientMessage::MakeMove(_move))
                .and_then(|_| Ok(true)),
            _ => {
                wrn!("Client want to send move in incorrect state: {:?}", state);
                Ok(false)
            },
        }
    }
    fn get_game(&self) -> Option<Game> {
        match &*self.online_match.lock().unwrap() {
            OnlineMatchState::PlayerMove(mov) => Some(mov.game.clone()),
            OnlineMatchState::OpponentMove(mov) => Some(mov.game.clone()),
            OnlineMatchState::MoveValidation(val) => Some(val.game.clone()),
            OnlineMatchState::Finished(fin) => Some(fin.game.clone()),
            _ => None,
        }
    }
}

fn message_received(state: OnlineMatchState, msg: ClientMessage) -> OnlineMatchState {
    match (state, msg) {
        (OnlineMatchState::Connecting(_), ClientMessage::GameCanceled) => todo!(),
        (OnlineMatchState::WaitingOpponent(_), ClientMessage::OpponentConnected) => todo!(),
        (OnlineMatchState::PlayerMove(_), ClientMessage::GameCanceled) => todo!(),
        (OnlineMatchState::OpponentMove(_), ClientMessage::OpponentDisconected) => todo!(),
        (OnlineMatchState::OpponentMove(_), ClientMessage::GameCanceled) => todo!(),
        (OnlineMatchState::OpponentMove(_), ClientMessage::GameFinished(_)) => todo!(),
        (OnlineMatchState::OpponentMove(_), ClientMessage::GameStateSync(_, _, _, _)) => todo!(),
        (OnlineMatchState::MoveValidation(_), ClientMessage::GameCanceled) => todo!(),
        (OnlineMatchState::MoveValidation(_), ClientMessage::GameStateSync(_, _, _, _)) => todo!(),
        (state, msg) => {
            wrn!("Invalid combination of message and state!");
            wrn!("Msg: {:#?} State {:#?}", &state, msg);
            state
        }
    }
}

fn game_client(
    url: Url,
    match_state: &Arc<Mutex<OnlineMatchState>>,
    tx: mpsc::Receiver<ClientMessage>,
) -> Result<(), tungstenite::Error> {
    let (mut socket, response) = tungstenite::connect(url)?;

    let initial_message = socket.read()?;

    if let Message::Binary(bytes) = initial_message {
        let state = &mut *match_state.lock().unwrap();
        let old_value = std::mem::replace(state, OnlineMatchState::InvalidDummy);
        assert!(matches!(old_value, OnlineMatchState::Connecting(..)));
        let connecting = if let OnlineMatchState::Connecting(val) = old_value {
            val
        } else {
            todo!()
        };
        *state = connecting.first_msg(bytes);
    }

    socket.split();

    loop {
        let player_msg = {
            let state = &*match_state.lock().unwrap();
            match state {
                OnlineMatchState::PlayerMove(_) => {
                    match tx.recv_timeout(Duration::from_millis(10)) {
                        Ok(msg) => Some(msg),
                        Err(_) => None,
                    }
                }
                _ => None,
            }
        };
        if let Some(player_msg) = player_msg {
            let _ = socket.write(player_msg.into());
        }
        let msg = socket.read()?;
        trc!("Received server msg: {}", msg);
        let msg = if let Message::Binary(bytes) = msg {
            from_bytes(bytes.as_slice())
        } else {
            todo!("Unexpected type of message received!")
        }
        .unwrap();
        let state = &mut *match_state.lock().unwrap();
        let old_state = std::mem::replace(state, OnlineMatchState::InvalidDummy);
        *state = message_received(old_state, msg);
    }

    Ok(())
}

enum SavedData {
    None,
    ChosenFigure((usize, usize)),
    InputText(String),
}

impl Default for SavedData {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
struct App {
    client: OnlineClient,
    saved_data: SavedData,
}

const CELL_SIZE: f32 = 45.0;

fn main() -> Result<(), eframe::Error> {
    println!("It's client!");
    env_logger::init();

    let app: App = Default::default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Test online client",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(app)
        }),
    )
}

enum ScreenData {
    ConnectScreen,
    GameScreen(Game),
    WaitScreen(String),
    ErrorScreen(Option<String>),
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let screen = {
            let state = self.client.online_match.lock().unwrap();
            match &*state {
                OnlineMatchState::Unconnected(_) => ScreenData::ConnectScreen,
                OnlineMatchState::Connecting(_) => ScreenData::WaitScreen("Connecting".into()),
                OnlineMatchState::WaitingOpponent(_) => {
                    ScreenData::WaitScreen("Opponent is not connected".into())
                }
                OnlineMatchState::PlayerMove(state) => ScreenData::GameScreen(state.game.clone()),
                OnlineMatchState::OpponentMove(state) => ScreenData::GameScreen(state.game.clone()),
                OnlineMatchState::MoveValidation(_) => {
                    ScreenData::WaitScreen("Move is validating".into())
                }
                OnlineMatchState::Finished(fin) => ScreenData::ErrorScreen(Some(format!(
                    "Game finished, winner: {:?}",
                    fin.winner
                ))),
                OnlineMatchState::Canceled(canceled) => {
                    ScreenData::ErrorScreen(canceled.reason.clone().into())
                }
                OnlineMatchState::InvalidDummy => todo!(),
            }
        };
        match screen {
            ScreenData::ConnectScreen => {
                let url = self.connect_screen(ctx, frame);
                if let Some(url) = url {
                    self.client.connect(url);
                }
            }
            ScreenData::GameScreen(game) => {
                let _move = self.game_screen(ctx, frame, game);
                if let Some(_move) = _move {
                    trc!("Sending move: {:?}", _move);
                    self.saved_data = SavedData::None;
                    let _ = self.client.send_move(_move);
                }
            }
            ScreenData::WaitScreen(text) => self.wait_screen(ctx, frame, text),
            ScreenData::ErrorScreen(reason) => self.error_screen(ctx, frame, reason),
        }
    }
}

impl App {
    fn connect_screen(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> Option<Url> {
        let mut url = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello!");
            if !matches!(&self.saved_data, SavedData::InputText(..)) {
                self.saved_data = SavedData::InputText(String::new())
            }
            let text = if let SavedData::InputText(text) = &mut self.saved_data {
                text
            } else {
                panic!("Hmm")
            };
            ui.text_edit_singleline(text);
            let resp = ui.button("Connect!");
            if resp.clicked() {
                let text: String = if text.is_empty() {
                    "ws://127.0.0.1:3030/ws".to_string()
                } else {
                    format!("ws://127.0.0.1:3030/ws/{}", text)
                };
                url = Url::parse(&text).and_then(|x| Ok(Some(x))).unwrap_or(None);
            }
        });
        url
    }

    fn game_screen(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        game: Game,
    ) -> Option<DefaultMove> {
        let mut new_click = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("That's game screen");
            new_click = self.grid(ui, frame, &game);
        });
        if new_click.is_some() {
            trc!("Clicked: {:?}", new_click);
        }
        if let SavedData::ChosenFigure((rank, file)) = &self.saved_data {
            if new_click.is_none() {
                return None;
            }
            match game
                .cell(*file, *rank)
                .expect("Invalid file and/or rank, bug")
            {
                FieldCell::Figure(_) => {
                    let _move = game.possible_moves(*rank as u32, *file as u32)
                        .and_then(|moves| {
                            moves
                                .into_iter()
                                .into_iter()
                                .find(|_move| _move.to == new_click.unwrap())
                                .and_then(|_move| Some(_move))
                        });
                    if _move.is_none() {
                        self.saved_data = SavedData::None;
                    }
                    _move
                }
                _ => {
                    self.saved_data = SavedData::None;
                    None
                }
            }
        } else if let Some((rank, file)) = new_click {
            self.saved_data = SavedData::ChosenFigure((rank as usize, file as usize));
            None
        } else {
            None
        }
    }

    fn wait_screen(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, text: String) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(text);
        });
    }

    fn error_screen(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        reason: Option<String>,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(text) = reason {
                ui.label("Here is something happen, check it out!");
                ui.label(text);
            } else {
                ui.label("Something went terribly wrong!");
            }
        });
    }

    fn grid(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
        game: &Game,
    ) -> Option<(u32, u32)> {
        let chosen_figure = if let SavedData::ChosenFigure(fig) = &self.saved_data {
            Some(fig)
        } else {
            None
        };
        let possible_moves =
            chosen_figure.and_then(|(rank, file)| game.possible_moves(*rank as u32, *file as u32));
        // trc!("Moves: {:?}", possible_moves);
        let board = game.current_board();
        let mut new_click = None;
        egui::Grid::new("main_grid")
            .striped(true)
            .min_col_width(CELL_SIZE)
            .max_col_width(CELL_SIZE)
            .min_row_height(CELL_SIZE)
            .show(ui, |ui| {
                for (rank, row) in board.iter().enumerate() {
                    for (file, cell) in row.iter().enumerate() {
                        let btn = if let Some(source) = piece_image(cell) {
                            egui::Button::image(source)
                        } else {
                            egui::Button::new("")
                        };
                        let selected = chosen_figure
                            .and_then(|fig| Some(fig == &(rank, file)))
                            .unwrap_or(false);
                        let btn = ui.add(
                            btn.frame(false)
                                .min_size(Vec2::new(CELL_SIZE, CELL_SIZE))
                                .fill(background_color(
                                    (rank, file),
                                    selected,
                                    possible_moves
                                        .as_ref()
                                        .and_then(|moves| {
                                            moves.iter().find(|_move| {
                                                _move.to == (rank as u32, file as u32)
                                            })
                                        })
                                        .is_some(),
                                )),
                        );
                        if btn.clicked() {
                            new_click = Some((rank as u32, file as u32));
                        }
                    }
                    ui.end_row();
                }
            });
        new_click
    }
}
