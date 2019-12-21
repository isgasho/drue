#[macro_use]
extern crate clap;
extern crate midir;

mod algorithms;
mod cli;
mod midi;
mod state;
mod utils;

use algorithms::*;
use dotenv;
use huemanity::Bridge;
use midi::*;
use state::*;
use std::env;
use std::error::Error;
// use std::thread::sleep;
// use std::time::Duration;

fn main() {
    dotenv::dotenv().ok();
    let ip = env::var("HUE_IP").unwrap();
    let key = env::var("HUE_KEY").unwrap();

    let bridge = Bridge::link(ip, key);

    // TODO: use this
    let matches = cli::create_app().get_matches();

    run(bridge, Blink);
}

fn run(bridge: huemanity::Bridge, callback: impl Callback) -> Result<(), Box<dyn Error>> {
    // TODO: Take in a function here and use that as a callback
    // acquire an input
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
    };

    // connect
    let conn_in = midi_in.connect(
        in_port,
        "Connection from Rust",
        move |stamp, message, data| {
            callback.execute(stamp, message, data, &bridge);
        },
        state,
    )?;

    // main loop
    while true {
        continue;
    }
    // TODO: figure out if this gets closed
    let (midi_in_, log_all_bytes) = conn_in.close();

    Ok(())
}
