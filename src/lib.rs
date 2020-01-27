pub mod algorithms;
pub mod core;
pub mod state;

#[macro_use]
extern crate clap;
extern crate midir;

use midir::{Ignore, MidiInput};
use std::error::Error;
use std::io::{stdin, stdout, Write};

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
