
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::{Message, WebSocket};

use crate::server::definitions::*;

pub async fn client_connection(ws: WebSocket, rooms: Rooms, room_name: Option<String>) {
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

pub async fn client_msg(msg: Message, rooms: &Rooms, room: &str, client_id: Uuid) {
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
