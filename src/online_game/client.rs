use std::{
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
};

use futures::{future, SinkExt, StreamExt};
use log::{error, info};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use url::Url;

use crate::{
    core::engine::{Game, Move},
    Color,
};

use crate::online_game::definitions::{ClientMessage, ServerMessage};

use super::definitions::GameId;

#[derive(Default, Clone, Copy, Debug)]
pub enum ClientState {
    #[default]
    Unconnected,
    WaitingOpponent,
    GameMyTurn,
    GameTurnValidation,
    GameEnemyTurn,
    GameFinished,
}

#[derive(Default, Clone)]
struct OnlineClientData {
    pub state: Arc<Mutex<ClientState>>,
    pub game: Arc<Mutex<Option<Game>>>,
    pub game_id: Arc<Mutex<String>>,
    pub color: Arc<Mutex<Color>>,
}

pub enum OnlineClientInput {
    Disconnect,
    Move(Move),
    Resign,
}

pub enum OnlineClientOutput {
    ReceivedGameId,
    StateChanged(ClientState),
    IncorrectInput,
}

pub struct OnlineClient {
    thread: std::thread::JoinHandle<()>,
    data: OnlineClientData,
    input: UnboundedSender<OnlineClientInput>,
    pub output: Receiver<OnlineClientOutput>,
}

impl Into<OnlineClientOutput> for ClientState {
    fn into(self) -> OnlineClientOutput {
        OnlineClientOutput::StateChanged(self)
    }
}

async fn handle_server_message(
    data: &OnlineClientData,
    message: ServerMessage,
) -> Result<Option<OnlineClientOutput>, ()> {
    // TODO: is it okay to do so?
    let state = { *data.state.lock().await };
    match state {
        ClientState::Unconnected => match message {
            ServerMessage::GameStateSync(board, last_move, current_player, client_color) => {
                *data.game.lock().await = Some(Game::new(board, current_player, last_move));
                *data.color.lock().await = client_color;
                *data.state.lock().await = ClientState::WaitingOpponent;
                Ok(Some(ClientState::WaitingOpponent.into()))
            }
            ServerMessage::RoomId(game_id) => {
                *data.game_id.lock().await = game_id;
                Ok(Some(OnlineClientOutput::ReceivedGameId))
            }
            _ => Err(()),
        },
        ClientState::WaitingOpponent => match message {
            ServerMessage::OpponentConnected => {
                let new_state = if data.game.lock().await.as_ref().unwrap().current_player()
                    == *data.color.lock().await
                {
                    ClientState::GameMyTurn
                } else {
                    ClientState::GameEnemyTurn
                };
                *data.state.lock().await = new_state;
                Ok(Some(new_state.into()))
            }
            _ => Err(()),
        },
        ClientState::GameMyTurn => match message {
            ServerMessage::OpponentDisconnected => {
                *data.state.lock().await = ClientState::WaitingOpponent;
                Ok(Some(ClientState::WaitingOpponent.into()))
            }
            _ => Err(()),
        },
        ClientState::GameTurnValidation => match message {
            ServerMessage::OpponentDisconnected => {
                *data.state.lock().await = ClientState::WaitingOpponent;
                Ok(Some(ClientState::WaitingOpponent.into()))
            }
            ServerMessage::GameStateSync(board, last_move, current_player, client_color) => {
                *data.game.lock().await = Some(Game::new(board, current_player, last_move));
                let new_state = if current_player == client_color {
                    ClientState::GameMyTurn
                } else {
                    ClientState::GameEnemyTurn
                };
                *data.state.lock().await = new_state;
                Ok(Some(new_state.into()))
            }
            _ => Err(()),
        },
        ClientState::GameEnemyTurn => match message {
            ServerMessage::OpponentDisconnected => {
                *data.state.lock().await = ClientState::WaitingOpponent;
                Ok(Some(ClientState::WaitingOpponent.into()))
            }
            ServerMessage::GameStateSync(board, last_move, current_player, _) => {
                *data.game.lock().await = Some(Game::new(board, current_player, last_move));
                *data.state.lock().await = ClientState::GameMyTurn;
                Ok(Some(ClientState::GameMyTurn.into()))
            }
            _ => Err(()),
        },
        ClientState::GameFinished => {
            match message {
                ServerMessage::OpponentDisconnected => {
                    // TODO: Potential to restart game!
                    Ok(None)
                }
                _ => Err(()),
            }
        }
    }
}

