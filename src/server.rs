use std::{collections::HashMap, convert::Infallible, sync::Arc};

use futures::{stream::StreamExt, FutureExt};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::RwLock;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    filters::ws::{Message, WebSocket},
    reject::Rejection,
    reply::Reply,
    Filter,
};

#[allow(unused_imports)]
use log::{debug as dbg, error as err, info as inf, trace as trc, warn as wrn};

pub struct Client {
    pub id: Uuid,
    pub sender: UnboundedSender<Result<Message, warp::Error>>,
    pub room: String,
}

type Rooms = Arc<RwLock<HashMap<String, Vec<Client>>>>;

#[tokio::main]
async fn main() {
    env_logger::init();
    inf!("It's server!");

    let rooms: Rooms = Arc::new(RwLock::new(HashMap::new()));

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

fn with<T>(value: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone
where
    T: Clone + std::marker::Send,
{
    warp::any().map(move || value.clone())
}

async fn new_room_handler(ws: warp::ws::Ws, rooms: Rooms) -> Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(|ws| client_connection(ws, rooms, None)))
}

async fn existing_room_handler(
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

async fn client_connection(ws: WebSocket, rooms: Rooms, room_name: Option<String>) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            err!("Failed sending websocket msg: {}", e);
        }
    }));

    let room_name = if let Some(name) = room_name {
        name
    } else {
        Uuid::new_v4().to_string()
    };
    let id = Uuid::new_v4();
    {
        // Adding new client to room
        let mut rooms = rooms.write().await;
        if let Some(clients) = rooms.get_mut(&room_name) {
            clients.push(Client {
                id,
                sender: client_sender,
                room: room_name.clone(),
            });
            trc!("Added new client {} to '{}' room", id, room_name);
        } else {
            rooms.insert(
                room_name.clone(),
                vec![Client {
                    id,
                    sender: client_sender,
                    room: room_name.clone(),
                }],
            );
            trc!("Created new room '{}' for new client {}", room_name, id);
        }
    }

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                err!("Failed receiving ws msg in room '{}': {}", room_name, e);
                break;
            }
        };
        client_msg(msg, &rooms, &room_name, id).await;
    }

    {
        let mut rooms = rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_name) {
            room.remove(
                room.iter()
                    .position(|x| x.id == id)
                    .expect("Room don't have client?"),
            );
            if room.is_empty() {
                rooms.remove(&room_name);
                trc!("Deleted room '{}' without clients", room_name);
            }
        } else {
            err!("Failed to find room '{}'", room_name);
        }
    }
    trc!("Client {} was disconnected...", id);
}

async fn client_msg(msg: Message, rooms: &Rooms, room: &str, client_id: Uuid) {
    trc!("Received message from room '{}': {:?}", room, msg);
    let message = match msg.to_str() {
        Ok(message) => message,
        Err(_) => {
            wrn!("Failed to parce string from message!");
            return;
        }
    };

    if let Some(room) = rooms.read().await.get(room) {
        room.iter().map(|client| &client.sender).for_each(|sender| {
            let _ = sender.send(Ok(Message::text(format!("[{}]: {}", client_id, message))));
        });
    }
}
