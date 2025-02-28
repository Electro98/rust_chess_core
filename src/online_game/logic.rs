use crate::{
    core::{
        definitions::ImplicitMove,
        engine::{Game, GameEndState},
    },
    Color,
};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::{Message, WebSocket};

use crate::online_game::definitions::*;

pub async fn client_connection(ws: WebSocket, rooms: Rooms, room_name: Option<String>) {
    let (client_ws_sender, client_ws_receiver) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("Failed sending websocket msg: {}", e);
        }
    }));

    let game_id = if let Some(name) = room_name {
        name
    } else {
        Uuid::new_v4().to_string()
    };
    let id = Uuid::new_v4();
    let (sender, player) = {
        // Adding new client to game
        let mut rooms_l = rooms.write().await;
        let mut client = Client {
            id,
            sender: client_sender,
            game_id: game_id.clone(),
            color: Color::White,
        };
        if let Some(game) = rooms_l.get_mut(&game_id) {
            let color = if game.white.is_none() {
                game.white = Some(client);
                Color::White
            } else {
                assert!(game.black.is_none(), "Game is full, but have new client!");
                client.color = Color::Black;
                game.black = Some(client);
                Color::Black
            };
            trace!("Added new client {} to '{}' room", id, game_id);
            (game.sender.clone(), color)
        } else {
            let (sender, receiver) = mpsc::unbounded_channel();
            let game = OnlineGame {
                id: game_id.clone(),
                game: Default::default(),
                sender: sender.clone(),
                white: Some(client),
                black: None,
            };
            rooms_l.insert(game_id.clone(), game);
            trace!("Created new room '{}' for new client {}", game_id, id);
            tokio::task::spawn(game_handler(receiver, rooms.clone(), game_id.clone()));
            (sender, Color::White)
        }
    };

    // if let Err(err) = sender.send((player, ClientMessage::Connected)) {
    //     error!("Failed to send msg to game handler! Is game ended?");
    //     debug!("Client id: {}", id);
    //     debug!("Game id: {}", game_id);
    //     error!("Got error: {}", err);
    // }
    client_ws_receiver
        .for_each(|msg| async {
            let client_msg = msg.unwrap().try_into();
            match client_msg {
                Ok(msg) => sender.send((player, msg)).expect("Something got wrong"),
                Err(err) => error!("Failed to parse client message! Err: {:?}", err),
            }
        })
        .await;

    let _ = sender.send((player, ClientMessage::Disconnect));
    trace!("Client {} was disconnected...", id);
}

#[derive(Debug)]
enum ServerState {
    NotStarted,
    UnconnectedPlayer,
    ActiveGame,
    GameCanceled,
    GameFinished,
}

