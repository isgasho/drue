pub mod algorithms;
pub mod core;
pub mod state;
pub mod utils;

extern crate midir;

use midir::{Ignore, MidiInput};
use std::error::Error;
use std::io::{stdin, stdout, Write};

/// This function just wraps the flow of acquiring a midi input stream.
/// It is based on the examples in the `midir` crate.
pub fn acquire_midi_input() -> Result<(usize, MidiInput), Box<dyn Error>> {
    let mut input = String::new(); // needs a string buffer
    let mut midi_in = MidiInput::new("drue")?;
    midi_in.ignore(Ignore::None);

    // Check which ports are available
    println!("Availabe midi inputs:");
    for input in 0..midi_in.port_count() {
        println!("{}: {}", input, midi_in.port_name(input).unwrap());
    }

    print!("Please select midi input number: ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let in_port: usize = input.trim().parse()?;
    Ok((in_port, midi_in))
}
