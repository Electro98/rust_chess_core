use chess_engine::{
    core::{
        definitions::ImplicitMove,
        engine::{Game, GameEndState, Move, Piece},
        utils::unpack_pos,
    },
    Color, PieceType,
};
use eframe::{egui, epaint::Vec2};
use gui::{background_color, piece_image};

mod gui;

struct App {
    game: Game,
    cell_size: f32,
    end_state: Option<GameEndState>,
    chosen_figure: Option<Piece>,
    selected_cell: Option<(usize, usize)>,
    moves: Option<Vec<Move>>,
    promotion_type: PieceType,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([572.0, 392.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            // let game = Game::from_fen(
            //     "r2k3r/p1p1qpb1/bn2pnp1/4N3/1p2P3/2N1BQ1p/PPP1BPPP/R3K2R w KQ - 0 1",
            // )
            // .unwrap();
            let game = Default::default();
            Box::new(App {
                cell_size: 45.0,
                game,
                end_state: None,
                chosen_figure: None,
                selected_cell: None,
                moves: None,
                promotion_type: PieceType::Queen,
            })
        }),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let move_to_exec = self.grid(ui);
                self.control_panel(ui);
                if let Some(mut _move) = move_to_exec {
                    if _move.promotion() {
                        _move.set_promotion_type(self.promotion_type);
                    }
                    println!(" - move: {_move} {_move:?}");
                    self.end_state = self.game.execute(_move);
                }
            });
        });
    }
}

impl App {
    fn control_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Test chess");
            ui.label(format!("Current player: {}", {
                if self.game.current_player() == Color::White {
                    "White"
                } else {
                    "Black"
                }
            }));
            ui.label(format!(
                "Is checked: {:?}",
                self.game.history().last_move().map(|_move| _move.check())
            ));
            if ui.button("Undo last move").clicked() && self.game.undo_last_move().is_ok() {
                self.end_state = None;
                self.chosen_figure = None;
                self.moves = None;
            }
            if let Some(end_state) = self.end_state {
                ui.label("Game finished!");
                ui.label(format!("Result: {:?}", end_state));
                if ui.button("Restart?").clicked() {
                    self.game = Game::default();
                    self.end_state = None;
                    self.chosen_figure = None;
                    self.moves = None;
                };
            }
            ui.radio_value(&mut self.promotion_type, PieceType::Queen, "Queen");
            ui.radio_value(&mut self.promotion_type, PieceType::Rook, "Rook");
            ui.radio_value(&mut self.promotion_type, PieceType::Bishop, "Bishop");
            ui.radio_value(&mut self.promotion_type, PieceType::Knight, "Third");
        });
    }

    fn grid(&mut self, ui: &mut egui::Ui) -> Option<Move> {
        let board = self.game.board();
        let mut move_to_exec = None;
        egui::Grid::new("main_grid")
            .striped(true)
            .min_col_width(self.cell_size)
            .max_col_width(self.cell_size)
            .min_row_height(self.cell_size)
            .show(ui, |ui| {
                for file in (0..8).rev() {
                    for rank in 0..8 {
                        let piece = board.get(file, rank);
                        let btn = if let Some(source) = piece_image(&piece) {
                            egui::Button::image(source)
                        } else {
                            egui::Button::new("")
                        };
                        let selected = self
                            .selected_cell
                            .map(|fig| fig == (file as usize, rank as usize))
                            .unwrap_or(false);
                        let btn = ui.add(
                            btn.frame(false)
                                .min_size(Vec2::new(self.cell_size, self.cell_size))
                                .fill(background_color(
                                    (file as usize, rank as usize),
                                    selected,
                                    self.moves
                                        .as_ref()
                                        .and_then(|moves| {
                                            moves.iter().find(|_move| {
                                                unpack_pos(_move.end_position())
                                                    == (file as u32, rank as u32)
                                            })
                                        })
                                        .is_some(),
                                    true,
                                )),
                        );
                        if btn.clicked() {
                            println!("Position {}-{} was clicked", file, rank);
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
                                                    == (file as u32, rank as u32)
                                            })
                                            .cloned();
                                        self.chosen_figure = None;
                                        self.selected_cell = None;
                                        None
                                    }
                                }
                            } else {
                                match piece.type_() {
                                    PieceType::Invalid | PieceType::EmptySquare => None,
                                    _ => {
                                        let moves: Vec<_> = self
                                            .game
                                            .get_possible_moves(false)
                                            .into_iter()
                                            .filter(|_move| _move.piece() == &piece)
                                            .collect();
                                        // dbg!(&moves);
                                        self.chosen_figure = if !moves.is_empty() {
                                            self.selected_cell =
                                                Some((file as usize, rank as usize));
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
