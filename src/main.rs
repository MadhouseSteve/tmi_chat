mod tmi;
use tmi::TMI;

fn main() {
    let mut tmi = TMI::new(
        std::env::var("TWITCH_TOKEN").unwrap(),
        "madhousesteve".into(),
    );
    tmi.verbose = true;
    let (rx, t) = tmi.read_message();
    loop {
        let msg = rx.recv();
        match msg {
            Ok(message) => match message.command.as_str() {
                "PRIVMSG" => println!("{} SAID {:?}", message.from, message.params[1]),
                "JOIN" => println!("{} JOINED", message.from),
                _ => {}
            },
            Err(_e) => break,
        }
    }
    t.join().unwrap();
}
