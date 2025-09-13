# Silly Chess Engine

That's just chess engine for my "Dark Chess" mobile game that started as my student diploma project.

It's still under construction, but the end of planned functionality is quite near.


## Current state of project

It's under non-active development in my free time. Some of the features are finished but some of them cut because of rework and will be rewritten! This's unfortunately takes time as well.

But I'm quite happy that now it's bugs free (at least from chess standpoint) and kinda playable! Still a long way to go to be polished and done! (^_^)

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

## Todo: Project Roadmap
 - [ ] Internals rework
   - [ ] Unused code cleanup
   - [ ] Exploring bottlenecks of current code
   - [ ] Trying different speedups and seeing how they affect different platforms
   - [ ] Try to find different approach to determining legality of moves
 - [ ] Making standalone apps playable
   - [ ] Finishing base egui app
   - [ ] Creating bot opponent
   - [ ] Something ?
 - [ ] Multiplayer
   - [ ] Deciding on protocol* architecture
   - [ ] Implementing strict server rules
   - [ ] Make it version dependent (?)
 - [ ] Tests
   - [ ] Review current tests
   - [ ] Create new ones for current undefined behaviors
 - [ ] Project structure
   - [ ] Separate unused parts from main engine
   - [ ] Serialization functionality should be by hidden by feature
   - [X] Hide network functionality by feature flags
   - [X] Make basic client as separate package (?)
 - [X] Create kotling bindings (Moved out current repository)


## Thanks!

I immensely grateful to **jniemann66** creator of [juddperft](https://github.com/jniemann66/juddperft). This's just a beautiful project that helps with developing and debugging your own chess engine!
