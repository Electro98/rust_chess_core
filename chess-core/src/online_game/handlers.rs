use warp::{reject::Rejection, reply::Reply};

use crate::online_game::definitions::*;
use crate::online_game::logic::client_connection;

pub async fn new_room_handler(ws: warp::ws::Ws, rooms: Rooms) -> Result<impl Reply, Rejection> {
    info!("Get new connection to websocket!");
    Ok(ws.on_upgrade(|ws| client_connection(ws, rooms, None)))
}

pub async fn existing_room_handler(
    room: String,
    ws: warp::ws::Ws,
    rooms: Rooms,
) -> Result<impl Reply, Rejection> {
    let result = if let Some(game) = rooms.clone().read().await.get(&room) {
        // Trying to connect in already created game
        if game.black.is_some() && game.white.is_some() {
            // Room is full
            return Err(warp::reject());
        }
        let name = game.id.clone();
        Ok(ws.on_upgrade(|ws| client_connection(ws, rooms, Some(name))))
    } else {
        Err(warp::reject())
    };
    result
}
