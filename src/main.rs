#![feature(fn_traits)]

#[macro_use]
extern crate clap;
extern crate midir;

#[macro_use]
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

// type Color = [f32; 2];

// TODO: Implement default algorithms
fn main() {
    // Get all the preliminary set up out of the way.
    dotenv::dotenv().ok();
    let ip = env::var("HUE_IP").unwrap();
    let key = env::var("HUE_KEY").unwrap();
    let bridge = Bridge::link(ip, key);

    let matches = create_app().get_matches();

    // default implementation of HPM based thresholding
    // some color tiers for hpm
    let mut tiers = BTreeMap::new();
    tiers.insert(0, [1.0, 1.0]); // midnight blue
    tiers.insert(200, [0.1585, 0.0884]); // midnight blue
    tiers.insert(600, [1.0, 0.0]); // redish

    // TODO: Might just go back to structs
    // default implementation of 'All Blink Debug'
    let default_blinkbug = algo!(debug(), blink(1, None));

    // default implementation for 'Hits-Per-Minute'
    let default_hpm = algo!(hpm_threshold([0.3174, 0.3207], &tiers, 0.7, 1));

    // default implementation for 'All Lights Blink'
    let default_blink = algo!(blink(1, None));

    // default implementation of 'All Blink Debug'
    let default_variety = algo!(variety_threshold([1.0, 1.0], [1.0, 0.0], 3, 0.7, 1));

    match matches.value_of("METHOD") {
        Some("hpm") => run(default_hpm, bridge),
        Some("blink") => {
            if let Some(pad) = matches.value_of("PAD") {
                if let Ok(ipad) = pad.parse::<u8>() {
                    let new_blink = blink(1, Some(vec![ipad]));
                    run(new_blink, bridge);
                } else {
                    panic!("Can not parse the pads")
                }
            } else {
                run(default_blink, bridge);
            }
        }
        Some("variety") => run(default_variety, bridge),
        Some("debug") => run(default_blinkbug, bridge),
        // Some("debug") => run(default_blinkbug, bridge),
        None => {
            println!("Incorrect method passed!");
        }
        _ => (),
    }
}

/// Simply creates a `clap` app with certain parsing capabilities and which is the main entry
/// point for the tool.
pub fn create_app() -> clap::App<'static, 'static> {
    let app = clap_app!(drue =>
                            (version: "0.1")
                            (author: "Art Eidukas <iwiivi@gmail.com>")
                            (about: "This app allows drum input to fire hue light commands.")
                            (@arg METHOD: -m --method +takes_value "Set which method to activate (blink|variety|hpm|debug)")
                            (@arg PAD: -p --pad +takes_value "Set which pad gets mapped to specialised blinking. Only works with `blink`")
    );

    // TODO: make sure this is consistent with the way the API is designed
    app
}

/// This function just wraps the flow of acquiring a midi input stream.
/// It is based on the examples in the `midir` crate.
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
