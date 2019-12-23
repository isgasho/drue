#[macro_use]
extern crate clap;
extern crate midir;

mod algorithms;
mod cli;
mod midi;
mod state;

use algorithms::*;
use dotenv;
use huemanity::Bridge;
use std::collections::BTreeMap;
use std::env;

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

    run(bridge, tiered);
}

// TODO: account for gammut?
