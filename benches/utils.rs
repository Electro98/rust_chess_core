use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chess_engine::{engine::{Board, Piece}, utils::compact_pos, Game, GameState, PieceType};


fn code_to_value(code: u8) -> u64 {
    match PieceType::from(code) {
        PieceType::Pawn => 100,
        PieceType::Knight => 325,
        PieceType::Bishop => 350,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 0,
        PieceType::Invalid => 0,
        PieceType::EmptySquare => 0,
    }
}

fn iter_raw_count(board: &Board) -> u64 {
    board.iter().map(code_to_value).sum()
}

fn iter_for_raw_count(board: &Board) -> u64 {
    let mut result = 0;
    for code in board.iter() {
        result += code_to_value(code);
    }
    result
}

fn for_raw_count(board: &Board) -> u64 {
    let mut result = 0;
    for file in 0..8u8 {
        for rank in 0..8u8 {
            let pos = compact_pos(rank, file);
            result += code_to_value(board.inside()[pos as usize]);
        }
    }
    result
}

fn type_to_value(_type: PieceType) -> u64 {
    match _type {
        PieceType::Pawn => 100,
        PieceType::Knight => 325,
        PieceType::Bishop => 350,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 0,
        PieceType::Invalid => 0,
        PieceType::EmptySquare => 0,
    }
}

fn iter_pieces(board: &Board) -> u64 {
    board.iter_pieces().map(|piece| type_to_value(piece.type_())).sum()
}

fn for_piece_count(board: &Board) -> u64 {
    let mut result = 0;
    for file in 0..8u8 {
        for rank in 0..8u8 {
            let pos = compact_pos(rank, file);
            result += type_to_value(Piece::from_code(board.inside()[pos as usize], pos).type_());
        }
    }
    result
}

fn stupid_game(mut game: Game, max_steps: usize) -> Game {
    for _ in 0..max_steps {
        if matches!(game.make_random_move(), GameState::Finished) {
            break;
        }
    }
    game
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("iter raw count", |b| b.iter(|| iter_raw_count(black_box(&Board::default()))));
    c.bench_function("iter-for raw count", |b| b.iter(|| iter_for_raw_count(black_box(&Board::default()))));
    c.bench_function("for raw count", |b| b.iter(|| for_raw_count(black_box(&Board::default()))));
    c.bench_function("iter piece count", |b| b.iter(|| iter_pieces(black_box(&Board::default()))));
    c.bench_function("for piece count", |b| b.iter(|| for_piece_count(black_box(&Board::default()))));
    c.bench_function("stupid game 100", |b| b.iter(|| {
        let game: Game = Default::default();
        stupid_game(game, 100)
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);