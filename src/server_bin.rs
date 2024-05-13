use warp::Filter;

mod server;

use crate::server::definitions::*;
use crate::server::handlers::*;

#[tokio::main]
async fn main() {
    env_logger::init();
    inf!("It's server!");

    let rooms: Rooms = Default::default();

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let new_room = warp::path("ws")
        .and(warp::ws())
        .and(with(rooms.clone()))
        .and_then(new_room_handler);

    let existing_room = warp::path!("ws" / String)
        .and(warp::ws())
        .and(with(rooms.clone()))
        .and_then(existing_room_handler);

    let routes = hello.or(existing_room).or(new_room);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn with<T>(value: T) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone
where
    T: Clone + std::marker::Send,
{
    warp::any().map(move || value.clone())
}
