use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::client::AutoStream;
use tungstenite::http::{HeaderValue, Request};
use tungstenite::protocol::WebSocket;
use tungstenite::Message;

pub struct TMI {
    ws: WebSocket<AutoStream>,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct DecodedMessage {
    pub metadata: HashMap<String, String>,
    pub from: String,
    pub command: String,
    pub params: Vec<String>,
}

impl TMI {
    pub fn start_loop(&mut self) -> (Receiver<DecodedMessage>, thread::JoinHandle<()>) {
        let (tx, rx) = channel::<DecodedMessage>();

        let shared_self = Arc::new(Mutex::new(self));
        let arc_tx = Arc::new(Mutex::new(tx));

        let t = thread::spawn(|| {
            let tx = arc_tx.clone();
            let local_self = shared_self.clone();
            // TODO - This could get large. We need some kind of TTL/cleanup
            let mut messages_handled: Vec<String> = Vec::new();
            loop {
                let msg = local_self
                    .lock()
                    .unwrap()
                    .ws
                    .read_message()
                    .unwrap()
                    .to_string();
                let lines = msg.trim().split("\n");
                for line in lines {
                    if local_self.lock().unwrap().verbose {
                        println!("< {}", line);
                    }

                    if line.starts_with("PING ") {
                        local_self
                            .lock()
                            .unwrap()
                            .send_message(line.replace("PING ", "PONG "))
                            .unwrap();
                        continue;
                    }

                    let mut segments: Vec<&str> = line.split(" ").collect();
                    let mut msg = DecodedMessage {
                        metadata: HashMap::new(),
                        from: String::new(),
                        command: String::new(),
                        params: Vec::new(),
                    };
                    if !segments[0].starts_with("@") {
                        continue;
                    }

                    let metadata = segments[0];
                    for entry in metadata.split(";") {
                        let mut pieces: Vec<&str> = entry.split("=").collect();
                        let key = pieces[0];
                        pieces.drain(0..1);
                        let val = pieces.join("");
                        msg.metadata.insert(key.into(), val);
                    }

                    if msg.metadata.get("id".into()).is_some() {
                        if messages_handled.contains(msg.metadata.get("id".into()).unwrap()) {
                            continue;
                        }
                        messages_handled.push(msg.metadata.get("id").unwrap().into());
                    }

                    segments.drain(0..1);

                    msg.from = segments[0].split("!").next().unwrap().into();
                    msg.command = segments[1].into();
                    segments.drain(0..1);
                    msg.params = segments.iter().map(|s| s.to_string()).collect();

                    tx.lock().unwrap().send(msg).unwrap();
                }
            }
        });

        (rx, t)
    }

    pub fn new(pass: String, name: String) -> TMI {
        let mut ws_request = Request::get("wss://irc-ws.chat.twitch.tv/")
            .body(())
            .unwrap();

        ws_request.headers_mut().insert(
            "Sec-Websocket-Protocol",
            HeaderValue::from_str("tmi".into()).unwrap(),
        );

        let (ws, _response) = tungstenite::connect(ws_request)
            .expect("Unable to connect to wss://tmi-ws.chat.twitch.tv");

        let mut tmi = TMI { ws, verbose: false };

        tmi.send_message("CAP REQ :twitch.tv/tags twitch.tv/commands twitch.tv/membership".into())
            .unwrap();
        tmi.send_message(format!("PASS {}", pass)).unwrap();
        tmi.send_message(format!("NICK {}", name)).unwrap();
        tmi.send_message(format!("JOIN #{}", name)).unwrap();

        tmi
    }

    pub fn send_message(&mut self, message: String) -> Result<(), tungstenite::Error> {
        if self.verbose {
            println!("> {}", message);
        }
        self.ws.write_message(Message::Text(message))
    }
}
