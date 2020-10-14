mod tmi;
use tmi::TMI;

fn main() {
    let mut tmi = TMI::new(
        std::env::var("TWITCH_TOKEN").unwrap(),
        "madhousesteve".into(),
    );
    tmi.verbose = true;
    tmi.read_message(|msg: tmi::DecodedMessage| {
        println!(">> {} {:?}", msg.command, msg.metadata);
    });
}