async fn handle_client_input(
    data: &OnlineClientData,
    input: OnlineClientInput,
) -> Result<(Option<OnlineClientOutput>, Option<ClientMessage>), ()> {
    let state = { *data.state.lock().await };
    match input {
        OnlineClientInput::Disconnect => {
            let mut lock = data.state.lock().await;
            if !matches!(*lock, ClientState::Unconnected) {
                *lock = ClientState::Unconnected;
                Ok((
                    Some(ClientState::Unconnected.into()),
                    Some(ClientMessage::Disconnect),
                ))
            } else {
                Ok((Some(OnlineClientOutput::IncorrectInput), None))
            }
        }
        OnlineClientInput::Move(client_move) => match state {
            ClientState::GameMyTurn => {
                *data.state.lock().await = ClientState::GameTurnValidation;
                Ok((
                    Some(ClientState::GameTurnValidation.into()),
                    Some(ClientMessage::MakeMove(client_move)),
                ))
            }
            _ => Err(()),
        },
        OnlineClientInput::Resign => match state {
            ClientState::GameMyTurn | ClientState::GameEnemyTurn => {
                *data.state.lock().await = ClientState::GameTurnValidation;
                Ok((
                    Some(ClientState::GameTurnValidation.into()),
                    Some(ClientMessage::Resigned),
                ))
            }
            _ => Ok((Some(OnlineClientOutput::IncorrectInput), None)),
        },
    }
}

impl OnlineClient {
    pub fn start_client(url: Url) -> Self {
        let (client_input_tx, client_input_rx) = mpsc::unbounded_channel();
        let (client_output_tx, client_output_rx) = std::sync::mpsc::channel();
        let online_data = OnlineClientData::default();
        let data = online_data.clone();
        let handle = std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(Self::connect(url, data, client_input_rx, client_output_tx))
                .expect("Failed to connect!");
        });
        Self {
            thread: handle,
            data: online_data,
            input: client_input_tx,
            output: client_output_rx,
        }
    }

    /// Wait and receive current online client state!
    ///  - Blocking function!
    pub fn current_state(&self) -> ClientState {
        *self.data.state.blocking_lock()
    }

    /// Wait and receive current online client GameID!
    ///  - Blocking function!
    pub fn game_id(&self) -> GameId {
        self.data.game_id.blocking_lock().clone()
    }

    /// Wait and receive current online client color!
    ///  - Blocking function!
    pub fn player_color(&self) -> Color {
        *self.data.color.blocking_lock()
    }

    /// Get current online client game mutex
    pub fn game(&self) -> &Arc<Mutex<Option<Game>>> {
        &self.data.game
    }

    /// Wait and execute function on current value of online client
    ///  - Blocking function!
    ///  - May be `None` if game is `None`
    pub fn in_game<F, T>(&self, func: F) -> Option<T>
    where
        F: FnMut(&Game) -> T,
    {
        self.data.game.blocking_lock().as_ref().map(func)
    }

    // TODO: To be real, better not to ignore this results
    pub fn resign(&self) {
        let _ = self.input.send(OnlineClientInput::Resign);
    }

    pub fn disconnect(&self) {
        let _ = self.input.send(OnlineClientInput::Disconnect);
    }

    pub fn make_move(&self, _move: Move) {
        let _ = self.input.send(OnlineClientInput::Move(_move));
    }

    /// Internal implementation
    async fn connect(
        url: Url,
        data: OnlineClientData,
        input_rx: UnboundedReceiver<OnlineClientInput>,
        output_tx: Sender<OnlineClientOutput>,
    ) -> Result<(), tungstenite::Error> {
        let (websocket, _) = tokio_tungstenite::connect_async(url).await?;
        let (mut write, read) = websocket.split();
        let input = UnboundedReceiverStream::new(input_rx);

        let _ = write.send(ClientMessage::Connected.into()).await;
        let data_cp = data.clone();
        let output_tx_cp = output_tx.clone();
        let client_handling = tokio::spawn(
            input
                .filter_map(move |client_input| {
                    let data = data_cp.clone();
                    let output_tx = output_tx_cp.clone();
                    async move {
                        match handle_client_input(&data, client_input).await {
                            Ok((output, client_msg)) => {
                                if let Some(output) = output {
                                    let _ = output_tx.send(output);
                                }
                                client_msg.map(|msg| Ok(msg.into()))
                            }
                            Err(err) => {
                                error!("Client provided incorrect input! Err: {:?}", err);
                                let _ = output_tx.send(OnlineClientOutput::IncorrectInput);
                                None
                            }
                        }
                    }
                })
                .forward(write),
        );

        let server_handling = read.for_each(|msg| async {
            let server_message = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    info!("Something went wrong with connection! Err: {}", err);
                    *data.state.lock().await = ClientState::Unconnected;
                    let _ = output_tx.send(ClientState::Unconnected.into());
                    return;
                }
            }
            .try_into();
            if let Ok(server_message) = server_message {
                match handle_server_message(&data, server_message).await {
                    Ok(output) => {
                        if let Some(output) = output {
                            let _ = output_tx.send(output);
                        }
                    }
                    Err(err) => {
                        error!("Server provided incorrect message! err: {:?}", err);
                    }
                }
            }
        });
        let _ = future::join(client_handling, server_handling).await;
        // future::join(tokio::task::spawn(client_handling), tokio::task::spawn(server_handling)).await;
        Ok(())
    }
}
