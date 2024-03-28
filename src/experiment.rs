use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
};

use chess_engine::{Game, MatchInterface};

fn main() {
    let filename = "results.csv";
    let file = Arc::new(Mutex::new(File::create(filename).unwrap()));
    while get_line_count(filename) <= 100_000 {
        let handles: Vec<_> = (0..10)
            .map(|idx| {
                let file = Arc::clone(&file);
                std::thread::spawn(move || {
                    let _ = std::panic::catch_unwind(|| {
                        let mut game = Game::default();
                        while !game.game_ended() {
                            game.make_random_move();
                            game.make_minimax_move(&file);
                        }
                    });
                    println!("[debug-{}]: Game finished!", idx)
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

fn get_line_count(path: &str) -> usize {
    BufReader::new(File::open(path).unwrap()).lines().count()
}
