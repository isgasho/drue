#[macro_use]
extern crate clap;
extern crate midir;

mod algorithms;
mod core;
mod state;

use crate::core::*;
use algorithms::*;
use dotenv;
use huemanity::Bridge;
use midir::{Ignore, MidiInput};
use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::io::{stdin, stdout, Write};

fn main() {
    dotenv::dotenv().ok();
    let ip = env::var("HUE_IP").unwrap();
    let key = env::var("HUE_KEY").unwrap();

    let bridge = Bridge::link(ip, key);

    let matches = create_app().get_matches();

    // Set up the algorithm
    let mut tiers = BTreeMap::new();
    tiers.insert(0, [1.0, 1.0]); // midnight blue
    tiers.insert(200, [0.1585, 0.0884]); // midnight blue
    tiers.insert(600, [1.0, 0.0]); // redish

    let tiered = TieredThreshold {
        base_color: [0.3174, 0.3207],
        tiers: tiers,
        measurement_seconds: 0.7,
        transition_milliseconds: 1,
    };

    let variety = VarietyThreshold {
        below: [1.0, 1.0],
        above: [1.0, 0.0],
        variety_threshold: 5,
        measurement_seconds: 0.7,
        transition_milliseconds: 1,
    };

    let blink = Blink { duration: 1 };

    match matches.value_of("METHOD") {
        Some("blink") => run(bridge, blink),
        Some("variety") => run(bridge, variety),
        Some("debug") => run(bridge, DummyPrint),
        Some("hpm") => run(bridge, tiered),
        None => {
            println!("Incorrect method passed!");
        }
        _ => (),
    }
}

pub fn create_app() -> clap::App<'static, 'static> {
    let app = clap_app!(drue =>
                            (version: "0.1")
                            (author: "Art Eidukas <iwiivi@gmail.com>")
                            (about: "This app allows drum input to fire hue light commands.")
                            (@arg METHOD: -m --method +takes_value "Set which method to activate (blink|variety|hpm|debug)")
    );
    app
}

pub fn acquire_midi_input() -> Result<(usize, MidiInput), Box<dyn Error>> {
    let mut input = String::new();
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Check which ports are available
    println!("Availabe input ports:");
    for input in 0..midi_in.port_count() {
        println!("{}: {}", input, midi_in.port_name(input).unwrap());
    }

    print!("Please select input port: ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let in_port: usize = input.trim().parse()?;
    Ok((in_port, midi_in))
}
