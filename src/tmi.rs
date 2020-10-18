extern crate native_tls;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::client::AutoStream;
use tungstenite::http::{HeaderValue, Request};
use tungstenite::protocol::WebSocket;
use tungstenite::Message;

pub struct TMI {
    ws: Arc<Mutex<WebSocket<AutoStream>>>,
    pub verbose: bool,
}

pub struct DecodedMessage {
    pub metadata: HashMap<String, String>,
    pub from: String,
    pub command: String,
    pub params: Vec<String>,
}

impl TMI {
    pub fn read_message(&self) -> (mpsc::Receiver<DecodedMessage>, thread::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let i_stream = self.ws.clone();
        let verbose = self.verbose;
        let t = thread::spawn(move || loop {
            let mut messages_handled: Vec<String> = Vec::new();
            let mut inner_stream = i_stream.lock().unwrap();
            let msg = inner_stream.read_message().unwrap().to_string();
            if verbose == true {
                println!("> {}", msg);
            }
            let lines = msg.trim().split("\n");
            for line in lines {
                if line.starts_with("PING ") {
                    if verbose == true {
                        println!("< {}", line.replace("PING ", "PONG "));
                    }
                    inner_stream
                        .write_message(Message::from(line.replace("PING ", "PONG ")))
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
                segments.drain(0..2);
                msg.params = segments.iter().map(|s| s.to_string()).collect();

                tx.send(msg).unwrap();
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

        let mut tmi = TMI {
            ws: Arc::new(Mutex::new(ws)),
            verbose: false,
        };

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
        self.ws
            .lock()
            .unwrap()
            .write_message(Message::Text(message))
    }
}
