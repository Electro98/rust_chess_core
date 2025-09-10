use chess_core::{core::definitions::Cell, core::engine::Piece, Color, PieceType};
use eframe::egui;

pub fn background_color(
    position: (usize, usize),
    selected: bool,
    possible_move: bool,
    visible: bool,
) -> egui::Color32 {
    let color = if (position.0 + position.1) % 2 == 0 {
        egui::Color32::LIGHT_GRAY
    } else {
        egui::Color32::DARK_GRAY
    };
    if !visible {
        color.gamma_multiply(0.2)
    } else if selected {
        egui::Color32::LIGHT_GREEN
    } else if possible_move {
        color.additive().gamma_multiply(1.3)
    } else {
        color
    }
}

pub fn piece_image(piece: &Piece) -> Option<egui::ImageSource<'static>> {
    use Color::*;
    use PieceType::*;
    match piece.type_() {
        EmptySquare => None,
        _ => match piece.color() {
            Black => match piece.type_() {
                Pawn => Some(egui::include_image!("../media/Chess_pdt45.svg.png")),
                Knight => Some(egui::include_image!("../media/Chess_ndt45.svg.png")),
                Bishop => Some(egui::include_image!("../media/Chess_bdt45.svg.png")),
                Rook => Some(egui::include_image!("../media/Chess_rdt45.svg.png")),
                Queen => Some(egui::include_image!("../media/Chess_qdt45.svg.png")),
                King => Some(egui::include_image!("../media/Chess_kdt45.svg.png")),
                Invalid => Some(egui::include_image!("../media/Chess_idt45.svg.png")),
                EmptySquare => unreachable!(),
            },
            White => match piece.type_() {
                Pawn => Some(egui::include_image!("../media/Chess_plt45.svg.png")),
                Knight => Some(egui::include_image!("../media/Chess_nlt45.svg.png")),
                Bishop => Some(egui::include_image!("../media/Chess_blt45.svg.png")),
                Rook => Some(egui::include_image!("../media/Chess_rlt45.svg.png")),
                Queen => Some(egui::include_image!("../media/Chess_qlt45.svg.png")),
                King => Some(egui::include_image!("../media/Chess_klt45.svg.png")),
                Invalid => Some(egui::include_image!("../media/Chess_ilt45.svg.png")),
                EmptySquare => unreachable!(),
            },
        },
    }
}

pub fn piece_image_cell(cell: &Cell) -> Option<egui::ImageSource<'static>> {
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
