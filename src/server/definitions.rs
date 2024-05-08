
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::Message;

pub use uuid::Uuid;

#[allow(unused_imports)]
pub use log::{debug as dbg, error as err, info as inf, trace as trc, warn as wrn};

pub struct Client {
    pub id: Uuid,
    pub sender: UnboundedSender<Result<Message, warp::Error>>,
    pub room: String,
}

pub type Rooms = Arc<RwLock<HashMap<String, Vec<Client>>>>;
