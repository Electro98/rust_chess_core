use chess_engine::{Cell, Color, DefaultMove, Figure, Game, MatchInterface};
use eframe::{egui, epaint::Vec2};
use gui::{background_color, piece_image};

mod gui;

struct App {
    game: Game,
    cell_size: f32,
    chosen_figure: Option<Figure>,
    selected_cell: Option<(usize, usize)>,
    moves: Option<Vec<DefaultMove>>,
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

            Box::new(App {
                game: Game::default(),
                cell_size: 45.0,
                chosen_figure: None,
                selected_cell: None,
                moves: None,
            })
        }),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let move_to_exec = self.grid(ui);
                ui.vertical(|ui| {
                    ui.heading("Test chess");
                    ui.label(format!("Current player: {}", {
                        if self.game.current_player() == Color::White {
                            "White"
                        } else {
                            "Black"
                        }
                    }));
                    ui.label(format!("Is checked: {}", self.game.checked()));
                    if self.game.game_ended() {
                        ui.label("Game finished!");
                        if ui.button("Restart?").clicked() {
                            self.game = Game::default();
                            self.chosen_figure = None;
                            self.moves = None;
                        };
                    }
                });
                if let Some(_move) = move_to_exec {
                    self.game.execute_move(_move);
                    self.game.wait_move();
                }
            });
        });
    }
}

impl App {
    fn grid(&mut self, ui: &mut egui::Ui) -> Option<DefaultMove> {
        let board = self.game.current_board();
        let mut move_to_exec = None;
        egui::Grid::new("main_grid")
            .striped(true)
            .min_col_width(self.cell_size)
            .max_col_width(self.cell_size)
            .min_row_height(self.cell_size)
            .show(ui, |ui| {
                for (rank, row) in board.iter().enumerate() {
                    for (file, cell) in row.iter().enumerate() {
                        let btn = if let Some(source) = piece_image(cell) {
                            egui::Button::image(source)
                        } else {
                            egui::Button::new("")
                        };
                        let selected = self
                            .selected_cell
                            .map(|fig| fig == (rank, file))
                            .unwrap_or(false);
                        let btn = ui.add(
                            btn.frame(false)
                                .min_size(Vec2::new(self.cell_size, self.cell_size))
                                .fill(background_color(
                                    (rank, file),
                                    selected,
                                    self.moves
                                        .as_ref()
                                        .and_then(|moves| {
                                            moves.iter().find(|_move| {
                                                _move.to == (rank as u32, file as u32)
                                            })
                                        })
                                        .is_some(),
                                    !matches!(cell, Cell::Unknown),
                                )),
                        );
                        if btn.clicked() {
                            println!("Position {}-{} was clicked", rank, file);
                            println!("Cell: {:?}", cell);
                            self.moves = if let Some(moves) = self.moves.as_mut() {
                                match cell {
                                    Cell::Unknown => {
                                        self.chosen_figure = None;
                                        self.selected_cell = None;
                                        None
                                    }
                                    _ => {
                                        move_to_exec = moves
                                            .iter()
                                            .find(|_move| _move.to == (rank as u32, file as u32))
                                            .cloned();
                                        self.chosen_figure = None;
                                        self.selected_cell = None;
                                        None
                                    }
                                }
                            } else {
                                match cell {
                                    Cell::Figure(figure) => {
                                        let moves = self.game.possible_moves(rank, file);
                                        self.chosen_figure = if moves.is_some() {
                                            self.selected_cell = Some((rank, file));
                                            Some(figure.clone())
                                        } else {
                                            None
                                        };
                                        moves
                                    }
                                    _ => None,
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
