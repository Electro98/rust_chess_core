use std::{collections::HashMap, io::BufRead};

use chess_engine::{Color, Figure, Game, MatchInterface, Move, PieceType};
use eframe::{egui, epaint::Vec2};

fn chess_symbol(figure: Figure) -> &'static str {
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
            let cell = board[rank][file];
            print!(
                "{} ",
                chess_symbol(match cell {
                    chess_engine::Cell::Figure(figure) => figure,
                    chess_engine::Cell::Empty => Figure {
                        kind: PieceType::Invalid,
                        color: Color::White,
                        last_move: false,
                        impose_check: false,
                        can_move: false
                    },
                    chess_engine::Cell::Unknown => Figure {
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
    current_color: Color,
    chosen_piece: Option<Piece>,
    moves: Option<Vec<UiMove>>,
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
                current_color: Color::White,
                chosen_piece: None,
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
                    if self.game.finished() {
                        ui.label("Game finished!");
                        if ui.button("Restart?").clicked() {
                            self.game = Game::default();
                            self.current_color = Color::White;
                            self.chosen_piece = None;
                            self.moves = None;
                        };
                    }
                });
                if let Some(_move) = move_to_exec {
                    self.game.make_move(_move.clone());
                }
            });
        });
    }
}

impl App {
    fn grid(&mut self, ui: &mut egui::Ui) -> Option<UiMove> {
        let board = self.game.board().inside();
        let mut move_to_exec = None;
        egui::Grid::new("main_grid")
            .striped(true)
            .min_col_width(self.cell_size)
            .max_col_width(self.cell_size)
            .min_row_height(self.cell_size)
            .show(ui, |ui| {
                for rank in 0..8u8 {
                    for file in 0..8u8 {
                        let pos = rank << 4 | file;
                        let piece = Piece::from_code(board[pos as usize], pos);
                        let widget = if let Some(source) = piece_image(piece.clone()) {
                            // ui.image(source)
                            // ui.add(egui::ImageButton::new(source).frame(false))
                            ui.add(
                                egui::Button::image(source)
                                    .frame(false)
                                    .min_size(Vec2::new(self.cell_size, self.cell_size))
                                    .fill(background_color(
                                        pos as usize,
                                        Some(piece) == self.chosen_piece,
                                        self.moves
                                            .as_ref()
                                            .and_then(|moves| {
                                                moves.iter().find(|ui_move| {
                                                    ui_move.position() == pos as usize
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
                                        pos as usize,
                                        false,
                                        self.moves
                                            .as_ref()
                                            .and_then(|moves| {
                                                moves.iter().find(|ui_move| {
                                                    ui_move.position() == pos as usize
                                                })
                                            })
                                            .is_some(),
                                    )),
                            )
                        };
                        if widget.clicked() {
                            let piece = Piece::from_code(board[pos as usize], pos);
                            println!("Position {} was clicked", pos);
                            println!("Piece: {:?}", piece);
                            self.moves = if let Some(moves) = self.moves.as_mut() {
                                self.current_color = piece.color();
                                move_to_exec = moves
                                    .into_iter()
                                    .find(|_move| _move.position() == piece.position())
                                    .and_then(|_move| Some(_move.clone()));
                                self.chosen_piece = None;
                                None
                            } else if piece.type_() != PieceType::Invalid
                                && piece.type_() != PieceType::EmptySquare
                            {
                                let moves = self.game.possible_moves(rank as u32, file as u32);
                                self.chosen_piece = if moves.is_some() {
                                    Some(piece.clone())
                                } else {
                                    None
                                };
                                println!("Moves: {:?}", moves);
                                moves
                            } else {
                                self.chosen_piece = None;
                                None
                            };
                        }
                    }
                    ui.end_row();
                }
            });
        move_to_exec
    }
}

fn background_color(position: usize, selected: bool, possible_move: bool) -> egui::Color32 {
    let color = if (position.wrapping_shr(4) + position & 0x0F) % 2 == 0 {
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

fn piece_image(piece: Piece) -> Option<egui::ImageSource<'static>> {
    use Color::*;
    use PieceType::*;
    match piece.color() {
        Black => match piece.type_() {
            Pawn => Some(egui::include_image!("../media/Chess_pdt45.svg.png")),
            Knight => Some(egui::include_image!("../media/Chess_ndt45.svg.png")),
            Bishop => Some(egui::include_image!("../media/Chess_bdt45.svg.png")),
            Rook => Some(egui::include_image!("../media/Chess_rdt45.svg.png")),
            Queen => Some(egui::include_image!("../media/Chess_qdt45.svg.png")),
            King => Some(egui::include_image!("../media/Chess_kdt45.svg.png")),
            Invalid => Some(egui::include_image!("../media/Chess_idt45.svg.png")),
            EmptySquare => None,
        },
        White => match piece.type_() {
            Pawn => Some(egui::include_image!("../media/Chess_plt45.svg.png")),
            Knight => Some(egui::include_image!("../media/Chess_nlt45.svg.png")),
            Bishop => Some(egui::include_image!("../media/Chess_blt45.svg.png")),
            Rook => Some(egui::include_image!("../media/Chess_rlt45.svg.png")),
            Queen => Some(egui::include_image!("../media/Chess_qlt45.svg.png")),
            King => Some(egui::include_image!("../media/Chess_klt45.svg.png")),
            Invalid => Some(egui::include_image!("../media/Chess_ilt45.svg.png")),
            EmptySquare => None,
        },
    }
}
