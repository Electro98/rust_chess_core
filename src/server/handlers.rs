use warp::{reject::Rejection, reply::Reply};

use crate::server::definitions::*;
use crate::server::logic::client_connection;

pub async fn new_room_handler(ws: warp::ws::Ws, rooms: Rooms) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(|ws| client_connection(ws, rooms, None)))
}

pub async fn existing_room_handler(
    room: String,
    ws: warp::ws::Ws,
    rooms: Rooms,
) -> Result<impl Reply, Rejection> {
    if rooms.read().await.contains_key(&room) {
        Ok(ws.on_upgrade(|ws| client_connection(ws, rooms, Some(room))))
    } else {
        Err(warp::reject())
    }
}
