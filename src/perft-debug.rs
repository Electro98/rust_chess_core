use std::env;

use chess_engine::engine::Game;

fn perf_test(game: Game, depth: u32, debug: bool) -> usize {
    match depth {
        0 => 1,
        1 => {
            let possible_moves = game.get_possible_moves(true);
            if debug {
                let moves: Vec<_> = possible_moves.iter().map(|elem| elem.to_string()).collect();
                println!("{moves:#?}")
            }
            possible_moves.len()
        }
        _ => game
            .get_possible_moves(true)
            .into_iter()
            .map(|_move| {
                let mut game = game.light_clone();
                if let Some(_end_state) = game.execute(_move, true) {
                    0
                } else {
                    perf_test(game, depth - 1, false)
                }
            })
            .sum(),
    }
}

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    let game = Game::from_fen(&args[1]).unwrap();
    let depth: u32 = args[2].parse().unwrap();
    let expected: usize = args[3].parse().unwrap();
    let result = perf_test(game, depth, true);
    if result == expected {
        Ok(())
    } else {
        println!("Found {result} moves.");
        Err(())
    }
}
