use std::{str::FromStr, sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

use eframe::egui;
use futures::StreamExt;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tungstenite::{Error, Message};
use url::Url;


struct App {
    message: String,
    display: String,
    input: UnboundedSender<String>,
    output: Receiver<String>,
}

async fn connect(url: Url, input: UnboundedReceiver<String>, output: Sender<String>) -> Result<(), Error> {
    let (websocket, _) = tokio_tungstenite::connect_async(url).await?;
    let (write, read) = websocket.split();
    let input = UnboundedReceiverStream::new(input);
    tokio::task::spawn(input.map(|msg| {
        Ok(Message::text(msg))
    }).forward(write));

    Ok(read.for_each(|msg| async {
        let data = msg.unwrap();
        if data.is_text() {
            let _ = output.send(data.into_text().unwrap());
        }
    }).await)
}

fn start_client_thread(url: Url) -> (UnboundedSender<String>, Receiver<String>, JoinHandle<()>) {
    let (client_input_tx, client_input_rx) = mpsc::unbounded_channel();
    let (client_output_tx, client_output_rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(
            connect(url, client_input_rx, client_output_tx)
        ).expect("Failed to connect!");
    });
    (client_input_tx, client_output_rx, handle)
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let url = Url::from_str(&std::env::args().nth(1).expect("Choose link")).expect("Failed to parse link");
    let (input, output, handle) = start_client_thread(url);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "Web Client",
        options,
        Box::new(|cc| {
            Box::new(App {
                message: String::new(),
                display: String::new(),
                input,
                output,
            })
        }),
    );
    handle.join().expect("Expected thread to finish");
    result
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let msg_edit = ui.text_edit_singleline(&mut self.message);
                if msg_edit.lost_focus() || ui.button("Send").clicked() {
                    let _ = self.input.send(self.message.clone());
                    // self.message.push('\n');
                    // self.display.push_str(&self.message);
                    self.message = String::new();
                    msg_edit.request_focus();
                }
            });
            if let Ok(mut text) = self.output.try_recv() {
                text.push('\n');
                self.display.push_str(&text);
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(&self.display);
            });
        });
    }
}
