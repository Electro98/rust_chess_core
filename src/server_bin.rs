use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc::{self, UnboundedReceiver, UnboundedSender}, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{filters::ws::Message, reject::Rejection, reply::Reply, Filter};

// use chess_engine::server::definitions::*;
// use chess_engine::server::handlers::*;
#[allow(unused_imports)]
use log::{error, warn, info, trace, debug};

struct Client {
    id: uuid::Uuid,
    sender: UnboundedSender<Result<Message, warp::Error>>,
}

type Clients = Arc<RwLock<Vec<Client>>>;

type ClientMessage = String;

async fn new_client_handler(ws: warp::ws::Ws, clients: Clients, sender: UnboundedSender<ClientMessage>) -> Result<impl Reply, Rejection> {
    info!("Get new connection to websocket!");
    Ok(ws.on_upgrade(|ws| client_connection(ws, clients, sender)))
}

async fn client_connection(ws: warp::ws::WebSocket, clients: Clients, sender: UnboundedSender<ClientMessage>) {
    let (ws_sender, mut ws_receiver) = ws.split();
    let (send_to_client, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(ws_sender).map(|result| {
        if let Err(err) = result {
            error!("Failed to send message to client using websocket! Err: {}", err);
        }
    }));

    let client_id = {
        let client_id = uuid::Uuid::new_v4();
        let mut clients = clients.write().await;
        clients.push(Client {
            id: client_id.clone(),
            sender: send_to_client,
        });
        client_id
    };
    debug!("Added new client: {}", client_id);

    while let Some(msg) = ws_receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(err) => {
                error!("Failed receiving message from client! Err: {}", err);
                break;
            },
        };
        if msg.is_text() {
            trace!("Received new text message: {:?}", msg);
            let _ = sender.send(String::from_utf8(msg.into_bytes()).unwrap());
        } else {
            debug!("Received non text message: {:?}", msg);
        }
    }

    {
        let mut clients = clients.write().await;
        let client_index = clients.iter().enumerate().find(|(_, client)| client.id == client_id).map(|(i, _)| i).unwrap();
        clients.remove(client_index);
        debug!("Removed client: {}", client_id);
    }
}

async fn message_router(mut rcv: UnboundedReceiver<ClientMessage>, clients: Clients) {
    while let Some(msg) = rcv.recv().await {
        let clients = clients.read().await;
        for client in &*clients {
            let _ = client.sender.send(Ok(Message::text(msg.clone())));
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("It's server!");

    let clients: Clients = Default::default();

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    let (sender, rcv) = mpsc::unbounded_channel::<ClientMessage>();

    tokio::task::spawn(message_router(rcv, clients.clone()));

    let new_room = warp::path("ws")
        .and(warp::ws())
        .and(with(clients.clone()))
        .and(with(sender.clone()))
        .and_then(new_client_handler);

    // let existing_room = warp::path!("ws" / String)
    //     .and(warp::ws())
    //     .and(with(clients.clone()))
    //     .and_then(existing_room_handler);

    // let routes = hello.or(existing_room).or(new_room);
    let routes = hello.or(new_room);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

fn with<T>(value: T) -> impl Filter<Extract = (T,), Error = std::convert::Infallible> + Clone
where
    T: Clone + std::marker::Send,
{
    warp::any().map(move || value.clone())
}