async fn game_handler(
    mut receiver: UnboundedReceiver<(Color, ClientMessage)>,
    rooms: Rooms,
    game_id: GameId,
) {
    let mut current_state = ServerState::NotStarted;
    let mut game = Game::default();
    // TODO: Game logic!
    while let Some((player, message)) = receiver.recv().await {
        match message {
            ClientMessage::Connected => {
                match &current_state {
                    ServerState::NotStarted => {
                        current_state = ServerState::UnconnectedPlayer;
                        trace!("Game #{} accepted {} player connection", game_id, player);
                    }
                    ServerState::UnconnectedPlayer => {
                        current_state = ServerState::ActiveGame;
                        trace!(
                            "Game #{} accepted missing {} player connection",
                            game_id,
                            player
                        );
                    }
                    state => {
                        error!(
                            "Game #{} Received invalid message {:?} in state {:?} from {} player",
                            game_id, message, state, player
                        );
                    }
                }
                if let Some(room) = rooms.read().await.get(&game_id) {
                    let client = room
                        .get_player(player)
                        .expect("Failed to get active player???");
                    send_message(client, ServerMessage::RoomId(game_id.clone()));
                    send_message(
                        client,
                        ServerMessage::GameStateSync(
                            game.board().clone(),
                            game.history().last_move(),
                            game.current_player(),
                            player,
                        ),
                    );
                    if matches!(current_state, ServerState::ActiveGame) {
                        send_message(client, ServerMessage::OpponentConnected);
                        send_message(
                            room.get_player(player.opposite())
                                .expect("Opponent must be connected!"),
                            ServerMessage::OpponentConnected,
                        );
                    }
                }
            }
            ClientMessage::Disconnect => match &current_state {
                ServerState::UnconnectedPlayer => {
                    current_state = ServerState::GameCanceled;
                    trace!(
                        "Game #{} last {} player is disconnected! Room is closed!",
                        game_id,
                        player
                    );
                    receiver.close();
                }
                ServerState::ActiveGame => {
                    current_state = ServerState::UnconnectedPlayer;
                    trace!("Game #{} {} player is disconnected!", game_id, player);
                    send_message_by_id(
                        &rooms,
                        &game_id,
                        player.opposite(),
                        ServerMessage::OpponentDisconnected,
                    )
                    .await;
                }
                ServerState::GameCanceled | ServerState::GameFinished => {
                    trace!("Game #{} {} player is disconnected!", game_id, player);
                    if let Some(room) = rooms.read().await.get(&game_id) {
                        if let Some(client) = room.get_player(player.opposite()) {
                            send_message(client, ServerMessage::OpponentDisconnected);
                        } else {
                            trace!("Game #{} All player have left! Room is closed!", game_id);
                            receiver.close();
                        }
                    }
                }
                state => {
                    error!(
                        "Game #{} Received invalid message {:?} in state {:?} from {} player",
                        game_id, message, state, player
                    );
                }
            },
            ClientMessage::MakeMove(client_move) => {
                if !matches!(current_state, ServerState::ActiveGame) {
                    error!(
                        "Game #{} Received MakeMove message in state {:?} from {} player",
                        game_id, current_state, player
                    );
                    continue;
                }
                if game.current_player() != player {
                    info!(
                        "Game #{} {} player is trying to make move not in turn order!",
                        game_id, player
                    );
                    continue;
                }
                let moves = game.get_possible_moves(true);
                let same_move = moves.into_iter().find(|_move| {
                    _move.end_position() == client_move.end_position()
                        && _move.piece().type_() == client_move.piece().type_()
                        && (!client_move.promotion()
                            || client_move.move_type() == _move.move_type())
                });
                let end_state = if let Some(_move) = same_move {
                    game.execute(_move)
                } else {
                    send_message_by_id(
                        &rooms,
                        &game_id,
                        player,
                        ServerMessage::GameStateSync(
                            game.board().clone(),
                            game.history().last_move(),
                            game.current_player(),
                            player,
                        ),
                    )
                    .await;
                    continue;
                };
                if let Some(room) = rooms.read().await.get(&game_id) {
                    send_message(
                        room.get_player(player).unwrap(),
                        ServerMessage::GameStateSync(
                            game.board().clone(),
                            game.history().last_move(),
                            game.current_player(),
                            player,
                        ),
                    );
                    send_message(
                        room.get_player(player.opposite()).unwrap(),
                        ServerMessage::GameStateSync(
                            game.board().clone(),
                            game.history().last_move(),
                            game.current_player(),
                            player.opposite(),
                        ),
                    );
                    if let Some(end_state) = end_state {
                        broadcast_msg(room, ServerMessage::GameFinished(end_state));
                    }
                }
            }
            ClientMessage::Resigned => todo!(),
        }
    }
    {
        let mut rooms = rooms.write().await;
        if let Some(online_game) = rooms.get_mut(&game_id) {
            // TODO: How to properly delete room?
            rooms.remove(&game_id);
            trace!("Deleted room '{}' with finished game", game_id);
        } else {
            error!("Failed to find room '{}'", game_id);
        }
    }
}

async fn broadcast_msg_by_id<T: Into<Message>>(rooms: &Rooms, game_id: &GameId, msg: T) {
    if let Some(room) = rooms.read().await.get(game_id) {
        broadcast_msg(room, msg);
    }
}

async fn send_message_by_id<T: Into<Message>>(
    rooms: &Rooms,
    game_id: &GameId,
    player: Color,
    msg: T,
) {
    if let Some(room) = rooms.read().await.get(game_id) {
        if let Some(client) = room.get_player(player) {
            send_message(client, msg);
        }
    }
}

fn broadcast_msg<T: Into<Message>>(room: &OnlineGame, msg: T) {
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
