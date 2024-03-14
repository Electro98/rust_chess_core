use std::io::BufRead;

use chess_engine::engine::Move as ImplMove;
use chess_engine::{Cell, Color, Figure, Game, MatchInterface, Move, PieceType};
use eframe::{egui, epaint::Vec2};

fn chess_symbol(figure: &Figure) -> &'static str {
    use Color::*;
    use PieceType::*;
    match figure.color {
        Black => match figure.kind {
            Pawn => "♟︎",
            Knight => "♞",
            Bishop => "♝",
            Rook => "♜",
            Queen => "♛",
            King => "♚",
            Invalid => "#",
            EmptySquare => " ",
        },
        White => match figure.kind {
            Pawn => "♙",
            Knight => "♘",
            Bishop => "♗",
            Rook => "♖",
            Queen => "♕",
            King => "♔",
            Invalid => "#",
            EmptySquare => " ",
        },
    }
}

fn draw_app(app: &App) {
    let board = app.game.current_board();
    for rank in 0..8 {
        for file in 0..8 {
            let cell = &board[rank][file];
            print!(
                "{} ",
                chess_symbol(match cell {
                    chess_engine::Cell::Figure(figure) => figure,
                    chess_engine::Cell::Empty => &Figure {
                        kind: PieceType::Invalid,
                        color: Color::White,
                        last_move: false,
                        impose_check: false,
                        can_move: false
                    },
                    chess_engine::Cell::Unknown => &Figure {
                        kind: PieceType::Invalid,
                        color: Color::White,
                        last_move: false,
                        impose_check: false,
                        can_move: false
                    },
                })
            );
        }
        print!("\n");
    }
}

fn read_pos() -> Option<u8> {
    let mut input = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut input)
        .expect("unable to read user input");
    let (file, rank) = match &input.to_uppercase().chars().collect::<Vec<_>>()[..] {
        &[first, second, ..] => (first, second),
        _ => return None,
    };
    if '1' <= rank && rank <= '8' && 'A' <= file && file <= 'G' {
        Some((rank as u8 - '1' as u8) << 4 | (file as u8 - 'A' as u8))
    } else {
        None
    }
}

fn get_pos(question: &str) -> u8 {
    use std::io::Write;
    loop {
        print!("{}: ", question);
        let _ = std::io::stdout().flush();
        if let Some(pos) = read_pos() {
            return pos;
        }
    }
}

struct App {
    game: Game,
    cell_size: f32,
    chosen_figure: Option<Figure>,
    selected_cell: Option<(usize, usize)>,
    moves: Option<Vec<Move<ImplMove>>>,
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
                }
            });
        });
    }
}

impl App {
    fn grid(&mut self, ui: &mut egui::Ui) -> Option<Move<ImplMove>> {
        let board = self.game.current_board();
        let mut move_to_exec = None;
        egui::Grid::new("main_grid")
            .striped(true)
            .min_col_width(self.cell_size)
            .max_col_width(self.cell_size)
            .min_row_height(self.cell_size)
            .show(ui, |ui| {
                for rank in 0..8 {
                    for file in 0..8 {
                        let cell = &board[rank][file];
                        let widget = if let Some(source) = piece_image(cell) {
                            // ui.image(source)
                            // ui.add(egui::ImageButton::new(source).frame(false))
                            let selected = self.selected_cell == Some((rank, file));
                            ui.add(
                                egui::Button::image(source)
                                    .frame(false)
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
                                    )),
                            )
                        } else {
                            ui.add(
                                egui::Button::new("")
                                    .frame(false)
                                    .min_size(Vec2::new(self.cell_size, self.cell_size))
                                    .fill(background_color(
                                        (rank, file),
                                        false,
                                        self.moves
                                            .as_ref()
                                            .and_then(|moves| {
                                                moves.iter().find(|_move| {
                                                    _move.to == (rank as u32, file as u32)
                                                })
                                            })
                                            .is_some(),
                                    )),
                            )
                        };
                        if widget.clicked() {
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
                                            .into_iter()
                                            .find(|_move| _move.to == (rank as u32, file as u32))
                                            .and_then(|_move| Some(_move.clone()));
                                        self.chosen_figure = None;
                                        self.selected_cell = None;
                                        None
                                    }
                                }
                            } else {
                                match cell {
                                    Cell::Figure(figure) => {
                                        let moves =
                                            self.game.possible_moves(rank as u32, file as u32);
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

fn background_color(
    position: (usize, usize),
    selected: bool,
    possible_move: bool,
) -> egui::Color32 {
    let color = if (position.0 + position.1) % 2 == 0 {
        egui::Color32::LIGHT_GRAY
    } else {
        egui::Color32::DARK_GRAY
    };
    if selected {
        egui::Color32::LIGHT_GREEN
    } else if possible_move {
        color.additive().gamma_multiply(1.3)
    } else {
        color
    }
}

fn piece_image(cell: &Cell) -> Option<egui::ImageSource<'static>> {
    use Color::*;
    use PieceType::*;
    match cell {
        Cell::Figure(figure) => match figure.color {
            Black => match figure.kind {
                Pawn => Some(egui::include_image!("../media/Chess_pdt45.svg.png")),
                Knight => Some(egui::include_image!("../media/Chess_ndt45.svg.png")),
                Bishop => Some(egui::include_image!("../media/Chess_bdt45.svg.png")),
                Rook => Some(egui::include_image!("../media/Chess_rdt45.svg.png")),
                Queen => Some(egui::include_image!("../media/Chess_qdt45.svg.png")),
                King => Some(egui::include_image!("../media/Chess_kdt45.svg.png")),
                Invalid => Some(egui::include_image!("../media/Chess_idt45.svg.png")),
                EmptySquare => None,
            },
            White => match figure.kind {
                Pawn => Some(egui::include_image!("../media/Chess_plt45.svg.png")),
                Knight => Some(egui::include_image!("../media/Chess_nlt45.svg.png")),
                Bishop => Some(egui::include_image!("../media/Chess_blt45.svg.png")),
                Rook => Some(egui::include_image!("../media/Chess_rlt45.svg.png")),
                Queen => Some(egui::include_image!("../media/Chess_qlt45.svg.png")),
                King => Some(egui::include_image!("../media/Chess_klt45.svg.png")),
                Invalid => Some(egui::include_image!("../media/Chess_ilt45.svg.png")),
                EmptySquare => None,
            },
        },
        Cell::Unknown => None,
        Cell::Empty => None,
    }
}
