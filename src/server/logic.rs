use crate::{Color, GameState, MatchInterface};
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
    let is_white = {
        // Adding new client to game
        let mut rooms = rooms.write().await;
        let client = Client {
            id,
            sender: client_sender,
            game_id: room_name.clone(),
        };
        if let Some(game) = rooms.get_mut(&room_name) {
            let color = if game.white.is_none() {
                game.white = Some(client);
                true
            } else {
                assert!(game.black.is_none(), "Game is full, but have new client!");
                game.black = Some(client);
                false
            };
            trc!("Added new client {} to '{}' room", id, room_name);
            color
        } else {
            let game = OnlineGame {
                id: room_name.clone(),
                game: Default::default(),
                white: Some(client),
                black: None,
            };
            rooms.insert(room_name.clone(), game);
            trc!("Created new room '{}' for new client {}", room_name, id);
            true
        }
    };
    let player = if is_white { Color::White } else { Color::Black };
    // send current state of game
    {
        let rooms = rooms.read().await;
        let room = rooms.get(&room_name).expect("Where is the room??? bug.");
        let (new_client, host) = if is_white {
            (room.white.as_ref(), room.black.as_ref())
        } else {
            (room.black.as_ref(), room.white.as_ref())
        };
        send_message(
            new_client.unwrap(),
            ClientMessage::GameStateSync(
                room.game.vision_board(player),
                room.game.current_player(),
                player,
                host.is_some(),
            ),
        );
        if let Some(host) = host {
            send_message(host, ClientMessage::OpponentConnected);
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
        // Game logic will be here
        client_msg(msg, &rooms, &room_name, id, player).await;
    }

    {
        let mut rooms = rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_name) {
            // TODO: How to properly delete room?
            if room.game.game_ended()
                || (player == Color::White && room.black.is_none())
                || (player == Color::Black && room.white.is_none())
            {
                if room.game.game_ended() {
                    brodcast_msg(
                        room,
                        ClientMessage::GameFinished(room.game.current_player().opposite()),
                    );
                } else {
                    brodcast_msg(room, ClientMessage::GameCanceled);
                }
                rooms.remove(&room_name);
                trc!("Deleted room '{}' with finished game", room_name);
            } else if let Some(client) = room.get_player(player.opposite()) {
                send_message(client, ClientMessage::OpponentDisconected);
                *room.get_player_mut(player) = None;
                trc!("Player '{}' has been disconnected!", id);
            }
        } else {
            err!("Failed to find room '{}'", room_name);
        }
    }
    trc!("Client {} was disconnected...", id);
}

async fn client_msg(msg: Message, rooms: &Rooms, room: &GameId, _client_id: Uuid, player: Color) {
    trc!("Received message from room '{}': {:?}", room, msg);
    if !msg.is_binary() {
        // TODO: do something
        return;
    }
    let client_msg: ClientMessage = msg.try_into().unwrap();
    match client_msg {
        ClientMessage::MakeMove(_move) => {
            let mut rooms = rooms.write().await;
            if let Some(room) = rooms.get_mut(room) {
                if room.game.current_player() == player {
                    match room.game.execute_move(_move) {
                        GameState::PlayerMove(player) => {
                            let opponent = player.opposite();
                            if let Some(client) = room.get_player(player) {
                                send_message(
                                    client,
                                    ClientMessage::GameStateSync(
                                        room.game.vision_board(player),
                                        player,
                                        player,
                                        room.get_player(opponent).is_some(),
                                    ),
                                );
                            }
                            if let Some(client) = room.get_player(opponent) {
                                send_message(
                                    client,
                                    ClientMessage::GameStateSync(
                                        room.game.vision_board(opponent),
                                        player,
                                        opponent,
                                        room.get_player(player).is_some(),
                                    ),
                                );
                            }
                        }
                        GameState::Finished => {}
                        _ => {}
                    }
                }
            } else {
                debg!(
                    "Probably bug, but room '{}' don't exist and received message.",
                    room
                );
            }
        }
        _ => {
            todo!("I don't know.");
        }
    }
}

async fn brodcast_msg_search(rooms: &Rooms, game_id: &GameId, msg: Message) {
    if let Some(room) = rooms.read().await.get(game_id) {
        brodcast_msg(room, msg);
    }
}

fn brodcast_msg<T: Into<Message>>(room: &OnlineGame, msg: T) {
    let message = msg.into();
    if let Some(client) = &room.white {
        let _ = client.sender.send(Ok(message.clone()));
    }
    if let Some(client) = &room.black {
        let _ = client.sender.send(Ok(message));
    }
}

fn send_message<T: Into<warp::ws::Message>>(client: &Client, msg: T) {
    let _ = client.sender.send(Ok(msg.into()));
}
