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
use std::collections::BTreeMap;
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

    // Set up the algorithm
    let mut tiers = BTreeMap::new();
    tiers.insert(300, [0.1585, 0.0884]); // midnight blue
    tiers.insert(600, [0.6531, 0.2834]); // redish

    let tiered = TieredColorSwap {
        base_color: [0.3174, 0.3207],
        tiers: tiers,
        measurement_seconds: 0.7,
        transition_milliseconds: 1,
    };

    // TODO: Move this to the callback implementation for the trait
    run(bridge, tiered);
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

    // do the setup for the algorithm
    callback.setup(&bridge);

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

// TODO: account for gammut?
