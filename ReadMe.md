# Silly Chess Engine

That's just chess engine for my "Dark Chess" game as my student diploma project.

It may be silly or/and goofy, but tremendous amount of work and passion were put into it!


## Current state of project

It's under non-active development in my free time. Most features are finished but some of them cut because of rework and will be rewritten! This's unfortunately takes time as well.

But I still happy that now it's bugs free (at least from chess standpoint) and kinda playable!

## Usage

This wonder of human mind have a few executables to build with `cargo`:
```Shell

# Runs the main binary that's just plain chess
cargo run --features="build-binary"

# Runs the online server on port 3030
cargo run --features="build-binary" --bin server

# Runs client for online game
#   first argument is address of server to connect to
#   also {game_id} is id of game and should be empty when first client is connected
#   when first client is connected in logs will be game-id for second client to connect to!
RUST_LOG=client=debug cargo run --features="build-binary" --bin client -- "ws://0.0.0.0:3030/ws/{game_id}"

```

## TODO:
 - [X] Create basic declarations for all game elements
   - [X] Chess board
   - [X] Chess piece
   - [X] Move
   - [X] History of moves
 - [X] Write logic for:
   - [X] Making a move
   - [X] Undo a move
   - [X] List all possible moves
   - [X] Allowing Checks and Mate
   - [X] Castling right check
 - [X] Make basic interface?
   - [X] Display board
   - [X] Make moves colorful
   - [X] Display checks and mates
 - [X] Tests for chess
 - [ ] Create kotling bindings (ABANDONED)
   - [ ] Compile it
   - [ ] Make it works


## Thanks!

I immensely grateful to **jniemann66** creator of [juddperft](https://github.com/jniemann66/juddperft). This's just a beautiful project that helps with developing and debugging your own chess engine!
