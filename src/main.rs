use std::{collections::HashMap, io::BufRead};

use chess_engine::{Board, Color, Move, Piece, PieceType};
use eframe::{egui, epaint::Vec2};

fn chess_symbol(piece: Piece) -> &'static str {
    use Color::*;
    use PieceType::*;
    match piece.color() {
        Black => match piece.type_() {
            Pawn => "♟︎",
            Knight => "♞",
            Bishop => "♝",
            Rook => "♜",
            Queen => "♛",
            King => "♚",
            Invalid => "#",
            EmptySquare => " ",
        },
        White => match piece.type_() {
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
    let board = app.board.inside();
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let pos = rank << 4 | file;
            let piece = Piece::from_code(board[pos as usize], pos);
            print!("{} ", chess_symbol(piece));
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
    board: Board,
    cell_size: f32,
    current_color: Color,
    chosen_piece: Option<Piece>,
    moves: Option<HashMap<u8, Move>>,
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(App {
                board: Board::default(),
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
            ui.heading("My chess game");

            // ui.image(egui::include_image!(
            //     "../media/Chess_bdt45.svg.png"
            // ));
            let board = self.board.inside();
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
                            let color = if (rank + file) % 2 == 0 {
                                egui::Color32::LIGHT_GRAY
                            } else {
                                egui::Color32::DARK_GRAY
                            };
                            let piece = Piece::from_code(board[pos as usize], pos);
                            let widget = if let Some(source) = piece_image(piece.clone()) {
                                // ui.image(source)
                                // ui.add(egui::ImageButton::new(source).frame(false))
                                ui.add(
                                    egui::Button::image(source)
                                        .frame(false)
                                        .min_size(Vec2::new(self.cell_size, self.cell_size))
                                        .fill({
                                            if Some(&piece) == self.chosen_piece.as_ref() {
                                                egui::Color32::LIGHT_GREEN
                                            } else if let Some(moves) = &self.moves {
                                                if moves.contains_key(&pos) {
                                                    color.additive().gamma_multiply(1.3)
                                                } else {
                                                    color
                                                }
                                            } else {
                                                color
                                            }
                                        }),
                                )
                            } else {
                                ui.add(
                                    egui::Button::new("")
                                        .frame(false)
                                        .min_size(Vec2::new(self.cell_size, self.cell_size))
                                        .fill({
                                            if let Some(moves) = &self.moves {
                                                if moves.contains_key(&pos) {
                                                    color.additive().gamma_multiply(1.3)
                                                } else {
                                                    color
                                                }
                                            } else {
                                                color
                                            }
                                        }),
                                )
                            };
                            if widget.clicked() {
                                let piece = Piece::from_code(board[pos as usize], pos);
                                println!("Position {} was clicked", pos);
                                println!("Piece: {:?}", piece);
                                if let Some(moves) = &mut self.moves {
                                    self.current_color = piece.color();
                                    move_to_exec = moves.remove(&pos);
                                    self.chosen_piece = None;
                                    self.moves = None;
                                }
                                if piece.type_() != PieceType::Invalid
                                    && piece.type_() != PieceType::EmptySquare
                                    && move_to_exec.is_none()
                                {
                                    self.chosen_piece = Some(piece.clone());
                                    let moves = self
                                        .board
                                        .get_possible_moves(piece.color(), Move::NullMove);
                                    println!("Moves: {:?}", moves);
                                    self.moves = Some(filter_moves(moves, &piece));
                                    println!("self.moves: {:?}", self.moves);
                                } else {
                                    self.chosen_piece = None;
                                    self.moves = None;
                                }
                            }
                        }
                        ui.end_row();
                    }
                });
            if let Some(_move) = move_to_exec {
                self.board.execute(_move);
                for color in [Color::Black, Color::White] {
                    let (checked, king) = self.board.is_checked(color);
                    if checked {
                        println!("{:?} is checked.", king);
                    }
                    self.board.castling_rights(king);
                }
            }
        });
    }
}

fn filter_moves(moves: Vec<Move>, piece: &Piece) -> HashMap<u8, Move> {
    HashMap::from_iter(
        moves
            .into_iter()
            .filter(|_move| match _move {
                Move::NullMove => true,
                Move::QuietMove(_piece, _) => piece == _piece,
                Move::Capture(_piece, _) => piece == _piece,
                Move::Castling(_piece, _, _) => piece == _piece,
                Move::PromotionQuiet(_piece, _, _) => piece == _piece,
                Move::PromotionCapture(_piece, _, _) => piece == _piece,
                Move::PawnDoublePush(_piece, _) => piece == _piece,
                Move::EnPassantCapture(_piece, _) => piece == _piece,
            })
            .map(|_move| {
                let pos: u8 = match &_move {
                    Move::NullMove => panic!("Null move is not valid move"),
                    Move::QuietMove(_, pos) => *pos,
                    Move::Capture(_, piece) => piece.position() as u8,
                    Move::Castling(_, _, rook) => rook.position() as u8,
                    Move::PromotionQuiet(_, pos, _) => *pos,
                    Move::PromotionCapture(_, piece, _) => piece.position() as u8,
                    Move::PawnDoublePush(_, pos) => *pos,
                    Move::EnPassantCapture(_, piece) => piece.position() as u8,
                };
                (pos, _move)
            }),
    )
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
