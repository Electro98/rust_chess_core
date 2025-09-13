use std::env;

use chess_core::utils::perf_test;

fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    let depth: usize = args[2].parse().unwrap();
    let expected: usize = args[3].parse().unwrap();
    let result = perf_test(&args[1], depth, expected, true, true);
    if result {
        Ok(())
    } else {
        Err(())
    }
}
