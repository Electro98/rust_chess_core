use tungstenite::Message;
use url::Url;

fn main() {
    println!("It's client!");
    env_logger::init();

    let (mut socket, response) = tungstenite::connect(
        Url::parse("ws://127.0.0.1:3030/ws").unwrap())
        .expect("Failed to connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket.send(Message::text("Connected to room")).unwrap();
    loop {
        let msg = socket.read().expect("Failed to read message");
        println!("Received: {}", msg);
    }
}
