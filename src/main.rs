mod tmi;
use tmi::TMI;

fn main() {
    let mut tmi = TMI::new(
        std::env::var("TWITCH_TOKEN").unwrap(),
        "madhousesteve".into(),
    );
    tmi.verbose = true;

    let (rx, t) = tmi.start_loop();

    loop {
        let msg = rx.recv().unwrap();
        println!(">> {:?}", msg);

        if msg.command == "QUIT" {
            break;
        }
    }

    t.join();
}
