use std::{
    str::FromStr,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use chess_engine::{
    core::{
        definitions::ImplicitMove,
        engine::{Game, GameEndState, Move, Piece},
        utils::unpack_pos,
    },
    online_game::client::{ClientState, OnlineClient, OnlineClientOutput},
    Color, PieceType,
};
use eframe::egui::{self, Vec2};
use futures::StreamExt;
use gui::{background_color, piece_image};
use log::{debug, error, info};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tungstenite::{Error, Message};
use url::Url;

mod gui;

struct App {
    client: OnlineClient,
    last_state: ClientState,
    cell_size: f32,
    end_state: Option<GameEndState>,
    chosen_figure: Option<Piece>,
    selected_cell: Option<(usize, usize)>,
    moves: Option<Vec<Move>>,
    promotion_type: PieceType,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let url = Url::from_str(&std::env::args().nth(1).expect("Choose link"))
        .expect("Failed to parse link");
    let online_client = OnlineClient::start_client(url);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([572.0, 392.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "Web Client",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(App {
                client: online_client,
                last_state: ClientState::Unconnected,
                cell_size: 45.,
                end_state: None,
                chosen_figure: None,
                selected_cell: None,
                moves: None,
                promotion_type: PieceType::Queen,
            })
        }),
    );
    result
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(message) = self.client.output.try_recv() {
            self.new_message(message);
        }
        let current_game = self.client.game().blocking_lock().clone();
        self.last_state = self.client.current_state();
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.last_state {
                ClientState::Unconnected => {
                    ui.label("Connecting...");
                }
                ClientState::WaitingOpponent => {
                    ui.label("Opponent is not connected!");
                }
                state => {
                    let game = current_game.expect(&format!("Game is None in '{state:?}'"));
                    ui.horizontal(|ui| {
                        let move_to_exec = self.grid(ui, &game);
                        self.control_panel(ui, &game);
                        if let Some(mut _move) = move_to_exec {
                            if _move.promotion() {
                                _move.set_promotion_type(self.promotion_type);
                            }
                            println!(" - move: {_move} {_move:?}");
                            // self.end_state = game.execute(_move);
                            self.client.make_move(_move);
                        }
                    });
                }
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.client.disconnect();
    }
}

impl App {
    fn new_message(&mut self, message: OnlineClientOutput) {
        match message {
            OnlineClientOutput::ReceivedGameId => {
                let game_id = self.client.game_id();
                info!("Received GameId: {}", game_id);
            }
            OnlineClientOutput::StateChanged(client_state) => {
                debug!("Switched to new state! State: {:?}", client_state);
                if matches!(client_state, ClientState::GameFinished) {
                    // TODO: asd
                }
            }
            OnlineClientOutput::IncorrectInput => {
                error!("Incorrect user action!");
            }
        }
    }

    fn control_panel(&mut self, ui: &mut egui::Ui, game: &Game) {
        ui.vertical(|ui| {
            ui.heading("Test chess");
            ui.label(format!("You are: {}", {
                if self.client.player_color() == Color::White {
                    "White"
                } else {
                    "Black"
                }
            }));
            ui.label(format!("Current player: {}", {
                if game.current_player() == Color::White {
                    "White"
                } else {
                    "Black"
                }
            }));
            ui.label(format!(
                "Is checked: {:?}",
                game.history().last_move().map(|_move| _move.check())
            ));
            // if ui.button("Undo last move").clicked() && game.undo_last_move().is_ok() {
            //     self.end_state = None;
            //     self.chosen_figure = None;
            //     self.moves = None;
            // }
            if let Some(end_state) = self.end_state {
                ui.label("Game finished!");
                ui.label(format!("Result: {:?}", end_state));
                // if ui.button("Restart?").clicked() {
                //     self.game = Game::default();
                //     self.end_state = None;
                //     self.chosen_figure = None;
                //     self.moves = None;
                // };
            }
            ui.radio_value(&mut self.promotion_type, PieceType::Queen, "Queen");
            ui.radio_value(&mut self.promotion_type, PieceType::Rook, "Rook");
            ui.radio_value(&mut self.promotion_type, PieceType::Bishop, "Bishop");
            ui.radio_value(&mut self.promotion_type, PieceType::Knight, "Third");
        });
    }

    fn grid(&mut self, ui: &mut egui::Ui, game: &Game) -> Option<Move> {
        let board = game.board();
        let mut move_to_exec = None;
        egui::Grid::new("main_grid")
            .striped(true)
            .min_col_width(self.cell_size)
            .max_col_width(self.cell_size)
            .min_row_height(self.cell_size)
            .show(ui, |ui| {
                for rank in (0..8).rev() {
                    for file in 0..8 {
                        let piece = board.get(rank, file);
                        let btn = if let Some(source) = piece_image(&piece) {
                            egui::Button::image(source)
                        } else {
                            egui::Button::new("")
                        };
                        let selected = self
                            .selected_cell
                            .map(|fig| fig == (rank as usize, file as usize))
                            .unwrap_or(false);
                        let btn = ui.add(
                            btn.frame(false)
                                .min_size(Vec2::new(self.cell_size, self.cell_size))
                                .fill(background_color(
                                    (rank as usize, file as usize),
                                    selected,
                                    self.moves
                                        .as_ref()
                                        .and_then(|moves| {
                                            moves.iter().find(|_move| {
                                                unpack_pos(_move.end_position())
                                                    == (rank as u32, file as u32)
                                            })
                                        })
                                        .is_some(),
                                    true,
                                )),
                        );
                        if btn.clicked() {
                            println!("Position {}-{} was clicked", rank, file);
                            println!("Cell: {:?}", piece);
                            self.moves = if let Some(moves) = self.moves.as_mut() {
                                match piece.type_() {
                                    PieceType::Invalid => {
                                        self.chosen_figure = None;
                                        self.selected_cell = None;
                                        None
                                    }
                                    _ => {
                                        move_to_exec = moves
                                            .iter()
                                            .find(|_move| {
                                                unpack_pos(_move.end_position())
                                                    == (rank as u32, file as u32)
                                            })
                                            .cloned();
                                        self.chosen_figure = None;
                                        self.selected_cell = None;
                                        None
                                    }
                                }
                            } else if piece.color() != self.client.player_color() {
                                self.chosen_figure = None;
                                self.selected_cell = None;
                                None
                            } else {
                                match piece.type_() {
                                    PieceType::Invalid | PieceType::EmptySquare => None,
                                    _ => {
                                        let moves: Vec<_> = game
                                            .get_possible_moves(false)
                                            .into_iter()
                                            .filter(|_move| _move.piece() == &piece)
                                            .collect();
                                        // dbg!(&moves);
                                        self.chosen_figure = if !moves.is_empty() {
                                            self.selected_cell =
                                                Some((rank as usize, file as usize));
                                            Some(piece)
                                        } else {
                                            None
                                        };
                                        if moves.is_empty() {
                                            None
                                        } else {
                                            Some(moves)
                                        }
                                    }
                                }
                            };
                        }
                    }
                    ui.end_row();
                }
            });
        move_to_exec
    }
}
